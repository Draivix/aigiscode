#!/usr/bin/env python3
"""
merge_registry.py – dedup of findings across batches/passes, capability registry, worker calibration, report

  python3 merge_registry.py                       # dedup registry.jsonl (same kind + same symbol set → one record, histories merged)
  python3 merge_registry.py --capabilities pass4  # merge capabilities from results/pass4/*.json into capabilities.jsonl
  python3 merge_registry.py --calibration         # how many of the worker's 'high' ended rejected
  python3 merge_registry.py --report              # writes out/REPORT.md
  python3 merge_registry.py --status              # writes out/STATUS.counts.md

registry.jsonl is an append-only log; this script produces registry.merged.jsonl (canonical state) – the log is never rewritten.
Status resolution: a later record that CONTINUES an earlier one's history (verify.py re-decision, pass-5 verdict) replaces its
status; two INDEPENDENT reports of the same finding (different batches) resolve by rank, where a verifier's 'rejected' never
loses to a model's 'confirmed' unless that confirmation came from a deterministic source.
"""
import re, argparse, json, pathlib, collections, difflib, datetime, csv

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / "out"
REG = OUT / "registry.jsonl"; MERGED = OUT / "registry.merged.jsonl"; CAPS = OUT / "capabilities.jsonl"

STATUS_RANK = {"rejected": 0, "candidate": 1, "needs_human": 2, "confirmed": 3}

def read_jsonl(p): return [json.loads(l) for l in p.read_text().splitlines() if l.strip()] if p.exists() else []
def write_jsonl(p, rows): p.write_text("".join(json.dumps(r, ensure_ascii=False) + "\n" for r in rows))

def dedup():
    rows = read_jsonl(REG); by = {}
    for r in rows:
        key = (r["kind"], tuple(sorted(r["symbols"])))
        if key not in by: by[key] = dict(r, sources=[r["source"]]); continue
        m = by[key]
        continuation = r.get("history", [])[:len(m.get("history", []))] == m.get("history", []) and len(r.get("history", [])) > len(m.get("history", []))
        if continuation:   # a re-decision or evidence refresh of this very record (verify.py --recheck-dead / --apply-pass5 / import --fix-quotes)
            m["status"], m["verification"], m["history"] = r["status"], r["verification"], r["history"]
            m["evidence"] = r.get("evidence", m["evidence"])
            continue
        m["sources"].append(r["source"]); m["history"] += r.get("history", [])
        m["evidence"] = {json.dumps(e, sort_keys=True): e for e in m["evidence"] + r["evidence"]}.values(); m["evidence"] = list(m["evidence"])
        if STATUS_RANK[r["status"]] > STATUS_RANK[m["status"]]: m["status"] = r["status"]; m["verification"] = r["verification"]
        m["seen_in_batches"] = len(m["sources"])
    merged = list(by.values()); write_jsonl(MERGED, merged)
    c = collections.Counter((r["kind"], r["status"]) for r in merged)
    print(f"merged {len(rows)} → {len(merged)} unique findings"); [print(f"  {k:20s} {s:12s} {n}") for (k, s), n in sorted(c.items())]
    return merged

def merge_capabilities(pass_dir):
    caps = {c["capability_id"]: c for c in read_jsonl(CAPS)}; next_id = len(caps) + 1; collisions = []
    for p in sorted((OUT / "results" / pass_dir).glob("*.json")):
        if re.search(r"_(rejected\d*|empty|gaps|missing)\.json$", p.name): continue  # driver side files, not worker answers
        res = json.loads(p.read_text())["result"]
        for c in res.get("capabilities", []) + res.get("new_capabilities", []):
            cid = c.get("capability_id") or ""
            if cid and cid in caps:
                caps[cid]["implementations"] = sorted(set(caps[cid]["implementations"]) | set(c["implementations"]))
                caps[cid]["entry_points"] = sorted(set(caps[cid].get("entry_points", [])) | set(c.get("entry_points", [])))
                continue
            # fuzzy name match against existing capabilities
            best = max(((difflib.SequenceMatcher(None, c["name"].lower(), e["name"].lower()).ratio(), k) for k, e in caps.items()), default=(0, None))
            if best[0] >= 0.85:
                k = best[1]; caps[k]["implementations"] = sorted(set(caps[k]["implementations"]) | set(c["implementations"])); collisions.append((c["name"], caps[k]["name"], round(best[0], 2)))
            else:
                k = f"CAP-{next_id:04d}"; next_id += 1
                caps[k] = {"capability_id": k, "name": c["name"], "description": c["description"], "implementations": sorted(set(c["implementations"])),
                           "entry_points": c.get("entry_points", []), "first_seen": p.name}
        for ms in res.get("merge_suggestions", []): collisions.append(("MERGE_SUGGESTION", ms["capability_ids"], ms["reason"]))
    for c in caps.values(): c["is_double_path"] = len(c["implementations"]) >= 2
    write_jsonl(CAPS, list(caps.values()))
    (OUT / "capability_collisions.json").write_text(json.dumps(collisions, ensure_ascii=False, indent=1))
    dp = sum(1 for c in caps.values() if c["is_double_path"])
    print(f"capabilities={len(caps)} double_path={dp} collisions_to_review={len(collisions)} (→ out/capability_collisions.json; send to the worker as a mini-batch to decide)")

def calibration():
    merged = read_jsonl(MERGED) or dedup()
    tab = collections.defaultdict(collections.Counter)
    for r in merged: tab[r.get("confidence_model", "?")][r["status"]] += 1
    lines = ["| model confidence | confirmed | candidate | needs_human | rejected | rejected % |", "|---|---|---|---|---|---|"]
    for conf in ("high", "medium", "low"):
        c = tab[conf]; tot = sum(c.values()) or 1
        lines.append(f"| {conf} | {c['confirmed']} | {c['candidate']} | {c['needs_human']} | {c['rejected']} | {c['rejected']/tot:.0%} |")
    txt = "\n".join(lines); print(txt)
    hi = tab["high"]; rate = hi["rejected"] / (sum(hi.values()) or 1)
    if rate > 0.10: print(f"\n⚠ high-confidence rejection rate {rate:.0%} > 10 % → raise the spot-check to 20 % and lower the weight of the worker's confidence")
    return txt

def status():
    merged = read_jsonl(MERGED) or dedup()
    c = collections.Counter(r["status"] for r in merged); k = collections.Counter(r["kind"] for r in merged)
    loc = sum(r["estimated_loc_saved"] for r in merged if r["status"] == "confirmed")
    batches = {p: len(list((OUT / "results" / p).glob("*.json"))) for p in ("pass2", "pass3", "pass4", "pass5") if (OUT / "results" / p).exists()}
    inc = (OUT / "INCIDENTS.md").read_text().count("\n") if (OUT / "INCIDENTS.md").exists() else 0
    md = [f"# LEAN AUDIT STATUS – {datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%S.%fZ")}", "",
          f"commit: `{(OUT/'baseline/commit.txt').read_text().strip() if (OUT/'baseline/commit.txt').exists() else '?'}`", "",
          "## Findings by status", *[f"- {s}: {n}" for s, n in sorted(c.items())], "",
          "## Findings by kind", *[f"- {s}: {n}" for s, n in sorted(k.items())], "",
          f"## Estimated saving (confirmed only): **{loc} LOC**", "",
          "## Batches done", *[f"- {p}: {n}" for p, n in batches.items()], "",
          f"## Incidents: {inc}", "", "## Worker calibration", calibration()]
    (OUT / "STATUS.counts.md").write_text("\n".join(md)); print("STATUS.counts.md written (STATUS.md is the director's narrative)")

def report():
    merged = read_jsonl(MERGED) or dedup()
    caps = read_jsonl(CAPS)
    hist = list(csv.DictReader((OUT / "metrics_history.csv").open())) if (OUT / "metrics_history.csv").exists() else []
    md = ["# LEAN AUDIT – REPORT", "", "## 1. Metrics per pass", ""]
    if hist:
        md += ["| " + " | ".join(hist[0].keys()) + " |", "|" + "---|" * len(hist[0])] + ["| " + " | ".join(r.values()) + " |" for r in hist]
    md += ["", "## 2. Findings (confirmed + needs_human), sorted by saving × confidence", ""]
    w = {"high": 1.0, "medium": 0.6, "low": 0.3}
    top = sorted((r for r in merged if r["status"] in ("confirmed", "needs_human")), key=lambda r: -r["estimated_loc_saved"] * w.get(r.get("confidence_model"), 0.5))
    for r in top[:300]:
        md += [f"### {r['id']} · {r['kind']} · {r['status']} · ~{r['estimated_loc_saved']} LOC", f"**{r['title']}**", "", f"Symbols: `" + "`, `".join(r["symbols"]) + "`", "", r["rationale"], ""]
        for e in r["evidence"][:3]: md += [f"- `{e['file']}:{e['line_start']}-{e['line_end']}`", "  ```php", "  " + e["quote"].replace("\n", "\n  "), "  ```"]
        if r.get("behavior_differences"): md += ["", "Behavior differences:"] + [f"- `{d['input']}` → A: {d['a_gives']} / B: {d['b_gives']}" for d in r["behavior_differences"]]
        md.append("")
    md += ["## 3. Top 50 `needs_human` decisions (the question the owner has to answer)", ""]
    for r in sorted((r for r in merged if r["status"] == "needs_human"), key=lambda r: -r["estimated_loc_saved"])[:50]:
        reasons = sorted({v.get("reason") or v.get("note") for v in r.get("verification", []) if isinstance(v, dict) and (v.get("reason") or v.get("note"))})
        md += [f"- **{r['id']}** · {r['kind']} · ~{r['estimated_loc_saved']} LOC · `" + "`, `".join(r["symbols"][:3]) + "`" + (" …" if len(r["symbols"]) > 3 else ""),
               f"  {human_question(r)}" + (f" Verifier notes: {'; '.join(reasons)}." if reasons else "")]
    md += ["", "## 4. Double paths (capabilities with ≥ 2 implementations)", ""] + [f"- **{c['name']}** ({c['capability_id']}): " + ", ".join(f"`{i}`" for i in c["implementations"]) for c in caps if c.get("is_double_path")]
    md += ["", "## 5. Entry points the audit assumed", "", "Check them – a missing entry point means falsely dead code.", ""]
    ep = OUT / "graph/entrypoints.json"
    if ep.exists(): md += [f"- `{e}`" for e in json.loads(ep.read_text())]
    md += ["", "## 6. Blind spots", "", "- dynamic calls from strings / DB / configuration loaded at runtime", "- reflection, `__call`, plugins", "- routes registered outside the code",
           "- `clients/*` overlays are grepped for string references (pass 1 and pass 5) but are not part of the call graph",
           "", "**Recommendation:** before deleting anything with status `candidate`/`needs_human`, deploy tombstones for 30 days in production.", ""]
    md += spend_section() + triage_section(merged) + twins_section() + collisions_section()
    for extra in sorted((OUT / "REPORT.sections").glob("*.md")) if (OUT / "REPORT.sections").exists() else []:
        md += ["", extra.read_text().rstrip(), ""]   # hand-written sections of the director (aigiscode signals, narrative)
    (OUT / "REPORT.md").write_text("\n".join(md)); print("REPORT.md written")


def human_question(r):
    k = r["kind"]; cand = r.get("canonical_candidate")
    if k == "dead_symbol": return "Is this symbol reached dynamically (string reference, reflection, route or job name from the database, a client overlay)? If not, it can go."
    if k in ("clone_exact", "clone_semantic", "variant_mergeable"):
        d = r.get("behavior_differences") or []
        return (f"Can the copies be merged into `{cand}`?" if cand else "Can the copies be merged into one implementation?") + (f" {len(d)} behavior difference(s) are listed – decide whether they are intended." if d else "")
    if k == "double_path": return "Two implementations of one capability: which one is canonical, and does the other carry behaviour the canonical one lacks?"
    if k == "redundant_layer": return "Does this layer add a contract (authorization, validation, transaction boundary) or only forward arguments?"
    if k == "dead_branch": return "Is the guard that makes this branch unreachable intended, or is the branch the bug?"
    if k == "unused_param": return "Is the parameter part of a public or plugin-facing signature that callers outside this repository rely on?"
    if k == "repeated_validation": return "Is the repeated validation a defence-in-depth decision or a copy?"
    if k == "unused_dependency": return "Is the composer package used by a client overlay, a deploy script or a service provider the graph does not see?"
    return "Decide whether the finding holds; the model and the deterministic checks disagree or lack a second source."


def _side_file(p): return re.search(r"_(rejected\d*|empty|gaps|missing)\.json$", p.name) is not None


def spend_section():
    """§7: token / time / cost per pass and worker model, from the meta block of every worker answer (push-backs and second looks included)."""
    rows = collections.defaultdict(collections.Counter)
    for pd in sorted((OUT / "results").glob("pass[0-9]")):
        for p in pd.glob("*.json"):
            if _side_file(p): continue
            try: m = json.loads(p.read_text()).get("meta") or {}
            except (json.JSONDecodeError, AttributeError): continue
            u = m.get("usage") or {}; c = rows[(pd.name, m.get("model", "?"), m.get("runner", "?"))]
            c["answers"] += 1; c["output_tokens"] += u.get("output_tokens", 0) or 0; c["cost_usd"] += u.get("cost_usd") or 0; c["seconds"] += m.get("elapsed_s") or 0
            c["input_tokens"] += (u.get("input_tokens", 0) or 0) + ((u.get("cache_creation_input_tokens", 0) or 0) + (u.get("cache_read_input_tokens", 0) or 0) if m.get("runner") == "claude" else 0)
    for prefix, label in (("t", "triage"), ("o", "triage second opinion")):
        for p in (OUT / "triage/results").glob(f"{prefix}_*.json"):
            m = json.loads(p.read_text()).get("meta") or {}; u = m.get("usage") or {}; c = rows[(label, m.get("model", "?"), "claude")]
            c["answers"] += 1; c["output_tokens"] += u.get("output_tokens") or 0; c["cost_usd"] += m.get("cost_usd") or 0; c["seconds"] += (m.get("duration_ms") or 0) / 1000
            c["input_tokens"] += sum(u.get(k) or 0 for k in ("input_tokens", "cache_creation_input_tokens", "cache_read_input_tokens"))
    md = ["## 7. Token and time spend per pass and worker", "", "Input tokens are the provider's own counters (for `claude -p` including cache writes and reads; for agy the billed input). Cost is the API list price the Claude CLI reports; the Max subscription and the Google AI Ultra quota were what actually paid.", "",
          "| pass | model | runner | answers | input tokens (M) | output tokens (M) | worker hours | nominal cost USD |", "|---|---|---|---|---|---|---|---|"]
    tot = collections.Counter()
    for (ps, model, runner), c in sorted(rows.items()):
        md.append(f"| {ps} | {model} | {runner} | {c['answers']} | {c['input_tokens'] / 1e6:.1f} | {c['output_tokens'] / 1e6:.2f} | {c['seconds'] / 3600:.1f} | {c['cost_usd']:.0f} |"); tot.update(c)
    md.append(f"| **total** | | | {tot['answers']} | {tot['input_tokens'] / 1e6:.1f} | {tot['output_tokens'] / 1e6:.2f} | {tot['seconds'] / 3600:.1f} | {tot['cost_usd']:.0f} |")
    return [""] + md + [""]


def triage_section(merged):
    """§8: the triage layer (Sonnet 5.5 medium, Opus second opinion) and the ranking of kept findings by severity, then LOC saved."""
    tp = OUT / "triage/triage.jsonl"
    if not tp.exists(): return []
    by_id = {r["id"]: r for r in merged}; rows = {}
    for r in read_jsonl(tp): rows[r["id"]] = r
    second = {r["id"]: r for r in read_jsonl(OUT / "triage/second_opinion.jsonl")} if (OUT / "triage/second_opinion.jsonl").exists() else {}
    v = collections.Counter(r["verdict"] for r in rows.values()); sev = collections.Counter((r["verdict"], r.get("severity")) for r in rows.values())
    md = ["## 8. Triage (independent reviewer per finding) and ranking", "",
          f"Triaged findings: {len(rows)} – keep {v['keep']}, reject {v['reject']}, unsure {v['unsure']}; script-rated clone classes: {sum(1 for r in rows.values() if r.get('model') == 'script')}.",
          f"Kept by severity: S1 {sev[('keep', 'S1')]}, S2 {sev[('keep', 'S2')]}, S3 {sev[('keep', 'S3')]}. Second opinions (Opus 5.5): {len(second)}" +
          (f", verdict agreement {sum(1 for i, s in second.items() if i in rows and s['verdict'] == rows[i]['verdict'])}/{len(second)}." if second else "."),
          "A finding both reviewers rejected was not sent to pass 5. Full table: `triage/triage.jsonl`, `TRIAGE.md`, `triage/agreement.txt`.", "",
          "### Ranking of kept findings (severity, then LOC saved) – top 150", "", "| rank | id | kind | sev | difficulty | ~LOC | status | title |", "|---|---|---|---|---|---|---|---|"]
    kept = [(r, by_id[i]) for i, r in rows.items() if r["verdict"] == "keep" and i in by_id and by_id[i]["status"] != "rejected"]
    kept.sort(key=lambda x: ({"S1": 0, "S2": 1, "S3": 2}.get(x[0].get("severity"), 3), -x[1]["estimated_loc_saved"]))
    for n, (t, f) in enumerate(kept[:150], 1):
        md.append(f"| {n} | {f['id']} | {f['kind']} | {t.get('severity')} | {t.get('difficulty')} | {f['estimated_loc_saved']} | {f['status']} | {f['title'].replace('|', '/')[:90]} |")
    return [""] + md + [""]


def twins_section():
    """§9: pass-3 singletons the worker matched to an existing registry finding (report material, no evidence quotes)."""
    tp = OUT / "singleton_twins.jsonl"
    if not tp.exists(): return []
    rows = read_jsonl(tp); c = collections.Counter(r["has_twin_in_registry"] for r in rows)
    md = ["## 9. Singleton symbols with a twin already in the registry (pass 3)", "",
          f"{c['true']} singletons were matched to an existing finding, {c['unsure']} are unsure; they carry no verbatim evidence and are not registry findings. Full list: `singleton_twins.jsonl`. First 80 confirmed matches:", ""]
    md += [f"- `{r['symbol']}` → {r.get('registry_match') or '?'}" for r in rows if r["has_twin_in_registry"] == "true"][:80]
    return [""] + md + [""]


def collisions_section():
    """§10: capability names the fuzzy merge (ratio ≥ 0.85) joined – the owner decides whether they are one capability."""
    cp = OUT / "capability_collisions.json"
    if not cp.exists(): return []
    rows = json.loads(cp.read_text())
    md = ["## 10. Capability name collisions to review", "", f"{len(rows)} fuzzy-name merges and merge suggestions from the capability registry (`capability_collisions.json`). First 60:", ""]
    for r in rows[:60]:
        if isinstance(r, (list, tuple)) and len(r) == 3 and isinstance(r[1], (list, tuple)): md.append(f"- {r[0]}: " + " / ".join(map(str, r[1])) + f" – {r[2]}")
        else: md.append(f"- {r}")
    return [""] + md + [""]

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--capabilities"); ap.add_argument("--calibration", action="store_true")
    ap.add_argument("--report", action="store_true"); ap.add_argument("--status", action="store_true")
    a = ap.parse_args()
    if a.capabilities: return merge_capabilities(a.capabilities)
    if a.calibration: return calibration()
    if a.report: dedup(); return report()
    if a.status: dedup(); return status()
    dedup()

if __name__ == "__main__":
    main()
