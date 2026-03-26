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
/repo/.aigiscode/ast-grep-scan.json
/repo/.aigiscode/external-analysis.json
/repo/.aigiscode/architecture-surface.json
/repo/.aigiscode/review-surface.json
/repo/.aigiscode/convergence-history.json
/repo/.aigiscode/guard-decision.json
/repo/.aigiscode/aigiscode-handoff.json
/repo/.aigiscode/agentic-review.json
/repo/.aigiscode/graph-packets.json
/repo/.aigiscode/repository-topology.json
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
7. Parse `ast-grep-scan.json` for provenance-rich structural rule hits from the secondary scanner plane. Today it covers loop-local expensive-operation clues (`collection scan`, `sort`, `regex compile`, `json decode/parse`, and `filesystem read/check` in loops) that strengthen `AlgorithmicComplexityHotspot`, dangerous-API clues (`eval`, `exec/system`, unsafe deserialization, unsafe HTML output) that can reinforce native `SecurityDangerousApi` findings, and narrow framework-misuse clues for both raw env access outside config/bootstrap boundaries and raw container/service-locator lookup outside provider/bootstrap or injection boundaries that can reinforce native `SanctionedPathBypass` findings. The engine stays generic, while framework-specific rule catalogs can now contribute findings with explicit provenance such as `ast_grep.pattern.laravel` and `ast_grep.pattern.django`; it is still evidence, not graph truth. `aigiscode-report.json.summary` and `architecture-surface.json.overview` now also expose family-level scanner counts for quick triage.
8. Parse `external-analysis.json` when the run included `--external-tool ...`.
9. Parse `architecture-surface.json` for hotspots, atlas views, and finding-first UI.
10. Parse `review-surface.json` for unreviewed finding state and policy/rule visibility.
11. Parse `convergence-history.json` to see which fingerprints are new, worsened, improved, unchanged, or resolved versus the previous run in the same output directory.
12. Parse `guard-decision.json` for the current allow/warn/block verdict and required review radius.
13. Parse `agentic-review.json` or run `aigiscode agent /repo` when you want the graph-backed AI review contract, prompt pair, artifact priorities, guardian packet bundle, diff-aware task packets, evidence chains, bounded typed graph traces, bounded code flows, explicit source/sink paths, bounded semantic state-flow evidence for mutable carriers, and the OpenAI-first execution/report contract for an agentic reviewer.
14. Parse `graph-packets.json` when you want bounded doctrine-aware graph neighborhoods for the top task packets or focus files without loading the entire graph family. Fallback focus-file packets now also carry semantic state-flow evidence when the slice supports it.
15. Parse `repository-topology.json` when you want a flatter repository map over top-level zones, manifests, runtime entries, contract-bearing directories, and cross-zone links. Route-declared files now also promote runtime-entry shape in this artifact. Scoped analyses now also expose explicit `boundary_truncated` truth so agents can tell “cropped slice” from “real orphan debt”. The artifact also carries a topology-level recommended start slice, zone-level triage summaries, structured triage steps, severity/priority rollups, preview cards, focus clusters for flat zones, explicit cross-zone pressure summaries with linked-zone previews, direct causal bridge summaries, topology-level semantic-state previews/counts for mutable carriers, proof-aware semantic-state summaries and flow-kind/proof-tier previews, compact semantic-state flow refs with stable IDs, spillover observations, convergence-state counts, lightweight owner hints with explicit basis metadata, and direct artifact refs back into packet/finding evidence.
16. Run `aigiscode agent-run /repo --adapter codex-exec` when you want AigisCode to execute a real local Codex review and write `agent-review.json`, `agent-review.md`, `agent-output-schema.json`, and `agent-execution.jsonl`.
17. Run `aigiscode agent-run /repo --adapter responses-http` when you want the same graph-backed review executed through the direct Rust OpenAI Responses adapter. This path requires `OPENAI_API_KEY`.
18. Run `aigiscode agent-spider /repo --adapter ... --limit N` when you want AigisCode to crawl the top `N` graph-backed task packets and write per-packet reports plus `agent-spider-report.json`.
19. Run `aigiscode tune /repo` when you want a conservative starting patch for `.aigiscode/policy.json`.
20. Fix a bounded set of issues.
21. Re-run `aigiscode report /repo`.

## Practical Guidance

- Treat the Rust artifact family as the source of truth.
- Prefer `deterministic-findings.json` when you want actionable queues.
- Prefer `ast-grep-scan.json` when you want syntax-local structural rule hits with explicit provenance. The current pilot catalog covers loop-local expensive-operation shapes such as collection scans, sort-in-loop, regex compile in loop, JSON decode/parse in loop, filesystem read/check in loop, a first dangerous-API family (`eval`, `exec/system`, unsafe deserialization, unsafe HTML output), and narrow framework-misuse families for raw env access outside config/bootstrap boundaries plus raw container/service-locator lookup outside provider/bootstrap or injection boundaries. Framework-specific catalogs can now stamp provenance such as `ast_grep.pattern.laravel` and `ast_grep.pattern.django`. Treat it as secondary evidence that can strengthen native detectors, not as reachability or doctrine truth.
- Prefer `external-analysis.json` when you want normalized external scanner evidence and raw-artifact locations.
- Prefer `architecture-surface.json` when you want topology, hotspots, or UI input. On scoped runs it now distinguishes confirmed orphans from `boundary_truncated` files.
- Prefer `review-surface.json` when you want review-state workflow over deterministic findings.
- Prefer `agentic-review.json` when you want the current graph-backed AI handoff contract. It now includes adapter plans for `codex exec`, direct OpenAI Responses HTTP, an optional TypeScript Codex SDK sidecar, and first-class context refs to both `graph-packets.json` and `repository-topology.json`.
- Prefer `graph-packets.json` when you want compact, doctrine-aware graph neighborhoods around the current top packets and focus files. Fallback focus-file packets now also carry bounded traces, code flows, source/sink paths, and semantic state-flow evidence on slice-only/degraded runs.
- Prefer `repository-topology.json` when you want a flatter zone/package/runtime view for orchestration and ownership reasoning. It now also links zones directly to visible findings and guardian packets, carries inline severity/priority previews, exposes generated-at and baseline-state context, gives agents structured zone-level triage steps, breaks flat zones into focus clusters when packet/finding density is otherwise hard to scan, surfaces semantic-state previews/counts and per-step/per-cluster state labels for mutable carriers, exposes semantic-state proof summaries plus flow-kind/proof-tier previews and stable flow refs, exposes owner-hint basis metadata instead of a bare confidence label, explains cross-zone drag through linked-zone pressure summaries, surfaces direct causal bridge summaries, recommends the single best starting slice when the repo is too noisy to triage manually, and exposes `boundary_truncated` truth on scoped runs so cropped slices do not masquerade as isolated architecture debt.
- Task packets now carry bounded typed `graph_traces` derived from the semantic graph, including directed support paths, reverse support paths, contextual support paths, alternate paths when available, and aggregate confidence plus relation sequences.
- Task packets now also carry bounded `code_flows`, so spiders can read a claim as a stepwise path instead of reconstructing one from raw hops.
- Task packets now also carry `source_sink_paths`, which expose explicit endpoints and supporting intermediate locations so an agent can reason about “where this starts” and “where it lands” without rebuilding endpoints from raw traces.
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
