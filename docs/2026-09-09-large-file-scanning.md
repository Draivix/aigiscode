# Large-file secondary scanning on Draivix

The secondary scanner no longer omits supported files above 150,000 bytes. On the
preserved Draivix corpus, all 18 former size gaps now reach the normal rule
pipeline: 17 enter AST scanning and one declaration file is lexically prefiltered.
All 9,497 preceding raw findings remain unchanged; 435 additional structural clues
are recorded. This closes the measured size-admission gap, not overall Q12 or audit
acceptance.

The [receipt](2026-09-09-large-file-scanning-evidence.json) records binary and source
identities, every formerly oversized file's result, complete artifact hashes,
source samples, CLI and MCP coverage, and measured time and memory. All 23,166
captured files still match the original inventory. This is the preserved capture,
not the current live Draivix tree: subsequent Mitel changes remain outside this
measurement. No original Draivix source was edited.

Files above the threshold run sequentially after smaller files finish their
parallel scan. Each eligible file keeps its complete AST, enclosing-loop context
and original line positions. The threshold now controls concurrency, not whether
rules run. Only one large secondary tree is live per scan invocation; individual
tree size, concurrent invocations and total analyzer memory remain unbounded by
this change. Existing language and lexical admission rules still apply. The
[coverage contract](SECONDARY_COVERAGE_CONTRACT.md) retains historical size-gap
fields for compatibility.

| Secondary coverage | Before | After |
| --- | ---: | ---: |
| Input files | 17,013 | 17,013 |
| AST-scanned files | 8,139 | 8,156 |
| Lexically prefiltered | 8,064 | 8,065 |
| Size omissions | 18 | 0 |
| Unsupported Vue files | 792 | 792 |

The additional clues comprise 89 complexity, 344 framework-misuse and two dangerous
API matches. Source inspection distinguishes their implications:

- `ChatService.php:2283` checks membership in `$summaries` inside a loop that grows
  that array. This is a real repeated collection scan; runtime impact still depends
  on input size. The assessment includes the exact operation site.
- `AccountingProductionEvidenceService.php:8436,8441` contains fallback `app(...)`
  lookups behind optional injected services. The scanner now exposes those calls;
  the relevant review is the injection/fallback boundary.
- `PortalDataService.php:7121,7124` checks fixed literal lists while normalizing
  boolean values. The raw catalog matches these calls, but the assessment does
  not promote those sites as expensive operations. Other pre-existing nested-loop
  evidence in that file remains.
- The two new dangerous-API clues are test code: clearing `document.body.innerHTML`
  to an empty string, and a Git command with escaped arguments. They are syntax
  matches, not evidence of newly discovered exploitable vulnerabilities.

The 2,597,200-byte `resources/js/types/entities.d.ts` receives
`no_family_prefilter_hit`. Two other formerly omitted files enter AST scanning
with zero findings: the large entity-schema migration and `ChatVoiceComposer.test.ts`.
None of those outcomes establishes universal semantic or security correctness.

The full updated audit took 84.03 seconds and peaked at 3,510,640 KiB RSS. Its
secondary scan took 19.246 seconds; the initial old-binary observation took 9.570
seconds for that phase. A subsequent old-binary run with full artifact publication
took 76.08 seconds, 3,506,216 KiB peak RSS and 10.018 seconds in the secondary scan.
These are individual shared-host observations, with
tracing enabled; the initial baseline used `--no-write`. Restoring this coverage
has an observed execution cost. The full semantic graph is byte-identical between
the published old and new runs.

Raw scanner, CLI, consolidated report and architecture overview agree on coverage.
A real MCP session reused the verified graph cache, first returned an explicit
indexing response, and served its useful overview after 45.16 seconds. Overview,
quality and coverage payloads carry the updated coverage, and all four queried
tools carry it in metadata with matching artifact/index inputs. Guard remains
`Block` and CLI exits 1: Vue coverage and the existing 81 recovered native sources,
792 scope-limited Vue extractions and 146 unsupported source files remain unresolved.
The self-audit also completed: 117 secondary inputs, 56 scanned, 61 prefiltered,
zero secondary gaps, 3.04 seconds and 152,672 KiB peak RSS; native coverage remains
incomplete and its exit code is 1.

The production release build succeeded. Two written regressions cover large-file
loop boundaries, source positions, calls outside loops, mixed-size results and
large-file prefilter/language accounting. Automated tests and CI remain paused
under David's instruction. Oversized implementation ownership, remaining language
coverage, broader correctness evidence and approved CI still require work.
