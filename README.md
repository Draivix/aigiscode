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
.aigiscode/external-analysis.json
.aigiscode/architecture-surface.json
.aigiscode/review-surface.json
.aigiscode/convergence-history.json
.aigiscode/guard-decision.json
.aigiscode/aigiscode-handoff.json
.aigiscode/agentic-review.json
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
code-flow style evidence paths plus explicit source/sink endpoints, and now
carries an adapter catalog with:
- local `codex exec`
- direct OpenAI Responses HTTP
- optional TypeScript Codex SDK sidecar

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
