#!/usr/bin/env python3
"""
pass_driver.py – run every batch of a pass through the worker with the DIRECTOR §6 quality protocol, unattended.

  python3 pass_driver.py --pass 2 [--batches 0000,0001] [--max-batches N]

Per batch (skips batches that already have a verified result):
  1. run_agy.py                         worker answer → results/pass<N>/<batch>.json
  2. 100 % quote + symbol check         every evidence.quote must be verbatim in the file, every symbol in symbols.sqlite;
                                        > 5 % mismatch → batch is `tainted` → push-back (reason hallucinated_symbol or
                                        missing_evidence) with the rejected items, up to 3 attempts; the last clean attempt wins
  3. thinness test                      < 1 finding per 3 000 input lines on a batch > 50k tokens → second look
                                        (files shuffled, attempt 2); union of both answers is passed on, intersection marked
                                        in verify.py by the duplicate id (same kind + symbols → same F-id)
  4. verify.py --pass <N> --result <file>   deterministic verification → registry.jsonl
  5. merge_registry.py                  registry.merged.jsonl + STATUS.counts.md
Everything the driver decides is logged to results/pass<N>/driver.log; nothing outside lean-audit/out is written.
"""
import argparse, concurrent.futures, json, os, pathlib, re, sqlite3, subprocess, sys, threading, time, datetime

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
OUT = pathlib.Path(os.environ.get("LEAN_OUT", ROOT / "out")).resolve()
REPO = pathlib.Path(os.environ.get("LEAN_REPO", OUT.parent.parent)).resolve()
DB = OUT / "symbols.sqlite"
TAINT_RATE, THIN_LINES, THIN_MIN_TOKENS = 0.05, 3000, 50_000
REGISTRY_LOCK = threading.Lock()  # verify + merge append to registry.jsonl; worker runs may overlap, registry writes may not


def log(pass_no, msg):
    line = f"{datetime.datetime.now(datetime.UTC).isoformat()}Z {msg}"
    print(line, flush=True)
    with (OUT / "results" / f"pass{pass_no}" / "driver.log").open("a") as f: f.write(line + "\n")


def run(cmd, **kw):
    return subprocess.run(cmd, cwd=REPO, text=True, capture_output=True, env=dict(os.environ, LEAN_OUT=str(OUT), LEAN_REPO=str(REPO)), **kw)


REPO_ROOT = REPO

def quote_matches(ev):
    """True if the quote is verbatim in the file. The model often misnumbers lines (observed: 686 vs real 124); a quote found
    elsewhere in the same file is accepted and the evidence lines are corrected in place (ev["line_relocated"] = True),
    because the DIRECTOR rule is about verifiable verbatim text, not about the model's line arithmetic."""
    try: lines = pathlib.Path(REPO_ROOT, ev["file"]).read_text(encoding="utf-8", errors="replace").splitlines()
    except (FileNotFoundError, KeyError, TypeError, OSError): return False
    norm = lambda s: re.sub(r"\s+", " ", s).strip()
    q = norm(ev.get("quote") or "")
    if not q: return False
    try: lo, hi = max(0, int(ev["line_start"]) - 3), min(len(lines), int(ev.get("line_end", ev["line_start"])) + 2)
    except (KeyError, TypeError, ValueError): lo, hi = 0, 0
    if q in norm("\n".join(lines[lo:hi])): return True
    first = norm(ev["quote"].strip().splitlines()[0]) if ev["quote"].strip() else ""
    n_lines = max(1, len(ev["quote"].strip().splitlines()))
    for i, l in enumerate(lines):
        if first and first in norm(l) and q in norm("\n".join(lines[i:i + n_lines + 2])):
            ev["line_start"], ev["line_end"], ev["line_relocated"] = i + 1, i + n_lines, True
            return True
    return False

def resolve_symbol(db, s):
    """Canonical fqn for a worker symbol, or None. Accepts the exact fqn, `path/File.php:Short::member` and bare
    `Short::member` (observed in pass 2 batch 0094: real symbols, wrong spelling) and resolves them through symbols.sqlite;
    a spelling that matches several symbols stays unresolved."""
    if db.execute("SELECT 1 FROM symbol WHERE fqn=? LIMIT 1", (s,)).fetchone(): return s
    path, short = (s.split(".php:", 1)[0] + ".php", s.split(".php:", 1)[1]) if ".php:" in s else ("", s)
    if "\\" in short or not short: return None
    rows = db.execute("SELECT fqn FROM symbol WHERE (fqn=? OR fqn LIKE ?)" + (" AND file=?" if path else ""),
                      (short, "%\\" + short, *([path] if path else []))).fetchall()
    return rows[0][0] if len(rows) == 1 else None


def check_result(path, db):
    """Returns (mismatch_rate, rejected_items, n_findings)."""
    res = json.loads(path.read_text())["result"]
    findings = res.get("findings", [])
    total = bad = 0; rejected = []
    for v in res.get("verifications", []):   # pass 5: a verdict's quotes are checked like a finding's (no verdict without verbatim evidence)
        bad_ev = [e for e in v.get("evidence", []) if not quote_matches(e)]
        total += len(v.get("evidence", [])); bad += len(bad_ev)
        if bad_ev: rejected.append({"finding": {k: v.get(k) for k in ("finding_id", "claim_type", "verdict")}, "reason": "missing_evidence", "bad_evidence": bad_ev, "bad_symbols": []})
    for f in findings:
        bad_ev = [e for e in f.get("evidence", []) if not quote_matches(e)]
        bad_sym = [s for s in f.get("symbols", []) if resolve_symbol(db, s) is None]
        total += len(f.get("evidence", [])) + len(f.get("symbols", [])); bad += len(bad_ev) + len(bad_sym)
        if bad_ev or bad_sym:
            rejected.append({"finding": {k: f[k] for k in ("kind", "title", "symbols") if k in f}, "reason": "hallucinated_symbol" if bad_sym else "missing_evidence",
                             "bad_evidence": bad_ev, "bad_symbols": bad_sym})
    return (bad / total if total else 0.0), rejected, len(findings) + len(res.get("verifications", []))


MERGE_VERDICTS = {"duplicate_exact", "duplicate_semantic", "variant_mergeable"}


def verdict_gaps(path):
    """Pass 3: group_verdicts[] entries (or mixed subgroups) with a mergeable verdict but no finding naming the cluster_id.
    Observed with claude-sonnet-5-5 (c_0055b: 18 mergeable verdicts, 5 findings); verify.py reads findings only."""
    res = json.loads(path.read_text())["result"]
    cited = " ".join(f.get("rationale", "") + " " + f.get("title", "") for f in res.get("findings", []))
    gaps = []
    for g in res.get("group_verdicts", []):
        subs = [sg for sg in g.get("subgroups") or [] if sg.get("verdict") in MERGE_VERDICTS]
        if (g.get("verdict") in MERGE_VERDICTS or subs) and g.get("cluster_id") and g["cluster_id"] not in cited:
            gaps.append({"cluster_id": g["cluster_id"], "verdict": g.get("verdict"), "canonical_candidate": g.get("canonical_candidate"),
                         "subgroups": subs, "reason": "verdict_without_finding"})
    return gaps


def batch_lines(batch_path):
    b = json.loads(batch_path.read_text())
    members = list(b.get("group", [])) + [m for g in b.get("groups", []) for m in g.get("group", [])]  # pass 3 packs several groups per batch
    return sum(f["content"].count("\n") for f in b.get("files", [])) + sum((m.get("body") or "").count("\n") for m in members), int(len(json.dumps(b)) / 2.6)


def run_agy(pass_no, batch, extra):
    r = run([sys.executable, "-I", str(HERE / "run_agy.py"), "--pass", str(pass_no), "--batch", batch, *extra])
    return r.returncode, (r.stdout + r.stderr).strip()[-600:]


def drive(pass_no, batch):
    res_dir = OUT / "results" / f"pass{pass_no}"; res_dir.mkdir(parents=True, exist_ok=True)
    done_marker = res_dir / f"{batch}.verified"
    if done_marker.exists(): return "skip"
    db = sqlite3.connect(DB)
    batch_path = OUT / "batches" / f"pass{pass_no}" / f"{batch}.json"
    lines, tokens = batch_lines(batch_path)
    chosen = None
    # attempt 1 + push-backs
    for attempt in (1, 2, 3):
        suffix = "" if attempt == 1 else f"_pb{attempt}"
        out = res_dir / f"{batch}{suffix}.json"
        if not out.exists():
            extra = [] if attempt == 1 else ["--pushback", pb_reason, "--attempt", str(attempt), "--rejected-file", str(res_dir / f"{batch}_rejected{attempt - 1}.json")]
            rc, tail = run_agy(pass_no, batch, extra)
            while rc == 75:  # quota window exhausted – wait for the reset the worker reported, then run the same attempt again
                m = re.search(r"QUOTA_RESET_SECONDS=(\d+)", tail); wait = (int(m.group(1)) if m else 1800) + 90
                log(pass_no, f"{batch} attempt {attempt}: agy quota exhausted, waiting {wait}s")
                time.sleep(wait); rc, tail = run_agy(pass_no, batch, extra)
            log(pass_no, f"{batch} attempt {attempt} rc={rc} {tail.splitlines()[-1] if tail else ''}")
            if rc != 0 or not out.exists(): log(pass_no, f"{batch} FAILED attempt {attempt}: {tail}"); return "failed"
        rate, rejected, n = check_result(out, db)
        log(pass_no, f"{batch} attempt {attempt}: findings={n} mismatch={rate:.1%} rejected={len(rejected)}")
        if rate <= TAINT_RATE: chosen, chosen_attempt = out, attempt; break
        pb_reason = "hallucinated_symbol" if any(r["bad_symbols"] for r in rejected) else "missing_evidence"
        (res_dir / f"{batch}_rejected{attempt}.json").write_text(json.dumps(rejected, ensure_ascii=False, indent=1))
        log(pass_no, f"{batch} TAINTED ({rate:.1%}) → push-back {pb_reason}")
    if chosen is None:
        log(pass_no, f"{batch} still tainted after 3 attempts – left for the director"); return "tainted"
    results = [chosen]
    # mergeable group verdicts without a finding → one fill-in push-back; union of both answers is verified
    gaps = verdict_gaps(chosen) if pass_no == 3 else []
    if gaps:
        fill = res_dir / f"{batch}_pb{chosen_attempt + 1}.json"
        if not fill.exists():
            (res_dir / f"{batch}_gaps.json").write_text(json.dumps(gaps, ensure_ascii=False, indent=1))
            rc, tail = run_agy(pass_no, batch, ["--pushback", "verdict_without_finding", "--attempt", str(chosen_attempt + 1), "--rejected-file", str(res_dir / f"{batch}_gaps.json")])
            while rc == 75:
                m = re.search(r"QUOTA_RESET_SECONDS=(\d+)", tail); wait = (int(m.group(1)) if m else 1800) + 90
                log(pass_no, f"{batch} fill: quota exhausted, waiting {wait}s"); time.sleep(wait)
                rc, tail = run_agy(pass_no, batch, ["--pushback", "verdict_without_finding", "--attempt", str(chosen_attempt + 1), "--rejected-file", str(res_dir / f"{batch}_gaps.json")])
            log(pass_no, f"{batch} fill rc={rc} ({len(gaps)} mergeable verdicts without finding)")
        if fill.exists():
            rate3, _rej, n3 = check_result(fill, db); left = len(verdict_gaps(fill))
            log(pass_no, f"{batch} fill: findings={n3} mismatch={rate3:.1%} gaps_left={left}")
            if rate3 <= TAINT_RATE: results.append(fill)
    if pass_no in (5, 6):   # verifier: coverage instead of thinness – every claim needs a verdict; the uncovered ones stay at their status and are listed
        want = {c["finding_id"] for c in json.loads(batch_path.read_text()).get("claims", [])}
        got = {v.get("finding_id") for r in results for v in json.loads(r.read_text())["result"].get("verifications", [])}
        missing = sorted(want - got)
        if missing: (res_dir / f"{batch}_missing.json").write_text(json.dumps(missing))
        log(pass_no, f"{batch} coverage: claims={len(want)} verified={len(want & got)} missing={len(missing)}")
    # thinness → second look
    n = len(json.loads(chosen.read_text())["result"].get("findings", []))
    if pass_no not in (5, 6) and tokens > THIN_MIN_TOKENS and n < lines / THIN_LINES:
        look = res_dir / f"{batch}_look2.json"
        if not look.exists():
            if not (res_dir / f"{batch}_empty.json").exists(): (res_dir / f"{batch}_empty.json").write_text("[]")  # must exist before the run (rc=1 otherwise)
            rc, tail = run_agy(pass_no, batch, ["--shuffle", "--attempt", "2", "--pushback", "too_thin", "--rejected-file", str(res_dir / f"{batch}_empty.json")])
            log(pass_no, f"{batch} second look rc={rc} ({n} findings for {lines} lines)")
        if look.exists():
            rate2, _rej, n2 = check_result(look, db)
            log(pass_no, f"{batch} second look: findings={n2} mismatch={rate2:.1%}")
            if rate2 <= TAINT_RATE: results.append(look)
    with REGISTRY_LOCK:
        for r in results:
            v = run([sys.executable, "-I", str(HERE / "verify.py"), "--pass", str(pass_no), "--result", str(r)])
            log(pass_no, f"{batch} verify {r.name}: rc={v.returncode} {(v.stdout + v.stderr).strip()[-200:]}")
            if v.returncode != 0: return "verify_failed"
        run([sys.executable, "-I", str(HERE / "merge_registry.py"), "--status"])
        done_marker.write_text(json.dumps({"ts": datetime.datetime.now(datetime.UTC).isoformat(), "accepted": [r.name for r in results]}))
    return "ok"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pass", dest="pass_no", type=int, required=True)
    ap.add_argument("--batches", help="comma list; default = all batch files of the pass")
    ap.add_argument("--max-batches", type=int, default=0)
    ap.add_argument("--parallel", type=int, default=1, help="concurrent worker runs (quota permitting)")
    a = ap.parse_args()
    bdir = OUT / "batches" / f"pass{a.pass_no}"
    batches = a.batches.split(",") if a.batches else sorted(p.stem for p in bdir.glob("*.json") if not p.name.endswith(".prompt.md"))
    if a.max_batches: batches = batches[:a.max_batches]
    (OUT / "results" / f"pass{a.pass_no}").mkdir(parents=True, exist_ok=True)
    summary = {}
    def one(b):
        t0 = time.time(); r = drive(a.pass_no, b)
        log(a.pass_no, f"{b} → {r} in {time.time() - t0:.0f}s"); return b, r
    with concurrent.futures.ThreadPoolExecutor(max_workers=a.parallel) as ex:
        for b, r in ex.map(one, batches): summary[b] = r
    print(json.dumps(summary, indent=1))


if __name__ == "__main__":
    main()
