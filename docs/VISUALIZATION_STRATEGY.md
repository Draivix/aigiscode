# Visualization Strategy

## Purpose

AigisCode is not a generic graph browser.
It is intended to be a whole-codebase scanner and architecture tool:

- detect structural risk across an entire repository
- explain why the system believes a finding is real
- show convergence over time through rules, policy, and AI review
- let a human move from repository-wide signal to exact evidence quickly

The visualization layer should be designed around those goals.

## What GitNexus Uses

Based on the local `../nexus` codebase:

- frontend: React 18 + TypeScript + Vite
- styling: Tailwind CSS v4
- icons: `lucide-react`
- large-graph renderer: `sigma`
- graph data structure: `graphology`
- layout: `graphology-layout-forceatlas2`, `graphology-layout-force`, `graphology-layout-noverlap`
- curved edges: `@sigma/edge-curve`
- diagrams for side views: `mermaid`
- browser-side analysis support: `web-tree-sitter`, `kuzu-wasm`, workers via `comlink`

Relevant local references:

- [gitnexus-web/package.json](/home/david/Work/Programming/nexus/gitnexus-web/package.json)
- [GraphCanvas.tsx](/home/david/Work/Programming/nexus/gitnexus-web/src/components/GraphCanvas.tsx)
- [useSigma.ts](/home/david/Work/Programming/nexus/gitnexus-web/src/hooks/useSigma.ts)
- [graph-adapter.ts](/home/david/Work/Programming/nexus/gitnexus-web/src/lib/graph-adapter.ts)

## What Is Worth Borrowing

- Sigma.js for large-graph rendering is a sound choice.
- Graphology is a reasonable frontend graph model and adapter layer.
- Separate layout handling from graph rendering.
- Use focused side panels and code reference panels instead of trying to do everything inside the canvas.
- Keep the graph interactive but avoid forcing all analysis understanding through one giant graph view.

## What We Should Do Better

GitNexus is graph-centric.
AigisCode should be finding-centric and architecture-centric.

That means the primary experience should not start with “here is the graph”.
It should start with “here is what matters in this codebase”.

The UI should privilege:

- strongest cycles
- layer violations
- bottleneck files
- god classes
- dead-code hotspots
- hardwiring clusters
- detector coverage gaps
- review/rules/policy lifecycle

Then the graph becomes evidence and navigation, not the product itself.

## Recommended Stack

Use a split visualization stack:

1. Sigma.js
   Use for the global repository atlas and large, dense interactive graphs.

2. ELK.js
   Use for deterministic layouts of focused subgraphs:
   - cycle explanation
   - dependency path view
   - layer map
   - import chain
   - call path evidence

3. React Flow
   Use for guided explainer views and architecture stories:
   - detector pipeline
   - feedback loop lifecycle
   - remediation flow
   - rules/policy/AI-review state transitions

Sigma should not be used for every view.
ELK-based focused diagrams will often explain architecture more clearly than a force layout.

## Product Views

### 1. Executive Overview

Top-level scan summary for the whole repo:

- architectural risk score
- highest-severity findings
- detector coverage by language
- trend vs previous run
- unresolved high-confidence findings
- newly accepted patterns and generated rules

### 2. Repository Atlas

Large interactive map of the codebase:

- communities / bounded regions
- modules and folders
- bottlenecks
- entry points
- strongly connected components
- heat overlays by finding family

### 3. Finding Explorer

Canonical workbench for triage:

- list grouped by detector and severity
- each finding opens:
  - why detected
  - source files
  - path evidence
  - confidence
  - policy/rule history
  - affected neighbors

### 4. Path Explainer

Deterministic subgraph for one problem:

- shortest dependency chain
- cycle ring
- layer break route
- exact call/import/extends edges used
- unresolved vs resolved references
- confidence per edge

### 5. Convergence View

This is where AigisCode should clearly beat generic graph tools.

Show:

- finding status changes across runs
- false positives accepted into policy
- rules generated from reviewed findings
- noise reduction over time
- new regressions vs historical baseline

### 6. Detector Coverage View

Show what the engine does and does not know:

- languages found
- detectors active by language
- unresolved-reference counts
- partial-parity warnings
- framework/runtime profile influence

This is critical for explainability and trust.

## Data Contract Requirements

The frontend should not reconstruct meaning from raw edges alone.
The engine should emit typed visualization DTOs or at minimum enough typed metadata for adapters to build them cleanly.

Needed concepts:

- resolved vs unresolved edges
- edge evidence and confidence tier
- finding-to-evidence mapping
- detector family and severity
- per-run lifecycle state
- community / module grouping
- path summaries for drilldowns
- coverage metadata

## Visual Design Direction

The UI should feel more like a serious architecture console than a developer toy.

Desired qualities:

- calm, dense, high-signal layout
- strong hierarchy and typography
- graph as one tool among several, not the whole page
- excellent side panels and inspectors
- minimal decorative motion
- color used semantically, not just aesthetically

Avoid:

- force-graph-first UX
- neon cyberpunk styling without information value
- overwhelming edge soup as the default screen
- “AI dashboard” clutter

## Near-Term Plan

1. Freeze the visualization product requirements before writing UI code.
2. Add visualization-oriented DTOs to the native analysis output or a thin adapter layer.
3. Build a static demo page from fixture JSON before wiring the full app.
4. Implement the three-view foundation:
   - overview
   - finding explorer
   - repository atlas
5. Add deterministic path explainer views after the evidence DTO contract is stable.

## Decision

For AigisCode, the right target is:

- better explanation than GitNexus
- better finding triage than GitNexus
- better multi-run convergence visibility than GitNexus
- equal or better large-graph performance than GitNexus

That is achievable if we treat visualization as a product surface for architecture analysis, not just as graph rendering.
