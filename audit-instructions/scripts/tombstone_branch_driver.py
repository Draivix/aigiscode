#!/usr/bin/env python3
"""Sonnet lane for dead_branch tombstones.

  python3 tombstone_branch_driver.py --root <draivix worktree> --branches branches.jsonl \
      --out <dir> [--filter <regex on group>] [--parallel 2] [--model claude-sonnet-5-5] [--effort high]

A dead_symbol gets its tombstone from the deterministic inserter; a dead_branch needs a reader
who finds the unreachable block from the evidence quote (line numbers drift) and puts
`\\App\\Support\\Tombstone::hit(__METHOD__.'#F-id'); // lean-audit F-id` as its first statement.
The worker is headless Claude Code with Read/Grep/Glob/Edit on the worktree. The director
verifies every batch mechanically: the diff of the batch's files may contain only added lines of
exactly that shape, no removed lines, `php -l` must pass; a batch that fails is reverted
(`git checkout -- <files>`) and retried once with the verifier's complaint, then left for a human.
Batches never share a file, so two workers can edit the same worktree.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import threading
import time
from collections import defaultdict
from pathlib import Path

LINE_RE = re.compile(r"^\+\s*\\App\\Support\\Tombstone::hit\(__METHOD__\s*\.\s*'#(F-[0-9a-f]+)'\); // lean-audit (F-[0-9a-f]+)\s*$")
MAX_PER_BATCH = 8
RUN_TIMEOUT_S = 1800
SCHEMA = {
    "type": "object",
    "properties": {
        "results": {
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "finding_id": {"type": "string"},
                    "action": {"type": "string", "enum": ["inserted", "skipped"]},
                    "file": {"type": "string"},
                    "line": {"type": "integer"},
                    "reason": {"type": "string"},
                },
                "required": ["finding_id", "action", "file", "reason"],
            },
        }
    },
    "required": ["results"],
}

BRIEF = """You insert runtime tombstones into code branches that a lean audit marked unreachable. You
work in a git worktree of the Draivix repository (Laravel, PHP 8.4). You change nothing except the
insertions described here.

For each finding below:
1. Open the file named in its evidence. Line numbers may have drifted; locate the code by the
   verbatim quote.
2. Identify the dead block: the branch the rationale and the verifier conditions call unreachable
   (an `if`/`elseif`/`else` body, a `case`/`default` body, a `catch` body, a loop body, the second of two
   duplicate guards, a code block after an unconditional return/throw). Only that block.
3. Insert, as the FIRST statement inside that block, on its own line with the block's indentation:
       \\App\\Support\\Tombstone::hit(__METHOD__.'#<finding_id>'); // lean-audit <finding_id>
   (replace <finding_id> twice with the finding id, e.g. F-14cb49ca95).
4. Exactly one inserted line per finding. If the finding names several dead spans, tombstone only
   the first one and say so in `reason`.

Skip (action `skipped`, with a precise `reason`) when: the quoted code is not in the file
(`not_found`); the dead thing is not a statement block (ternary, `match` arm, arrow function,
expression, a whole method or class, a parameter) (`not_a_block`); the block already contains a
`Tombstone::hit(` (`already`); or the block is code after `return`/`throw` with no braces to hold a
statement (`unreachable_tail`). Never delete, move or reformat any existing line; never add a `use`
statement; never touch another file; never insert into a method that is not the one in the evidence.

Report every finding in the JSON result, inserted or skipped, with the file and the line number of
the inserted statement.

Findings:
"""


def sh(cmd: list[str], cwd: str, timeout: int | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=cwd, text=True, capture_output=True, timeout=timeout)


def group_of(rel: str) -> str:
    m = re.match(r"app/Modules/([^/]+)/", rel)
    if m:
        return "module-" + m.group(1)
    m = re.match(r"app/([^/]+)/", rel)
    if m:
        return "core-" + m.group(1)
    return "other-" + rel.split("/")[0]


def usage_limit_wait_s(text: str, default: int = 900) -> int:
    m = re.search(r"resets? (\d{1,2})(?::(\d{2}))?\s*(am|pm)", text, re.I)
    if not m:
        return default
    h, mi, ap = int(m.group(1)), int(m.group(2) or 0), m.group(3).lower()
    h = h % 12 + (12 if ap == "pm" else 0)
    now = time.localtime()
    target = time.mktime((now.tm_year, now.tm_mon, now.tm_mday, h, mi, 0, 0, 0, -1))
    if target < time.time():
        target += 86400
    return max(60, min(int(target - time.time()) + 60, 6 * 3600))


def build_batches(rows: list[dict], flt: re.Pattern | None) -> list[dict]:
    by_file: dict[str, list[dict]] = defaultdict(list)
    for r in rows:
        ev = r.get("evidence") or []
        rel = ev[0]["file"] if ev and isinstance(ev[0], dict) and ev[0].get("file") else None
        if not rel:
            continue
        if flt and not flt.search(group_of(rel)):
            continue
        by_file[rel].append(r)
    batches: list[dict] = []
    by_group: dict[str, list[tuple[str, list[dict]]]] = defaultdict(list)
    for rel, rs in sorted(by_file.items()):
        by_group[group_of(rel)].append((rel, rs))
    for group, files in sorted(by_group.items()):
        cur: list[dict] = []
        cur_files: list[str] = []
        for rel, rs in files:
            if cur and len(cur) + len(rs) > MAX_PER_BATCH:
                batches.append({"group": group, "files": cur_files, "findings": cur})
                cur, cur_files = [], []
            cur.extend(rs)
            cur_files.append(rel)
        if cur:
            batches.append({"group": group, "files": cur_files, "findings": cur})
    for i, b in enumerate(batches):
        b["id"] = f"tb_{i:04d}"
    return batches


def verify(root: str, files: list[str], expected_ids: set[str]) -> tuple[bool, str, dict[str, str]]:
    """Return (ok, complaint, {finding_id: 'file:line'}) for the diff of the batch's files."""
    diff = sh(["git", "diff", "--unified=0", "--", *files], root).stdout
    placed: dict[str, str] = {}
    cur_file = None
    cur_line = 0
    for ln in diff.splitlines():
        if ln.startswith("+++ b/"):
            cur_file = ln[6:]
            continue
        if ln.startswith(("---", "diff ", "index ")):
            continue
        if ln.startswith("@@"):
            m = re.search(r"\+(\d+)", ln)
            cur_line = int(m.group(1)) if m else 0
            continue
        if ln.startswith("-"):
            return False, f"removed/modified line in {cur_file}: {ln[:120]}", {}
        if ln.startswith("+"):
            m = LINE_RE.match(ln)
            if not m or m.group(1) != m.group(2):
                return False, f"added line is not a tombstone statement in {cur_file}: {ln[:160]}", {}
            if m.group(1) not in expected_ids:
                return False, f"tombstone for a finding not in this batch: {m.group(1)}", {}
            if m.group(1) in placed:
                return False, f"two tombstones for {m.group(1)}", {}
            placed[m.group(1)] = f"{cur_file}:{cur_line}"
            cur_line += 1
    for f in files:
        if sh(["git", "diff", "--quiet", "--", f], root).returncode != 0:
            r = sh(["php", "-l", f], root)
            if r.returncode != 0:
                return False, f"php -l failed for {f}: {r.stdout.strip()[:200]}", {}
    return True, "", placed


def run_batch(b: dict, args: argparse.Namespace, out: Path, log) -> None:
    root = args.root
    expected = {f["finding_id"] for f in b["findings"]}
    result_file = out / f"{b['id']}.json"
    if result_file.exists():
        log(f"{b['id']} already done")
        return
    complaint = ""
    for attempt in (1, 2):
        findings = [{k: f.get(k) for k in ("finding_id", "title", "evidence", "rationale", "conditions")} for f in b["findings"]]
        prompt = BRIEF + json.dumps(findings, ensure_ascii=False, indent=1)
        if complaint:
            prompt += f"\n\nYour previous attempt was reverted by the verifier: {complaint}\nDo it again within the rules above."
        cmd = ["env", "-u", "CLAUDECODE", "claude", "-p", prompt, "--model", args.model, "--effort", args.effort,
               "--output-format", "json", "--json-schema", json.dumps(SCHEMA), "--tools", "Read,Grep,Glob,Edit",
               "--allowedTools", "Read,Grep,Glob,Edit", "--permission-mode", "acceptEdits", "--max-turns", "80"]
        t0 = time.time()
        while True:
            try:
                r = sh(cmd, root, timeout=RUN_TIMEOUT_S)
            except subprocess.TimeoutExpired:
                log(f"{b['id']} attempt {attempt}: timeout")
                r = None
                break
            if "usage_limit_reached" in (r.stdout + r.stderr) or "limit reset" in (r.stdout + r.stderr).lower():
                w = usage_limit_wait_s(r.stdout + r.stderr)
                log(f"{b['id']} usage limit, waiting {w}s")
                time.sleep(w)
                continue
            break
        answer = None
        if r is not None and r.returncode == 0:
            try:
                env = json.loads(r.stdout)
                answer = env.get("structured_output") or json.loads(env.get("result") or "{}")
            except (json.JSONDecodeError, TypeError):
                answer = None
        ok, complaint, placed = verify(root, b["files"], expected)
        log(f"{b['id']} attempt {attempt}: {len(placed)} placed, ok={ok} {complaint[:100]} ({int(time.time() - t0)}s)")
        if ok:
            result_file.write_text(json.dumps({"batch": b["id"], "group": b["group"], "files": b["files"],
                                               "placed": placed, "worker": answer, "attempt": attempt,
                                               "seconds": int(time.time() - t0)}, ensure_ascii=False, indent=1))
            return
        sh(["git", "checkout", "--", *b["files"]], root)
    (out / f"{b['id']}.failed.json").write_text(json.dumps({"batch": b["id"], "files": b["files"], "complaint": complaint,
                                                            "findings": sorted(expected)}, indent=1))


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--branches", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--filter")
    ap.add_argument("--parallel", type=int, default=2)
    ap.add_argument("--model", default="claude-sonnet-5-5")
    ap.add_argument("--effort", default="high")
    ap.add_argument("--plan", action="store_true", help="print the batches and exit")
    args = ap.parse_args()
    rows = [json.loads(l) for l in open(args.branches) if l.strip()]
    batches = build_batches(rows, re.compile(args.filter) if args.filter else None)
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    (out / "batches.json").write_text(json.dumps(batches, ensure_ascii=False, indent=1))
    if args.plan:
        for b in batches:
            print(b["id"], b["group"], len(b["findings"]), "findings", len(b["files"]), "files")
        print(len(batches), "batches", sum(len(b["findings"]) for b in batches), "findings")
        return
    lock = threading.Lock()
    logf = open(out / "driver.log", "a")

    def log(msg: str) -> None:
        with lock:
            line = f"{time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())} {msg}"
            print(line, flush=True)
            logf.write(line + "\n")
            logf.flush()

    queue = list(batches)
    qlock = threading.Lock()

    def worker() -> None:
        while True:
            with qlock:
                if not queue:
                    return
                b = queue.pop(0)
            run_batch(b, args, out, log)

    threads = [threading.Thread(target=worker, daemon=True) for _ in range(max(1, args.parallel))]
    for t in threads:
        t.start()
        time.sleep(5)
    for t in threads:
        t.join()
    done = len([p for p in out.glob("tb_*.json") if re.fullmatch(r"tb_\d{4}\.json", p.name)])
    failed = len(list(out.glob("*.failed.json")))
    log(f"finished: {done} batches ok, {failed} failed")


if __name__ == "__main__":
    main()
