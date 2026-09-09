# Secondary coverage on the full Draivix snapshot

The updated auditor accounts for all 17,013 sources passed to its secondary
scanner. It keeps 9,446 existing findings unchanged while exposing 828 omitted
files: 18 exceed the size limit, and 810 have no secondary language rules
(792 Vue files and 18 `.mjs` files). Another 8,058 files were deliberately
prefiltered, and 8,127 entered rule scanning.

The [machine receipt](2026-09-09-secondary-coverage-evidence.json) identifies the
engine and source fingerprints, coverage projections, baseline assessment, guard
trigger and artifact hashes. The source capture is the unchanged, broad Draivix
snapshot documented in the [preceding review](2026-09-09-draivix-revalidation.md).
No Draivix source or configuration was edited.

## Observed behavior

The audit finished in 78.67 seconds with 3,524,588 KiB peak RSS and exit code 1.
These are single-run observations on a shared host, not a performance claim.
Coverage is identical in the raw scanner artifact, CLI summary, consolidated
report, architecture surface and convergence artifact. The largest-gap preview
includes AccountingProductionEvidenceService; ordinary prefilter skips no longer
hide these gaps in the new coverage field. Existing aggregate skip fields remain
available for compatibility.

The run used copies of the preceding real baseline family. Its hashes verified,
but comparison is `not_compared`, with explicit current and previous secondary
coverage reasons, native input limitations and the changed engine identity.
All 3,218 logical findings are `NotCompared`; new, worsened, improved and resolved
counts are zero, and graph/contract deltas are null. The new secondary-coverage
`Block` trigger appears independently of the existing native-coverage trigger.
Omitted scanner evidence is not presented as code improvement.

A real MCP session reused the verified custom-directory graph cache. Its first
useful overview arrived in 43.37 seconds after an explicit indexing response.
Overview, quality and coverage carried the same secondary coverage as the raw
scanner artifact; all four queried tools, including cycles, included its incomplete
status and gap counts in metadata. The quality recommendation names the 18 size
gaps and 810 missing language backends. The process exited successfully after
serving the partial index.

## Implementation and remaining work

`scanners/coverage.rs` owns classification of scanner completion. The driver now
records sources without a backend, and report/MCP projections reuse the typed
result. The report also reuses the surface's existing skip projection instead of
recomputing it. Guard, baseline comparison and analytical CLI exit status consume
the completion contract. MCP metadata and quality/coverage tools expose it;
otherwise-zero affected quality dimensions become unknown when evidence is missing.

The [contract](SECONDARY_COVERAGE_CONTRACT.md) distinguishes rule execution from
semantic or security soundness. The 150,000-byte limit remains; this change does
not add large-file scanning or a Vue backend. The `.mjs` omissions identify a
concrete remaining language-alias gap. Q12 still includes separating oversized
implementation responsibilities and removing the optional Node runtime path.
The broader architecture-priority and incremental-processing work also remains open.

The production release build passed without warnings. Existing size/prefilter
regressions were extended and a mixed-source coverage regression was added, but
automated tests and CI were not run under David's stop instruction. The real full
audit demonstrates the new coverage and guard trigger; an isolated case where
secondary coverage alone determines the verdict was not dynamically replayed.
