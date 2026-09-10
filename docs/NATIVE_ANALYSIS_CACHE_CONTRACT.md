# Native analysis restoration

`AIGISCORE_FAST_LOAD=1` remains opt-in. A verified semantic graph can now reuse
the native analysis already stored in `deterministic-findings.json`. This adds
no artifact path, dependency or runtime. The manifest has an optional
`deterministic_findings_xxh3` field; old manifests omit it and recompute analysis.

The existing snapshot reader pins and verifies the published generation. Graph
loading then verifies engine identity, scope, admitted input inventory, semantic
environment, resolver configuration, supported source bytes and graph checksum.
An engine/input mismatch declines graph reuse. An invalid published generation
remains an explicit integrity error rather than a clean empty result.

Native findings are eligible only when both publication and current manifest
identity contain no external tool evidence and external checks are complete.
Current policy and doctrine are loaded once for this analysis, and their shared
assessment fingerprint must match the manifest. Configuration loading errors
propagate. The loader deserializes typed required fields while hashing the same
opened stream, including trailing bytes. Its hash, input coverage and all five
file/symbol/reference/edge counts must agree with the validated graph project.

Restored fields are graph analysis, architectural assessment, contract inventory,
hardwiring, secondary scanner results and security analysis. Dead-code analysis
is recomputed because its supplemental sweep can read outside the parsed slice;
it must equal the cached result before reuse is accepted. A failed eligibility,
decoding, checksum, count, coverage or dead-code comparison recomputes the entire
native analysis from the verified graph and current captured policy/doctrine.

Neither old guard/convergence decisions nor old agent context are restored.
Those continue to derive from the current analysis and pinned previous baseline
through the existing MCP/artifact layers. External tools are not replayed or
represented as freshly executed. Existing incomplete native/secondary coverage
is preserved and still affects guard and CLI behavior.

`Scan` measures inventory/source validation, `LoadGraph` measures graph decoding
and validation, and `Structure` measures structural assembly. A successful native
restore adds `LoadAnalysis`, which includes current configuration loading,
findings decoding and dead-code verification. A fallback adds `Analyze` instead.
`VerifyInputs` now measures the final admitted-input/configuration validation
after either path. Prior run timings are never substituted for current work. MCP trace output
states whether native analysis was restored.

This is unchanged-snapshot reuse, not incremental parsing or global analysis.
The per-file secondary scanner cache is not seeded by this restoration; a later
changed scan may need to warm it. Full graph decoding and MCP surfaces still
consume time and memory. Shared configuration capture and final validation are
described in [input capture](INPUT_CAPTURE_CONTRACT.md), including the remaining
non-atomic filesystem and supplemental-reader limits.

The existing graph round-trip regression now checks all native fields and a
logically unrelated findings payload with an updated hash; the external-evidence
regression checks manifest ineligibility. They were written but not run or built
as test targets. Tests, CI and local quality gates remain paused. Runtime evidence
and its narrower limits are recorded in the [Draivix observation](2026-09-10-analysis-restore.md).
