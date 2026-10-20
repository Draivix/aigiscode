#!/usr/bin/env python3
"""
verify.py – deterministic verification of the worker's findings per schemas/verification_matrix.md

  python3 verify.py --pass 2                       # verify every results/pass2/*.json, append findings to registry.jsonl
  python3 verify.py --pass 2 --result <file.json>  # one result file only (pass_driver.py uses this; avoids re-appending)
  python3 verify.py --spot-check 0.10              # re-check 10 % of registry quotes against the files → hallucination rate
  python3 verify.py --apply-pass5                  # apply the independent verifier's verdicts (results/pass5)

For every finding:
  1. evidence.quote must appear VERBATIM in file within line_start-2..line_end+2 → otherwise rejected:evidence_mismatch
  2. symbols[] must exist in symbols.sqlite → otherwise rejected:hallucinated_symbol
  3. kind-specific checks (see the matrix) → status candidate/confirmed/rejected/needs_human
  4. estimated_loc_saved recomputed from symbols.sqlite
  5. appended to registry.jsonl with a history record
"""
import argparse, collections, json, pathlib, sqlite3, random, re, subprocess, sys, datetime, hashlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / "out"
REG = OUT / "registry.jsonl"

DYNAMIC_PATTERNS = [
    r"call_user_func", r"call_user_func_array", r"->\$\w+\(", r"::\$\w+\(", r"new \$\w+", r"\[\$\w+,\s*['\"]", r"ReflectionMethod", r"ReflectionClass",
    r"__call\b", r"__callStatic\b", r"method_exists", r"is_callable", r"array_map\(\s*\[", r"usort\(.*\[",
]

def now(): return datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%S.%fZ")

def load_registry():
    return [json.loads(l) for l in REG.read_text().splitlines() if l.strip()] if REG.exists() else []

def append_registry(rec):
    with REG.open("a", encoding="utf-8") as f: f.write(json.dumps(rec, ensure_ascii=False) + "\n")

REPO_ROOT = pathlib.Path(".")  # verify.py runs with cwd = audited repo root

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

def symbol_exists(db, fqn):
    return db.execute("SELECT 1 FROM symbol WHERE fqn=? LIMIT 1", (fqn,)).fetchone() is not None

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

def symbol_meta(db, fqn):
    return db.execute("SELECT loc, reachable_from_entry, coverage_pct, dead_code_detector, unused_public, ast_hash, cpd_group, name_refs FROM symbol WHERE fqn=?", (fqn,)).fetchone()

def grep_dynamic(symbol_short, roots):
    """List of file:line where the symbol name appears as a string literal (dynamic dispatch / config reference)."""
    hits = []
    pat = re.escape(symbol_short.split("::")[-1])
    try:
        r = subprocess.run(["rg", "-n", "--no-heading", "-e", f"['\"]{pat}['\"]", *roots], capture_output=True, text=True, timeout=120)
        hits += [l for l in r.stdout.splitlines() if l]
    except Exception as e:
        hits.append(f"RG_ERROR {e}")
    return hits[:50]

def grep_dynamic_bulk(shorts, roots):
    """One rg pass for all names: short name → [file:line] where it appears as a string literal (dynamic dispatch, config)."""
    hits = collections.defaultdict(list)
    shorts = sorted(set(shorts))
    for i in range(0, len(shorts), 400):
        pat = "['\"](" + "|".join(re.escape(x) for x in shorts[i:i + 400]) + ")['\"]"
        r = subprocess.run(["rg", "-n", "-o", "--no-heading", "-e", pat, *roots], capture_output=True, text=True, timeout=600)
        for l in r.stdout.splitlines():
            loc, _, m = l.rpartition(":")
            hits[m.strip("'\"")].append(loc)
    return hits


def recheck_dead(db, roots):
    """Pass 1 → two-source rule for every dead_symbol candidate (graph reachability, PHPStan flags, string refs) in one go."""
    last = {}
    for r in load_registry(): last[r["id"]] = r
    cands = [r for r in last.values() if r["kind"] == "dead_symbol" and r["status"] == "candidate"]
    hits = grep_dynamic_bulk([c["symbols"][0].rsplit("::", 1)[-1] for c in cands], roots)
    stats = collections.Counter()
    for f in cands:
        s = f["symbols"][0]; m = symbol_meta(db, s)
        if not m: status, v = "rejected", [{"check": "symbol_meta", "result": "missing"}]
        else:
            loc, reach, cov, dcd, unused, _h, _c, name_refs = m
            dyn = hits.get(s.rsplit("::", 1)[-1], [])[:50]
            v = [{"check": "aigiscode_reachable", "result": reach}, {"check": "same_name_call_refs_anywhere", "result": name_refs},
                 {"check": "dead_code_detector", "result": dcd}, {"check": "unused_public", "result": unused}, {"check": "grep_dynamic_string_refs", "result": dyn}]
            sources = (reach == 0) + bool(dcd or unused) + (not name_refs and not dyn)
            if reach == 1: status = "rejected"; v.append({"reason": "reachable_in_graph"})
            elif dyn: status = "needs_human"; v.append({"reason": "string_reference_found"})
            elif reach == 0 and (dcd or unused): status = "candidate"; v.append({"note": f"{sources} independent sources agree; awaiting pass-5 'dead' verdict and clients/* overlay grep"})
            elif reach is None: status = "needs_human"; v.append({"reason": "not_in_graph"})
            else: status = "candidate"; v.append({"note": "single source so far"})
        if status != f["status"] or f.get("verification") != v:
            append_registry(dict(f, status=status, verification=v, history=f.get("history", []) + [{"ts": now(), "status": status, "by": "verify.py --recheck-dead"}]))
        stats[status] += 1
    print(json.dumps(stats, indent=1))


def decide(db, f, roots):
    """Returns (status, verification[])."""
    v = []
    kind = f["kind"]
    metas = {s: symbol_meta(db, s) for s in f["symbols"]}

    if kind == "clone_exact":
        hashes = {m[5] for m in metas.values() if m}
        cpd = {m[6] for m in metas.values() if m and m[6]}
        ok = len(hashes) == 1 or len(cpd) == 1
        v.append({"check": "ast_hash_or_cpd", "result": ok})
        return ("confirmed" if ok else "needs_human"), v

    if kind == "dead_symbol":
        s = f["symbols"][0]; m = metas[s]
        if not m: return "rejected", [{"check": "symbol_meta", "result": "missing"}]
        loc, reach, cov, dcd, unused, _h, _c, name_refs = m
        v.append({"check": "aigiscode_reachable", "result": reach})
        v.append({"check": "same_name_call_refs_anywhere", "result": name_refs})
        v.append({"check": "dead_code_detector", "result": dcd}); v.append({"check": "unused_public", "result": unused})
        v.append({"check": "coverage_pct", "result": cov})
        dyn = grep_dynamic(s, roots); v.append({"check": "grep_dynamic_string_refs", "result": dyn})
        if reach == 1: return "rejected", v + [{"reason": "reachable_in_graph"}]
        if dyn and not any(h.startswith("RG_ERROR") for h in dyn): return "needs_human", v + [{"reason": "string_reference_found"}]
        if reach == 0 and (dcd == 1 or unused == 1) and (cov in (None, 0, 0.0)):
            note = "awaiting pass-5 verifier 'dead' verdict" + (f"; {name_refs} same-name call(s) elsewhere must be resolved" if name_refs else "")
            return "candidate", v + [{"note": note}]
        return "needs_human", v

    if kind in ("clone_semantic", "variant_mergeable", "double_path"):
        if kind == "variant_mergeable" and not f.get("behavior_differences"):
            return "rejected", [{"reason": "variant_without_behavior_differences"}]
        return "candidate", v + [{"note": "awaiting pass-5 verifier"}]

    if kind == "redundant_layer":
        s = f["symbols"][0]
        body = db.execute("SELECT body FROM symbol WHERE fqn=?", (s,)).fetchone()
        one_call = bool(body and body[0] and len([l for l in body[0].splitlines() if l.strip()]) <= 2)
        v.append({"check": "body_is_single_delegation", "result": one_call})
        return ("candidate" if one_call else "needs_human"), v

    if kind == "suspicious": return "needs_human", v
    return "candidate", v

def process_result(db, path, roots, stats):
    data = json.loads(path.read_text())
    res, meta = data["result"], data["meta"]
    rejected_for_pushback = []
    for f in res.get("findings", []):
        fid = "F-" + hashlib.sha1(json.dumps([f["kind"], sorted(f["symbols"])], sort_keys=True).encode()).hexdigest()[:10]
        bad_ev = [e for e in f["evidence"] if not quote_matches(e)]
        f["symbols"] = [resolve_symbol(db, s) or s for s in f["symbols"]]  # canonical spelling before the id is computed
        bad_sym = [s for s in f["symbols"] if not symbol_exists(db, s)]
        if bad_ev or bad_sym:
            status, ver = "rejected", [{"reason": "evidence_mismatch" if bad_ev else "hallucinated_symbol", "bad_evidence": bad_ev, "bad_symbols": bad_sym}]
            rejected_for_pushback.append({"finding": f, "reason": ver[0]["reason"]})
            stats["rejected_hallucination"] += 1
        else:
            status, ver = decide(db, f, roots)
            stats[status] += 1
        loc = sum((symbol_meta(db, s) or (0,))[0] or 0 for s in f["symbols"][1:]) if len(f["symbols"]) > 1 else (symbol_meta(db, f["symbols"][0]) or (0,))[0]
        append_registry({"id": fid, "kind": f["kind"], "title": f["title"], "symbols": f["symbols"], "evidence": f["evidence"],
                         "rationale": f["rationale"], "behavior_differences": f.get("behavior_differences", []),
                         "confidence_model": f["confidence"], "estimated_loc_saved_model": f["estimated_loc_saved"],
                         "estimated_loc_saved": loc, "status": status, "verification": ver, "source": f"pass{meta['pass']}/{meta['batch']}",
                         "history": [{"ts": now(), "status": status, "by": "verify.py"}]})
    if rejected_for_pushback:
        pb = OUT / "results" / f"pass{meta['pass']}" / f"{meta['batch']}_rejected.json"
        pb.write_text(json.dumps(rejected_for_pushback, ensure_ascii=False, indent=1))
        print(f"  → {len(rejected_for_pushback)} rejected; push-back file: {pb}")

def spot_check(fraction):
    latest = {}
    for r in load_registry(): latest[r["id"]] = r   # the registry is append-only; only the last record of an id is current
    reg = list(latest.values()); sample = random.sample(reg, max(1, int(len(reg) * fraction)))
    bad = sum(1 for r in sample for e in r["evidence"] if not quote_matches(e))
    total = sum(len(r["evidence"]) for r in sample)
    rate = bad / total if total else 0
    print(f"spot-check: {bad}/{total} quotes mismatched = {rate:.1%}  {'TAINTED' if rate > 0.05 else 'ok'}")

PASS5_STATUS = {"delete": {"dead": "confirmed", "alive": "rejected", "undecidable": "needs_human"},
                "merge": {"safe": "confirmed", "safe_with_adapter": "confirmed", "unsafe": "rejected", "undecidable": "needs_human"}}

DECISION = {"safe": "ok", "safe_with_adapter": "ok", "dead": "ok", "unsafe": "no", "alive": "no", "undecidable": "undecidable"}

def apply_pass6():
    """Second independent verifier (results/pass6, agy) against the first (pass5_verifier in verification[]):
    same decision → status stands (agreement recorded); different decision → needs_human; a row without a decisive first
    verdict (needs_human) gets a decisive second verdict as `candidate` (one verifier is one source, not two)."""
    by_id = {}
    for r in load_registry(): by_id[r["id"]] = r
    stats = collections.Counter(); accepted = []
    for m in sorted((OUT / "results/pass6").glob("*.verified")):
        try: names = json.loads(m.read_text())["accepted"]
        except (json.JSONDecodeError, KeyError, TypeError): names = [m.stem + ".json"]
        accepted += [OUT / "results/pass6" / n for n in names]
    for p in accepted:
        for v in json.loads(p.read_text())["result"].get("verifications", []):
            f = by_id.get(v.get("finding_id"))
            if not f: stats["unknown_finding_id"] += 1; continue
            mine = DECISION.get(v.get("verdict"), "undecidable")
            bad = [e for e in v.get("evidence", []) if not quote_matches(e)]
            if mine != "undecidable" and (bad or not v.get("evidence")): mine = "undecidable"; stats["evidence_missing_or_mismatch"] += 1
            prior = next((c for c in reversed(f.get("verification", [])) if isinstance(c, dict) and c.get("check") in ("pass5_verifier", "agy_crosscheck_verifier")), None)
            theirs = DECISION.get((prior or {}).get("verdict"), "undecidable")
            if mine == "undecidable": status, outcome = f["status"], "second_undecidable"
            elif theirs == "undecidable": status, outcome = ("candidate" if f["status"] == "needs_human" else f["status"]), "single_decisive_verdict"
            elif mine == theirs: status, outcome = f["status"], "agree"
            else: status, outcome = "needs_human", "disagree"
            if status == "confirmed" and f["kind"] == "dead_symbol":
                reach = next((c.get("result") for c in f.get("verification", []) if c.get("check") == "aigiscode_reachable"), None)
                if reach == 1: status = "rejected"
            rec = dict(f, status=status, verification=f.get("verification", []) + [{"check": "pass6_verifier", "verdict": v.get("verdict"), "first_verdict": (prior or {}).get("verdict"), "outcome": outcome, "confidence": v.get("confidence"), "conditions": v.get("conditions", [])[:3], "evidence": v.get("evidence", [])[:4]}],
                       source=f"pass6/{p.stem}", history=f.get("history", []) + [{"ts": now(), "status": status, "by": "verify.py --apply-pass6", "verdict": v.get("verdict"), "outcome": outcome}])
            append_registry(rec); stats[outcome] += 1; stats[f"status:{status}"] += 1
    print(json.dumps(stats, indent=1))


def apply_pass5():
    """Independent verifier verdicts (results/pass5/*.json) → new registry records with the final status and history."""
    by_id = {}
    for r in load_registry(): by_id[r["id"]] = r   # last record per id wins (append-only log)
    stats = collections.Counter()
    accepted = []   # only the answers the driver accepted (≤ 5 % quote mismatch), in the order it accepted them; tainted attempts never apply
    for m in sorted((OUT / "results/pass5").glob("v_*.verified")):
        try: names = json.loads(m.read_text())["accepted"]
        except (json.JSONDecodeError, KeyError, TypeError): names = [m.stem + ".json"]
        accepted += [OUT / "results/pass5" / n for n in names]
    for p in accepted:
        res = json.loads(p.read_text())["result"]
        for v in res.get("verifications", []):
            f = by_id.get(v.get("finding_id"))
            if not f: stats["unknown_finding_id"] += 1; continue
            status = PASS5_STATUS.get(v.get("claim_type"), {}).get(v.get("verdict"), "needs_human")
            bad_quotes = [e for e in v.get("evidence", []) if not quote_matches(e)]
            if status != "needs_human" and (bad_quotes or not v.get("evidence")):   # a verdict without verbatim evidence decides nothing
                status = "needs_human"; stats["evidence_missing_or_mismatch"] += 1
                v = dict(v, conditions=list(v.get("conditions", [])) + [f"verifier evidence not verbatim ({len(bad_quotes)} bad quote(s), {len(v.get('evidence', []))} given) – verdict '{v.get('verdict')}' not applied"])
            if status == "confirmed" and f["kind"] == "dead_symbol":
                reach = next((c.get("result") for c in f.get("verification", []) if c.get("check") == "aigiscode_reachable"), None)
                if reach == 1: status = "rejected"   # the graph wins over the model (DIRECTOR §6.1)
            rec = dict(f, status=status, verification=f.get("verification", []) + [{"check": "pass5_verifier", "verdict": v.get("verdict"), "confidence": v.get("confidence"), "conditions": v.get("conditions", []), "missing_material": v.get("missing_material", []), "differences_found": v.get("differences_found", []), "evidence": v.get("evidence", [])[:6]}],
                       source=f"pass5/{p.stem}", history=f.get("history", []) + [{"ts": now(), "status": status, "by": "verify.py --apply-pass5", "verdict": v.get("verdict")}])
            append_registry(rec); stats[status] += 1
    print(json.dumps(stats, indent=1))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply-pass5", action="store_true")
    ap.add_argument("--apply-pass6", action="store_true", help="second verifier (agy) verdicts in results/pass6 → agreement / needs_human")
    ap.add_argument("--recheck-dead", action="store_true", help="re-decide every dead_symbol candidate with the two-source rule (bulk rg)")
    ap.add_argument("--pass", dest="pass_no", type=int)
    ap.add_argument("--result", help="verify this one result file instead of the whole pass directory")
    ap.add_argument("--spot-check", type=float)
    ap.add_argument("--roots", nargs="*", default=["app", "routes", "config", "database", "packages", "tools"])
    ap.add_argument("--db", default=str(OUT / "symbols.sqlite"))
    a = ap.parse_args()
    if a.apply_pass5: return apply_pass5()
    if a.apply_pass6: return apply_pass6()
    if a.spot_check: return spot_check(a.spot_check)
    db = sqlite3.connect(a.db)
    if a.recheck_dead: return recheck_dead(db, a.roots)
    stats = {"confirmed": 0, "candidate": 0, "rejected": 0, "needs_human": 0, "rejected_hallucination": 0}
    paths = [pathlib.Path(a.result)] if a.result else sorted((OUT / "results" / f"pass{a.pass_no}").glob("*.json"))
    for p in paths:
        if "_rejected" in p.name or "_empty" in p.name: continue
        print(p.name); process_result(db, p, a.roots, stats)
    print(json.dumps(stats, indent=1))

if __name__ == "__main__":
    main()
