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
aigiscode report /path/to/project
aigiscode info /path/to/project
aigiscode plugins
aigiscode tune /path/to/project
aigiscode surface /path/to/project
aigiscode --version
```

`analyze` writes:

- `.aigiscode/deterministic-analysis.json`
- `.aigiscode/semantic-graph.json`
- `.aigiscode/dependency-graph.json`
- `.aigiscode/evidence-graph.json`
- `.aigiscode/contract-inventory.json`
- `.aigiscode/doctrine-registry.json`
- `.aigiscode/deterministic-findings.json`
- `.aigiscode/external-analysis.json`
- `.aigiscode/architecture-surface.json`
- `.aigiscode/review-surface.json`
- `.aigiscode/convergence-history.json`
- `.aigiscode/guard-decision.json`
- `.aigiscode/aigiscode-handoff.json`
- `.aigiscode/aigiscode-report.json`
- `.aigiscode/aigiscode-report.md`

## Recommended Agent Workflow

1. Run `aigiscode analyze /repo`.
2. Parse `.aigiscode/aigiscode-report.json` for the consolidated machine contract.
3. Read `.aigiscode/deterministic-findings.json` for raw detector output.
4. Use `.aigiscode/dependency-graph.json` for low-noise architecture queries.
5. Use `.aigiscode/evidence-graph.json` for detailed call-site and runtime evidence.
6. Use `.aigiscode/contract-inventory.json` for declared routes, hooks, env/config keys, and symbolic runtime contracts.
7. Use `.aigiscode/doctrine-registry.json` for the machine-readable guardian doctrine and default clause disposition.
8. Use `.aigiscode/architecture-surface.json` and `.aigiscode/review-surface.json` for topology and triage.
9. Use `.aigiscode/convergence-history.json` to compare the current run against the previous artifact baseline in the same output directory.
10. Use `.aigiscode/guard-decision.json` for the current allow/warn/block judgment and required review radius.
11. Use `.aigiscode/aigiscode-handoff.json` when handing the repository to another agent.
12. Use `aigiscode tune /repo` when you want a conservative starting patch for `.aigiscode/policy.json`.
13. Re-run `aigiscode report /repo` after fixes.

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
