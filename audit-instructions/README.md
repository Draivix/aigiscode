# lean-audit

Read-only audit Draivix backendu před verzí 2.0. Ředitel = Claude Code (čte `DIRECTOR.md`), pracovník = Gemini 4 Argon přes API (`scripts/run_gemini.py`), pravda = deterministické nástroje + aigiscode graf.

## Jak to spustit (člověk)

```bash
# 1. zkopíruj složku do kořene repa
cp -r lean-audit /path/to/draivix-backend/
cd /path/to/draivix-backend

# 2. nástroje žijí MIMO repo composer (nic se v repu nemění)
mkdir -p lean-audit/out/tools && cd lean-audit/out/tools
composer init -n --name draivix/lean-audit-tools
composer require --dev nikic/php-parser phpstan/phpstan shipmonk/dead-code-detector tomasvotruba/unused-public \
  icanhazstring/composer-unused qossmic/deptrac phpmetrics/phpmetrics
# phpstan.neon: zkopíruj z repa a přidej extensiony dead-code-detector + unused-public
cd ../../..
pip install google-genai numpy hnswlib   # (hnswlib volitelné)
npm i -g jscpd                           # nebo PMD CPD

# 3. hook – zaregistruj v .claude/settings.json (viz hlavička hooks/block-writes.sh)

# 4. aigiscode artefakty
aigiscode analyze . --output-dir lean-audit/out/graph

# 5. spusť Claude Code v kořeni repa a řekni mu:
#    "Přečti lean-audit/DIRECTOR.md a jsi ředitel auditu. Začni kolem 0."
```

## Pořadí kol (salám)
0. baseline + příprava (`metrics.sh baseline`, `extract_symbols.php`, entrypointy → **potvrdit člověkem**)
1. deterministika (bez LLM) → `verify.py` importuje do registru
2. vertikální (G): entrypoint + call chain
3. horizontální (G): sémantické sousedy (`embed_cluster.py` → `build_batches.py --pass 3`)
4. doménové (G): zóny + capability registr
5. ověření: nezávislý verifier (`prompts/verify_independent.md`), `merge_registry.py --report`

Po každém kole: `metrics.sh after_passN` a `merge_registry.py --status`. Konec, když se dvě kola po sobě pohnou o < 3 % (viz DIRECTOR.md §7.2).

## Co tu NENÍ a schválně
- žádný refaktor, žádný Rector bez `--dry-run`, žádné PR. Výstup je `out/REPORT.md` a `out/registry.merged.jsonl`.
- `agy -p` (Antigravity CLI headless) – má v 10/2026 otevřené bugy v non-TTY režimu (visí / zahodí stdout). Používá se API.

## Co si musíš ověřit sám (ředitel to nemůže vědět)
- názvy klíčů v `semantic-graph.json` a `repository-topology.json` (uprav `build_batches.py::load_graph/load_zones`)
- seznam entrypointů (cron, workery, webhooky, pluginové hooky) – nejdůležitější vstup
- `SRC_DIRS` v `metrics.sh` a `--root` v `extract_symbols.php`
- PHPStan level a baseline repa
