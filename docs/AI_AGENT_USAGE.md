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
aigiscode report /repo
aigiscode analyze /repo --external-tool gitleaks
```

Primary outputs:

```text
/repo/.aigiscode/deterministic-analysis.json
/repo/.aigiscode/semantic-graph.json
/repo/.aigiscode/deterministic-findings.json
/repo/.aigiscode/external-analysis.json
/repo/.aigiscode/architecture-surface.json
/repo/.aigiscode/review-surface.json
/repo/.aigiscode/aigiscode-report.json
```

## Recommended Agent Loop

1. Run `aigiscode analyze /repo`.
2. Parse `aigiscode-report.json` for the consolidated graph, detector, and summary contract.
3. Parse `deterministic-findings.json` for dead-code and hardwiring findings.
4. Parse `external-analysis.json` when the run included `--external-tool ...`.
5. Parse `architecture-surface.json` for hotspots, atlas views, and finding-first UI.
6. Parse `review-surface.json` for unreviewed finding state and future policy/review overlays.
7. Fix a bounded set of issues.
8. Re-run `aigiscode report /repo`.

## Practical Guidance

- Treat the Rust artifact family as the source of truth.
- Prefer `deterministic-findings.json` when you want actionable queues.
- Prefer `external-analysis.json` when you want normalized external scanner evidence and raw-artifact locations.
- Prefer `architecture-surface.json` when you want topology, hotspots, or UI input.
- Prefer `review-surface.json` when you want review-state workflow over deterministic findings.
- Use `.aigiscode/policy.json` for project-wide suppression patterns and detector thresholds.
- Use `.aigiscode/policy.json` for native and external finding acceptance patterns, including `external.skip_tools`, `external.skip_categories`, and `external.allowed_rule_ids`.
- Use `.aigiscode/rules.json` for narrow per-finding exclusions that should persist across runs, including external-finding matches by type, file pattern, and optional `tool`.
- Use `--output-dir <path>` when you need artifacts outside `.aigiscode/`.
- Use `--no-write` if you only want JSON on stdout.
- Use `--external-tool <name>` or `--external-tools all` when you want Rust to run supported external analyzers and archive raw evidence under `.aigiscode/reports/<run_id>/raw/`.

## Current Scope

Supported product commands today:

- `aigiscode analyze <path>`
- `aigiscode report <path>`
- `aigiscode analyze-rust <path>`
- `aigiscode surface <path>`
- `aigiscode mcp <path>`
- `aigiscode version`

Legacy Python `report`, `tune`, and `plugins` flows are removed from the
repository. MCP is now provided by the native Rust stdio server.
