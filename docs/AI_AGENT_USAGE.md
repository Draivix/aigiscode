# AI Agent Usage

## Purpose

Use `aigiscode` as a local deterministic analysis engine for repository-scale
structural triage.

## Setup

From this repository:

```bash
cargo build
```

From GitHub:

```bash
cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode
```

## Baseline Run

```bash
aigiscode analyze /repo
aigiscode agent /repo
aigiscode agent-run /repo --adapter codex-exec
aigiscode agent-run /repo --adapter responses-http
aigiscode agent-spider /repo --adapter codex-exec --limit 3
aigiscode report /repo
aigiscode analyze /repo --external-tool gitleaks
```

Primary outputs:

```text
/repo/.aigiscode/deterministic-analysis.json
/repo/.aigiscode/semantic-graph.json
/repo/.aigiscode/dependency-graph.json
/repo/.aigiscode/evidence-graph.json
/repo/.aigiscode/contract-inventory.json
/repo/.aigiscode/doctrine-registry.json
/repo/.aigiscode/deterministic-findings.json
/repo/.aigiscode/external-analysis.json
/repo/.aigiscode/architecture-surface.json
/repo/.aigiscode/review-surface.json
/repo/.aigiscode/convergence-history.json
/repo/.aigiscode/guard-decision.json
/repo/.aigiscode/aigiscode-handoff.json
/repo/.aigiscode/agentic-review.json
/repo/.aigiscode/aigiscode-report.json
/repo/.aigiscode/aigiscode-report.md
```

## Recommended Agent Loop

1. Run `aigiscode analyze /repo`.
2. Parse `aigiscode-report.json` for the consolidated graph, detector, and summary contract.
3. Parse `dependency-graph.json` for low-noise architecture dependencies and `evidence-graph.json` for detailed call-site/runtime evidence.
4. Parse `contract-inventory.json` for declared routes, hooks, env/config keys, and symbolic runtime contracts.
5. Parse `doctrine-registry.json` for machine-readable guardian doctrine and default clause disposition.
6. Parse `deterministic-findings.json` for dead-code and hardwiring findings.
7. Parse `external-analysis.json` when the run included `--external-tool ...`.
8. Parse `architecture-surface.json` for hotspots, atlas views, and finding-first UI.
9. Parse `review-surface.json` for unreviewed finding state and policy/rule visibility.
10. Parse `convergence-history.json` to see which fingerprints are new, worsened, improved, unchanged, or resolved versus the previous run in the same output directory.
11. Parse `guard-decision.json` for the current allow/warn/block verdict and required review radius.
12. Parse `agentic-review.json` or run `aigiscode agent /repo` when you want the graph-backed AI review contract, prompt pair, artifact priorities, guardian packet bundle, diff-aware task packets, evidence chains, bounded typed graph traces, bounded code flows, and the OpenAI-first execution/report contract for an agentic reviewer.
13. Run `aigiscode agent-run /repo --adapter codex-exec` when you want AigisCode to execute a real local Codex review and write `agent-review.json`, `agent-review.md`, `agent-output-schema.json`, and `agent-execution.jsonl`.
14. Run `aigiscode agent-run /repo --adapter responses-http` when you want the same graph-backed review executed through the direct Rust OpenAI Responses adapter. This path requires `OPENAI_API_KEY`.
15. Run `aigiscode agent-spider /repo --adapter ... --limit N` when you want AigisCode to crawl the top `N` graph-backed task packets and write per-packet reports plus `agent-spider-report.json`.
16. Run `aigiscode tune /repo` when you want a conservative starting patch for `.aigiscode/policy.json`.
17. Fix a bounded set of issues.
18. Re-run `aigiscode report /repo`.

## Practical Guidance

- Treat the Rust artifact family as the source of truth.
- Prefer `deterministic-findings.json` when you want actionable queues.
- Prefer `external-analysis.json` when you want normalized external scanner evidence and raw-artifact locations.
- Prefer `architecture-surface.json` when you want topology, hotspots, or UI input.
- Prefer `review-surface.json` when you want review-state workflow over deterministic findings.
- Prefer `agentic-review.json` when you want the current graph-backed AI handoff contract. It now includes adapter plans for `codex exec`, direct OpenAI Responses HTTP, and an optional TypeScript Codex SDK sidecar.
- Task packets now carry bounded typed `graph_traces` derived from the semantic graph, including directed support paths, reverse support paths, contextual support paths, alternate paths when available, and aggregate confidence plus relation sequences.
- Task packets now also carry bounded `code_flows`, so spiders can read a claim as a stepwise path instead of reconstructing one from raw hops.
- Prefer a direct OpenAI `v1/responses` integration over browser-only Codex OAuth when you automate the AI layer. The current official Codex SDK is TypeScript-only, and there is no official Rust OpenAI SDK, so Rust should keep a typed HTTP adapter as the primary boundary.
- `responses-http` sends the graph-backed review contract directly to the Responses API; it does not depend on browser OAuth or local file access.
- Use the `execution` block inside `agentic-review.json` as the source of truth for automation. It now names the preferred local/service adapters, report targets, required markdown sections, the task packets that must be covered, the generated JSON Schema for the agent's structured JSON report, and concrete adapter invocation plans.
- Use `agent-spider` when you want bounded packet-by-packet crawling instead of one monolithic review. It keeps the same graph/doctrine contract but persists one report per task packet.
- Use `.aigiscode/policy.json` for project-wide suppression patterns and detector thresholds.
- Use `.aigiscode/policy.json` for native and external finding acceptance patterns, including `external.skip_tools`, `external.skip_categories`, and `external.allowed_rule_ids`.
- Use `.aigiscode/rules.json` for narrow per-finding exclusions that should persist across runs, including external-finding matches by type, file pattern, and optional `tool`.
- Use `--output-dir <path>` when you need artifacts outside `.aigiscode/`.
- Use `--no-write` if you only want JSON on stdout.
- Use `--external-tool <name>` or `--external-tools all` when you want Rust to run supported external analyzers and archive raw evidence under `.aigiscode/reports/<run_id>/raw/`.

## Current Scope

Supported product commands today:

- `aigiscode analyze <path>`
- `aigiscode agent <path>`
- `aigiscode agent-run <path>`
- `aigiscode agent-spider <path>`
- `aigiscode report <path>`
- `aigiscode analyze-rust <path>`
- `aigiscode info <path>`
- `aigiscode plugins`
- `aigiscode tune <path>`
- `aigiscode surface <path>`
- `aigiscode mcp <path>`
- `aigiscode version`

Legacy Python command flows are removed from the repository. These Rust-native
commands now provide the supported product surface, and MCP is provided by the
native Rust stdio server.
