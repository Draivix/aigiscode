# AGENTS.md

> AigisCode is a native Rust whole-codebase analyzer. This file gives AI coding
> agents the current project contract.

## Deployment

- Website host: `21.davidstrejc.cz:/home/agisicode.com/public_html`
- Website build: `cd website && npm run build`
- Domain: `aigiscode.com`

## Product Direction

- `GOAL.md` is the top-level product contract. Read it before making product,
  graph, detector, plugin, or reporting decisions.
- `ZEUS_SHIELD.md` defines the guardian doctrine for preventing architectural
  drift, dependency cancer, and AI-generated incoherence. Use it when making
  decisions about doctrine, enforcement, convergence, and codebase guidance.
- The implementation target is native Rust end-to-end.
- Do not add new Python bridges, Python tooling, or Python-hosted runtime paths.
- The website may describe roadmap items, but only the Rust CLI is a supported
  product surface today.

## Current CLI

```bash
aigiscode analyze /path/to/project
aigiscode agent /path/to/project
aigiscode agent-run /path/to/project
aigiscode agent-spider /path/to/project
aigiscode report /path/to/project
aigiscode info /path/to/project
aigiscode plugins
aigiscode tune /path/to/project
aigiscode surface /path/to/project
aigiscode mcp /path/to/project           # native Rust stdio MCP server (one-shot snapshot)
aigiscode mcp /path/to/project --watch   # live daemon: re-analyze on file changes, serve a freshness contract
aigiscode --version
```

The `mcp --watch` daemon keeps the graph live: a filesystem watcher re-analyzes
on change and atomically republishes the snapshot, and the `repo_overview` tool
returns a `Freshness` contract (`revision`/`indexed_revision`/`observed_revision`/
`is_stale`/`dirty_paths`/`generated_at_unix_ms`) plus optional
`min_revision`/`consistency`/`wait_ms` params, so an agent editing files is never
silently served a stale graph. See `docs/ONLINE_CODE_GRAPH_ARCHITECTURE.md`.

`analyze` writes:

- `.aigiscode/deterministic-analysis.json`
- `.aigiscode/semantic-graph.json`
- `.aigiscode/dependency-graph.json`
- `.aigiscode/evidence-graph.json`
- `.aigiscode/contract-inventory.json`
- `.aigiscode/doctrine-registry.json`
- `.aigiscode/deterministic-findings.json`
- `.aigiscode/ast-grep-scan.json`
- `.aigiscode/external-analysis.json`
- `.aigiscode/architecture-surface.json`
- `.aigiscode/review-surface.json`
- `.aigiscode/convergence-history.json`
- `.aigiscode/guard-decision.json`
- `.aigiscode/aigiscode-handoff.json`
- `.aigiscode/agentic-review.json`
- `.aigiscode/graph-packets.json`
- `.aigiscode/repository-topology.json`
- `.aigiscode/aigiscode-report.json`
- `.aigiscode/aigiscode-report.md`
- `.aigiscode/scan-manifest.json`

`scan-manifest.json` is the fast-load fingerprint: with `AIGISCORE_FAST_LOAD=1`,
`aigiscode mcp` verifies it (version, resolver-config hash, per-file content
hashes) and skips Parse+Resolve when everything matches, declining to a full
analysis on any mismatch.

## Recommended Agent Workflow

1. Run `aigiscode analyze /repo`.
2. Parse `.aigiscode/aigiscode-report.json` for the consolidated machine contract.
3. Read `.aigiscode/deterministic-findings.json` for raw detector output.
4. Use `.aigiscode/ast-grep-scan.json` when you want provenance-rich structural rule hits from the secondary scanner plane. Today it covers loop-local expensive-operation clues for `AlgorithmicComplexityHotspot`, dangerous-API clues that can reinforce native `SecurityDangerousApi` findings (`eval`, `exec/system`, unsafe deserialization, unsafe HTML output), and narrow framework-misuse clues for both raw env access outside config/bootstrap boundaries and raw container/service-locator lookup outside provider/bootstrap or injection boundaries that can reinforce native `SanctionedPathBypass` findings. The engine stays generic, while framework-specific rule catalogs can now contribute findings with explicit provenance such as `ast_grep.pattern.laravel` and `ast_grep.pattern.django`; this artifact is evidence, not graph truth. `aigiscode-report.json.summary` and `architecture-surface.json.overview` now also expose family-level scanner counts so the scanner mix is visible without loading the raw scanner artifact.
5. Use `.aigiscode/dependency-graph.json` for low-noise architecture queries.
6. Use `.aigiscode/evidence-graph.json` for detailed call-site and runtime evidence.
7. Use `.aigiscode/contract-inventory.json` for declared routes, hooks, env/config keys, and symbolic runtime contracts.
8. Use `.aigiscode/doctrine-registry.json` for the machine-readable guardian doctrine and default clause disposition.
9. Use `.aigiscode/architecture-surface.json` and `.aigiscode/review-surface.json` for topology and triage. On cropped/scoped analyses these artifacts now distinguish confirmed orphan debt from `boundary-truncated` files whose callers may live outside the analyzed slice.
10. Use `.aigiscode/convergence-history.json` to compare the current run against the previous artifact baseline in the same output directory.
11. Use `.aigiscode/guard-decision.json` for the current allow/warn/block judgment and required review radius.
12. Use `.aigiscode/aigiscode-handoff.json` when handing the repository to another agent.
13. Use `.aigiscode/agentic-review.json` or `aigiscode agent /repo` when you want the graph-backed AI review contract, prompts, diff-aware task packets, evidence chains, bounded typed multi-path graph traces, bounded code flows, explicit source/sink paths, bounded semantic state-flow evidence for mutable carriers, artifact priorities, and the adapter catalog for execution. The contract now also treats `.aigiscode/graph-packets.json` and `.aigiscode/repository-topology.json` as first-class agent context, not side artifacts.
14. Use `.aigiscode/graph-packets.json` when you want bounded doctrine-aware graph neighborhoods for agents or reviewers without loading the entire graph artifact family. Fallback focus-file packets now also carry bounded traces, code flows, source/sink paths, and semantic state-flow evidence instead of empty shells.
15. Use `.aigiscode/repository-topology.json` when you want a flatter agent-facing repository map over zones, manifests, runtime entries, contract-bearing directories, cross-zone links, direct zone-to-finding / zone-to-packet links, and now topology-level semantic-state previews for mutable-carrier flows. Route-declared files now also promote runtime-entry shape in this artifact, scoped analyses now expose explicit `boundary_truncated` truth instead of fake orphan pressure, and it carries a topology-level recommended start slice, zone-level triage briefs, structured triage steps, focus clusters for flat zones, explicit cross-zone pressure summaries and linked-zone previews, direct causal bridge summaries, spillover observations, evidence refs, convergence-state hints, lightweight ownership hints with explicit basis metadata, per-step/per-cluster semantic-state labels, semantic-state proof summaries plus flow-kind/proof-tier previews, and compact semantic-state flow refs with stable IDs when that evidence exists.
16. Use `aigiscode agent-run /repo --adapter codex-exec` when you want AigisCode to execute a real local agent review and write `agent-review.json`, `agent-review.md`, `agent-output-schema.json`, and `agent-execution.jsonl`.
17. Use `aigiscode agent-run /repo --adapter responses-http` when you want the same graph-backed review executed through the direct Rust OpenAI Responses adapter. This path requires `OPENAI_API_KEY`.
18. Use `aigiscode agent-spider /repo --adapter ... --limit N` when you want AigisCode to crawl the top graph-backed task packets and persist one report per packet plus `agent-spider-report.json`.
19. Use `aigiscode tune /repo` when you want a conservative starting patch for `.aigiscode/policy.json`.
20. Re-run `aigiscode report /repo` after fixes.

## Project Structure

```text
Cargo.toml                            # Root Rust workspace
rust/crates/aigiscore/
├── Cargo.toml                        # Main crate manifest
└── src/
    ├── artifacts.rs                  # Native artifact writing
    ├── cli.rs                        # Shared CLI implementation
    ├── ingestion/                    # Scan and pipeline orchestration
    ├── parsing/                      # Language adapters
    ├── resolve/                      # Semantic resolution
    ├── semantic_models/              # Framework/library semantic model packs
    ├── graph/                        # Structural analysis
    ├── detectors/                    # Dead code and hardwiring
    ├── surface/                      # Architecture surface contract
    └── bin/
        ├── aigiscode.rs              # Product binary
        └── aigiscore.rs              # Compatibility binary
website/                              # Marketing site and cockpit prototype
docs/                                 # Product and architecture docs
```

## Build And Test

```bash
cargo fmt
cargo test
cd website && npm ci && npm run build
```

## Architecture Rules

- Keep parsing, resolution, graph analysis, detectors, and surface generation in
  Rust crates with typed contracts.
- `cli.rs` is orchestration only. Do not move detector heuristics or serialization
  logic into the command layer.
- `artifacts.rs` owns artifact paths and file emission. Do not duplicate artifact
  writing in other modules.
- `surface/` owns the architecture-facing contract used by MCP/UI work.
- Do not reintroduce secondary sources of truth for findings outside the Rust
  artifact family.

## Quality Bar

- No repo-specific heuristics in core without a clear generalized reason.
- No silent failures that look like clean analysis.
- No undocumented artifact path changes.
- No new language-specific sidecars in other runtimes.
