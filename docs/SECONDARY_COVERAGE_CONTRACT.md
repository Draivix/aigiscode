# Secondary rule coverage

`ast-grep-scan.json.coverage` accounts for every source passed to the secondary
scanner. The same typed value is exposed as `ast_grep_coverage` in the CLI summary,
consolidated report, architecture overview, convergence history, and MCP overview,
quality and coverage tools. MCP tool/resource metadata carries its status and gap
counts alongside native input coverage and freshness.

The scanner owns this value. Report projections copy it rather than calculating
another interpretation of scanner completion.

| Field | Meaning |
| --- | --- |
| `status` | `complete`, `incomplete`, `no_inputs`, or `unknown` |
| `input_files` | All sources passed to this scanner, including those without a language backend |
| `scanned_files` | Sources admitted to AST rule scanning |
| `prefiltered_files` | Sources skipped by the configured lexical family prefilters |
| `oversized_files` | Legacy size-limit omissions; retained for historical artifacts, zero in current scans |
| `unsupported_files` | Sources for which this scanner has no language rules |
| `other_gap_files` | Skips with another reason, conservatively treated as gaps |
| `scope_limited_files` | Scanned or prefiltered files with only part of the source in scope; overlaps those counts |
| `gap_bytes` | Full source sizes of files with omissions or limited scope; not a count of individual unexamined bytes |
| `gap_files_preview` | Up to five gaps, largest first with path as the tie-breaker |

`input_files` equals the sum of scanned, prefiltered and the three gap categories.
The complete per-file omission list remains in `skipped_files`, including its
existing reason strings. Partially covered files appear separately in
`scope_limited_files` with a typed scope, and their count is not added to
`input_files` a second time. Ordinary prefilter skips do not occupy the gap preview;
a prefiltered Vue file does appear there because its template remains outside scope.
Files without a backend now appear with reason `no_rules_for_language`; they are
no longer silently absent from scanner accounting.

`complete` means no recorded gaps in executing the configured rule catalog. It
does not establish that the prefilters are sound, that every language has every
rule family, that parsing captured all semantics, or that the source is secure.
`no_inputs` means the scanner received no source; native input coverage separately
controls the validity of an empty analysis. Missing legacy coverage deserializes
as `unknown`.

Size, language, extraction or scope gaps preserve partial findings but block a clean guard verdict
and make full analytical CLI commands return 1. The reason identifies missing
evidence, not a discovered defect. A baseline with current or previous secondary
coverage gaps is not comparable, so a missing finding cannot silently be called
resolved or improved. MCP continues serving the partial graph with explicit
coverage. Quality marks otherwise-zero affected dimensions unknown and includes
the coverage caveat in its recommendations.

Files above 150,000 bytes now run sequentially after the parallel smaller-file
scan. This threshold controls concurrency, not admission: eligible files retain
their complete AST and original line positions. Lexical prefilters and language
coverage accounting apply equally at every size. Only one large secondary AST is
live per scan invocation; this does not bound a single tree's memory, simultaneous
analysis invocations, native parsing, or accumulated findings. Large-file execution does not resolve oversized implementation
responsibilities or establish that the rule catalog is complete.

Vue SFC files use the shared native extractor, backed by the installed HTML parser,
to select top-level JavaScript/JSX, TypeScript or TSX script blocks. Comments,
templates, styles and custom blocks are masked with byte positions and newlines
preserved. Existing JS/TS rules run against those scripts at the original `.vue`
path. A file with no applicable script rule is prefiltered normally. Every admitted
Vue file also has scope `vue_script_only`, including scriptless components: no
template or style coverage is implied, and secondary coverage stays incomplete.

Recovered component structure, external script sources and unsupported script
languages/types produce an `extraction_gap` with a reason and original source line:
`vue_structure_recovery`, `vue_external_script`, `vue_unsupported_script_language`
or `vue_embedded_nul`.
Individually intact, top-level supported blocks can still contribute evidence;
their scope record retains the extraction gap. If no intact supported script was
found, the whole-file skip occupies `other_gap_files`, without also being counted
as partially scanned. Native `ParseOutcome.extraction_gap` records the same
limitation while its scope remains `vue_script_only`. This field identifies the
first observed limitation, not an exhaustive diagnostic list or a Vue compiler verdict.
The extractor does not compile Vue templates or resolve external script contents.
Embedded NUL bytes are replaced with spaces only in the HTML boundary parser's
input to avoid premature end-of-input; extracted JS/TS keeps the original bytes
and the extraction gap remains visible.
Missing legacy scope counters deserialize as zero.

Backend, prefilter and built-in rule-family selection share one language mapping.
JavaScript includes `.js`, `.jsx`, `.mjs` and `.cjs`; TypeScript includes `.ts`,
`.mts` and `.cts`, with `.tsx` using its TSX parser. Extensions are case-insensitive.
