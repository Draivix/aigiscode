# Internet Research: Code Graphing in 2026

Date: 2026-03-18

Purpose: summarize current public best practice for code graphing, repository-scale code understanding, graph storage/query patterns, and AI-oriented graph retrieval, then translate that research into concrete product guidance for AigisCode.

This document is based on official product/documentation sources where possible, plus recent public research papers where the topic is forward-looking or AI-specific.

## Executive Summary

There is no single "best graph for code." The current best practice is a layered architecture with multiple graph projections:

- A canonical semantic graph with typed nodes, typed edges, evidence, and language-specific truth.
- One or more overlays for control/data flow, framework/runtime behavior, or higher-level semantics.
- A low-noise dependency projection for architecture reporting, parity comparisons, SCC analysis, and impact queries.
- A navigation-oriented projection for definition/reference workflows.
- A query/read model optimized for interactive exploration, agent workflows, and graph algorithms.

The strongest public systems do not flatten these use cases into one graph:

- Joern uses a Code Property Graph with overlays and a custom graph engine.
- CodeQL uses per-language extraction into a relational database with AST/CFG/data-flow views.
- Sourcegraph uses SCIP for precise code navigation and strongly emphasizes deterministic index generation and build-aware indexing.
- Recent repository-level AI work uses repository graphs for retrieval, planning, and long-horizon repository reasoning.

The main implication for AigisCode is straightforward:

- Keep the Rust semantic graph as canonical truth.
- Keep `dependency_view` as a normalized architecture/read model.
- Add a first-class `evidence_view` instead of overloading the canonical graph JSON for all deep analysis use cases.
- Consider a future `navigation_view` or SCIP export for definition/reference interoperability.
- Keep Kuzu as an optional query/read model, not the source of truth for extraction or detection.

## Research Questions

1. What graph models are currently considered effective for real code analysis?
2. How do mature systems separate extraction, analysis, and storage/query concerns?
3. What do current 2025-2026 repository/AI papers imply for product direction?
4. What should AigisCode build next if the goal is architectonic quality, security, and AI guidance?

## Findings

### 1. The strongest public pattern is a layered graph, not a flat dependency graph

Joern documents the Code Property Graph as a single intermediate representation that merges different classic program views into one property graph, then extends it with overlays. The docs explicitly describe nodes, typed edges, and properties as core building blocks, and also note that the representation has evolved to support multiple language frontends and multiple levels of abstraction through overlays.

Implication:

- AigisCode should not treat "graph" as only file-to-file imports or calls.
- Structural edges, runtime/framework edges, and future control/data-flow evidence should be distinct layers or overlays.
- The existing `GraphLayer` direction in Rust is aligned with public best practice.

What matters most from Joern:

- use a single canonical graph with typed structure
- support higher-level overlays instead of mutating core truth
- preserve enough richness to ask different classes of questions over the same base representation

### 2. Language-specific extraction remains non-negotiable

CodeQL's official docs still emphasize language-specific extractors and per-language database schemas. For compiled languages, extraction observes the real build process and captures syntax plus semantic binding/type information. For interpreted languages, extraction runs directly on source while resolving dependencies. CodeQL databases contain AST, control flow, and data flow.

Implication:

- AigisCode should keep language truth in per-language extraction and resolution logic, not attempt one universal shallow parser layer.
- Build-aware extraction matters. For languages and ecosystems where real builds materially improve truth, the analyzer should eventually integrate build context instead of relying only on file discovery.
- Multi-language support should remain explicit and typed, not blended into one lossy schema.

What this means for our roadmap:

- preserve parser/resolver accuracy work as a core priority
- treat framework/plugin logic as a separate concern from language extraction
- eventually add build-aware adapters where semantic truth depends on real project configuration

### 3. Navigation graphs and architecture graphs are different products

SCIP is explicitly a language-agnostic code intelligence protocol for navigation features such as go-to-definition, references, and implementations. Sourcegraph's docs make two important recommendations:

- indexers should use real compiler frontends or language servers after semantic analysis
- index output should be deterministic, and snapshot testing is recommended

SCIP is powerful, but it is intentionally navigation-oriented. It is not a replacement for a richer architecture/security graph.

Implication:

- AigisCode should not confuse navigation truth with architecture truth.
- A future `navigation_view` or SCIP export would be useful, but it should sit beside `dependency_view` and `evidence_view`, not replace them.
- Determinism and snapshot/golden testing should be treated as hard quality requirements for graph generation.

Concrete product takeaway:

- exporting a SCIP-compatible projection would likely improve interoperability and code-understanding workflows
- but AigisCode's differentiated value remains the richer architecture/runtime/policy graph, not only navigation indexes

### 4. Storage engine choice is secondary to graph contract quality

Joern's docs are unusually direct on this point: older Joern versions used general-purpose graph databases, but the project later replaced both storage backend and query language with its own custom graph engine after the limitations of that approach became apparent.

Kuzu's current docs position it as an embedded graph database with official client APIs, transactional query support, and graph algorithms executed on projected graphs. It is a strong fit for local query/read workloads and interactive graph exploration.

Implication:

- "Which database?" is not the first architecture question.
- The first question is: what is the canonical graph contract, and which projections are materialized for which workloads?
- Kuzu is a good optional read model for AigisCode because it supports local embedding, Cypher-style querying, and projected-graph algorithms.
- But the analyzer's source of truth should remain the Rust semantic graph plus typed artifacts, not the Kuzu schema.

This supports the current AigisCode direction:

- Rust semantic graph as canonical truth
- Kuzu as optional materialized query/read model
- normalized projections for parity, reporting, and agent exploration

### 5. Projected graphs are now a normal requirement, not a luxury

Kuzu's algorithm extension docs describe algorithms operating on projected graphs rather than directly on all tables. This is consistent with how real code analysis systems behave: they run different algorithms over different graph slices because different tasks want different notions of adjacency and relevance.

Implication:

- AigisCode should keep expanding graph projections:
  - `dependency_view`
  - `evidence_view`
  - likely future `navigation_view`
  - eventually detector- or task-specific projected views
- The same canonical graph should feed SCCs, hotspots, impact analysis, AI retrieval, and security traversal through different projections rather than one monolithic export.

### 6. Determinism is a first-class graph quality requirement

Sourcegraph's SCIP indexer docs explicitly recommend deterministic output and snapshot testing. This is an important design signal: for graphing systems, correctness is not only semantic. It is also operational:

- identical inputs should produce identical indexes
- graph generation should be testable with golden snapshots
- source ranges, symbol identities, and bindings should be auditable

Implication:

- AigisCode should continue moving toward deterministic graph artifacts with testable schemas.
- `dependency_view` and future `evidence_view` should have stable ordering and stable identifiers.
- Snapshot tests over representative repositories or fixtures should be part of the graphing quality bar.

### 7. The major AI trend is repository-level graph retrieval, not plain vector search

Recent public papers point in the same direction:

- RepoGraph argues repository-level graphs improve AI software engineering tasks by providing repository-wide navigation and code context.
- CGM argues repository code graph structures can improve repository-level software engineering performance when graph structure is integrated into model processing.
- RANGER argues for graph-enhanced retrieval, using fast graph queries for entity-centric questions and guided graph exploration for natural-language repository questions.

The exact methods differ, but the product lesson is consistent:

- function/file retrieval is not enough for repository-scale tasks
- graphs help agents localize code, reason across files, and retrieve relevant context
- different query types want different graph access patterns

Implication:

- AigisCode should treat graph artifacts as machine interfaces for agents, not only human reports
- MCP resources and tool outputs should continue evolving toward graph-guided code understanding
- the product should support both:
  - fast deterministic entity/path queries
  - broader AI-guided repository exploration using graph context

### 8. History and diff-awareness are still underused but strategically important

CodeQL's older Team Insight publication remains relevant because it focuses on tracking violations over time rather than only reporting one snapshot. The core point still holds:

- a single snapshot is not enough to understand code quality evolution
- you need reliable matching of issues across revisions
- dynamic, change-aware quality information is more useful than static counts alone

Implication:

- AigisCode's Zeus Shield direction is correct
- diff-aware graph judgment, convergence history, and historical artifact comparison are not "nice to have"
- they are part of what makes the system a guardian instead of a one-shot analyzer

## Comparative Model Map

### Code Property Graph / overlay model

Best at:

- deep static analysis
- multi-view reasoning
- vulnerability and data-flow expansion
- layered semantics

Public example:

- Joern

What AigisCode should borrow:

- overlays/layers
- typed multigraph semantics
- one canonical graph with multiple higher-level projections

### Relational analysis database

Best at:

- precise language extraction
- large query libraries
- build-aware semantic capture
- path queries over flow graphs

Public example:

- CodeQL

What AigisCode should borrow:

- build-aware extraction discipline
- language-specific schemas
- path/result interpretation discipline

### Navigation protocol / index

Best at:

- definitions, references, implementations
- IDE and agent navigation
- cross-repo symbol navigation
- deterministic index testing

Public example:

- SCIP / Sourcegraph

What AigisCode should borrow:

- deterministic index generation
- snapshot testing
- possible future export/interoperability layer

### Embedded graph read model

Best at:

- interactive local graph queries
- read-model materialization
- algorithm execution over projected graphs
- agent-friendly graph access

Public example:

- Kuzu

What AigisCode should borrow:

- local embedded querying
- projected graph workflows
- optional graph algorithms over selected views

### Repository graph for AI retrieval/planning

Best at:

- repository-scale agent localization
- graph-guided retrieval
- long-horizon software engineering tasks
- bridging structured entities with natural-language queries

Public examples:

- RepoGraph
- CGM
- RANGER

What AigisCode should borrow:

- graph-guided retrieval surfaces
- explicit machine-readable graph context for agents
- separation of entity queries from broader exploratory queries

## Recommendations for AigisCode

### 1. Keep the Rust semantic graph as canonical truth

Do not move the analyzer's source of truth into Kuzu or another DB. The public systems above reinforce that extraction and semantic truth should remain storage-independent.

### 2. Treat graph projections as product APIs

The product should explicitly maintain at least these projections:

- `semantic_graph`
  - canonical internal truth
- `dependency_view`
  - low-noise structural/runtime dependency projection
- `evidence_view`
  - call-site multiplicity, lines, reasons, confidence, plugin/runtime evidence
- `navigation_view`
  - future definition/reference index, potentially SCIP-compatible

### 3. Keep runtime/framework logic in overlays/plugins

The research strongly supports overlays and projected graphs. For AigisCode this means:

- core owns language truth
- plugins own framework/runtime expansion
- policy/rules own repo-specific accepted behavior

Do not flatten plugin/runtime edges into indistinguishable structural truth.

### 4. Add deterministic snapshot tests for graph projections

This is now a direct quality requirement, not an optional nicety.

Recommended:

- stable ordering for nodes and edges
- golden snapshot tests for `dependency_view`
- golden snapshot tests for future `evidence_view`
- fixture-based checks for source ranges, relation kinds, and identity stability

### 5. Add build-aware/indexer-aware extraction over time

The strongest public systems reuse real semantic context. AigisCode should gradually add:

- build-aware Rust/TS/Python/PHP integrations where practical
- language-specific project-model awareness
- framework-aware plugin expansion after language truth is established

### 6. Keep Kuzu as a projected read model

Kuzu fits well for:

- MCP graph exploration
- architecture queries
- impact analysis
- low-noise parity checks
- graph algorithms over selected views

But it should remain a materialized read model derived from canonical Rust artifacts.

### 7. Prioritize history and guard mode after graph contract stabilization

The research supports the Zeus Shield direction:

- compare graph states over time
- track issue lifecycles
- classify regressions versus accepted behavior
- feed results into guard decisions for AI and human workflows

## Specific Next Steps

Based on this research, the highest-value next steps are:

1. Add a first-class `evidence_view` artifact and MCP resource.
2. Define stable schemas and snapshot tests for all graph projections.
3. Consider a future `navigation_view` or SCIP export for definition/reference interoperability.
4. Extend WordPress/Django/Laravel plugins as graph overlays instead of detector-side heuristics.
5. Add diff-aware graph comparison and violation-history tracking once projection contracts stabilize.

## Bottom Line

The public state of the art does not point toward one giant graph dumped into one database.

It points toward:

- language-accurate extraction
- a rich canonical graph
- multiple projected views
- deterministic indexing
- storage/query separation
- graph-guided agent workflows at repository scale

That is the direction AigisCode should continue to follow.

## Sources

Official docs and protocols:

- Joern overview: https://docs.joern.io/
- Joern Code Property Graph: https://docs.joern.io/code-property-graph/
- Joern glossary and overlays: https://docs.joern.io/glossary/
- CodeQL overview: https://codeql.github.com/docs/codeql-overview/about-codeql/
- CodeQL docs index: https://codeql.github.com/docs/
- SCIP protocol repo: https://github.com/sourcegraph/scip
- Sourcegraph precise code navigation: https://sourcegraph.com/docs/code-navigation/precise-code-navigation
- Sourcegraph writing an indexer: https://sourcegraph.com/docs/code-navigation/writing-an-indexer
- Kuzu getting started: https://docs.kuzudb.com/get-started/
- Kuzu client APIs: https://docs.kuzudb.com/client-apis/
- Kuzu Rust API: https://docs.kuzudb.com/client-apis/rust/
- Kuzu installation/system requirements: https://docs.kuzudb.com/installation
- Kuzu algorithms/projected graphs: https://docs.kuzudb.com/extensions/algo/

Research papers:

- RepoGraph (ICLR 2025): https://arxiv.org/abs/2410.14684
- Code Graph Model (CGM), 2025: https://arxiv.org/abs/2505.16901
- RANGER, 2025: https://arxiv.org/abs/2509.25257
- Team Insight / tracking static analysis violations over time: https://codeql.github.com/publications/tracking-analysis-violations.pdf
