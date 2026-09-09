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
| `oversized_files` | Sources with an available backend omitted by the size limit |
| `unsupported_files` | Sources for which this scanner has no language rules |
| `other_gap_files` | Skips with another reason, conservatively treated as gaps |
| `gap_bytes` | Total bytes in oversized, unsupported and other omitted sources |
| `gap_files_preview` | Up to five gaps, largest first with path as the tie-breaker |

`input_files` equals the sum of scanned, prefiltered and the three gap categories.
The complete per-file omission list remains in `skipped_files`, including its
existing reason strings. Ordinary prefilter skips do not occupy the gap preview.
Files without a backend now appear with reason `no_rules_for_language`; they are
no longer silently absent from scanner accounting.

`complete` means no recorded gaps in executing the configured rule catalog. It
does not establish that the prefilters are sound, that every language has every
rule family, that parsing captured all semantics, or that the source is secure.
`no_inputs` means the scanner received no source; native input coverage separately
controls the validity of an empty analysis. Missing legacy coverage deserializes
as `unknown`.

Size or language gaps preserve partial findings but block a clean guard verdict
and make full analytical CLI commands return 1. The reason identifies missing
evidence, not a discovered defect. A baseline with current or previous secondary
coverage gaps is not comparable, so a missing finding cannot silently be called
resolved or improved. MCP continues serving the partial graph with explicit
coverage. Quality marks otherwise-zero affected dimensions unknown and includes
the coverage caveat in its recommendations.

The existing 150,000-byte secondary file limit remains in place. This contract
exposes that limitation; it does not implement chunking, enable Vue scanning,
or claim to finish the broader Q12 ownership and native-runtime work.

Backend, prefilter and built-in rule-family selection share one language mapping.
JavaScript includes `.js`, `.jsx`, `.mjs` and `.cjs`; TypeScript includes `.ts`,
`.mts` and `.cts`, with `.tsx` using its TSX parser. Extensions are case-insensitive.
