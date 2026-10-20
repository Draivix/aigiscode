#!/usr/bin/env python3
"""
run_agy.py – send one batch to the worker through the Antigravity CLI (`agy -p`) and store the raw answer.

Replaces run_gemini.py when the worker is driven through `agy` instead of the Gemini API
(operator decision 2026-10-18: Gemini 4 Argon High via agy, 1M context, permissions skipped).

Headless contract verified on agy 1.4.2 (2026-10-18):
  - stdin is NOT read in print mode and a single argv string is capped at 128 KB, so the prompt is
    written to <out>/batches/pass<N>/<batch>.prompt.md and `-p=` tells the worker to read it with the
    `view_whole_file` MCP tool (2 MB limit; view_file would truncate at 800 lines). Verified: a 1.45 MB
    batch arrived whole (322k context tokens) and was answered correctly in 3 minutes.
  - `--output-format json` prints {"conversation_id","status","response","structured_output","usage",...}
  - `--json-schema <file>` enforces the response schema; `structured_output` carries the parsed object
Safety: the worker runs with cwd = the audit worktree, whose source tree is chmod a-w; after
every run `git status` of the worktree is checked and any change is logged to INCIDENTS.md.
Never pass `--sandbox` together with `--dangerously-skip-permissions` (agy issue #36).

Usage:
  python3 run_agy.py --pass 2 --batch 0017
  python3 run_agy.py --pass 3 --batch c_00412 --pushback missing_evidence --attempt 2 --rejected-file out/results/pass3/c_00412_rejected.json
  python3 run_agy.py --selftest

Env:
  LEAN_REPO   worktree the worker reads (default: parent of LEAN_OUT)
  LEAN_OUT    output dir (default: <script>/../out)
  LEAN_MODEL  agy model id (default: gemini-4-argon-high)
  LEAN_RUNNER agy (default) | claude – headless Claude Code as the worker (LEAN_MODEL then is e.g. claude-opus-5-5,
              LEAN_CLAUDE_EFFORT high); same prompt, schema, checks and result format, billed to the Anthropic account
  LEAN_MAX_INPUT_TOKENS (default 450000)
"""
import argparse, re, datetime, hashlib, json, os, pathlib, subprocess, sys, time

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = pathlib.Path(os.environ.get("LEAN_OUT", ROOT / "out")).resolve()
REPO = pathlib.Path(os.environ.get("LEAN_REPO", OUT.parent.parent)).resolve()
MODEL = os.environ.get("LEAN_MODEL", "gemini-4-argon-high")
RUNNER = os.environ.get("LEAN_RUNNER", "agy")  # agy (default) | claude = headless Claude Code (`claude -p`, billed to the Anthropic account)
CLAUDE_EFFORT = os.environ.get("LEAN_CLAUDE_EFFORT", "high")
MAX_IN = int(os.environ.get("LEAN_MAX_INPUT_TOKENS", "450000"))
CHARS_PER_TOKEN = 2.6  # measured on batch 0000: 561k real input tokens for 1.3 MB of batch JSON (PHP tokenizes dense)
AGY_SYSTEM_OVERHEAD = 32_000  # measured: an empty prompt costs ~31k input tokens

PROMPTS, SCHEMAS = ROOT / "prompts", ROOT / "schemas"
PASS_PROMPT = {2: "pass2_vertical.md", 3: "pass3_horizontal.md", 4: "pass4_domain.md", 5: "verify_independent.md"}
PASS_SCHEMA = {2: "findings.schema.json", 3: "findings.schema.json", 4: "findings.schema.json", 5: "verification_batch.schema.json"}

TOOL_RULES = """
## Tool use in this session
You run inside the audited repository checkout. You MAY use read-only tools (view file, grep, list) to
verify a caller, a dynamic dispatch or a test before you report. You MUST NOT write, edit, create or
delete any file, run package managers, run git commands that change state, or run the application.
The checkout is read-only; any attempt is logged as an incident. The batch below already contains the
material you are expected to audit in full; tools are for verification, not for skipping the reading.

## Evidence quotes – the director checks every one mechanically
`quote` is 1 to 3 CONSECUTIVE lines copied character for character from the file (whitespace inside may differ,
nothing else). Never condense, never skip lines with "...", never join two places into one quote: a quote that is
not found verbatim rejects the whole finding and the batch is sent back. If the proof spans more lines, give several
evidence items of ≤ 3 lines each. `line_start`/`line_end` must be the real line numbers of those lines.
"""

INSTRUCTION = ("Read the whole file {prompt_file} with the view_whole_file tool (NOT view_file, which truncates). "
               "It contains the audit rules followed by a JSON batch. Follow the rules exactly and audit the batch. "
               "Output only JSON matching the required schema.")


def load(p): return pathlib.Path(p).read_text(encoding="utf-8")


def build_prompt(pass_no, batch, pushback=None, attempt=1, rejected=None):
    parts = [load(PROMPTS / "common_rules.md"), TOOL_RULES, "\n\n---\n\n", load(PROMPTS / PASS_PROMPT[pass_no])]
    if pushback:
        pb = load(PROMPTS / "push_back.md").replace("{{reason}}", pushback).replace("{{attempt}}", str(attempt))
        pb = pb.replace("{{rejected_items}}", json.dumps(rejected or [], ensure_ascii=False, indent=1))
        parts += ["\n\n---\n\n", pb]
    parts += ["\n\n---\n\n# BATCH INPUT (JSON)\n", json.dumps(batch, ensure_ascii=False)]
    return "".join(parts)


def strip_schema(schema):
    """agy's --json-schema rejects $schema/$ref; inline the simple local refs."""
    defs = schema
    def walk(node):
        if isinstance(node, dict):
            node = {k: v for k, v in node.items() if k not in ("$schema", "title", "$id")}
            if "$ref" in node:
                target = defs
                for p in node["$ref"].lstrip("#/").split("/"): target = target[p]
                return walk(target)
            return {k: walk(v) for k, v in node.items()}
        if isinstance(node, list): return [walk(x) for x in node]
        return node
    return walk(schema)


def schema_file(pass_no):
    dst = OUT / "tools" / f"agy-schema-pass{pass_no}.json"
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_text(json.dumps(strip_schema(json.loads(load(SCHEMAS / PASS_SCHEMA[pass_no])))))
    return dst


def worktree_dirty():
    r = subprocess.run(["git", "-C", str(REPO), "status", "--porcelain", "--", ".", ":!lean-audit", ":!.aigiscode"],
                       capture_output=True, text=True)
    return r.stdout.strip()


def incident(msg):
    OUT.mkdir(parents=True, exist_ok=True)
    with (OUT / "INCIDENTS.md").open("a", encoding="utf-8") as f:
        f.write(f"- {datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%S.%fZ")} {msg}\n")


def agy(prompt, schema, timeout_s, model, prompt_file=None):
    if RUNNER == "claude":  # same prompt and schema; the whole prompt goes in on stdin, read-only tools only
        cmd = ["claude", "-p", "--model", model, "--effort", CLAUDE_EFFORT, "--output-format", "json",
               "--json-schema", schema.read_text(), "--tools", "Read,Grep,Glob", "--max-turns", "60", "--permission-mode", "dontAsk"]
        env = {k: v for k, v in os.environ.items() if k != "CLAUDECODE"}
        t0 = time.time()
        r = subprocess.run(cmd, input=prompt, capture_output=True, text=True, timeout=timeout_s, cwd=REPO, env=env)
        return r, round(time.time() - t0, 1)
    instruction = INSTRUCTION.format(prompt_file=prompt_file) if prompt_file else prompt
    cmd = ["agy", f"-p={instruction}", "--model", model, "--output-format", "json",
           "--json-schema", str(schema), "--print-timeout", "0", "--dangerously-skip-permissions"]
    env = dict(os.environ, AGY_CLI_DISABLE_AUTO_UPDATE="1")
    t0 = time.time()
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout_s, cwd=REPO, env=env, stdin=subprocess.DEVNULL)
    return r, round(time.time() - t0, 1)


def parse_response(stdout):
    outer = json.loads(stdout)
    if RUNNER == "claude":  # normalise the claude -p envelope to the agy one the rest of the kit reads
        outer.setdefault("conversation_id", outer.get("session_id"))
        outer.setdefault("status", "ERROR" if outer.get("is_error") else "SUCCESS")
        outer["usage"] = dict(outer.get("usage") or {}, cost_usd=outer.get("total_cost_usd"))
        if outer.get("is_error"):
            msg = str(outer.get("result"))
            if re.search(r"rate.?limit|usage limit|limit reached|429|overloaded", msg, re.I):  # subscription window – wait like an agy quota
                m = re.search(r"resets? (?:in|at) (?:(\d+)h)?\s*(?:(\d+)m)?", msg, re.I)
                secs = sum(int(x or 0) * k for x, k in zip(m.groups(), (3600, 60))) if m and any(m.groups()) else 1800
                incident(f"claude -p usage limit: {msg[:200]} – waiting {secs}s"); print(f"QUOTA_RESET_SECONDS={secs}", file=sys.stderr); sys.exit(75)
            raise ValueError(f"claude -p error: {msg[:300]}")
        if not isinstance(outer.get("structured_output"), dict): outer["response"] = outer.get("result", "")
    if isinstance(outer.get("structured_output"), dict):
        return outer, outer["structured_output"]
    text = outer.get("response", "").strip()
    if text.startswith("```"):
        text = text.split("\n", 1)[1].rsplit("```", 1)[0]
    return outer, json.loads(text)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pass", dest="pass_no", type=int)
    ap.add_argument("--batch")
    ap.add_argument("--pushback", help="missing_evidence|too_thin|vague|skimmed|hallucinated_symbol")
    ap.add_argument("--attempt", type=int, default=1)
    ap.add_argument("--rejected-file")
    ap.add_argument("--shuffle", action="store_true", help="second look: shuffle file order")
    ap.add_argument("--model", default=MODEL)
    ap.add_argument("--timeout-min", type=int, default=45)
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()

    if a.selftest:
        s = OUT / "tools" / "agy-selftest-schema.json"; s.parent.mkdir(parents=True, exist_ok=True)
        s.write_text('{"type":"object","required":["ok","model"],"properties":{"ok":{"type":"boolean"},"model":{"type":"string"}}}')
        r, el = agy('Return {"ok": true, "model": "<your model id>"}', s, 300, a.model)
        print(r.stdout.strip() or r.stderr.strip(), f"({el}s, dirty={bool(worktree_dirty())})"); return

    if not (a.pass_no and a.batch): sys.exit("--pass and --batch are required")
    batch = json.loads(load(OUT / "batches" / f"pass{a.pass_no}" / f"{a.batch}.json"))
    if a.shuffle and batch.get("files"):
        import random; random.Random(a.attempt).shuffle(batch["files"])
    rejected = json.loads(load(a.rejected_file)) if a.rejected_file else None
    prompt = build_prompt(a.pass_no, batch, a.pushback, a.attempt, rejected)

    est = int(len(prompt) / CHARS_PER_TOKEN) + AGY_SYSTEM_OVERHEAD
    if est > MAX_IN:
        sys.exit(f"BATCH TOO BIG: ~{est} > {MAX_IN} tokens – split the batch")

    res_dir = OUT / "results" / f"pass{a.pass_no}"; res_dir.mkdir(parents=True, exist_ok=True)
    # second look (shuffled re-read) gets its own name: a `_pb{n}` from a push-back may already exist for the same attempt number
    suffix = f"_look{a.attempt}" if a.shuffle else (f"_pb{a.attempt}" if a.pushback else "")
    out_path = res_dir / f"{a.batch}{suffix}.json"
    if out_path.exists(): sys.exit(f"Result already exists, refusing to overwrite: {out_path}")

    if worktree_dirty():
        sys.exit(f"worktree dirty before run – resolve first:\n{worktree_dirty()}")

    schema = schema_file(a.pass_no)
    prompt_path = OUT / "batches" / f"pass{a.pass_no}" / f"{a.batch}{suffix}.prompt.md"
    prompt_path.write_text(prompt, encoding="utf-8")
    if prompt_path.stat().st_size > 2_000_000:
        sys.exit(f"PROMPT FILE TOO BIG for view_whole_file: {prompt_path.stat().st_size} bytes – split the batch")
    delay = 60
    for try_no in range(3):
        err = None
        try:
            r, elapsed = agy(prompt, schema, a.timeout_min * 60, a.model, prompt_file=str(prompt_path.relative_to(REPO)))
            dirty = worktree_dirty()
            if dirty:
                incident(f"pass{a.pass_no}/{a.batch} WORKER CHANGED THE TREE:\n{dirty}")
                sys.exit(f"INCIDENT: worker modified the worktree. See INCIDENTS.md. Restore the tree before continuing.")
            (res_dir / f"{a.batch}{suffix}.raw.txt").write_text(r.stdout + "\n--- stderr ---\n" + r.stderr)
            if r.returncode != 0 and RUNNER == "claude" and "usage_limit_reached" in r.stdout:  # claude -p exits 1 on a 429 envelope
                m = re.search(r"resets (\d{1,2})(?::(\d{2}))?\s*(am|pm)", r.stdout, re.I)
                secs = 1800
                if m:
                    import zoneinfo
                    now = datetime.datetime.now(zoneinfo.ZoneInfo("Europe/Prague")); h = int(m.group(1)) % 12 + (12 if m.group(3).lower() == "pm" else 0)
                    at = now.replace(hour=h, minute=int(m.group(2) or 0), second=0, microsecond=0)
                    if at <= now: at += datetime.timedelta(days=1)
                    secs = int((at - now).total_seconds())
                incident(f"pass{a.pass_no}/{a.batch}: claude -p usage limit ({re.sub(r'\s+', ' ', r.stdout)[-160:]}), resets in {secs}s")
                print(f"QUOTA_RESET_SECONDS={secs}", file=sys.stderr); sys.exit(75)
            if r.returncode != 0:
                err = f"agy exit {r.returncode}: {r.stderr.strip()[-400:]}"
                if "RESOURCE_EXHAUSTED" in r.stderr + r.stdout:  # 5-hour quota window; retrying inside the window is futile
                    m = re.search(r"Resets in (?:(\d+)h)?(?:(\d+)m)?(?:(\d+)s)?", r.stderr + r.stdout)
                    secs = sum(int(x or 0) * k for x, k in zip(m.groups(), (3600, 60, 1))) if m else 1800
                    incident(f"pass{a.pass_no}/{a.batch}: agy quota exhausted, resets in {secs}s")
                    print(f"QUOTA_RESET_SECONDS={secs}", file=sys.stderr); sys.exit(75)
            else:
                outer, data = parse_response(r.stdout)
                meta = {"model": a.model, "runner": RUNNER, "pass": a.pass_no, "batch": a.batch, "attempt": a.attempt,
                        "pushback": a.pushback, "input_tokens_est": est, "usage": outer.get("usage"),
                        "conversation_id": outer.get("conversation_id"), "status": outer.get("status"),
                        "elapsed_s": elapsed, "prompt_sha256": hashlib.sha256(prompt.encode()).hexdigest(),
                        "ts": datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%S.%fZ")}
                out_path.write_text(json.dumps({"meta": meta, "result": data}, ensure_ascii=False, indent=1), encoding="utf-8")
                print(f"OK {out_path} findings={len(data.get('findings', []))} est_in={est} "
                      f"real_in={(outer.get('usage') or {}).get('input_tokens')} {elapsed}s")
                return
        except subprocess.TimeoutExpired:
            err = f"timeout after {a.timeout_min} min"
        except FileNotFoundError as e:  # worker binary briefly missing (claude self-update replaced it mid-run, 2026-10-19 16:03 UTC)
            err = f"worker binary not found: {e}"
        except (json.JSONDecodeError, ValueError, KeyError) as e:
            err = f"invalid JSON from worker: {e}"
        print(f"try {try_no + 1} failed: {err}", file=sys.stderr)
        incident(f"pass{a.pass_no}/{a.batch} try{try_no + 1}: {err}")
        time.sleep(delay); delay *= 2
    sys.exit("FAILED after 3 tries")


if __name__ == "__main__":
    main()
