# AigisCode

[![CI](https://github.com/Draivix/aigiscode/actions/workflows/ci.yml/badge.svg)](https://github.com/Draivix/aigiscode/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.82%2B-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Native Rust whole-codebase analysis for AI agents.

AigisCode scans mixed-language repositories, builds a semantic graph, and emits
machine-readable artifacts for structural triage. The current product surface is
the Rust CLI. The legacy Python implementation has been removed from this
repository.

## Quick Start

```bash
cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode
cd your-project
aigiscode analyze .
```

For local development from this repository:

```bash
cargo run --bin aigiscode -- analyze .
```

## Commands

```text
aigiscode analyze <path>      Run deterministic analysis and write native artifacts
aigiscode agent <path>        Print the graph-backed AI review contract
aigiscode agent-run <path>    Execute the AI review through a concrete adapter and write agent reports
aigiscode agent-spider <path> Crawl top task packets through a concrete adapter and write per-packet reports
aigiscode report <path>       Compatibility alias that also writes aigiscode-report.json
aigiscode analyze-rust <path> Compatibility alias for analyze
aigiscode info <path>         Inspect existing Rust-native artifact state
aigiscode plugins             List built-in runtime/framework overlay plugins
aigiscode tune <path>         Suggest a conservative policy patch from current analysis signals
aigiscode surface <path>      Emit architecture surface JSON
aigiscode mcp <path>          Start the native Rust stdio MCP server for one repository
aigiscode version             Print CLI version
```

Use `--output-dir <path>` to write artifacts outside `.aigiscode/`.
Use `--no-write` to print JSON without writing artifacts.
Use `--external-tool <name>` or `--external-tools all` to run native Rust
external adapters alongside deterministic analysis.

## Artifacts

`aigiscode analyze` writes:

```text
.aigiscode/deterministic-analysis.json
.aigiscode/semantic-graph.json
.aigiscode/dependency-graph.json
.aigiscode/evidence-graph.json
.aigiscode/contract-inventory.json
.aigiscode/doctrine-registry.json
.aigiscode/deterministic-findings.json
.aigiscode/ast-grep-scan.json
.aigiscode/external-analysis.json
.aigiscode/architecture-surface.json
.aigiscode/review-surface.json
.aigiscode/convergence-history.json
.aigiscode/guard-decision.json
.aigiscode/aigiscode-handoff.json
.aigiscode/agentic-review.json
.aigiscode/graph-packets.json
.aigiscode/repository-topology.json
.aigiscode/aigiscode-report.json
.aigiscode/aigiscode-report.md
```

When external tools are enabled, raw scanner artifacts are archived under:

```text
.aigiscode/reports/<run_id>/raw/
```

`aigiscode surface` prints the architecture surface JSON and also writes:

```text
.aigiscode/architecture-surface.json
```

`aigiscode mcp` serves tools, resources, and prompts over stdio from the same
native artifact family.

`aigiscode agent` runs the normal analysis pipeline, writes the same artifact
family, and prints `agentic-review.json` as the primary machine contract for an
AI reviewer. The AI contract is graph-backed, includes diff-aware task packets,
trace-style evidence chains, bounded typed multi-path graph traces, and bounded
code-flow style evidence paths plus explicit source/sink endpoints, bounded
semantic state-flow evidence for mutable carriers when the slice supports it, and now
carries an adapter catalog with:
- local `codex exec`
- direct OpenAI Responses HTTP
- optional TypeScript Codex SDK sidecar

`ast-grep-scan.json` is the first secondary scanner-plane artifact. It carries
typed, provenance-rich structural rule hits from in-process `ast-grep`
evaluation. Today it covers three pilot families:
- loop-local expensive-operation rules (`collection scan`, `sort`, `regex compile`,
  `json decode/parse`, and `filesystem read/check` in loops) that strengthen
  `AlgorithmicComplexityHotspot`
- dangerous-API rules (`eval`, `exec/system`, unsafe deserialization, unsafe HTML
  output`) that strengthen native `SecurityDangerousApi` findings
- narrow framework-misuse rules (`raw env outside config/bootstrap`,
  `raw container lookup outside provider/bootstrap or injection boundaries`)
  that strengthen native `SanctionedPathBypass` findings

The engine stays in core, but framework-specific rule catalogs are now allowed
to contribute findings with explicit provenance such as
`ast_grep.pattern.laravel` and `ast_grep.pattern.django`, so framework misuse
can scale without turning the core scanner file into a framework registry.

It is secondary evidence, not semantic-graph truth, reachability truth, or
doctrine truth. `aigiscode-report.json.summary` and
`architecture_surface.overview` now also break those scanner hits down by
family so the scanner mix is visible without opening the raw artifact.

`graph-packets.json` complements `agentic-review.json` with bounded,
doctrine-aware graph neighborhoods for the current top packets and focus files.
Fallback focus-file packets now also carry bounded traces, code flows,
source/sink paths, and semantic state-flow evidence when guardian packets are
absent, so the packet layer stays useful even on degraded or slice-only runs.

`repository-topology.json` complements both with a flatter orchestration map
over top-level zones, manifests, runtime entries, contract-bearing directories,
cross-zone links, direct zone-to-finding / zone-to-packet links, zone-level
triage briefs, structured triage steps, focus clusters for flat zones,
explicit cross-zone pressure summaries with linked-zone previews, direct causal
bridge summaries, topology-level semantic-state previews/counts and proof-aware
summaries for mutable carriers, a topology recommended start slice, spillover observations,
convergence-state hints, and lightweight ownership hints with explicit basis
metadata so agents can reason about repository layout, cross-zone drag,
semantic propagation, and the next slice without loading the full evidence
graph. Route-declared files now also promote runtime-entry shape here, so
modern Symfony/Laravel-style controller surfaces are no longer invisible in
topology. Scoped/cropped analyses now also expose explicit `boundary_truncated`
truth here instead of implying fake orphan debt for files
whose real callers live outside the analyzed slice.
Topology semantic-state previews now also expose stable flow IDs, flow kind,
and proof tier (`exact_resolved`, `receiver_typed`, `heuristic`), and
zones/steps/clusters carry proof summaries plus compact flow refs so agents can
tell strong propagation evidence from weak heuristic hints and jump back to one
exact flow instead of only reading labels.

`aigiscode agent-run` is the first real executor. It materializes the normal
artifact family, selects an adapter, and writes:

```text
.aigiscode/agent-review.json
.aigiscode/agent-review.md
.aigiscode/agent-output-schema.json
.aigiscode/agent-execution.jsonl
```

`aigiscode agent-spider` crawls the top task packets from `agentic-review.json`
through the same adapter boundary and writes:

```text
.aigiscode/agent-spider-report.json
.aigiscode/agent-spider/<packet>/agent-review.json
.aigiscode/agent-spider/<packet>/agent-review.md
.aigiscode/agent-spider/<packet>/agent-output-schema.json
.aigiscode/agent-spider/<packet>/agent-execution.jsonl
```

Current working adapters:
- `codex-exec` for local Codex CLI execution
- `responses-http` for direct Rust `v1/responses` execution with `OPENAI_API_KEY`

Planned adapter:
- `codex-sdk` as a thin optional TypeScript sidecar around the official Codex SDK

## What It Finds

- Circular dependencies
- Bottlenecks and orphan files
- Boundary-truncated files on scoped analyses
- Dead code candidates
- Hardwired values
- Declared routes, hooks, env keys, config keys, and symbolic runtime contracts
- Architecture-surface summaries for UI and agent workflows

## Policy And Rules

The Rust review/report layer now reads optional suppression files from the
target repository:

- `.aigiscode/policy.json` for project-wide patterns such as `orphan_entry_patterns`,
  `abandoned_entry_patterns`, `skip_path_patterns`, `allowed_literals`, and
  `repeated_literal_min_occurrences`, plus external-finding controls like
  `external.skip_tools`, `external.skip_categories`, and `external.allowed_rule_ids`
- `.aigiscode/rules.json` for narrow per-finding exclusions by finding type,
  file pattern, and optional symbol/value or external tool match

Raw deterministic analysis remains in `deterministic-analysis.json`. The
policy/rule overlay is reflected in `review-surface.json`, `aigiscode-report.json`,
and the native MCP server.

## External Tools

The Rust CLI can also orchestrate external analyzers and normalize their output
into `external-analysis.json`, `review-surface.json`, `aigiscode-report.json`,
and MCP finding workflows.

Current native adapters:

- `ruff`
- `gitleaks`
- `pip-audit`
- `osv-scanner`
- `composer-audit`
- `npm-audit`
- `cargo-deny`
- `cargo-clippy`

## Supported Languages

- Rust
- PHP
- Python
- TypeScript / JavaScript
- Ruby

Language support here means parsing and graph extraction in the Rust engine. Parity
is still in progress for some higher-level detectors.

## Development

```bash
cargo fmt
cargo test
cd website && npm ci && npm run build
```

## Current Direction

- Rust is the only product runtime.
- Python packaging, CLI, MCP host, report shell, and tests have been removed.
- Public docs and website examples now target the Rust CLI.

## License

MIT. See [LICENSE](LICENSE).
