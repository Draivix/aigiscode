#!/usr/bin/env bash
# metrics.sh – přepočítá metriky a přidá řádek do out/metrics_history.csv
# Spouštět: po kole 0 (baseline) a po každém dalším kole. Nic v repu nemění.
# Nástroje žijí v lean-audit/out/tools (samostatný composer), ne v repu.
set -euo pipefail
LABEL="${1:-unlabeled}"            # např. baseline, after_pass1, after_pass2…
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/out"; TOOLS="$OUT/tools/vendor/bin"; DET="$OUT/deterministic"
SRC="${SRC_DIRS:-app routes config database packages tools}"   # backend PHP roots of this repo
PHPSTAN="${PHPSTAN_BIN:-vendor/bin/phpstan}"   # the repo's own phpstan (larastan lives there); extensions come from out/tools
mkdir -p "$DET"

echo "== $LABEL ==" >&2

# The source tree is one immutable commit for the whole audit (document-only), so the tool runs below are computed once
# (on the compute server, owner rule: no CPU-bound jobs on the workstation) and reused while their artifact exists.
# Set METRICS_RECOMPUTE=1 to force a recomputation.
reuse() { [ -z "${METRICS_RECOMPUTE:-}" ] && [ -s "$1" ]; }

# LOC
if reuse "$DET/cloc.csv"; then LOC=$(awk -F, '/PHP/{print $5}' "$DET/cloc.csv"); else
LOC=$(cloc --quiet --csv --include-lang=PHP $SRC 2>/dev/null | tee "$DET/cloc.csv" | awk -F, '/PHP/{print $5}' || echo 0); fi

# duplicity (jscpd je nejjednodušší na instalaci: npx jscpd; PMD CPD alternativa)
reuse "$DET/jscpd/jscpd-report.json" || npx --yes jscpd --silent --reporters json --output "$DET/jscpd" --min-tokens 40 --format php --max-size 3mb --ignore '**/vendor/**,**/node_modules/**,**/tests/**,**/tests_fast/**,**/Tests/**,**/Fixtures/**,**/fixtures/**' $SRC >/dev/null 2>&1 || true
DUP_PCT=$(jq -r '.statistics.total.percentage // 0' "$DET/jscpd/jscpd-report.json" 2>/dev/null || echo 0)
DUP_GROUPS=$(jq -r '.duplicates | length' "$DET/jscpd/jscpd-report.json" 2>/dev/null || echo 0)

# mrtvý kód – PHPStan s dead-code-detector + unused-public (config v out/tools/phpstan.neon, level dle baseline repa)
reuse "$DET/phpstan.json" || "$PHPSTAN" analyse -c "$OUT/tools/phpstan.neon" -a "$OUT/tools/vendor/autoload.php" --error-format=json --no-progress --memory-limit=8G > "$DET/phpstan.json" 2>"$DET/phpstan.err" || true
DEAD=$(jq '[.files[].messages[] | select((.identifier // "")|test("shipmonk\\.dead|public\\..*\\.unused"))] | length' "$DET/phpstan.json" 2>/dev/null || echo 0)

# composer-unused
reuse "$DET/composer-unused.json" || "$TOOLS/composer-unused" --output-format=json --no-progress --no-ansi 2>/dev/null | sed -n '/^{/,$p' > "$DET/composer-unused.json" || true
UNUSED_PKG=$(jq '.["unused-packages"] | length' "$DET/composer-unused.json" 2>/dev/null || echo 0)

# deptrac (jen pokud má repo config)
DEPTRAC=0
if [ -f deptrac.yaml ]; then "$TOOLS/deptrac" analyse --formatter=json --output="$DET/deptrac.json" >/dev/null 2>&1 || true; DEPTRAC=$(jq '.Report.Violations // 0' "$DET/deptrac.json" 2>/dev/null || echo 0); fi

# složitost
reuse "$DET/phpmetrics.json" || "$TOOLS/phpmetrics" --report-json="$DET/phpmetrics.json" $(echo $SRC | tr ' ' ',') >/dev/null 2>&1 || true
CCN=$(jq '[.[] | .ccn // empty] | add / length' "$DET/phpmetrics.json" 2>/dev/null || echo 0)

# nálezy z registru
CONF=0; NH=0
if [ -f "$OUT/registry.merged.jsonl" ]; then
  # top-level status only: a grep over the line would also count the "status" entries inside history[] (inflated after_pass5 once)
  read -r CONF NH < <(python3 -I -c 'import json,sys,collections; c=collections.Counter(json.loads(l)["status"] for l in open(sys.argv[1]) if l.strip()); print(c["confirmed"], c["needs_human"])' "$OUT/registry.merged.jsonl")
fi

CSV="$OUT/metrics_history.csv"
[ -f "$CSV" ] || echo "ts,label,loc,dup_pct,dup_groups,dead_symbols,unused_packages,deptrac_violations,avg_ccn,confirmed_findings,needs_human" > "$CSV"
echo "$(date -u +%FT%TZ),$LABEL,$LOC,$DUP_PCT,$DUP_GROUPS,$DEAD,$UNUSED_PKG,$DEPTRAC,$CCN,$CONF,$NH" | tee -a "$CSV"
