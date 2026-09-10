# Supplemental orphan evidence and the Draivix boundary counterexample

Backend orphan analysis previously ignored supplemental directory/read failures,
invalid text and files above 1 MiB. It now reports bounded typed coverage and
defers the affected judgment while retaining independent findings. Its content
fingerprint is part of snapshot identity and final publication validation.
See the [contract](BACKEND_ORPHAN_COVERAGE_CONTRACT.md).

Two actual parts of the preserved Draivix source provide useful evidence:

- `tools/espo_migration`: all 100 PHP sources have complete native and secondary
  coverage. The supplemental scan reads 550 files / 5,154,121 bytes and records
  three existing oversized JSON reports. Coverage is explicitly incomplete, CLI
  exits 1, and MCP carries the same gap status. No migration script was executed.
- `app/Modules/SmsService`: 40 PHP sources and 16 supplemental JSON files /
  19,013 bytes. The first observation exposed an unjustified orphan candidate
  for `SmsServiceServiceProvider`. Its actual discovery lives outside the slice:
  `bootstrap/providers.php:47–49` and `ModuleProviderDiscovery.php:94–97` construct
  and register module providers. The final analyzer reports `deferred_boundary`
  and removes that orphan candidate. This is a generic scope repair, with no
  SmsService-specific exception. The module source is unchanged.

The final production build passed without warnings in 54.11 s. Its full preserved
Draivix audit completed in 81.04 s with 3,566,012 KiB peak RSS. Entire semantic,
dependency/evidence graphs, contract inventory and secondary scanner remain
byte-identical to the preceding version. The broad corpus still defers dead-code
analysis for incomplete native coverage; the new status makes that explicit.
There remain 17,067 supported files, 132,657 symbols, 331,060 edges, 19 strong
and 22 total cyclic components. These are individual observations, not a benchmark.

Thirteen full-corpus MCP requests completed successfully, with a first useful
response in 18.73 s and matching indexed/artifact inputs. Five additional MCP
requests observed the real migration scope and its supplemental gaps. A separate
self-audit and source-integrity evidence are retained. Tests and CI remain paused.
The new combined gap/boundary regression and two updated filesystem-backed orphan
regressions were not run or compiled as test targets. Permission/missing-input,
invalid-UTF8, file-growth and concurrent-change branches remain unverified at runtime.

To preserve evidence while making room for this audit, 32 closed working binaries
were archived: 2,788,804,560 bytes became a 748,811,840-byte archive. `tar --compare`
passed with no output, and every original SHA-256 was checked again before its
unpacked copy was removed. Current and preceding binaries remain available;
published analysis artifact families were preserved. The receipt lists archive
members, hashes and the archive location for exact restoration.

[Machine evidence](2026-09-10-supplemental-coverage-evidence.json) records builds,
artifact comparisons, source scopes, actual RPCs and archive provenance. Report
filenames containing personal identifiers are represented by hashes in this
receipt; complete gap records remain in the local raw artifacts. The separate
[current Draivix quality assessment](2026-09-10-draivix-quality.md) gives the
actionable implementation priorities. Overall Q01–Q12 acceptance remains open.
