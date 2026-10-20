#!/usr/bin/env python3
"""
import_graph.py – join the aigiscode evidence graph onto symbols.sqlite and compute reachability.

Inputs (cwd = audited repo root):
  <graph>/evidence-graph.json   resolved method-level edges (relation_kind call|dispatch|containerresolution|overrides)
  <graph>/semantic-graph.json   raw Call references (by name) – used for the "name is referenced somewhere" signal
  <graph>/entrypoints.json + entrypoints.convention.json   reachability roots

Writes:
  symbol.reachable_from_entry   1 = reached from a root over resolved edges, 0 = in graph but not reached, NULL = not in graph
  symbol.name_refs              number of Call references anywhere whose target_name equals the method/function name
  <graph>/callees.json, callers.json   fqn → [fqn] over resolved edges (build_batches.py reads these)
  <graph>/graph-join.json       join statistics (how many graph nodes mapped onto table rows)

Nodes join on (file_path, start_line); the symbol table was extracted with repo-relative roots so paths agree.
A call edge whose target is a CLASS node means `new X` / static class use → mapped to X::__construct when it exists.
`overrides` edges are added in reverse (parent → child) so an interface/abstract dispatch reaches every implementation.
"""
import argparse, collections, json, pathlib, sqlite3, sys

CALL_KINDS = {"call", "dispatch", "containerresolution"}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--db", required=True); ap.add_argument("--graph", required=True)
    a = ap.parse_args()
    g = pathlib.Path(a.graph); db = sqlite3.connect(a.db)
    db.execute("ALTER TABLE symbol ADD COLUMN name_refs INTEGER") if "name_refs" not in {r[1] for r in db.execute("PRAGMA table_info(symbol)")} else None

    by_pos, ctor_of, fqn_kind = {}, {}, {}
    for sid, kind, fqn, file, line in db.execute("SELECT id,kind,fqn,file,line_start FROM symbol"):
        by_pos[(file, line)] = fqn; fqn_kind[fqn] = kind
        if kind == "method" and fqn.endswith("::__construct"): ctor_of[fqn.rsplit("::", 1)[0]] = fqn

    print("loading evidence graph…", file=sys.stderr)
    ev = json.loads((g / "evidence-graph.json").read_text())["evidence_graph"]
    node_fqn, unmapped = {}, collections.Counter()
    for n in ev["nodes"]:
        fqn = by_pos.get((n["file_path"], n["start_line"]))
        if fqn is None: unmapped[n["kind"]] += 1; continue
        if n["kind"] == "CLASS" and fqn in ctor_of: fqn = ctor_of[fqn]
        node_fqn[n["id"]] = fqn

    callees, callers = collections.defaultdict(set), collections.defaultdict(set)
    edge_stats = collections.Counter()
    for e in ev["edges"]:
        rk = e.get("relation_kind")
        if rk in CALL_KINDS:
            s, t = node_fqn.get(e["from"]), node_fqn.get(e["to"])
        elif rk == "overrides":  # child overrides parent → parent reaches child
            t, s = node_fqn.get(e["from"]), node_fqn.get(e["to"])
        else:
            continue
        if s and t and s != t:
            callees[s].add(t); callers[t].add(s); edge_stats[rk] += 1
        else:
            edge_stats[f"{rk}_unmapped"] += 1

    roots = json.loads((g / "entrypoints.json").read_text()) + json.loads((g / "entrypoints.convention.json").read_text())
    # a root method also makes its class constructor reachable (the framework instantiates the class)
    seen, stack = set(), [r for r in roots if r in fqn_kind]
    for r in list(stack):
        c = ctor_of.get(r.rsplit("::", 1)[0]) if "::" in r else None
        if c: stack.append(c)
    while stack:
        s = stack.pop()
        if s in seen: continue
        seen.add(s); stack.extend(callees.get(s, ()))
    in_graph = set(node_fqn.values())

    print("loading semantic graph for name references…", file=sys.stderr)
    sem = json.loads((g / "semantic-graph.json").read_text())
    name_refs = collections.Counter(r["target_name"] for r in sem.get("references", []) if r.get("kind") == "Call" and r.get("target_name"))
    del sem

    rows = []
    for fqn, kind in fqn_kind.items():
        if kind not in ("method", "function"): continue
        reach = 1 if fqn in seen else (0 if fqn in in_graph else None)
        rows.append((reach, name_refs.get(fqn.rsplit("::", 1)[-1], 0), fqn))
    db.executemany("UPDATE symbol SET reachable_from_entry=?, name_refs=? WHERE fqn=?", rows); db.commit()

    (g / "callees.json").write_text(json.dumps({k: sorted(v) for k, v in callees.items()}))
    (g / "callers.json").write_text(json.dumps({k: sorted(v) for k, v in callers.items()}))
    stats = {"graph_nodes": len(ev["nodes"]), "nodes_mapped": len(node_fqn), "nodes_unmapped_by_kind": dict(unmapped),
             "edges": dict(edge_stats), "roots": len(roots), "roots_in_table": sum(1 for r in roots if r in fqn_kind),
             "reachable": sum(1 for r in rows if r[0] == 1), "unreached_in_graph": sum(1 for r in rows if r[0] == 0),
             "not_in_graph": sum(1 for r in rows if r[0] is None)}
    (g / "graph-join.json").write_text(json.dumps(stats, indent=1)); print(json.dumps(stats, indent=1))


if __name__ == "__main__":
    main()
