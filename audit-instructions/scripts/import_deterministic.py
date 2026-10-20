#!/usr/bin/env python3
"""
import_deterministic.py – pass 1: turn deterministic tool output into registry findings and symbol-table flags.

  python3 import_deterministic.py --db lean-audit/out/symbols.sqlite --out lean-audit/out [--only jscpd,ast,composer,phpstan]

Sources (all under <out>/deterministic/):
  jscpd/jscpd-report.json      exact clone pairs → union-find over the enclosing symbols → one `clone_exact` finding per
                               group, status confirmed (CPD does not guess); symbol.cpd_group is set. Pairs where both
                               sides are generated schema migrations are counted but not reported.
  clones_normalized.json       same normalized AST hash (type-2 clones, from extract_symbols.php) → `clone_exact`,
                               confirmed, verification ast_hash
  composer-unused.json         → `unused_dependency`, candidate (runtime-only packages such as roadrunner must be judged)
  phpstan.json                 shipmonk.dead* / public.*.unused → `dead_symbol`, candidate; sets symbol.dead_code_detector
                               and symbol.unused_public so verify.py can apply the two-source rule
  (symbol table)               graph: reachable_from_entry = 0 AND name_refs = 0 (import_graph.py) → `dead_symbol`, candidate;
                               the two signals are one source each (aigiscode reachability, dynamic name grep)

Registry records are appended to <out>/registry.jsonl with source "pass1/<tool>"; nothing is ever rewritten.
"""
import argparse, collections, datetime, hashlib, json, pathlib, sqlite3


def now(): return datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%S.%fZ")


class Table:
    def __init__(self, db):
        self.db = db
        self.by_file = collections.defaultdict(list)
        for fqn, file, a, b, loc, kind in db.execute("SELECT fqn,file,line_start,line_end,loc,kind FROM symbol WHERE kind IN ('method','function','class','trait','enum','interface')"):
            self.by_file[file].append((a, b, fqn, loc, kind))
        for v in self.by_file.values(): v.sort(key=lambda r: (r[1] - r[0], r[0]))

    def enclosing(self, file, start, end, prefer_callable=True):
        """Smallest symbol covering [start, end]; callables first, class as fallback."""
        rows = [r for r in self.by_file.get(file, []) if r[0] <= start and r[1] >= end]
        if prefer_callable:
            c = [r for r in rows if r[4] in ("method", "function")]
            if c: return c[0]
        return rows[0] if rows else None

    def loc(self, fqn):
        r = self.db.execute("SELECT loc FROM symbol WHERE fqn=?", (fqn,)).fetchone(); return r[0] if r else 0


def record(out, rec):
    with (out / "registry.jsonl").open("a", encoding="utf-8") as f: f.write(json.dumps(rec, ensure_ascii=False) + "\n")


def fid(kind, symbols): return "F-" + hashlib.sha1(json.dumps([kind, sorted(symbols)], sort_keys=True).encode()).hexdigest()[:10]


def quote_of(file, start, n=3):
    try: lines = pathlib.Path(file).read_text(encoding="utf-8", errors="replace").splitlines()
    except FileNotFoundError: return ""
    return "\n".join(lines[start - 1:start - 1 + n])


def import_jscpd(t, out, stats):
    """One finding per CPD clone class = all occurrences of one identical fragment (grouping by fragment text, never
    transitively through symbols – transitive union chained 2 649 symbols into one useless blob on the first attempt)."""
    rep = json.loads((out / "deterministic/jscpd/jscpd-report.json").read_text())
    roots = ("app", "routes", "config", "database", "packages", "tools")  # jscpd names are relative to the scanned root
    def resolve(name):
        hits = [f"{r}/{name}" for r in roots if f"{r}/{name}" in t.by_file]
        if not hits: hits = [f"{r}/{name}" for r in roots if pathlib.Path(r, name).exists()]
        return hits[0] if hits else name
    classes = collections.defaultdict(dict)   # fragment key → {(file,start,end): fqn}
    frag_lines, frag_text = {}, {}
    for d in rep["duplicates"]:
        fa, fb = resolve(d["firstFile"]["name"]), resolve(d["secondFile"]["name"])
        if fa.startswith("database/migrations") and fb.startswith("database/migrations"):
            stats["jscpd_pairs_generated_migrations_skipped"] += 1; continue
        occ = []
        for f, loc in ((fa, d["firstFile"]), (fb, d["secondFile"])):
            s = t.enclosing(f, loc["start"], loc["end"], prefer_callable=True)
            if not s or s[4] not in ("method", "function"): stats["jscpd_sides_outside_callable"] += 1; continue
            occ.append(((f, loc["start"], loc["end"]), s[2]))
        if len(occ) < 2: continue
        if occ[0][1] == occ[1][1]: stats["jscpd_pairs_inside_one_symbol"] += 1; continue
        key = hashlib.sha1(" ".join(d["fragment"].split()).encode()).hexdigest()
        for pos, fqn in occ: classes[key][pos] = fqn
        frag_lines[key] = d["lines"]; frag_text.setdefault(key, d["fragment"]); stats["jscpd_pairs_used"] += 1
    n = 0
    for key, occs in classes.items():
        g = sorted(set(occs.values()))
        if len(g) < 2: continue
        cpd = "cpd_" + key[:10]
        t.db.executemany("UPDATE symbol SET cpd_group=COALESCE(cpd_group || ',', '') || ? WHERE fqn=?", [(cpd, s) for s in g])
        # quote read from each occurrence's own file: the first import copied the fragment text of the first occurrence to
        # every evidence entry, and jscpd fragments are token-normalised, so 23 % of the copied quotes did not match their file
        ev = [{"file": f, "line_start": a, "line_end": b, "quote": quote_of(f, a)[:600]} for (f, a, b) in sorted(occs)][:8]
        record(out, {"id": fid("clone_exact", g), "kind": "clone_exact", "title": f"jscpd clone class: {frag_lines[key]} identical lines in {len(occs)} places / {len(g)} symbols",
                     "symbols": g, "evidence": ev, "rationale": f"jscpd found the same {frag_lines[key]}-line fragment (≥40 tokens) at {len(occs)} locations (type-1 clone).",
                     "behavior_differences": [], "confidence_model": "high", "estimated_loc_saved_model": 0,
                     "estimated_loc_saved": frag_lines[key] * (len(occs) - 1), "status": "confirmed", "verification": [{"check": "cpd", "result": cpd}],
                     "source": "pass1/jscpd", "history": [{"ts": now(), "status": "confirmed", "by": "import_deterministic.py"}]})
        n += 1
    t.db.commit(); stats["jscpd_clone_classes"] = n; stats["jscpd_symbols_in_classes"] = len({s for o in classes.values() for s in o.values()})


def import_ast(t, out, stats):
    rows = json.loads((out / "deterministic/clones_normalized.json").read_text())
    for r in rows:
        g = sorted(set(r["fqns"].split(" | ")))
        if len(g) < 2: continue
        ev = []
        for s in g[:6]:
            m = t.db.execute("SELECT file,line_start,line_end FROM symbol WHERE fqn=? AND ast_hash=?", (s, r["ast_hash"])).fetchone()
            if m: ev.append({"file": m[0], "line_start": m[1], "line_end": m[2], "quote": quote_of(m[0], m[1])[:600]})
        record(out, {"id": fid("clone_exact", g), "kind": "clone_exact", "title": f"normalized-AST clone group of {len(g)} symbols: {g[0].rsplit('::', 1)[-1]}",
                     "symbols": g, "evidence": ev, "rationale": "Identical normalized AST (variables renamed, literals abstracted, comments dropped) – type-2 clone found without a model.",
                     "behavior_differences": [], "confidence_model": "high", "estimated_loc_saved_model": 0,
                     "estimated_loc_saved": sum(t.loc(s) for s in g[1:]), "status": "confirmed", "verification": [{"check": "ast_hash", "result": r["ast_hash"]}],
                     "source": "pass1/ast_hash", "history": [{"ts": now(), "status": "confirmed", "by": "import_deterministic.py"}]})
        stats["ast_groups"] += 1


def import_composer(t, out, stats):
    d = json.loads((out / "deterministic/composer-unused.json").read_text())
    for p in d.get("unused-packages", []):
        name = p["name"] if isinstance(p, dict) else p
        record(out, {"id": fid("unused_dependency", [name]), "kind": "unused_dependency", "title": f"composer package {name} has no code reference", "symbols": [name],
                     "evidence": [{"file": "composer.json", "line_start": 1, "line_end": 1, "quote": name}], "rationale": "composer-unused found no symbol of this package used from the analysed code. Runtime-only packages (workers, CLIs) must be judged by a human.",
                     "behavior_differences": [], "confidence_model": "medium", "estimated_loc_saved_model": 0, "estimated_loc_saved": 0, "status": "candidate",
                     "verification": [{"check": "composer_unused", "result": True}], "source": "pass1/composer-unused", "history": [{"ts": now(), "status": "candidate", "by": "import_deterministic.py"}]})
        stats["unused_packages"] += 1


def import_phpstan(t, out, stats):
    d = json.loads((out / "deterministic/phpstan.json").read_text())
    root = str(pathlib.Path(".").resolve()) + "/"
    per_symbol = collections.defaultdict(list)
    for file, body in d["files"].items():
        rel = file.replace(root, "")
        for m in body["messages"]:
            ident = m.get("identifier") or ""
            if not (ident.startswith("shipmonk.dead") or ident.startswith("public.")): continue
            s = t.enclosing(rel, m["line"], m["line"])
            if not s: stats["phpstan_messages_without_symbol"] += 1; continue
            per_symbol[s[2]].append((ident, m["message"], rel, m["line"]))
    for fqn, msgs in per_symbol.items():
        idents = {i for i, *_ in msgs}
        dcd = int(any(i.startswith("shipmonk.deadMethod") or i.startswith("shipmonk.deadFunction") for i in idents))
        unused = int(any(i == "public.method.unused" for i in idents))
        t.db.execute("UPDATE symbol SET dead_code_detector=?, unused_public=? WHERE fqn=?", (dcd, unused, fqn))
        if not (dcd or unused): stats["phpstan_member_only_flags"] += 1; continue  # constants / properties: kept as flags via messages, not symbol-level findings
        _i, text, rel, line = msgs[0]
        record(out, {"id": fid("dead_symbol", [fqn]), "kind": "dead_symbol", "title": f"PHPStan dead code: {fqn.rsplit(chr(92), 1)[-1]}", "symbols": [fqn],
                     "evidence": [{"file": rel, "line_start": line, "line_end": line, "quote": quote_of(rel, line, 1)[:600]}],
                     "rationale": "; ".join(sorted({m[1] for m in msgs}))[:1200], "behavior_differences": [], "confidence_model": "high" if dcd and unused else "medium",
                     "estimated_loc_saved_model": 0, "estimated_loc_saved": t.loc(fqn), "status": "candidate",
                     "verification": [{"check": "dead_code_detector", "result": dcd}, {"check": "unused_public", "result": unused}, {"note": "awaiting second source (graph reachability + dynamic grep) in verify.py"}],
                     "source": "pass1/phpstan", "history": [{"ts": now(), "status": "candidate", "by": "import_deterministic.py"}]})
        stats["phpstan_dead_symbol_candidates"] += 1
    t.db.commit()


def import_graph_dead(t, out, stats):
    rows = t.db.execute("""SELECT fqn,file,line_start,loc,kind,visibility,is_abstract FROM symbol
                           WHERE kind IN ('method','function') AND reachable_from_entry=0 AND COALESCE(name_refs,0)=0""").fetchall()
    for fqn, file, line, loc, kind, vis, is_abstract in rows:
        if kind == "method" and fqn.rsplit("::", 1)[-1] in ("__construct", "__destruct", "__toString", "__invoke", "__get", "__set", "__call", "__callStatic", "jsonSerialize"):
            stats["graph_dead_magic_skipped"] += 1; continue
        record(out, {"id": fid("dead_symbol", [fqn]), "kind": "dead_symbol", "title": f"unreachable and never named: {fqn.rsplit(chr(92), 1)[-1]}", "symbols": [fqn],
                     "evidence": [{"file": file, "line_start": line, "line_end": line, "quote": quote_of(file, line, 1)[:600]}],
                     "rationale": "Not reachable from any entry point over resolved aigiscode call edges, and no Call reference anywhere in the analysed code targets this name (so no dynamic same-name dispatch either).",
                     "behavior_differences": [], "confidence_model": "high", "estimated_loc_saved_model": 0, "estimated_loc_saved": loc or 0, "status": "candidate",
                     "verification": [{"check": "aigiscode_reachable", "result": 0}, {"check": "same_name_call_refs_anywhere", "result": 0}, {"note": "pass 5 must grep clients/* overlays and config strings before confirming"}],
                     "source": "pass1/graph", "history": [{"ts": now(), "status": "candidate", "by": "import_deterministic.py"}]})
        stats["graph_dead_candidates"] += 1; stats["graph_dead_loc"] += loc or 0


def fix_quotes(out, stats):
    """Re-read every pass1/jscpd evidence quote from its own file and append a corrected continuation record where it differs
    (registry stays append-only; merge_registry takes evidence from the continuation)."""
    latest = {}
    for line in (out / "registry.jsonl").read_text(encoding="utf-8").splitlines():
        r = json.loads(line); latest[r["id"]] = r
    for r in latest.values():
        if r.get("source") != "pass1/jscpd": continue
        ev = [dict(e, quote=quote_of(e["file"], e["line_start"])[:600]) for e in r["evidence"]]
        if ev == r["evidence"]: stats["jscpd_quotes_already_correct"] += 1; continue
        record(out, dict(r, evidence=ev, history=r.get("history", []) + [{"ts": now(), "status": r["status"], "by": "import_deterministic.py --fix-quotes",
                                                                           "note": "evidence quotes re-read from each file (tooling bug, see INCIDENTS.md)"}]))
        stats["jscpd_quotes_fixed"] += 1


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--db", required=True); ap.add_argument("--out", required=True); ap.add_argument("--only", default="jscpd,ast,composer,phpstan,graph"); ap.add_argument("--fix-quotes", action="store_true")
    a = ap.parse_args()
    out = pathlib.Path(a.out); t = Table(sqlite3.connect(a.db)); stats = collections.Counter()
    if a.fix_quotes: fix_quotes(out, stats); print(json.dumps(dict(stats), indent=1)); return
    for name, fn in (("jscpd", import_jscpd), ("ast", import_ast), ("composer", import_composer), ("phpstan", import_phpstan), ("graph", import_graph_dead)):
        if name in a.only.split(","): fn(t, out, stats)
    print(json.dumps(dict(stats), indent=1))


if __name__ == "__main__":
    main()
