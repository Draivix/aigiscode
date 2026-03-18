# Graphing System Progress

Date: 2026-03-18

Purpose:
- internal engineering report
- source material for future blog posts
- technical explanation of how AigisCode is building its graphing system and why

## Executive Summary

AigisCode is being built as a whole-codebase graphing and reasoning system, not
as a flat static-analysis report generator.

The current graphing direction is:

- canonical graph in native Rust
- typed structural and runtime/framework edges
- plugin-expanded behavior for framework conventions
- optional Kuzu read model for querying, MCP, and repository understanding
- normalized dependency view for low-noise graph reporting
- richer evidence view retained in JSON artifacts for deep analysis

This matters because our product goal is not simply to count nodes and edges.
The real goal is to build a graph that helps humans and AI understand how a
codebase is actually wired, where the architecture is healthy, where it is
degrading, and how security and runtime behavior fit into that picture.

That is the difference between a noisy code graph and a useful guardian system.

## Product Context

The graphing system is being built under the contracts in:

- [GOAL.md](/home/david/Work/Programming/aigiscode.com/GOAL.md)
- [ZEUS_SHIELD.md](/home/david/Work/Programming/aigiscode.com/ZEUS_SHIELD.md)

The short version:

- AigisCode should ensure architectonic quality and security of analyzed software
- it should do this through explainable, typed, evidence-preserving analysis
- it should outperform `../nexus` not by being noisier, but by having stronger
  graph meaning and stronger architectural judgment

## Current Graphing Architecture

### 1. Canonical Semantic Graph in Rust

The canonical source of truth is the Rust semantic graph.

Relevant code:

- [graph/mod.rs](/home/david/Work/Programming/aigiscode.com/rust/crates/aigiscore/src/graph/mod.rs)
- [resolve/mod.rs](/home/david/Work/Programming/aigiscode.com/rust/crates/aigiscore/src/resolve/mod.rs)
- [parsing/mod.rs](/home/david/Work/Programming/aigiscode.com/rust/crates/aigiscore/src/parsing/mod.rs)

The graph currently contains:

- files
- symbols
- semantic references
- resolved edges

Resolved edges now carry typed metadata:

- `ReferenceKind`
- `RelationKind`
- `GraphLayer`
- `EdgeStrength`
- `EdgeOrigin`
- `ResolutionTier`
- confidence and reason

This is the first important architectural choice:

- the graph is not just “A depends on B”
- it is “A depends on B through this relation, at this layer, with this
  confidence, for this reason”

That is critical for explainability and later doctrine-based judgment.

### 2. Layered Meaning

We have moved away from a flat edge set.

The current model distinguishes:

- structural edges
- runtime edges
- framework edges
- policy-overlay edges

This allows us to ask different questions against different graph views:

- structural cycles
- runtime-expanded cycles
- mixed cycles
- likely framework artifacts

This is already a material architectural advantage over flatter graph systems,
because real software often looks different at runtime than it does in plain
source imports and method calls.

### 3. Plugin-Expanded Runtime Behavior

Framework behavior is not being hardcoded into core language semantics.

Relevant code:

- [plugins/queue.rs](/home/david/Work/Programming/aigiscode.com/rust/crates/aigiscore/src/plugins/queue.rs)
- [plugins/container.rs](/home/david/Work/Programming/aigiscode.com/rust/crates/aigiscore/src/plugins/container.rs)
- [plugins/wordpress.rs](/home/david/Work/Programming/aigiscode.com/rust/crates/aigiscore/src/plugins/wordpress.rs)

Current plugin-expanded behavior includes:

- queue/job dispatch edges
- Laravel container-resolution edges
- WordPress hook publish/subscribe edges

This is the second major architectural choice:

- language truth belongs in core
- framework truth belongs in plugins
- repo-specific accepted behavior belongs in policy/rules

Without this separation, the product would collapse into repo-specific hacks.

### 4. Optional Kuzu Read Model

The Rust graph remains canonical.
Kuzu is used as a query-optimized read model.

Relevant code:

- [kuzu_index.rs](/home/david/Work/Programming/aigiscode.com/rust/crates/aigiscore/src/kuzu_index.rs)
- [kuzu_bridge.mjs](/home/david/Work/Programming/aigiscode.com/tools/kuzu_bridge.mjs)
- [cli.rs](/home/david/Work/Programming/aigiscode.com/rust/crates/aigiscore/src/cli.rs)
- [mcp/mod.rs](/home/david/Work/Programming/aigiscode.com/rust/crates/aigiscore/src/mcp/mod.rs)

Why this architecture was chosen:

- analysis logic should not be coupled to storage mechanics
- JSON artifacts remain the portable source of truth
- Kuzu is excellent for graph querying, MCP access, and architecture exploration
- this gives us graph-database power without making the analyzer itself
  storage-led

The CLI currently exposes this through:

- `aigiscode graph <repo> --kuzu`
- `aigiscode cypher <repo> "<query>"`

The MCP server also exposes raw graph querying for code understanding.

## Raw Graph vs Dependency View

This was the most important graphing correction in the latest iteration.

Originally, the Kuzu export was too noisy for parity comparison because it
included:

- one synthetic `MODULE` node per analyzed file
- one `CONTAINS` edge per symbol
- repeated call-site edges counted individually

That made the graph look much larger than `../nexus`, but not all of that
difference was real architectural value.

We now separate two concepts:

### Canonical Graph

This remains rich and evidence-heavy.

It is allowed to keep:

- repeated call sites
- detailed runtime/plugin edges
- fine-grained semantic information
- evidence needed for deep investigation

### Dependency View

This is the Kuzu export used for parity, reporting, and query workloads.

It now:

- omits synthetic `MODULE` nodes
- omits `CONTAINS` edges
- remaps module-targeted edges onto file nodes
- collapses repeated dependencies into one edge with `occurrenceCount`

This was the right move because we do not want to throw away useful canonical
graph detail, but we also do not want our query/read model to be polluted by
representational noise.

In other words:

- canonical graph optimizes for truth and evidence
- dependency view optimizes for low-noise architectural interpretation

## WordPress Benchmark Progress

The real benchmark repo is:

- `/home/david/Work/Programming/wordpress`

Reference system:

- `/home/david/Work/Programming/nexus/gitnexus`

### Older Raw Export

Initial AigisCode Kuzu totals on WordPress:

- `36,202` nodes
- `156,050` relationships

This was too inflated to use as a fair parity metric.

### Current Normalized Dependency View

Current AigisCode release-binary run:

- command:
  - `./target/release/aigiscode graph ../wordpress --kuzu`
- wall clock:
  - `22.78s`

Current normalized totals:

- nodes: `32,862`
- relationships: `95,878`

Current relationship breakdown:

- `CALL`: `85,451`
- `EVENTPUBLISH`: `3,662`
- `OVERRIDES`: `1,947`
- `EVENTSUBSCRIBE`: `1,868`
- `EXTENDS`: `1,489`
- `IMPORT`: `764`
- `TYPEUSE`: `625`
- `IMPLEMENTS`: `72`

GitNexus on the same repo:

- wall clock: `22.4s`
- nodes: `19,692`
- edges: `64,453`

Interpretation:

- we are now essentially on par in throughput
- we are still denser than GitNexus
- but the density is now much less contaminated by export artifacts
- the remaining difference is much more likely to be semantic coverage rather
  than graph inflation

## What We Learned From Hand Checks

We did not rely only on totals.
We hand-checked real WordPress files against the graph.

### `src/wp-includes/class-wp-block-supports.php`

Finding:

- AigisCode captures much stronger inbound usage coverage than GitNexus
- incoming `CALL` edges:
  - AigisCode: `27`
  - GitNexus: `6`

This matters because the file is a reusable internal service and architectural
understanding depends heavily on seeing who actually reaches into it.

### `src/wp-content/themes/twentyten/search.php`

Finding:

- AigisCode captures stronger template-level call coverage than GitNexus
- after normalization, the file is no longer polluted by a synthetic `MODULE`
  node in the parity graph

This suggests our graph is not simply “fatter.” On simple theme files, it can
already be more truthful.

### `src/wp-admin/includes/credits.php`

This file exposed a real resolver defect during the earlier comparison.

Before the fix:

- `wp_remote_get()` missed
- `esc_attr()` missed
- `translate()` resolved to a method target instead of the global function

Root cause:

- PHP parameter extraction was undercounting some real functions
- call resolution used too-rigid arity matching
- free calls could still compete with same-named methods

Fixes implemented:

- required-parameter counts added to symbols
- call resolution now accepts arities within `[required, total]`
- free calls prefer free-function candidates
- PHP parameter parsing now counts simple parameters correctly even without type
  annotations

After the fix, hand-checked on the real repo:

- `wp_remote_get()` resolves at line `42`
- `esc_attr()` resolves at line `147`
- `translate()` resolves as a global `FUNCTION` at lines `104` and `156`

This is exactly the kind of improvement loop we want:

1. compare graph against reality
2. find concrete falsehood
3. fix generic semantics
4. rerun the real repo
5. hand-check again

## Why These Design Choices Matter

### 1. Better Signal For Humans

A noisy graph is not useful for architectural work.

By separating canonical evidence from normalized dependency view, we make the
graph usable for:

- architecture reports
- parity comparisons
- cycle reasoning
- hotspot analysis

without destroying the deeper evidence needed for investigation.

### 2. Better Inputs For AI

The graph is not only for visualizations.
It is becoming the memory and reasoning surface for AI agents.

That means the graph must support:

- precise dependency queries
- runtime/framework interpretation
- low-noise architectural summaries
- evidence-preserving drill-downs

If the graph is too flat, AI misses context.
If it is too noisy, AI drifts and hallucinates structure.

### 3. Better Foundation For Zeus Shield

Zeus Shield depends on high-quality codebase understanding.

The guardian cannot judge:

- architectural drift
- dependency overkill
- pattern incoherence
- framework misuse

unless the graph distinguishes:

- structural truth
- runtime/plugin expansion
- repo-specific policy truth
- evidence vs normalized dependency shape

This graphing work is not a side feature.
It is the substrate for the whole guardian system.

## Technical Direction From Here

The next graphing work should focus on the remaining high-value questions.

### 1. Keep The Two-View Model Explicit

We now have the right conceptual direction:

- canonical evidence graph
- normalized dependency view

That distinction should become first-class in more surfaces, not only the Kuzu
export.

### 2. Continue Parity Hand Checks

The remaining gap with GitNexus is not obvious representational noise anymore.
The next question is:

- is the remaining extra `CALL` density mostly real signal
- or do file-scope PHP calls still need a narrower dependency projection for
  architectural reporting

That must be answered with more hand-checked samples, not by intuition.

### 3. Keep Fixing Generic Language Semantics

Recent progress came from generic fixes, not WordPress hacks.

That is the correct path.

The rule remains:

- language truth in core
- framework truth in plugins
- repo truth in rules/policy

### 4. Expand Runtime and Security Meaning

The graph is already carrying:

- WordPress hook edges
- queue edges
- container-resolution edges

That needs to continue, because a guardian system must see more than imports and
method calls.

## Current Position

We are no longer in the phase where the graph is obviously polluted by export
artifacts.

We are now in the more serious phase:

- improving semantic precision
- comparing against real repositories
- measuring graph quality by truth and usefulness
- preparing the graph as the foundation for architecture guidance and AI guard
  rails

That is a much better place to be.

## Suggested Blog Angles

This report can seed multiple technical blog posts.

Possible directions:

1. `Why Flat Code Graphs Fail For Architectural Reasoning`
   - canonical graph vs dependency view
   - evidence vs architectural signal

2. `How We Benchmarked Our Code Graph Against GitNexus On WordPress`
   - real repo
   - hand-checked truth
   - why raw node counts are misleading

3. `Why We Use Rust For Canonical Graphing And Kuzu For Querying`
   - analyzer core vs read model
   - typed semantics vs graph-database ergonomics

4. `How Optional Parameters Broke Our PHP Graph And How We Fixed It`
   - WordPress example
   - generic parser/resolver improvement
   - why hand checks matter

5. `Building Zeus Shield: The Graph Layer Beneath AI Code Governance`
   - graphing as substrate for architecture guard rails
   - structural vs runtime truth
   - doctrine-aware analysis
