# Draivix: current-source audit and acceptance evidence

The full-source audit processed 17,013 supported sources from 23,166 copied files
in 83.31 seconds, using 3,446,044 KiB peak RSS. It produced useful partial evidence
and returned exit code 1: 81 files required parser recovery, 792 Vue files have
script-only coverage, and 146 recognized source files are unsupported. **This is
not a clean-code or security acceptance.**

The measured engine is commit `f1cbaf0`, fingerprint `11937782ece2d943`. The
[machine receipt](2026-09-09-draivix-revalidation-evidence.json) records binary,
source-inventory and artifact hashes, timings, coverage, graph anchors, baseline
state and actual MCP responses. Raw artifacts are retained at
`target/reliability-2026-09-09/draivix-full/`.

## Inputs and limits

Draivix HEAD remains `95276a526f6c9e684f81ea4b83639894e2f02c38`, with existing
working-tree changes. Every one of the 17,013 supported sources from September 8
still matches the original by SHA-256. The new snapshot includes other file types
so unsupported Java, Go, shell and SQL inputs are visible. All 23,166 copied files
were compared against the originals and the copied-byte inventory without a
mismatch. No Draivix source, policy, configuration or Git state was modified.

The original `.aigiscode/scan.json` restricts analysis to selected production
directories. This audit uses a separate copy with no include-prefix restriction,
retains the original doctrine, and includes tests and client overlays. It retains
the scanner's default directory exclusions and excludes the same three generated
paths as the earlier corpus: `phpstan/cache`, `phpstan/resultCache.php`, and
`android/app/src/main/assets/public`. Hidden directories, symlinks and dependency
trees are outside this capture. Its standalone Git boundary is initialized without
commits; this is a source observation, not a review of a Git diff or release.

The 81 recovered files and 811 diagnostics do not establish 81 invalid source
files. For example, `ErpMakeEntityCommand.php:386` is inside a heredoc-based code
generator; parser grammar support requires separate investigation. The audit
preserves partial graph facts and weakens edges involving recovered files.

## What the current artifacts establish

| Observation | Result and meaning |
| --- | --- |
| Native graph | 129,454 symbols, 1,086,615 references, 325,573 resolved edges; these counts do not measure resolution recall |
| Cycles | 29 full-view and 19 strong components, with 116 closed witness steps |
| Witness provenance | 115 steps occur in the dependency projection; the remaining inferred DataBox constructor step occurs in the evidence projection |
| Strong witnesses | None uses an inferred or type-only edge |
| Dead code | Checks are deferred because input coverage is incomplete; zero findings is not evidence of absence |
| Baseline | Missing / initial snapshot, 3,218 logical findings marked `FirstObserved`, previous count and deltas are null |
| Guard | `Block` for incomplete native evidence; no claimed regressions or resolutions |
| External security checks | Not requested; 65 native dangerous-API findings are leads, not 65 validated vulnerabilities |
| Secondary scanner | 8,127 scanned files; 8,058 prefilter skips and 18 files skipped for size |
| Artifact volume | 1,660,448,436 bytes; large graph representations remain an operational limitation |

The dependency projection intentionally omits the inferred DataBox step. Source
`DataBoxClient.php:40` actually constructs `DataBoxApi`; its current `global` /
`inferred` classification is conservative and loses precision for this same-namespace
case. A full-view cycle is not automatically a strong dependency cycle.

## Positive and negative source evidence

1. **Receiver ownership remains repaired.** In
   `app/Entities/_Core/Authoring/StructureApplier.php`, `$operation->isNoop()` at
   lines 114 and 183 does not bind to `LocaleFileWriter`. The genuine
   `$this->localeWriter->isNoop(...)` at line 328 still binds to that method.
   This is one positive target and two rejected false targets; it does not prove
   complete PHP receiver inference.
2. **Runtime and type cycles remain distinct.** The create-submit loader imports
   `normalizeEntityCreateSubmitModule` at line 4, and its registry imports the
   loader at line 2. That pair has a strong runtime witness. The entity-form
   loader's line-2 import is `import type`; its pair remains in the full view
   but is absent from the strong view. Removing that type declaration would not
   be a runtime-cycle repair.
3. **Repeated decoding is not established redundant work.**
   `app/Entities/_Core/ConversionLoader.php:164–178` decodes fields belonging to
   each individual row. The complexity finding has real source anchors, but
   hoisting those operations out of the loop changes their inputs. Its graph
   pressure path begins in a test; it does not demonstrate production hotness.
4. **A dangerous API is not an injection proof.**
   `ErpDevResetCommand.php:191` executes a fixed build command.
   `ErpLintEntityCommand.php:677` applies `escapeshellarg` to the filename.
   Both are real execution sites. Neither inspected expression establishes an
   attacker-controlled command injection. Their reported reachability paths
   start in tests, not an established production ingress.
5. **The top automatic recommendation is not yet reliable.** Topology recommends
   `ActivityArtifactMetadataResolverRegistry.php` for abstraction sprawl.
   That 41-line class holds a provider map with registration and lookup.
   `CRMFoundationServiceProvider.php:40–47` registers it and enrolls it in tenant
   context reset; two live consumers inject it. The reported naming-role group
   does not establish redundant responsibility or justify deleting this owner.
   The recommendation's 32,635 cross-zone relations describe the broad `app` zone,
   not that registry's individual importance.

These are deliberately selected, source-reviewed cases. They are not a random
sample, a precision/recall score, or proof that all remaining edges are correct.

## Actionable Draivix order

The earlier [source-backed remediation ledger](2026-09-08-draivix-review/README.md)
still applies to unchanged source. Prioritize the following work with the owners;
this audit does not authorize modifying Draivix.

1. Complete actor identity integration. `EntityBroadcaster.php:184` accepts explicit
   `actorId`, while `AssignmentNotificationService.php:120` only reads `user->id`.
   Static input `['actorId' => 41]` therefore produces different results. The
   existing `app/Support/ActorContext.php:13` has no production caller. Define the
   shared precedence and missing-actor behavior, migrate the consumers, and verify
   actual notification/persistence/realtime paths. No notification was sent here.
2. Reconcile the existing replacement contracts with their production consumers:
   ACL membership writing, filter type/sort resolution and view-column selection.
   Test-only or unwired replacements are incomplete integration, not automatically
   obsolete files. Preserve concurrent work.
3. Keep removals source-specific. The old ledger's private
   `EntityViewHandler::hydrateManyToManyForShow` and three unused imports still have
   their reviewed source evidence. The new engine's deferred dead-code check does
   not grant a fresh deletion verdict or invalidate that manual review.
4. Separate the create-submit normalizer from its registry/loader pair. Preserve
   the entity-form type relation and legitimate asynchronous Email dispatch paths.
5. Narrow real ownership seams: artifact/path handling versus domain validation in
   AccountingProductionEvidenceService; existing send/sync/access owners in Email;
   generation, tools and telemetry in Chat. Keep tenant-aware query state and
   invalidation together in the large Email Vue components. Line count alone is
   not the justification.

## MCP and remaining acceptance work

The corrected real MCP session initialized in 9 ms. The same session received
retryable `indexing` responses at 30 and 61 seconds, then a useful overview at
76.81 seconds. Cycle, quality and coverage queries succeeded immediately afterward.
Every result carried revision 1 and `input_coverage.status=incomplete`; quality
described partial evidence and dead-code severity `unknown`.

That session performed a fresh parse, despite enabling fast load: source inspection found that
MCP's cache reader ignored `--output-dir` and selected only `<root>/.aigiscode`.
The fix passes the selected directory into ingestion. That measurement
must not be described as cache performance. A preceding client used an incorrect
quality tool name and ended before retrying the overview; its error log is retained,
and the corrected session above is the basis for successful API claims.

The fixed release build, fingerprint `b552b2988ce88f06`, completed a separate full
audit in 79.73 seconds with 3,465,192 KiB peak RSS and identical summary fields.
It wrote artifacts into another custom directory, preserving the first audit.
MCP then explicitly confirmed cache reuse and skipped Parse+Resolve. Its first
useful overview arrived in **43.72 seconds**, followed by successful cycle,
quality and coverage queries. Before readiness, the same process returned an
honest retryable indexing response at 30 seconds. The cycle and quality results
and input-coverage contract match the preceding source-built session.

The directory is selected consistently for both the manifest and graph. Without
an explicit directory it remains `<root>/.aigiscode`; an unavailable custom cache
declines to a source build rather than silently choosing another cache. The
existing round-trip regressions were updated, and a selected-directory regression
was added but not run. The production build completed without warnings. This is
one sequential observation with warm filesystem caches on a shared host; it does
not establish incremental performance or a statistical speedup.

The current data still leave Q11–Q12 open: startup and artifact memory are large,
the watcher rebuilds the whole graph, concurrent load and single-change latency
remain unmeasured, and file-by-file atomic writes are not whole-family publication.
Eighteen large files miss secondary scanning, including the production evidence
service, ChatService and Email Vue components. The automatic priority example above
also prevents accepting the current architecture guidance as independently reliable.
The optional Kuzu execution path still uses its existing Node bridge, so the
native Rust target is not met by every optional path.

Later follow-ups implement [complete analytical generations](2026-09-09-artifact-publication.md)
and [native Cypher with immutable database exports](2026-09-09-native-cypher.md).
Those receipts supersede the corresponding implementation limitations above;
they do not close the broader acceptance conditions. The Cypher receipt also
records later live-source drift from this saved capture.

Production CLI and real read-only MCP requests were exercised. Automated tests and
CI remained disabled under David's instruction. Approved regression gates, broader
labeled positive/negative evidence, and the remaining Q01–Q12 acceptance requirements
still need completion. No overall goal or separate work lock is closed.
