#!/usr/bin/env python3
"""
build_batches.py – builds the batches of one pass from symbols.sqlite + the aigiscode graph + clusters.json

  --pass 2  : entry point + transitive callee closure (vertical)
  --pass 3  : cluster + callers + tests (horizontal)
  --pass 4  : zone derived from file paths (domain)
  --check-coverage : only prints what would stay uncovered, writes nothing
  --split <batch_id> : splits an existing batch in two (when the runner reports TOO BIG)

NOTE: the graph comes from import_graph.py (callees.json / callers.json joined onto the symbol table), not from the raw
aigiscode artifacts; zones are derived from paths because repository-topology.json has only 6 coarse zones here.

Token estimate: 1 token ≈ 3.5 PHP characters (conservative).
"""
import argparse, json, pathlib, sqlite3, sys, collections, random

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / "out"
CHARS_PER_TOKEN = 2.6  # measured on batch 0000: 561k real input tokens for 1.3 MB of batch JSON (PHP tokenizes dense)
MAX_TOKENS = 380_000  # batch JSON only; run_agy.py adds rules + ~32k agy system overhead under the 450k input cap
SHARED_UTILITY_CALLER_THRESHOLD = 20   # a file reached from > N units is shown as signatures only (and audited in a hub batch)

def est_tokens(s): return int(len(s) / CHARS_PER_TOKEN)

def load_graph():
    """(callees, callers, entrypoints) from import_graph.py output (resolved aigiscode edges joined onto the symbol table)."""
    callees = {k: set(v) for k, v in json.loads((OUT / "graph/callees.json").read_text()).items()}
    callers = {k: set(v) for k, v in json.loads((OUT / "graph/callers.json").read_text()).items()}
    ep = json.loads((OUT / "graph/entrypoints.json").read_text())
    return collections.defaultdict(set, callees), collections.defaultdict(set, callers), ep

def zone_of(file):
    """Domain zone = module directory or second-level core namespace directory (aigiscode topology zones are too coarse here)."""
    parts = file.split("/")
    if len(parts) > 3 and parts[:2] == ["app", "Modules"]: return "/".join(parts[:3])
    if len(parts) > 2 and parts[0] == "app": return "/".join(parts[:2]) if parts[1] in ("Console", "Http", "Hooks", "Actions", "Support", "Providers") else "/".join(parts[:3])
    return parts[0]

def load_zones(db):
    zones = collections.defaultdict(list)
    for (f,) in db.execute("SELECT DISTINCT file FROM symbol"): zones[zone_of(f)].append(f)
    return zones

def symbol_rows(db, fqns):
    q = ",".join("?" * len(fqns))
    return db.execute(f"SELECT fqn,file,line_start,line_end,signature,body FROM symbol WHERE fqn IN ({q})", list(fqns)).fetchall()

def file_text(path):
    return pathlib.Path(path).read_text(encoding="utf-8", errors="replace")

def write_batch(pass_no, bid, batch):
    d = OUT / "batches" / f"pass{pass_no}"; d.mkdir(parents=True, exist_ok=True)
    p = d / f"{bid}.json"; p.write_text(json.dumps(batch, ensure_ascii=False))
    return p

def known_registry(files=None, sym_file=None):
    """Compact registry view for the prompt; with `files` only findings that touch a file of this batch (the full registry
    is tens of thousands of symbols and would eat the batch budget)."""
    reg = OUT / "registry.merged.jsonl" if (OUT / "registry.merged.jsonl").exists() else OUT / "registry.jsonl"; cap = OUT / "capabilities.jsonl"
    findings = [json.loads(l) for l in reg.read_text().splitlines() if l.strip()] if reg.exists() else []
    caps = [json.loads(l) for l in cap.read_text().splitlines() if l.strip()] if cap.exists() else []
    if files is not None and sym_file is not None:
        findings = [f for f in findings if any(sym_file.get(s) in files for s in f["symbols"])]
    return {"findings": [{"id": f["id"], "kind": f["kind"], "symbols": f["symbols"], "status": f["status"]} for f in findings if f["status"] != "rejected"],
            "capabilities": [{"capability_id": c["capability_id"], "name": c["name"], "implementations": c["implementations"]} for c in caps]}

# ---------- pass 2 ----------
def build_pass2(db, check_only):
    callees, callers, entrypoints = load_graph()
    sym_file = dict(db.execute("SELECT fqn, file FROM symbol"))
    file_tok = {f: est_tokens(file_text(f)) for f in set(sym_file.values())}
    # unit = all entry points of one class (a controller, a command, a hook file's service) → one closure
    units = collections.defaultdict(list)
    for ep in entrypoints: units[ep.rsplit("::", 1)[0] if "::" in ep else ep].append(ep)
    closures, reach_count = {}, collections.Counter()
    for u, eps in units.items():
        seen, stack = set(), list(eps)
        while stack:
            x = stack.pop()
            if x in seen: continue
            seen.add(x); stack.extend(callees.get(x, ()))
        closures[u] = seen
        for f in {sym_file[x] for x in seen if x in sym_file}: reach_count[f] += 1
    shared_limit = max(SHARED_UTILITY_CALLER_THRESHOLD, int(0.03 * len(units)))
    shared_files = {f for f, c in reach_count.items() if c > shared_limit}
    shared_sigs = [{"fqn": fqn, "signature": sig} for (fqn, sig) in db.execute("SELECT fqn, signature FROM symbol WHERE kind IN ('method','function')") if sym_file.get(fqn) in shared_files]

    base_tok = est_tokens(json.dumps(shared_sigs)) + 20_000
    budget = MAX_TOKENS - base_tok - 40_000  # JSON escaping and per-batch metadata overshoot the raw file estimate

    def unit_files(u):
        """Closure files in BFS depth order (entry class first), shared files removed; cut at the token budget.
        Returns (kept, truncated) – truncated files stay visible to the worker as paths + depth in batch_meta."""
        depth, frontier, d = {}, list(units[u]), 0
        while frontier:
            nxt = []
            for x in frontier:
                f = sym_file.get(x)
                if f and f not in shared_files and f not in depth: depth[f] = d
                nxt.extend(y for y in callees.get(x, ()) if y not in depth and sym_file.get(y) not in depth)
            frontier, d = list(set(nxt)), d + 1
            if d > 12: break
        kept, trunc, tok = [], [], 0
        for f in sorted(depth, key=lambda f: (depth[f], f)):
            if tok + file_tok[f] <= budget or not kept: kept.append(f); tok += file_tok[f]
            else: trunc.append({"path": f, "depth": depth[f]})
        return kept, trunc
    plan = {u: unit_files(u) for u in units}
    order = sorted(units, key=lambda u: -sum(file_tok[f] for f in plan[u][0]))
    batches, cur, cur_files, bid, covered = [], [], set(), 0, set()
    def flush():
        nonlocal cur, cur_files, bid
        if not cur: return
        eps = [ep for u in cur for ep in units[u]]
        batch = {"batch_meta": {"pass": 2, "batch_id": f"{bid:04d}", "entry_points": eps, "units": cur,
                                "truncated_files": {u: plan[u][1] for u in cur if plan[u][1]},
                                "description": "entry points (grouped by owning class) and their transitive callee closure in call-depth order; widely shared files appear as signatures only; files listed in truncated_files are part of the chain but did not fit the batch (audited in their own unit or hub batch)"},
                 "shared_signatures": shared_sigs,
                 "graph_context": {ep: {"direct_callees": sorted(x for x in callees.get(ep, ()) if x in sym_file), "closure_size": len(closures[u])} for u in cur for ep in units[u]},
                 "files": [{"path": f, "content": file_text(f)} for f in sorted(cur_files)], "known_registry": known_registry(cur_files, sym_file)}
        if not check_only: write_batch(2, f"{bid:04d}", batch)
        batches.append((f"{bid:04d}", len(eps), est_tokens(json.dumps(batch))))
        covered.update(eps); bid += 1; cur, cur_files = [], set()
    for u in order:
        uf = set(plan[u][0]); add = sum(file_tok[f] for f in uf - cur_files)
        if cur and base_tok + sum(file_tok[f] for f in cur_files) + add > MAX_TOKENS: flush()
        cur.append(u); cur_files |= uf
    flush()
    # hub batches: the shared files themselves, with their caller counts, so every file is read in full exactly once
    hub_files = sorted(shared_files, key=lambda f: (f.rsplit("/", 1)[0], f))
    caller_count = {fqn: len(callers.get(fqn, ())) for fqn in sym_file}
    i, hid = 0, 0
    while i < len(hub_files):
        chunk, tok = [], 0
        while i < len(hub_files) and (not chunk or tok + file_tok[hub_files[i]] <= MAX_TOKENS - 30_000):
            chunk.append(hub_files[i]); tok += file_tok[hub_files[i]]; i += 1
        batch = {"batch_meta": {"pass": 2, "batch_id": f"h{hid:03d}", "entry_points": [], "units": [], "hub": True,
                                "description": "hub files reached from many entry points (signatures were shown in every unit batch); audited here in full with the number of distinct callers per symbol"},
                 "shared_signatures": [], "graph_context": {fqn: caller_count[fqn] for fqn, f in sym_file.items() if f in set(chunk)},
                 "files": [{"path": f, "content": file_text(f)} for f in chunk], "known_registry": known_registry(set(chunk), sym_file)}
        if not check_only: write_batch(2, f"h{hid:03d}", batch)
        batches.append((f"h{hid:03d}", 0, est_tokens(json.dumps(batch)))); hid += 1
    missing = [ep for ep in entrypoints if ep not in covered]
    big = [b for b in batches if b[2] > MAX_TOKENS]
    truncated_units = sum(1 for u in units if plan[u][1])
    print(f"pass2: batches={len(batches)} (hub {hid}) units={len(units)} truncated_units={truncated_units} entrypoints={len(entrypoints)} uncovered={len(missing)} shared_files={len(shared_files)} (limit {shared_limit} units) over_limit={len(big)}")
    for b in big[:10]: print("  OVER", b)
    for m in missing[:20]: print("  UNCOVERED", m)
    return missing

# ---------- pass 3 ----------
def build_pass3(db, check_only):
    """Many clusters per batch (one agy call costs ~3 min + 32k overhead, so 2 088 single-cluster calls are not an option).
    Each cluster keeps its own members, callers, tests and similarity edges under groups[]; the worker answers per group."""
    cl = json.loads((OUT / "clusters.json").read_text())
    _callees, callers, _ep = load_graph()
    sym_file = dict(db.execute("SELECT fqn, file FROM symbol"))
    test_rows = db.execute("SELECT fqn, file FROM symbol WHERE file LIKE '%/tests/%' OR file LIKE '%Test.php'").fetchall()
    groups = []
    for c in cl["clusters"]:
        fqns = [m["fqn"] for m in c["members"]]
        # members by symbol id, never by fqn: anonymous-class methods share one fqn (`anon::up` has 1 239 rows), and the first
        # build expanded 30-member clusters into 2 500-member, 1.2 M-token groups that the runner refused
        q = ",".join("?" * len(c["members"]))
        rows = db.execute(f"SELECT fqn,file,line_start,line_end,signature,body FROM symbol WHERE id IN ({q})", [m["id"] for m in c["members"]]).fetchall()
        members = [{"fqn": fqn, "file": f, "line_start": a, "line_end": b, "signature": sig, "body": body} for (fqn, f, a, b, sig, body) in rows]
        caller_fqns = set().union(*(callers.get(f, set()) for f in fqns))
        caller_rows = [{"fqn": fqn, "file": f, "signature": sig, "context": (body or "")[:600]} for (fqn, f, _a, _b, sig, body) in symbol_rows(db, list(caller_fqns)[:60])[:60]] if caller_fqns else []
        shorts = {m.split("::")[0].split("\\")[-1] for m in fqns}
        tests = [{"fqn": fqn, "file": f} for (fqn, f) in test_rows if any(sh in f for sh in shorts)][:20]
        g = {"cluster_id": c["cluster_id"], "similarity_edges": c["edges"], "group": members, "callers": caller_rows, "callers_truncated": max(0, len(caller_fqns) - 60),
             "tests": tests, "graph_context": {f: len(callers.get(f, ())) for f in fqns}}
        if est_tokens(json.dumps(g)) > MAX_TOKENS - 30_000:
            for r in g["callers"]: r["context"] = r["context"][:150]
        if est_tokens(json.dumps(g)) > MAX_TOKENS - 30_000:   # still too big: very long members – keep the head of each body
            for m in g["group"]:
                if len(m["body"] or "") > 6000: m["body"], m["body_truncated"] = m["body"][:6000], True
        groups.append(g)
    n_batches, cur, cur_tok, cur_files = 0, [], 0, set()
    def flush():
        nonlocal n_batches, cur, cur_tok, cur_files
        if not cur: return
        batch = {"batch_meta": {"pass": 3, "batch_id": f"c_{n_batches:04d}", "clusters": [g["cluster_id"] for g in cur],
                                "description": f"{len(cur)} semantic neighbour groups (no required call edge between members); answer every group, key findings by cluster_id in rationale"},
                 "groups": cur, "shared_signatures": [], "files": [], "graph_context": {}, "known_registry": known_registry(cur_files, sym_file)}
        if not check_only: write_batch(3, f"c_{n_batches:04d}", batch)
        n_batches += 1; cur, cur_tok, cur_files = [], 0, set()
    for g in groups:
        t = est_tokens(json.dumps(g))
        if cur and cur_tok + t > MAX_TOKENS - 30_000: flush()
        cur.append(g); cur_tok += t; cur_files |= {m["file"] for m in g["group"]}
    flush()
    # singletons in chunks of 200, signatures + 10 lines only – question: "does it have a twin in the registry?"
    ids = cl["singleton_ids"]
    for i in range(0, len(ids), 200):
        chunk = ids[i:i+200]
        q = ",".join("?" * len(chunk))
        rows = db.execute(f"SELECT fqn,file,signature,substr(body,1,600) FROM symbol WHERE id IN ({q})", chunk).fetchall()
        batch = {"batch_meta": {"pass": 3, "batch_id": f"s_{i//200:04d}", "description": "singletons: answer only has_twin_in_registry"},
                 "groups": [], "singletons": [{"fqn": a, "file": b, "signature": c, "head": d} for (a, b, c, d) in rows],
                 "shared_signatures": [], "files": [], "graph_context": {}, "known_registry": known_registry({b for (_a, b, _c, _d) in rows}, sym_file)}
        if not check_only: write_batch(3, f"s_{i//200:04d}", batch)
    print(f"pass3: clusters={len(groups)} cluster batches={n_batches} singleton batches={(len(ids)+199)//200}")
    return []

# ---------- pass 4 ----------
def build_pass4(db, check_only):
    zones = load_zones(db)
    sym_file = dict(db.execute("SELECT fqn, file FROM symbol"))  # known_registry filtered to the batch's files: the full view is ~1 M tokens after pass 3
    bid = 0; uncovered_files = []
    all_files = {r[0] for r in db.execute("SELECT DISTINCT file FROM symbol")}
    zoned = set()
    for name, files in zones.items():
        files = [f for f in files if f.endswith(".php")]
        zoned.update(files)
        cur, cur_tok, part = [], 0, 0
        def flush():
            nonlocal cur, cur_tok, part, bid
            if not cur: return
            batch = {"batch_meta": {"pass": 4, "batch_id": f"{bid:04d}", "zone": name, "part": part, "description": "all code of one domain zone"},
                     "files": [{"path": f, "content": file_text(f)} for f in cur], "shared_signatures": [], "graph_context": {},
                     "known_registry": known_registry(set(cur), sym_file)}
            if not check_only: write_batch(4, f"{bid:04d}", batch)
            bid += 1; part += 1; cur, cur_tok = [], 0
        for f in files:
            t = est_tokens(file_text(f))
            if cur and cur_tok + t > MAX_TOKENS - 40_000: flush()  # headroom for the filtered known_registry
            cur.append(f); cur_tok += t
        flush()
    uncovered_files = sorted(all_files - zoned)
    print(f"pass4: batches={bid} zones={len(zones)} files_not_in_any_zone={len(uncovered_files)}")
    if uncovered_files:
        (OUT / "batches/pass4_unzoned_files.txt").write_text("\n".join(uncovered_files))
        print("  → unzoned files written to out/batches/pass4_unzoned_files.txt; build an extra zone 'unzoned' by namespace")
    return uncovered_files

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pass", dest="pass_no", type=int, required=True)
    ap.add_argument("--check-coverage", action="store_true")
    ap.add_argument("--db", default=str(OUT / "symbols.sqlite"))
    a = ap.parse_args()
    db = sqlite3.connect(a.db)
    {2: build_pass2, 3: build_pass3, 4: build_pass4}[a.pass_no](db, a.check_coverage)

if __name__ == "__main__":
    main()
