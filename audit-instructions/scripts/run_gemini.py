#!/usr/bin/env python3
"""
run_gemini.py – pošle jeden batch Gemini 4 Argon přes API a uloží syrovou odpověď.

Proč API a ne `agy -p`: Antigravity CLI má v říjnu 2026 otevřené bugy, kdy v non-TTY
režimu (subprocess, pipe) visí nebo tiše zahodí stdout. API dává navíc vynucené
JSON schéma (response_schema) a žádné nástroje → G fyzicky nemůže nic změnit.

Použití:
  python3 run_gemini.py --pass 2 --batch 0017
  python3 run_gemini.py --pass 3 --batch c_00412 --pushback missing_evidence --attempt 2
  python3 run_gemini.py --selftest

Env:
  GEMINI_API_KEY   (nebo GOOGLE_API_KEY)
  LEAN_MODEL       (default: gemini-4-argon)
  LEAN_OUT         (default: lean-audit/out)
  LEAN_MAX_INPUT_TOKENS (default: 450000) – batch nad limit se odmítne, nic se neposílá
"""
import argparse, json, os, sys, time, hashlib, pathlib, datetime

try:
    from google import genai
    from google.genai import types
except ImportError:
    sys.exit("pip install google-genai")

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = pathlib.Path(os.environ.get("LEAN_OUT", ROOT / "out"))
MODEL = os.environ.get("LEAN_MODEL", "gemini-4-argon")
MAX_IN = int(os.environ.get("LEAN_MAX_INPUT_TOKENS", "450000"))

PROMPTS = ROOT / "prompts"
SCHEMAS = ROOT / "schemas"

PASS_PROMPT = {2: "pass2_vertical.md", 3: "pass3_horizontal.md", 4: "pass4_domain.md", 5: "verify_independent.md"}
PASS_SCHEMA = {2: "findings.schema.json", 3: "findings.schema.json", 4: "findings.schema.json", 5: "verification.schema.json"}


def load(p): return pathlib.Path(p).read_text(encoding="utf-8")


def build_prompt(pass_no, batch, pushback=None, attempt=1, rejected=None):
    parts = [load(PROMPTS / "common_rules.md"), "\n\n---\n\n", load(PROMPTS / PASS_PROMPT[pass_no])]
    if pushback:
        pb = load(PROMPTS / "push_back.md")
        pb = pb.replace("{{reason}}", pushback).replace("{{attempt}}", str(attempt))
        pb = pb.replace("{{rejected_items}}", json.dumps(rejected or [], ensure_ascii=False, indent=1))
        parts += ["\n\n---\n\n", pb]
    parts += ["\n\n---\n\n# BATCH INPUT (JSON)\n", json.dumps(batch, ensure_ascii=False)]
    return "".join(parts)


def strip_schema_for_gemini(schema):
    """Gemini response_schema nepodporuje $schema/$ref/title – odstraníme a rozbalíme jednoduché $ref."""
    defs = schema
    def walk(node):
        if isinstance(node, dict):
            node = {k: v for k, v in node.items() if k not in ("$schema", "title", "$id")}
            if "$ref" in node:
                # jen jednoduché "#/properties/x"
                path = node["$ref"].lstrip("#/").split("/")
                target = defs
                for p in path: target = target[p]
                return walk(target)
            return {k: walk(v) for k, v in node.items()}
        if isinstance(node, list):
            return [walk(x) for x in node]
        return node
    return walk(schema)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pass", dest="pass_no", type=int)
    ap.add_argument("--batch")
    ap.add_argument("--pushback", help="reason code: missing_evidence|too_thin|vague|skimmed|hallucinated_symbol")
    ap.add_argument("--attempt", type=int, default=1)
    ap.add_argument("--rejected-file", help="JSON se seznamem odmítnutých nálezů pro push-back")
    ap.add_argument("--temperature", type=float, default=0.2)
    ap.add_argument("--thinking", default="high", choices=["low", "medium", "high"])
    ap.add_argument("--shuffle", action="store_true", help="second look: promíchá pořadí souborů v batchi")
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()

    client = genai.Client()  # čte GEMINI_API_KEY / GOOGLE_API_KEY

    if a.selftest:
        r = client.models.generate_content(
            model=MODEL, contents="Return JSON {\"ok\": true, \"model\": \"<your model name>\"}",
            config=types.GenerateContentConfig(response_mime_type="application/json", temperature=0))
        print(r.text); return

    if not (a.pass_no and a.batch): sys.exit("--pass a --batch jsou povinné")

    batch_path = OUT / "batches" / f"pass{a.pass_no}" / f"{a.batch}.json"
    batch = json.loads(load(batch_path))
    if a.shuffle and "files" in batch:
        import random; random.Random(a.attempt).shuffle(batch["files"])

    rejected = json.loads(load(a.rejected_file)) if a.rejected_file else None
    prompt = build_prompt(a.pass_no, batch, a.pushback, a.attempt, rejected)

    # tokenový strop – nikdy neposílat víc, než se vejde i s thinkingem a výstupem
    cnt = client.models.count_tokens(model=MODEL, contents=prompt).total_tokens
    if cnt > MAX_IN:
        sys.exit(f"BATCH TOO BIG: {cnt} > {MAX_IN} tokens – rozděl batch (build_batches.py --split {a.batch})")

    schema = strip_schema_for_gemini(json.loads(load(SCHEMAS / PASS_SCHEMA[a.pass_no])))
    cfg = types.GenerateContentConfig(
        temperature=a.temperature,
        response_mime_type="application/json",
        response_schema=schema,
        thinking_config=types.ThinkingConfig(thinking_level=a.thinking),
        max_output_tokens=65536,
        tools=[],  # ŽÁDNÉ nástroje – G nemůže číst ani psát mimo to, co dostal
    )

    res_dir = OUT / "results" / f"pass{a.pass_no}"; res_dir.mkdir(parents=True, exist_ok=True)
    suffix = f"_pb{a.attempt}" if a.pushback else (f"_look{a.attempt}" if a.attempt > 1 else "")
    out_path = res_dir / f"{a.batch}{suffix}.json"
    if out_path.exists():
        sys.exit(f"Result already exists, refusing to overwrite: {out_path}")

    delay = 30
    for try_no in range(3):
        t0 = time.time()
        try:
            r = client.models.generate_content(model=MODEL, contents=prompt, config=cfg)
            text = r.text
            data = json.loads(text)
            meta = {
                "model": MODEL, "pass": a.pass_no, "batch": a.batch, "attempt": a.attempt,
                "pushback": a.pushback, "input_tokens": cnt,
                "usage": getattr(r, "usage_metadata", None) and r.usage_metadata.model_dump() if hasattr(r.usage_metadata, "model_dump") else str(getattr(r, "usage_metadata", "")),
                "elapsed_s": round(time.time() - t0, 1),
                "prompt_sha256": hashlib.sha256(prompt.encode()).hexdigest(),
                "ts": datetime.datetime.utcnow().isoformat() + "Z",
            }
            out_path.write_text(json.dumps({"meta": meta, "result": data}, ensure_ascii=False, indent=1), encoding="utf-8")
            print(f"OK {out_path}  findings={len(data.get('findings', []))}  in={cnt}  {meta['elapsed_s']}s")
            return
        except (json.JSONDecodeError, ValueError) as e:
            err = f"invalid JSON from model: {e}"
        except Exception as e:  # rate limit, timeout, 5xx
            err = f"{type(e).__name__}: {e}"
        print(f"try {try_no+1} failed: {err}", file=sys.stderr)
        (OUT / "INCIDENTS.md").open("a").write(f"- {datetime.datetime.utcnow().isoformat()}Z pass{a.pass_no}/{a.batch} try{try_no+1}: {err}\n")
        time.sleep(delay); delay *= 2
    sys.exit("FAILED after 3 tries")


if __name__ == "__main__":
    main()
