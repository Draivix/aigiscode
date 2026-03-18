# BE + FE + MCP Architecture

## Product Thesis

AigisCode is not a graph toy and not a single-detector linter.
It is meant to become the one whole-codebase architecture scanner and architect console:

- deterministic enough to trust
- explainable enough to review
- broad enough to cover mixed-language repositories
- pluggable enough to absorb framework/runtime knowledge
- usable by both humans and AI agents through the same typed evidence

The product has three equally important surfaces:

1. backend analysis engine
2. frontend architecture cockpit
3. MCP server for deep architectural analysis and guided codebase reasoning

Those three surfaces must share one typed source of truth.

## Design Reference: What To Borrow From GitNexus

Based on the local `../nexus` codebase:

- GitNexus is strong at graph-backed agent workflows.
- It exposes a clean MCP surface with tools, resources, and prompts.
- Its web UI uses proven graph tooling: React, Vite, Sigma, Graphology.
- It treats MCP as a first-class product surface, not an afterthought.

We should borrow:

- MCP as a real agent interface
- a typed server-side contract
- Sigma for the large-graph atlas
- graph adapters instead of binding UI directly to raw storage
- multi-repo thinking where it improves agent ergonomics

We should not copy:

- graph-first product framing
- raw knowledge-graph tooling as the main user experience
- a UI centered on “query the graph” instead of “understand the architecture”

GitNexus is closest to “deep code graph for AI”.
AigisCode should be “deep architecture operating system for codebases”.

## North Star

Every output surface should answer the same core questions:

- What is structurally wrong in this repository?
- Why does AigisCode believe that?
- How severe is it?
- What exact files, paths, symbols, and runtime contracts support that conclusion?
- What changed since the last run?
- What noise was absorbed into policy/rules so the next run is quieter?

That means the product center is findings plus evidence, not just nodes plus edges.

## System Shape

```text
Repository
  -> Rust ingestion + semantic graph + deterministic detectors
  -> typed run artifact
  -> Rust orchestration / policy / review / report pipeline
  -> shared architecture surface
      -> frontend cockpit
      -> MCP server
      -> archived reports
```

## Backend Architecture

## 1. Rust Core Becomes Canonical For Deterministic Analysis

Rust owns:

- repository scan
- structure graph
- semantic symbol graph
- resolution evidence
- deterministic graph analysis
- deterministic dead-code analysis
- deterministic hardwiring analysis
- visualization/MCP-friendly typed surfaces

Rust output must stay explainable:

- resolved vs unresolved references
- resolution tier
- confidence
- reason
- finding-to-evidence linkage

## 2. Legacy Python Must Be Replaced, Not Preserved

The current repository still contains Python implementations for CLI, review, reporting, and MCP work.
That is migration debt, not target architecture.

Rust should absorb:

- native CLI orchestration
- policy loading and plugin composition
- review DTO preparation and durable rule lifecycle
- external tool execution and normalization
- report assembly and archival
- MCP transport and artifact serving

Constraints:

- do not add new Python bridge or compatibility-shell layers
- do not preserve Python as a long-term host for runtime concerns
- delete Python implementation paths after Rust replacements are verified

## 3. Artifact Model

Each run should produce one canonical typed artifact family under `.aigiscode/`:

- raw deterministic run artifact
- review/policy-enriched artifact
- archived run snapshots

Minimum layers:

1. `semantic_graph`
2. `deterministic_findings`
3. `architecture_surface`
4. `review_surface`

The frontend and MCP should read `architecture_surface` first, then drill into the richer underlying artifacts when needed.

## 4. Data Ownership

- Rust owns structural facts.
- Rust owns policy/review/report/runtime concerns as migration completes.
- Frontend owns presentation state only.
- MCP owns agent-friendly access patterns only.

No surface should invent a second meaning for the same concept.

## Architecture Surface Contract

The shared surface should be smaller than the full semantic graph, but richer than a summary blob.
It should contain:

- overview counts
- language coverage
- hotspot files
- normalized finding highlights
- repository atlas nodes
- repository atlas edges
- evidence-grade metadata for drilldowns

This surface is the bridge between deterministic analysis and product UX.

## MCP Architecture

## Why MCP Matters For AigisCode

AigisCode needs MCP because AI agents are one of the primary consumers of architectural truth.
The MCP surface should give agents the same deep, explainable architecture context that the frontend gives humans.

This is not generic “repo chat”.
It is deep architectural analysis of codebases.

## Protocol Direction

Per the current MCP specification, the server should expose all three server primitives:

- tools for agent-driven actions and focused queries
- resources for stable context objects and artifacts
- prompts for guided architecture workflows

That matches AigisCode well:

- tools for analysis and explanation
- resources for static run artifacts and repo context
- prompts for guided review, refactor planning, and remediation

## Server Placement

Best near-term path:

- implement the MCP server in Rust on top of the typed Rust artifacts
- keep transport simple: stdio first

Why Rust first:

- it matches the target runtime instead of creating another bridge to delete later
- it keeps artifact ownership, transport, and lifecycle in one implementation language
- it prevents the MCP surface from becoming another reason to retain Python product code

The protocol contract still matters more than the transport library choice, but the implementation target is Rust.

## Repository Scope

AigisCode should support both:

- single-repo mode from the current working repository
- optional multi-repo registry mode later

We should not front-load global registry complexity before the single-repo contract is excellent.

Recommended progression:

1. single-repo MCP backed by local `.aigiscode/` artifacts
2. optional registry of analyzed repos
3. multi-repo lookup only when the single-repo workflows are strong

## Initial MCP Tools

The first tool set should be architecture-first:

1. `repo_overview`
   Returns architectural summary, detector coverage, top risks, stale-run metadata.

2. `list_findings`
   Filter by family, severity, file, language, confidence, visibility state.

3. `explain_finding`
   Returns exact evidence for one finding, including related files, paths, edges, and policy/review state.

4. `trace_path`
   Explains the shortest or strongest path between files/symbols/modules.

5. `show_cycles`
   Returns strong cycles and total cycles with focused subgraph data.

6. `show_hotspots`
   Returns bottlenecks, god files/classes, orphan files, runtime entry candidates, coupling metrics.

7. `compare_runs`
   Shows regressions, resolved findings, accepted noise, and convergence over time.

8. `coverage_report`
   Shows language support, detector coverage, unresolved-reference pressure, and parity gaps.

9. `suggest_remediation_plan`
   Produces a deterministic architecture-first remediation outline from existing findings.

These tools should return structured content first, with readable text as a compatibility layer.

## Initial MCP Resources

Resources should expose stable, reusable context:

- `aigiscode://repo/current/overview`
- `aigiscode://repo/current/coverage`
- `aigiscode://repo/current/findings`
- `aigiscode://repo/current/atlas`
- `aigiscode://repo/current/cycles`
- `aigiscode://repo/current/hotspots`
- `aigiscode://repo/current/run/{run_id}`
- `aigiscode://repo/current/finding/{finding_id}`

Later:

- `aigiscode://repos`
- `aigiscode://repo/{name}/...`

## Initial MCP Prompts

Prompts should guide repeatable architecture workflows:

- `triage_repo`
- `explain_architecture`
- `review_cycle`
- `plan_refactor`
- `compare_runs`
- `generate_architecture_brief`

Prompts are not substitutes for tools.
They are guided workflows over the typed artifacts and tool outputs.

## Frontend Architecture

## Product Shape

The product UI should be an architecture cockpit.
Default screen should answer “what matters now?” rather than “here is a graph”.

Core views:

1. overview
2. finding explorer
3. repository atlas
4. path explainer
5. convergence over time
6. coverage and trust surface

## Recommended Frontend Stack

Use:

- React + TypeScript + Vite
- Sigma.js for the large repository atlas
- Graphology as the in-browser graph adapter/model
- ELK.js for deterministic layouts of cycles, paths, and layered views
- React Flow only for guided process/explainer views where node-based UX is clearer than a graph atlas

This combines the strongest parts of the GitNexus stack with a better product focus.

## FE Data Flow

```text
architecture_surface JSON
  -> loader / cache
  -> view adapters
      -> overview cards
      -> finding explorer
      -> atlas graph model
      -> drilldown explainers
```

The UI should never derive architectural truth from raw graph traversal alone when the engine already knows it.

## Frontend Interaction Rules

- findings are primary navigation
- graphs are evidence and exploration
- every highlighted edge must answer “why does this exist?”
- every finding must answer “why is this here?”
- every severity must answer “why is this important?”
- every view must expose confidence and coverage context

## Better Than GitNexus

To beat GitNexus, AigisCode should be better at:

- triage of architectural debt
- explainability of findings
- cross-run convergence tracking
- policy/rules lifecycle visibility
- mixed-language whole-repo architectural understanding

We do not need to beat it by making a prettier force graph.

## Execution Plan

## Phase A

- freeze the shared architecture surface contract
- expose it from the Rust engine
- document MCP tool/resource/prompt shapes

## Phase B

- add native Rust MCP server on top of typed artifacts
- add single-repo stdio integration
- support architecture-first agent workflows

## Phase C

- build a fixture-driven frontend cockpit prototype
- implement overview, finding explorer, and atlas first

## Phase D

- add run comparison and convergence views
- add policy/review overlays
- decide whether multi-repo registry is worth the complexity

## Immediate Decisions

These are the decisions this plan locks in:

- Rust is the canonical deterministic engine.
- Rust is the target orchestration and MCP host.
- The shared product contract is a typed architecture surface, not raw graph dumps.
- MCP is a first-class AigisCode surface.
- The frontend is a finding-first architecture cockpit.
- GitNexus is an inspiration source, not the target product shape.
