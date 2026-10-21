#!/usr/bin/env bash
# One tombstone insertion MR for one group of dead_symbol targets.
#   tombstone_mr.sh <slug> <group-regex>        e.g. tombstone_mr.sh aichat '^module-AiChat$'
# Cuts ../draivix-tombstone-<slug> off origin/main with scripts/wt.sh, runs tombstone_insert.php
# with --write for the groups matching the regex, commits exactly the written files, pushes, opens
# the MR with a file → finding table, arms auto-merge and prints the MR iid. The insertion report
# is kept at lean-audit/out/tombstones/insert-<slug>.json for tombstone_record.py.
set -euo pipefail
SLUG="${1:?slug}"; FILTER="${2:?group regex}"
SHARED="${DRAIVIX_SHARED:-/home/david/Work/Programming/draivix}"
KIT="$(cd "$(dirname "$0")" && pwd)"
OUT="${LEAN_OUT:-/home/david/Work/Programming/draivix-lean-audit-2026-10/lean-audit/out}/tombstones"
WT="$(dirname "$SHARED")/draivix-tombstone-$SLUG"
BRANCH="chore/tombstone-$SLUG"
REPORT="$OUT/insert-$SLUG.json"

cd "$SHARED" && git fetch -q origin
[ -d "$WT" ] || bash scripts/wt.sh new "$BRANCH" >/dev/null
php -d memory_limit=2G "$KIT/tombstone_insert.php" --root="$WT" --targets="$OUT/targets.jsonl" \
    --filter="$FILTER" --write --report="$REPORT" > "$OUT/insert-$SLUG.summary.json"
python3 -I - "$REPORT" <<'EOF' > "$OUT/insert-$SLUG.files"
import json, sys
rows = [r for r in json.load(open(sys.argv[1])) if r["action"] == "inserted"]
print("\n".join(sorted({r["file"] for r in rows})))
EOF
FILES_N=$(grep -c . "$OUT/insert-$SLUG.files" || true)
[ "$FILES_N" -gt 0 ] || { echo "nothing inserted for $FILTER"; exit 1; }
INS_N=$(python3 -I -c "import json,sys;print(sum(r['action']=='inserted' for r in json.load(open(sys.argv[1]))))" "$REPORT")
GROUPS=$(python3 -I -c "import json,sys;print(', '.join(sorted({r['group'] for r in json.load(open(sys.argv[1])) if r['action']=='inserted'})))" "$REPORT")

cd "$WT"
xargs -a "$OUT/insert-$SLUG.files" git add --
git commit -q -F - <<EOF
chore(lean-audit): tombstones for $INS_N dead symbols ($SLUG)

One added line per method the lean audit (docs/lean-audit-2026-10, registry
rows of kind dead_symbol with status confirmed/candidate/needs_human) could not
reach from any static source: \\App\\Support\\Tombstone::hit(__METHOD__) tagged
with the finding id. No behaviour change; a hit in production rejects the
finding, no hit after the observation window deletes the method with its marker
(lean-audit/out/TOMBSTONES.md). Groups: $GROUPS. $FILES_N files.

Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
EOF
git push -q -u origin "$BRANCH" 2>/dev/null

DESC="$(python3 -I - "$REPORT" "$SLUG" "$INS_N" "$FILES_N" "$GROUPS" <<'EOF'
import json, sys
from collections import defaultdict
report, slug, ins, files, groups = sys.argv[1:6]
rows = [r for r in json.load(open(report)) if r["action"] == "inserted"]
by_file = defaultdict(list)
for r in rows:
    by_file[r["file"]].append(f"{r['method']} ({r['finding_id']}, {r['status']})")
print(f"""## What

{ins} added lines in {files} files ({groups}): `\\App\\Support\\Tombstone::hit(__METHOD__); // lean-audit F-…` as the first statement of every method the lean audit (`docs/lean-audit-2026-10`, `lean-audit/out/registry`) marks `dead_symbol` with status confirmed, candidate or needs_human. Nothing else in these files changed (parser-positioned text splice, `php -l` after write).

## Why

No static source (aigiscode call graph, PHPStan, grep over core and client overlays) reaches these methods; production traffic is the oracle for string-built dispatch and runtime plugins. One attributable line per symbol per worker per hour goes to the `tombstone` channel (`App\\Support\\Tombstone`, draivix!6330); `tombstone:report` aggregates. After the observation window (`lean-audit/out/TOMBSTONES.md`) a symbol with hits has its finding rejected, a symbol without hits is deleted together with its marker.

## Layer verdict

Core/module code, true for every tenant, no behaviour change, no metadata, no migration.

## Proof

The marker itself is covered by `tests/Fast/Support/TombstoneTest.php` (draivix!6330). This MR adds no behaviour; pipeline (PHPStan on changed files, Pint, fast tests) is the proof that the splice is syntactically and type-wise inert.

<details><summary>File → method (finding id, audit status)</summary>

| File | Methods |
|---|---|""")
for f in sorted(by_file):
    print(f"| `{f}` | {'; '.join(by_file[f])} |")
print("""
</details>

🤖 Generated with [Claude Code](https://claude.com/claude-code)""")
EOF
)"
URL=$(glab mr create --source-branch "$BRANCH" --target-branch main \
    --title "chore(lean-audit): tombstones for $INS_N dead symbols ($SLUG)" \
    --description "$DESC" --remove-source-branch --yes 2>&1 | grep -oE 'merge_requests/[0-9]+' | tail -1)
IID="${URL##*/}"
[ -n "$IID" ] || { echo "MR creation failed"; exit 1; }
glab mr merge "$IID" --auto-merge --remove-source-branch --yes >/dev/null 2>&1 || true
echo "MR $IID $BRANCH $INS_N insertions $FILES_N files report=$REPORT"
