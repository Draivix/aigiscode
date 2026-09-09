# Secondary rules now cover module extensions

All 18 `.mjs` files in the preserved full Draivix snapshot now enter the existing
JavaScript rule pipeline. Twelve are scanned and six are prefiltered. Secondary
coverage now reports 8,139 scanned files, 8,064 prefilters, 18 size gaps and 792
unsupported Vue files. The source inventory remains 17,013 files.

The subsequent [large-file scan](2026-09-09-large-file-scanning.md) closes the
18 size gaps on this same capture. The measurements below precede that change.

The [receipt](2026-09-09-module-extension-evidence.json) records source and engine
identities, counts, retained evidence, source examples and artifact hashes. The
full audit took 78.56 seconds with 3,535,208 KiB peak RSS and exit code 1. Remaining
native and secondary coverage gaps still block a clean audit; the observation is
not a performance benchmark. Draivix was not edited.

## What changed and what was observed

The scanner had JavaScript rules but omitted `.mjs` from its backend mapping.
Separate extension switches for prefilters and the three rule families made
adding only a parser mapping insufficient. They now use the existing typed
`SupportLang` mapping. `.mjs`/`.cjs` select JavaScript, `.mts`/`.cts` select
TypeScript, and TSX keeps its own parser with TypeScript rule sets. Extension
matching is case-insensitive, consistent with native source ingestion.

The audit retains all 9,446 earlier scanner findings and adds 51 raw environment
access clues in seven `.mjs` files. The semantic graph fingerprint, graph-analysis
results, native security analysis and architectural assessment are unchanged.
The new raw clues did not turn into new corroborated architecture or security
findings.

Two source checks bound the interpretation:

- `hocuspocus-server.mjs:10,20,83,84` really reads `process.env`. This is server
  bootstrap code that loads configuration and requires a token-signing key.
  The exact syntax hits establish scanner execution; they do not independently
  prove a sanctioned-path violation or justify moving this initialization.
- `apps/desktop/src/config.mjs:52` accepts an explicit environment parameter with
  default `process.env`, then reads properties through that parameter. The file
  now enters scanning and has no secondary findings. The direct-member rule does
  not match that adapter shape; this is not a universal safety verdict.

The baseline remains `not_compared` because both observations have incomplete
coverage and the engine changed. Extra scanner evidence from a repaired backend
is not reported as a newly introduced Draivix defect.

A real MCP session reused the graph cache and served a useful overview after
45.46 seconds. Its overview, quality, coverage and response metadata expose the
updated coverage. It returned an explicit indexing response before readiness.

The production release build passed without warnings. A regression spanning JS,
module extensions, case variation and TSX checks all three existing rule families,
but it was not run. The actual corpus evidence here covers `.mjs`; it does not
claim independent runtime verification of every added alias. Automated tests and
CI remain disabled under David's instruction.
