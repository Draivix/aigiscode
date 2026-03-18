# Architecture

## Purpose

AigisCode is a native Rust analysis engine for whole-codebase structural
inspection. The product contract is typed artifacts plus a small CLI surface.

## Current Pipeline

1. Scan repository structure
2. Parse supported languages into a semantic graph
3. Resolve symbols and edges across files
4. Run deterministic structural analysis
5. Run native detector passes
6. Emit machine-readable artifacts and architecture surface JSON

## Module Ownership

- `ingestion/`: repository scan and pipeline execution
- `parsing/`: language extraction adapters
- `resolve/`: import and symbol resolution
- `graph/`: cycles, bottlenecks, orphan detection, and graph summaries
- `detectors/`: dead code and hardwiring
- `surface/`: architecture-focused output contract
- `artifacts.rs`: canonical artifact writer
- `cli.rs`: command routing and output policy

## Artifact Contract

`analyze` writes:

- `.aigiscode/deterministic-analysis.json`
- `.aigiscode/semantic-graph.json`
- `.aigiscode/dependency-graph.json`
- `.aigiscode/evidence-graph.json`
- `.aigiscode/deterministic-findings.json`
- `.aigiscode/external-analysis.json`
- `.aigiscode/architecture-surface.json`
- `.aigiscode/review-surface.json`
- `.aigiscode/aigiscode-handoff.json`
- `.aigiscode/aigiscode-report.json`

`surface` writes:

- `.aigiscode/architecture-surface.json`

## Design Rules

- Rust is the only supported runtime.
- The artifact family is the only supported machine interface.
- New product features must build on Rust contracts, not sidecar runtimes.
- Keep the CLI thin and push real logic into typed library modules.

## Removed Legacy Surfaces

The previous Python CLI, report pipeline, plugin-module system, analytical mode,
and Python-hosted MCP slice are removed from the repository. New work should
target Rust-native equivalents instead of recreating them in another runtime.
