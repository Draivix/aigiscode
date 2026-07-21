# Full Draivix Analysis — Shit-Code Removal Ledger

**Date:** 2026-07-21
**Target:** `/home/david/Work/Programming/draivix` (Laravel + Cycle ORM + Inertia/Vue + Capacitor monolith)
**Mode:** ANALYSIS ONLY — no draivix code changes. Every claim is verified against source before it lands here.
**Instrument:** aigiscode (this repo), 2026-07-21 binary — channels + http_calls contracts, `$this->prop`/docblock/`app()` receiver resolution, determinism-purged outputs, fast-load.

## Goal

Find as much removable/simplifiable code as honestly possible: legacy shims,
converters, normalizers, duplicate mechanisms, dead paths. The deliverable is
`09-removal-candidates.md` — a ranked ledger where every entry carries
evidence (file:line), a verification note (a human actually read the code),
an impact estimate (blast radius), and a verdict: `delete`, `merge`,
`simplify`, `keep`, or `owner-decision`.

## Method

1. Fresh `aigiscode analyze` on draivix (as configured by its own
   `.aigiscode/scan.json` — note: it excludes `Commands/` dirs; blind spot
   documented in `01-inventory.md`).
2. Per-axis mining of the artifact family (`deterministic-findings`,
   `review-surface`, `semantic-graph`, `contract-inventory`, `convergence-history`).
3. **Every candidate is hand-verified against source.** Detector output is a
   lead, not a verdict. False positives are recorded as such in
   `10-tool-fit-log.md` — that file is also the honest account of where the
   tool helped and where it wasted time.

## Documents (written incrementally, in order)

| File | Axis |
|---|---|
| `01-inventory.md` | Scale, languages, scan config, artifact baseline |
| `02-dead-code-and-orphans.md` | Dead functions, unused imports, orphan files |
| `03-legacy-and-conversion.md` | Legacy markers, EspoCRM shims, conversion machinery |
| `04-normalizers-converters.md` | Normalizer/converter/mapper inventory + overlap |
| `05-duplicate-mechanisms.md` | Dual paths for the same concern |
| `06-god-classes-hotspots.md` | Oversized files, logic concentration |
| `07-hardwiring-config.md` | Hardcoded URLs, env access, literal sprawl |
| `08-cycles-and-coupling.md` | The entity-core knot, job cycles |
| `09-removal-candidates.md` | **The ranked removal ledger (main deliverable)** |
| `10-tool-fit-log.md` | How aigiscode fit in hand: wins, noise, gaps, fixes |

## Status — COMPLETE (2026-07-21)

- [x] Fresh analysis (≈54s pipeline)
- [x] 01-inventory — 7,688 analyzed files; 49% of findings = scan-config echo
- [x] 02-dead-code-and-orphans — 100% verified: 4 files/orphans + 13 methods
- [x] 03-legacy-and-conversion — 1 dead alias + 3 dead constants; ModuleRegistry comment debt; EspoMigration owner decision
- [x] 04-normalizers-converters — 42 classes; 1 merge candidate; 1 guard FP
- [x] 05-duplicate-mechanisms — fan-out = architecture; ServiceProvider sprawl = idiom; XLSX dual-path found
- [x] 06-god-classes-hotspots — 6 survivors post-discount; 8.5k-line evidence service is top split candidate
- [x] 07-hardwiring-config — 22-URL config-centralization batch
- [x] 08-cycles-and-coupling — knot referenced to existing plan; 7 small pairs ranked
- [x] 09-removal-candidates — **the ledger: A (immediate deletions) through F (comment debt)**
- [x] 10-tool-fit-log — instrument review: precision good, suppressible noise mapped, gaps listed

Headline: the repo is well-kept; the verified immediate pool is 3 files +
13 methods + 1 alias + 3 constants + 1 test-coupled pair, one 22-finding
config batch, and three suppression decisions — plus a ranked restructure lane.
