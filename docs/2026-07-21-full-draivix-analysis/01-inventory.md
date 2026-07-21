# 01 — Inventory & Baseline

Fresh analysis: **2026-07-21**, aigiscode binary with channels/http_calls/property-resolution/determinism fixes.
Pipeline: Scan 0.2s · Structure 0.35s · Parse 3.4s · Resolve 25.4s · Analyze 24.3s (≈54s wall on a loaded machine).

## Corpus

| Language | Files |
|---|---|
| PHP | 6,221 |
| TypeScript | 1,458 |
| JavaScript | 9 |
| Data (json/md/css/assets) | 2,063 |
| Unsupported (sh/go/swift/…) | 109 |

Note: "Data" and "Unsupported" are now separate labels (this session's change);
previous runs reported 2,121 "Unsupported", which was ~95% assets.

## Scan configuration (their own `.aigiscode/scan.json`) — blind spots

- `include_path_prefixes`: `app`, `routes`, `config`, `resources/js`
- `ignored_dir_names`: `Tests, tests, tests_fast, __tests__, Fixtures, fixtures, __mocks__, lang, database, **Commands, commands**`

Consequences the ledger must respect:

1. **Console commands are invisible** (~hundreds of `app/Modules/*/Console/Commands/*.php`).
   Everything discovered in this analysis is scoped to the runtime web slice.
   **1,241 of 2,544 findings (49%) are `Boundary-truncated file`** — files whose
   callers may live in the excluded dirs. That is a *config artifact*, not code
   quality, and it inflates "High" counts.
2. Tests are out of slice — dead-code findings there are unreachable by design.
3. Recommendation for the owner (not acted on): drop `Commands`/`commands`
   from `ignored_dir_names` when console coverage is wanted.

## Findings baseline (review-surface)

- **2,544 visible findings** — High 1,112 · Medium 59 · Low 1,373
- Precision: exact 1,241 · heuristic 982 · certain 255 · modeled 66
- Families: Graph 2,364 · Hardwiring 134 · Security 28 · DeadCode 18

Top titles:

| Count | Title | Read |
|---|---|---|
| 1,241 | Boundary-truncated file | config artifact, see above |
| 633 | Algorithmic complexity hotspot | mostly heuristic; needs ranking, not counting |
| 255 | Layer contract violation | entities→services / core→modules; real, doctrine-backed |
| 155 | Sanctioned path bypass | env/container lookups outside boundaries |
| 109 | Repeated literal | hardwiring candidates for config/constants |
| 22 | Hardcoded network | external API URLs in source |
| 20 | Unsafe HTML output API | innerHTML-family sinks (mitigated per prior audits — verify) |
| 19 | Abstraction sprawl | candidate dual-path clusters |
| 14 | Duplicate mechanism | explicit dual-path findings |
| 14 | Unused private function | dead code |
| 10 | Bottleneck file | coupling hubs |
| 8 | Hand-rolled parsing | homegrown parsers |

Contract inventory: routes 432/560 (unique/occurrences) · hooks 21/279 ·
config_keys 303/500 · env_keys 450/573 · symbolic_literals 922/1856 ·
http_calls 336/412 (new) · channels 5 (new).

Convergence vs previous run: +99 new, 84 resolved, 0 worsened — repo is
roughly flat; the tool's own detector changes explain most of the delta.

## Working rule for the ledger

With 49% of findings being scan-config echo, the effective pool is ~1,300.
Within it, triage order: `certain`/`modeled` precision first, `heuristic`
only where the family is a named removal axis (duplicates, sprawl, legacy).
Every removal candidate gets a source read + blast-radius check before it is
listed in `09`.
