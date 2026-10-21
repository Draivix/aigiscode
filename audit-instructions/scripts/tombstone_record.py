#!/usr/bin/env python3
"""Record tombstone events in the append-only registry.

  python3 tombstone_record.py --inserted <inserter report.json> --mr <iid> [--commit <sha>]
  python3 tombstone_record.py --branch-lane <dir with tb_*.json> --mr <iid> [--commit <sha>]
  python3 tombstone_record.py --hits <tombstone:report --json output> --instance <domain> [--collected <date>]

`--inserted` / `--branch-lane` append one record per finding with a `tombstone` verification
entry (state `inserted`, MR, site) and a history line; the status does not change.
`--hits` marks every finding whose tombstone fired as `rejected` (alive in production), with the
tenants and callers the report saw. Rows are matched by finding id; a `#F-id` suffix on a branch
tombstone symbol names the finding directly, a plain symbol is matched through its targets row.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

OUT = Path(__file__).resolve().parents[1] / "out"
REG = OUT / "registry.jsonl"
MERGED = OUT / "registry.merged.jsonl"
TARGETS = OUT / "tombstones" / "targets.jsonl"


def now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def latest() -> dict[str, dict]:
    by_id: dict[str, dict] = {}
    src = MERGED if MERGED.exists() else REG
    for line in open(src):
        if line.strip():
            r = json.loads(line)
            by_id[r["id"]] = r
    return by_id


def append(rec: dict) -> None:
    with open(REG, "a") as f:
        f.write(json.dumps(rec, ensure_ascii=False) + "\n")


def record_inserted(by_id: dict[str, dict], placements: dict[str, str], mr: str, commit: str | None, lane: str) -> int:
    n = 0
    for fid, site in placements.items():
        f = by_id.get(fid)
        if f is None:
            print(f"unknown finding {fid}", file=sys.stderr)
            continue
        check = {"check": "tombstone", "state": "inserted", "lane": lane, "mr": mr, "site": site, "ts": now()}
        if commit:
            check["commit"] = commit
        rec = dict(f, verification=f.get("verification", []) + [check], source=f"tombstone/{mr}",
                   history=f.get("history", []) + [{"ts": now(), "status": f["status"], "by": "tombstone_record.py", "event": "tombstone_inserted", "mr": mr}])
        append(rec)
        by_id[fid] = rec
        n += 1
    return n


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--inserted")
    ap.add_argument("--branch-lane")
    ap.add_argument("--hits")
    ap.add_argument("--mr")
    ap.add_argument("--commit")
    ap.add_argument("--instance")
    ap.add_argument("--collected")
    a = ap.parse_args()
    by_id = latest()

    if a.inserted:
        if not a.mr:
            sys.exit("--mr required")
        placements: dict[str, str] = {}
        for r in json.load(open(a.inserted)):
            if r.get("action") == "inserted":
                placements.setdefault(r["finding_id"], f"{r['file']}:{r.get('line')}")
        print("symbol lane recorded:", record_inserted(by_id, placements, a.mr, a.commit, "symbol"))

    if a.branch_lane:
        if not a.mr:
            sys.exit("--mr required")
        placements = {}
        for p in sorted(Path(a.branch_lane).glob("tb_*.json")):
            if not re.fullmatch(r"tb_\d{4}\.json", p.name):
                continue
            placements.update(json.load(open(p)).get("placed", {}))
        print("branch lane recorded:", record_inserted(by_id, placements, a.mr, a.commit, "branch"))

    if a.hits:
        if not a.instance:
            sys.exit("--instance required")
        symbol_to_fid: dict[str, str] = {}
        if TARGETS.exists():
            for line in open(TARGETS):
                t = json.loads(line)
                symbol_to_fid.setdefault(t["symbol"], t["finding_id"])
        n = 0
        for h in json.load(open(a.hits)):
            sym = h["symbol"]
            m = re.search(r"#(F-[0-9a-f]+)$", sym)
            fid = m.group(1) if m else symbol_to_fid.get(sym)
            f = by_id.get(fid) if fid else None
            if f is None:
                print(f"hit without a finding: {sym}", file=sys.stderr)
                continue
            check = {"check": "tombstone", "state": "hit", "instance": a.instance, "collected": a.collected or now()[:10],
                     "hits": h.get("hits"), "first": h.get("first"), "last": h.get("last"),
                     "tenants": sorted((h.get("tenants") or {}).keys()), "callers": sorted((h.get("callers") or {}).keys())[:10]}
            rec = dict(f, status="rejected", verification=f.get("verification", []) + [check], source=f"tombstone/hits/{a.instance}",
                       history=f.get("history", []) + [{"ts": now(), "status": "rejected", "by": "tombstone_record.py", "event": "tombstone_hit", "instance": a.instance}])
            append(rec)
            by_id[fid] = rec
            n += 1
        print("hits recorded (findings rejected):", n)
    print("now run: python3 merge_registry.py && python3 merge_registry.py --status")


if __name__ == "__main__":
    main()
