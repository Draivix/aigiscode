#!/usr/bin/env python3
"""
triage_driver.py – triage layer over the registry: nonsense elimination, difficulty and severity per finding.

  python3 triage_driver.py --build                 # clone_exact difficulty by script; model + dead_symbol findings → triage/batches/t_NNNN.json
  python3 triage_driver.py --run --parallel 10     # one headless `claude -p` (Sonnet 5.5, medium, read-only tools) per batch
  python3 triage_driver.py --summary               # TRIAGE.md

Writes only under <out>/triage/ (triage.jsonl keyed by finding id, results/, batches/, driver.log) and <out>/TRIAGE.md.
The registry is never touched: a triage verdict becomes a registry status only through a continuation record written by
the director. Incremental: --build skips ids that already have a triage line, --run skips batches with a result.
Reviewer choice (2026-10-19 test, 30 findings): Sonnet 5.5 medium agrees with Opus 5.5 medium on 22/30 verdicts with no
factual slip at half the wall time; Opus is used only as a second opinion on S1 (--model override).
"""
import argparse, collections, concurrent.futures, datetime, json, os, pathlib, re, sqlite3, subprocess, sys, threading, time

HERE = pathlib.Path(__file__).resolve().parent
OUT = pathlib.Path(os.environ.get("LEAN_OUT", HERE.parent / "out")).resolve()
REPO = pathlib.Path(os.environ.get("LEAN_REPO", OUT.parent.parent)).resolve()
TR = OUT / "triage"
PER_BATCH, MAX_TURNS, RUN_TIMEOUT_S = 30, 80, 1800
LOCK = threading.Lock()

SCHEMA = {"type": "object", "required": ["items"], "properties": {"items": {"type": "array", "items": {
    "type": "object", "required": ["id", "verdict", "reason", "difficulty", "severity", "confidence"],
    "properties": {"id": {"type": "string"}, "verdict": {"type": "string", "enum": ["keep", "reject", "unsure"]},
                   "reason": {"type": "string"}, "difficulty": {"type": "string", "enum": ["delete", "merge_local", "merge_cross", "design"]},
                   "severity": {"type": "string", "enum": ["S1", "S2", "S3"]}, "confidence": {"type": "string", "enum": ["high", "medium", "low"]}}}}}}

PROMPT = """You are a code-audit triage reviewer for a Laravel 11 / Cycle ORM PHP backend (Draivix ERP). Read-only task: you have only Read, Grep and Glob; never try to change anything.

Input: {batch} – {n} findings produced by an LLM worker (or by PHPStan/graph analysis for kind dead_symbol) in a dead-code / duplication audit. Each has id, kind, title, symbols, rationale, evidence quotes (file + line) and symbol_details with the symbol bodies, graph caller counts, reachability and same-name reference counts. The file is long: read it completely with offset/limit until the end before answering. Paths are relative to the current directory (the audited git worktree). Read the real files whenever the excerpts are not enough; grep for callers when a claim depends on who calls what. Dynamic dispatch exists: strings in config/*.php, app/Actions/actions.php, config/jobs.php, routes/**, hook files (*.hooks.php), entity metadata JSON and Blade/Vue templates can reference a method by name.

For EVERY finding decide:
1. verdict: "keep" (real, and the recommendation would be safe to carry out), "reject" (wrong, not equivalent, not dead, not redundant, or nonsense), "unsure" (cannot decide from the code; name the evidence that would settle it). Be strict: clone_semantic / variant_mergeable only "keep" if the behaviours really match or the stated differences are complete; dead_branch / redundant_layer only "keep" if the guard really cannot fire or the layer really adds nothing; dead_symbol only "keep" if you found no caller, no override/interface contract, no string or convention reference (routes, hooks, actions, jobs, metadata) and no test-only usage that keeps it alive on purpose. A missed edge case (null, empty, exceptions, side effects, callers relying on the difference) means "reject" or "unsure" with the edge case named.
2. reason: one or two sentences with file:line references you actually read.
3. difficulty: "delete" (remove code, nothing to adapt) | "merge_local" (same module, one canonical survivor, callers trivially moved) | "merge_cross" (across modules or layers, adapter or signature change needed) | "design" (needs a product or ownership decision).
4. severity: "S1" (risk of divergent behaviour today: double paths, variant validation, two sources of truth) | "S2" (maintenance cost only) | "S3" (cosmetic).
5. confidence: "high" | "medium" | "low".

Answer with the structured output only: {{"items": [{{"id","verdict","reason","difficulty","severity","confidence"}}, …]}} with every one of the {n} ids exactly once."""


def now(): return datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%SZ")


def log(msg):
    line = f"{now()} {msg}"; print(line, flush=True)
    with (TR / "driver.log").open("a") as f: f.write(line + "\n")


def latest_registry():
    latest = {}
    for l in (OUT / "registry.merged.jsonl").read_text(encoding="utf-8").splitlines():
        if l.strip(): r = json.loads(l); latest[r["id"]] = r
    return latest


def triaged_ids():
    p = TR / "triage.jsonl"
    return {json.loads(l)["id"] for l in p.read_text().splitlines() if l.strip()} if p.exists() else set()


def append_triage(rows, target="triage.jsonl"):
    with LOCK, (TR / target).open("a", encoding="utf-8") as f:
        for r in rows: f.write(json.dumps(r, ensure_ascii=False) + "\n")


def module_of(path):
    m = re.match(r"app/Modules/([^/]+)/", path)
    return m.group(1) if m else path.split("/")[0] + ("/" + path.split("/")[1] if path.startswith(("app/", "database/")) and "/" in path else "")


def build():
    TR.mkdir(exist_ok=True); (TR / "batches").mkdir(exist_ok=True); (TR / "results").mkdir(exist_ok=True)
    latest, done = latest_registry(), triaged_ids()
    for b in TR.glob("batches/t_*.json"): done |= {x["id"] for x in json.loads(b.read_text())}   # already batched, result pending
    db = sqlite3.connect(OUT / "symbols.sqlite")
    callers = json.load((OUT / "graph/callers.json").open())
    sym_file = dict(db.execute("SELECT fqn, file FROM symbol"))
    script_rows, todo = [], []
    for r in latest.values():
        if r["status"] == "rejected" or r["id"] in done: continue
        src = str(r.get("source", ""))
        if r["kind"] == "clone_exact" and src.startswith("pass1/"):   # deterministic fact (jscpd / normalized AST): difficulty from where the copies live
            files = [sym_file.get(s) or "" for s in r["symbols"]]
            if files and all(f.startswith("database/migrations") for f in files): diff = "design"
            else: diff = "merge_local" if len({module_of(f) for f in files if f}) <= 1 else "merge_cross"
            script_rows.append({"id": r["id"], "verdict": "keep", "reason": f"deterministic clone class ({src}); copies in {len({module_of(f) for f in files if f})} module(s)",
                                "difficulty": diff, "severity": "S2", "confidence": "high", "model": "script", "batch": None, "ts": now()})
        elif src.startswith(("pass2/", "pass3/", "pass4/")) or (r["kind"] == "dead_symbol" and r["status"] in ("candidate", "needs_human")):
            todo.append(r)
    append_triage(script_rows)
    todo.sort(key=lambda r: (r["kind"], r["id"]))
    existing = sorted(TR.glob("batches/t_*.json")); n0 = int(existing[-1].stem[2:]) + 1 if existing else 0
    for i in range(0, len(todo), PER_BATCH):
        items = []
        for r in todo[i:i + PER_BATCH]:
            syms = []
            for s in r["symbols"][:6]:
                row = db.execute("SELECT file, line_start, line_end, body, reachable_from_entry, name_refs, dead_code_detector, unused_public FROM symbol WHERE fqn=? LIMIT 1", (s,)).fetchone()
                if row: syms.append({"fqn": s, "file": row[0], "lines": f"{row[1]}-{row[2]}", "callers_in_graph": len(callers.get(s, [])), "reachable_from_entry": row[4],
                                     "same_name_refs_anywhere": row[5], "phpstan_dead": row[6], "phpstan_unused_public": row[7], "body": (row[3] or "")[:3500]})
            items.append({"id": r["id"], "kind": r["kind"], "status": r["status"], "title": r["title"], "symbols": r["symbols"], "rationale": (r.get("rationale") or "")[:1200],
                          "behavior_differences": (r.get("behavior_differences") or [])[:5], "verification": r.get("verification"),
                          "evidence": [{"file": e["file"], "line_start": e.get("line_start"), "quote": e["quote"][:400]} for e in r["evidence"][:4]],
                          "estimated_loc_saved": r.get("estimated_loc_saved"), "symbol_details": syms})
        bid = f"t_{n0 + i // PER_BATCH:04d}"
        (TR / "batches" / f"{bid}.json").write_text(json.dumps(items, ensure_ascii=False, indent=1))
    print(f"triage build: script-triaged clone_exact={len(script_rows)}, model/dead findings={len(todo)} → {(len(todo) + PER_BATCH - 1) // PER_BATCH} new batches (from {n0:04d})")


def usage_limit_wait_s(stdout, default=1800):
    """Seconds until the reset time quoted by claude -p ("… resets 10:30pm (Europe/Prague)"), or `default` when absent."""
    import datetime, zoneinfo
    m = re.search(r"resets (\d{1,2})(?::(\d{2}))?\s*(am|pm)", stdout, re.I)
    if not m: return default
    now = datetime.datetime.now(zoneinfo.ZoneInfo("Europe/Prague"))
    h = int(m.group(1)) % 12 + (12 if m.group(3).lower() == "pm" else 0)
    at = now.replace(hour=h, minute=int(m.group(2) or 0), second=0, microsecond=0)
    if at <= now: at += datetime.timedelta(days=1)
    return int((at - now).total_seconds()) + 30


def run_one(batch_path, model, target="triage.jsonl"):
    bid = batch_path.stem; out = TR / "results" / f"{bid}.json"
    if out.exists(): return bid, "skip"
    items = json.loads(batch_path.read_text()); ids = {x["id"] for x in items}
    prompt = PROMPT.format(batch=str(batch_path.relative_to(REPO)) if batch_path.is_relative_to(REPO) else str(batch_path), n=len(items))
    cmd = ["env", "-u", "CLAUDECODE", "claude", "-p", prompt, "--model", model, "--effort", "medium", "--output-format", "json", "--json-schema", json.dumps(SCHEMA),
           "--tools", "Read,Grep,Glob", "--max-turns", str(MAX_TURNS), "--permission-mode", "dontAsk"]
    attempt = 0
    while attempt < 2:
        attempt += 1
        try: r = subprocess.run(cmd, cwd=REPO, text=True, capture_output=True, timeout=RUN_TIMEOUT_S)
        except subprocess.TimeoutExpired: log(f"{bid} attempt {attempt}: timeout"); continue
        if r.returncode != 0 and "usage_limit_reached" in r.stdout:   # Claude Max 5-hour window: wait it out, the attempt does not count
            secs = usage_limit_wait_s(r.stdout); log(f"{bid}: claude usage limit, waiting {secs}s"); time.sleep(secs); attempt -= 1; continue
        (TR / "results" / f"{bid}.raw{attempt}.txt").write_text(r.stdout + "\n--- stderr ---\n" + r.stderr)
        try:
            outer = json.loads(r.stdout); data = outer.get("structured_output")
            if not isinstance(data, dict):
                txt = re.sub(r"^```(?:json)?|```$", "", (outer.get("result") or "").strip(), flags=re.M).strip(); data = json.loads(txt)
            rows = data["items"]
        except (json.JSONDecodeError, KeyError, TypeError, AttributeError) as e:
            log(f"{bid} attempt {attempt}: unparsable answer ({e}); rc={r.returncode} {r.stderr.strip()[-200:]}"); continue
        got = {x["id"] for x in rows}; missing = ids - got
        rows = [x for x in rows if x["id"] in ids]
        for m in missing: rows.append({"id": m, "verdict": "unsure", "reason": "reviewer returned no verdict for this id", "difficulty": "design", "severity": "S3", "confidence": "low"})
        usage = outer.get("usage") or {}
        out.write_text(json.dumps({"meta": {"model": model, "batch": bid, "attempt": attempt, "ts": now(), "cost_usd": outer.get("total_cost_usd"), "duration_ms": outer.get("duration_ms"),
                                            "usage": {k: usage.get(k) for k in ("input_tokens", "output_tokens", "cache_read_input_tokens", "cache_creation_input_tokens")}, "missing": sorted(missing)},
                                   "items": rows}, ensure_ascii=False, indent=1))
        append_triage([dict(x, model=model, batch=bid, ts=now()) for x in rows], target)
        c = collections.Counter(x["verdict"] for x in rows)
        log(f"{bid} ok: {dict(c)} missing={len(missing)} cost=${outer.get('total_cost_usd', 0):.2f} {int((outer.get('duration_ms') or 0) / 1000)}s")
        return bid, "ok"
    log(f"{bid} FAILED after 2 attempts"); return bid, "failed"


def run(parallel, model, only, prefix="t", target="triage.jsonl"):
    batches = sorted(TR.glob(f"batches/{prefix}_*.json"))
    if only: batches = [b for b in batches if b.stem in only.split(",")]
    batches = [b for b in batches if not (TR / "results" / f"{b.stem}.json").exists()]
    log(f"run: {len(batches)} batches, parallel={parallel}, model={model}")
    with concurrent.futures.ThreadPoolExecutor(max_workers=parallel) as ex:
        summary = collections.Counter(s for _b, s in ex.map(lambda b: run_one(b, model, target), batches))
    log(f"run done: {dict(summary)}"); summarize()


def build_second_opinion(fraction=0.10, seed=11):
    """Opus re-judges every S1 keep plus a random sample of the other model-reviewed ids; batches o_NNNN from the same items."""
    import random
    rows = {}
    for l in (TR / "triage.jsonl").read_text().splitlines():
        if l.strip(): r = json.loads(l); rows[r["id"]] = r
    model_ids = [i for i, r in rows.items() if r["model"] != "script"]
    s1 = {i for i in model_ids if rows[i]["verdict"] == "keep" and rows[i]["severity"] == "S1"}
    random.seed(seed); sample = set(random.sample([i for i in model_ids if i not in s1], int(len(model_ids) * fraction)))
    items = {}
    for b in TR.glob("batches/t_*.json"):
        for x in json.loads(b.read_text()): items[x["id"]] = x
    done = set()   # incremental: ids Opus already judged (earlier wave) keep their verdict; new batches continue the o_ numbering
    if (TR / "second_opinion.jsonl").exists():
        done = {json.loads(l)["id"] for l in (TR / "second_opinion.jsonl").read_text().splitlines() if l.strip()}
    for b in TR.glob("batches/o_*.json"): done |= {x["id"] for x in json.loads(b.read_text())}
    chosen = [items[i] for i in sorted(s1 | sample) if i in items and i not in done]
    start = len(list(TR.glob("batches/o_*.json")))
    for n, i in enumerate(range(0, len(chosen), PER_BATCH), start):
        (TR / "batches" / f"o_{n:04d}.json").write_text(json.dumps(chosen[i:i + PER_BATCH], ensure_ascii=False, indent=1))
    print(f"second opinion: S1 keeps {len(s1)} + sample {len(sample)}, already judged {len(done & (s1 | sample))} → {(len(chosen) + PER_BATCH - 1) // PER_BATCH} new batches o_ from {start:04d}")


def agreement():
    """Sonnet vs Opus on the second-opinion set; S1 keeps that Opus does not confirm are downgraded to unsure in triage.jsonl."""
    first, second = {}, {}
    for l in (TR / "triage.jsonl").read_text().splitlines():
        if l.strip(): r = json.loads(l); first[r["id"]] = r
    for l in (TR / "second_opinion.jsonl").read_text().splitlines():
        if l.strip(): r = json.loads(l); second[r["id"]] = r
    agree = collections.Counter(); downgraded = []
    for i, o in second.items():
        f = first.get(i)
        if not f: continue
        for k in ("verdict", "difficulty", "severity"): agree[k] += f[k] == o[k]
        if f["verdict"] == "keep" and f["severity"] == "S1" and not (o["verdict"] == "keep" and o["severity"] == "S1"):
            downgraded.append(dict(f, verdict="unsure" if o["verdict"] != "reject" else "reject", severity=o["severity"],
                                   reason=f"S1 not confirmed by second opinion ({o['verdict']}/{o['severity']}): {o['reason'][:300]} | first: {f['reason'][:200]}",
                                   confidence="low", model="sonnet+opus", ts=now()))
    append_triage(downgraded)
    n = len(second)
    lines = [f"Second opinion (Opus 5.5 medium) on {n} ids: agreement verdict {agree['verdict']}/{n}, difficulty {agree['difficulty']}/{n}, severity {agree['severity']}/{n}; "
             f"S1 keeps not confirmed: {len(downgraded)} (set to unsure/reject in triage.jsonl)."]
    (TR / "agreement.txt").write_text("\n".join(lines) + "\n"); print(lines[0])


def summarize():
    rows = {}
    for l in (TR / "triage.jsonl").read_text().splitlines():
        if l.strip(): r = json.loads(l); rows[r["id"]] = r   # last line per id wins
    latest = latest_registry()
    kind = {i: latest[i]["kind"] for i in rows if i in latest}
    by = lambda key: collections.Counter(key(r) for r in rows.values())
    model_rows = [r for r in rows.values() if r["model"] != "script"]
    md = [f"# TRIAGE – {now()}", "", f"Triaged findings: {len(rows)} (script-rated clone_exact: {len(rows) - len(model_rows)}, reviewed by model: {len(model_rows)})", "",
          "## Verdict (model-reviewed)", *[f"- {k}: {v}" for k, v in sorted(collections.Counter(r['verdict'] for r in model_rows).items())], "",
          "## Verdict by kind (model-reviewed)"]
    kv = collections.defaultdict(collections.Counter)
    for r in model_rows: kv[kind.get(r["id"], "?")][r["verdict"]] += 1
    md += [f"- {k}: " + ", ".join(f"{v} {c}" for v, c in sorted(kv[k].items())) for k in sorted(kv)]
    md += ["", "## Severity (kept only)", *[f"- {k}: {v}" for k, v in sorted(collections.Counter(r['severity'] for r in rows.values() if r['verdict'] == 'keep').items())],
           "", "## Difficulty (kept only)", *[f"- {k}: {v}" for k, v in sorted(collections.Counter(r['difficulty'] for r in rows.values() if r['verdict'] == 'keep').items())],
           "", "## S1 kept findings (divergent behaviour risk)"]
    s1 = [r for r in rows.values() if r["verdict"] == "keep" and r["severity"] == "S1"]
    for r in sorted(s1, key=lambda r: r["id"]):
        t = latest.get(r["id"], {}).get("title", "")[:110]
        md.append(f"- `{r['id']}` [{kind.get(r['id'], '?')}/{r['difficulty']}/{r['confidence']}] {t} — {r['reason'][:220]}")
    cost = sum((json.loads(p.read_text())["meta"].get("cost_usd") or 0) for p in TR.glob("results/t_*.json"))
    md += ["", f"Reviewer runs: {len(list(TR.glob('results/t_*.json')))}, cost ${cost:.2f}; log `triage/driver.log`."]
    if (TR / "agreement.txt").exists(): md += ["", "## Second opinion", (TR / "agreement.txt").read_text().strip()]
    (OUT / "TRIAGE.md").write_text("\n".join(md) + "\n")
    print(f"TRIAGE.md written: {len(rows)} triaged, S1 kept {len(s1)}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--build", action="store_true"); ap.add_argument("--run", action="store_true"); ap.add_argument("--summary", action="store_true")
    ap.add_argument("--parallel", type=int, default=10); ap.add_argument("--model", default="claude-sonnet-5-5"); ap.add_argument("--only", help="comma list of batch ids")
    ap.add_argument("--second-opinion", action="store_true", help="build o_* batches (S1 keeps + 10 %% sample) and run them with Opus into second_opinion.jsonl")
    ap.add_argument("--agreement", action="store_true")
    a = ap.parse_args()
    TR.mkdir(exist_ok=True)
    if a.build: build()
    if a.run: run(a.parallel, a.model, a.only)
    if a.second_opinion: build_second_opinion(); run(a.parallel, "claude-opus-5-5", None, prefix="o", target="second_opinion.jsonl")
    if a.agreement: agreement()
    if a.summary: summarize()


if __name__ == "__main__":
    main()
