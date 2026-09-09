# Vue script evidence on the preserved Draivix corpus

The secondary scanner now processes script regions in all 792 captured Vue
components. It AST-scans 206 files and lexically prefilters 586, retains all 9,932
previous raw clues and adds 46: 41 complexity and five HTML-output matches.
Every Vue file remains explicitly `vue_script_only`; template, style and custom
block behavior has not been audited. Secondary coverage is still incomplete.

The [receipt](2026-09-09-vue-script-coverage-evidence.json) contains the final binary
and artifact fingerprints, all added clues, extraction limitations, graph
comparisons, MCP metadata and source integrity. All 23,166 files in the preserved
capture still match its original size/hash inventory. Original Draivix source was
not edited. Subsequent live Mitel changes are outside this saved-corpus evidence.

The native Vue adapter, lexical masking and secondary scanner share the same
extractor. It now uses the already-installed HTML parser to select intact top-level
script blocks instead of searching raw text for tags. Comments and nested examples
cannot become component code. Attribute values retain quoted delimiters; JS/JSX,
TypeScript and TSX scripts select their corresponding existing grammar. Masking
preserves original byte offsets and line numbers.

Recovered component structure or unavailable script content is recorded with a
reason and original line. Intact selected scripts can still contribute evidence;
if none can be extracted, the scanner records a whole-file gap. These are parser
limitations, not Vue compiler verdicts. The [contract](SECONDARY_COVERAGE_CONTRACT.md)
defines the counters and distinguishes partial scope from whole-file omissions.

Two source-backed boundary cases explain the native changes:

- `FleetPanel.vue:1` has `>` inside its quoted `generic` attribute. The old text
  search copied the end of that attribute into the script and caused a false
  recovery. The corrected boundary removes that error. Five incident edges retain
  their exact identity, location and resolution tier while changing from inferred
  strength/confidence 500 to hard strength/confidence 900. The parser-recovery
  qualification disappears from their reasons.
- `ReportResultView.vue:185` contains embedded NUL bytes. The HTML scanner treats
  these as a raw-text terminator. Replacing them with spaces only for HTML boundary
  discovery preserves the full script; the native and secondary JS/TS parsers
  still receive original bytes. `vue_embedded_nul` remains explicit, and its
  original 22 JS/TS recovery diagnostics remain. This is not a clean parse claim.

The complete files, symbols and references arrays are byte-identical to the
preceding large-file scan. All 325,573 resolved edges were compared in order across
every serialized property: only the five FleetPanel-related confidence/strength/
reason changes occur. Remaining unsupported-source and other-input fields match.
There are 32 parse-outcome changes: the FleetPanel correction, one NUL limitation
and 30 HTML structure-recovery annotations. Counts remain 129,454 symbols and
1,086,615 references. Native recovered-file count is 80, down from 81 solely because
of FleetPanel; all 792 Vue files still have limited native extraction scope.

The new raw clues require interpretation. Source inspection found:

- `GlobalSearch.vue:1143` constructs a regex for every query word inside a loop.
  This establishes repeated construction; it does not measure user-visible cost.
- `CsvImportModal.vue:223` checks a normalized string against aliases inside a loop.
  Its template also produces an HTML recovery annotation at line 528, but the
  intact script and its source position remain available.
- `ItemsGrid.vue:962,1192` checks a fixed six-value literal array. Those syntax
  matches do not establish an unbounded collection-scan hotspot.
- `MapyCzMapField.vue:706,810` clears a container with an empty string;
  `ShadowHtmlPreview.vue:23` calls `sanitizeTemplatePreviewHtml` before assignment.
  HTML-output syntax alone does not establish an exploitable injection.

The architecture assessment, security analysis, hardwiring and graph-analysis
sections of `deterministic-findings.json` remain unchanged. The existing native complexity dispatcher excludes Vue, so the
41 complexity additions remain raw secondary evidence. Connecting that evidence to
architectural triage with suitable actionability filters remains outstanding;
this change does not claim to finish Vue analysis or overall Q01–Q12 acceptance.

The final full audit took 84.48 seconds, with 3,494,756 KiB peak RSS and 18.906
seconds in the secondary scan. The preceding large-file observation took 84.03
seconds and 19.246 seconds respectively. These are individual shared-host runs,
not evidence of a speedup. A real MCP session reused the verified graph cache,
returned an explicit indexing response, then served the useful overview after
45.00 seconds. Overview, quality and coverage payloads agree with CLI/report
coverage; all four queried tools carry 792 scope-limited files in metadata with
matching artifact/index inputs. Guard remains `Block` and CLI exits 1.

The final self-audit finished in 2.91 seconds with 154,564 KiB peak RSS. Its
secondary coverage is complete for 117 inputs; native coverage remains incomplete
and CLI exits 1. The production release build succeeded. Eight added and two
updated regressions cover the boundary and coverage cases but remain unexecuted,
as do automated quality gates and CI, under David's instruction.
