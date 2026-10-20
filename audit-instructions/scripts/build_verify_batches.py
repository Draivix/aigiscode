#!/usr/bin/env python3
"""
build_verify_batches.py – pass 5 input: independent-verifier batches for findings that would lead to a delete or a merge.

  python3 build_verify_batches.py --db lean-audit/out/symbols.sqlite --out lean-audit/out [--statuses candidate,confirmed] [--per-batch 40]

For every registry.merged.jsonl finding with kind in MERGE_KINDS (claim `merge`) or DELETE_KINDS (claim `delete`) and a
status in --statuses, the batch carries ONLY: the claim type, the symbols with full bodies, every known caller (bodies,
from graph/callers.json), tests whose file mentions the class short name, and for delete claims the deterministic
reachability evidence. The original rationale is deliberately NOT included (DIRECTOR §6.6).
"""
import argparse, collections, json, pathlib, re, sqlite3

MERGE_KINDS = {"clone_exact", "clone_semantic", "variant_mergeable", "double_path", "redundant_layer"}
DELETE_KINDS = {"dead_symbol", "dead_branch", "unused_param", "unused_dependency"}
CHARS_PER_TOKEN, MAX_TOKENS = 2.6, 380_000  # measured on real agy runs (pass 2): PHP batch JSON ≈ 2.6 chars/token


def est(s): return int(len(s) / CHARS_PER_TOKEN)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--db", required=True); ap.add_argument("--out", required=True)
    ap.add_argument("--statuses", default="candidate,confirmed"); ap.add_argument("--per-batch", type=int, default=40)
    ap.add_argument("--kinds", default=",".join(sorted(MERGE_KINDS | DELETE_KINDS)))
    ap.add_argument("--pass-no", type=int, default=5, help="5 = first independent verifier; 6 = second verifier (agy) over the same claim set")
    ap.add_argument("--prefix", default="v")
    a = ap.parse_args()
    out = pathlib.Path(a.out); db = sqlite3.connect(a.db)
    callers = json.loads((out / "graph/callers.json").read_text()) if (out / "graph/callers.json").exists() else {}
    tests_by_short = collections.defaultdict(list)
    for p in pathlib.Path("tests").rglob("*.php") if pathlib.Path("tests").exists() else []:
        txt = p.read_text(encoding="utf-8", errors="replace")
        for short in set(re.findall(r"\b([A-Z][A-Za-z0-9]+)(?:::class|\()", txt)): tests_by_short[short].append(str(p))
    for p in pathlib.Path("app").rglob("*Test.php"):
        txt = p.read_text(encoding="utf-8", errors="replace")
        for short in set(re.findall(r"\b([A-Z][A-Za-z0-9]+)(?:::class|\()", txt)): tests_by_short[short].append(str(p))

    def sym(fqn):
        r = db.execute("SELECT fqn,file,line_start,line_end,signature,body,reachable_from_entry,dead_code_detector,unused_public,coverage_pct,name_refs FROM symbol WHERE fqn=?", (fqn,)).fetchone()
        if not r: return None
        k = ["fqn", "file", "line_start", "line_end", "signature", "body", "reachable_from_entry", "dead_code_detector", "unused_public", "coverage_pct", "name_refs"]
        return dict(zip(k, r))

    wanted_statuses, wanted_kinds = set(a.statuses.split(",")), set(a.kinds.split(","))
    triage = {}   # triage layer (triage_driver.py): a finding both reviewers rejected is not sent to the verifier
    tp = out / "triage/triage.jsonl"
    if tp.exists():
        for line in tp.read_text().splitlines():
            if line.strip(): t = json.loads(line); triage[t["id"]] = t
    skipped_triage = 0
    claims = []
    for line in (out / "registry.merged.jsonl").read_text().splitlines():
        f = json.loads(line)
        if f["status"] not in wanted_statuses or f["kind"] not in wanted_kinds: continue
        if triage.get(f["id"], {}).get("verdict") == "reject": skipped_triage += 1; continue
        if f["id"] in triage: f["triage"] = {k: triage[f["id"]][k] for k in ("verdict", "difficulty", "severity", "reason")}
        if f["kind"] == "unused_dependency": continue  # composer packages: judged by the owner, not by code reading
        claim = "merge" if f["kind"] in MERGE_KINDS else "delete"
        symbols = [s for s in (sym(x) for x in f["symbols"]) if s]
        if not symbols: continue
        caller_fqns = sorted({c for x in f["symbols"] for c in callers.get(x, [])})
        caller_rows = [s for s in (sym(c) for c in caller_fqns[:40]) if s]
        shorts = {x.rsplit("::", 1)[0].rsplit("\\", 1)[-1] for x in f["symbols"]}
        tests = sorted({t for s in shorts for t in tests_by_short.get(s, [])})[:20]
        c = {"finding_id": f["id"], "claim_type": claim, "kind": f["kind"], "symbols": symbols, "callers": caller_rows,
             "callers_truncated": max(0, len(caller_fqns) - 40), "tests": tests,
             "canonical_candidate": f.get("canonical_candidate"), "behavior_differences_claimed": f.get("behavior_differences", []),
             "triage": f.get("triage")}
        if claim == "delete":
            c["reachability_evidence"] = [v for v in f.get("verification", []) if isinstance(v, dict)]
        claims.append(c)

    d = out / f"batches/pass{a.pass_no}"; d.mkdir(parents=True, exist_ok=True)
    bid, cur, cur_tok, n = 0, [], 0, 0
    def flush():
        nonlocal bid, cur, cur_tok, n
        if not cur: return
        batch = {"batch_meta": {"pass": a.pass_no, "batch_id": f"{a.prefix}_{bid:04d}", "description": "independent verification of merge/delete claims; verify EVERY claim in claims[] and return one verification per finding_id"},
                 "claims": cur, "known_registry": {}}
        (d / f"{a.prefix}_{bid:04d}.json").write_text(json.dumps(batch, ensure_ascii=False)); bid += 1; n += len(cur); cur, cur_tok = [], 0
    for c in claims:
        t = est(json.dumps(c))
        if cur and (len(cur) >= a.per_batch or cur_tok + t > MAX_TOKENS): flush()
        cur.append(c); cur_tok += t
    flush()
    print(f"pass{a.pass_no}: claims={len(claims)} batches={bid} skipped_triage_rejected={skipped_triage}")


if __name__ == "__main__":
    main()
