# MCP for Agents — Design Contract

> Written from the consumer's seat: an AI coding agent (Claude) audited AigisCode
> and the draivix validation repo *through* `aigiscode mcp`, and recorded every
> point of friction. This document is the design contract for making the MCP
> surface serve the product's actual assignment.

## The Assignment

AigisCode exists so that AIs produce code the way senior human programmers do.

A senior programmer working in a codebase holds a mental model that no file
shows directly:

1. **A map** — what the system is, where the entries are, which parts are load-bearing.
2. **Conventions** — how things are done *here*: the sanctioned mechanism for
   config, dispatch, persistence; what the doctrine forbids.
3. **Blast-radius intuition** — before touching a function, they feel who
   depends on it and what will break.
4. **Drift sense** — they notice when a change makes the codebase *worse*
   (a second mechanism for an existing concept, a new bypass, dead weight).
5. **History** — "we removed that", "this is legacy kept for X".

An AI agent has none of this between tasks. Its context window is its entire
working memory, and every token spent on unfocused JSON is a token not spent
on the user's problem. The MCP server's job is to be that mental model,
**queryable, honest, and cheap in tokens**.

Everything below follows from one sentence: *answer the question the agent
actually asked, at the size a colleague would answer it.*

## Hard Rules

1. **Token budget per tool result.** Orientation tools ≤ ~3 KB. Query tools
   ≤ ~8 KB with explicit `limit`/`cursor` params. Nothing returns unbounded
   inline dumps — measured today, `repo_overview` returned **89.7 KB**, which
   is not an overview, it is context poison. A human colleague answers "what
   is this repo?" in a paragraph, not by reciting the filing cabinet.
2. **Answer-first, evidence-on-demand.** Every response leads with the
   verdict/summary a human would say out loud; file paths, line anchors, and
   raw evidence hang off IDs the agent can expand via `explain_finding` /
   dedicated detail calls. Never inline what an ID can defer.
3. **Honesty markers are load-bearing.** `is_stale`, `boundary_truncated`,
   proof tiers (`certain`/`strong`/`heuristic`), and unresolved-reference
   pressure must ride along with every answer they qualify. An agent cannot
   eyeball skepticism; the payload must carry it.
4. **Deterministic.** Same repo state → byte-identical answers. (Resolver
   nondeterminism was found and fixed during this audit; treat any HashSet
   iteration reaching an output as a bug.)
5. **No aggregate without discrimination.** "675 hardwiring findings,
   severity: high" is useless when 573 are repeated string keys in one
   artifact writer. Aggregates must always break down by severity band and by
   proof tier, or they mislead more than they inform.

## The Agent's Edit Loop (target workflow)

The current tool set is *audit-shaped* (findings-first). The assignment is
*edit-loop-shaped*. A coding agent's session looks like:

```
orient → locate → plan the change → make the change → verify the change
```

| Loop stage | Question the agent asks | Today | Needed |
|---|---|---|---|
| Orient | "What is this repo? Where do I start?" | `repo_overview` (89 KB dump) | `repo_brief` ≤ 3 KB: purpose-guess, zones, entries, top-3 pressures, doctrine headline |
| Locate | "Where is symbol/concept X?" | ❌ nothing (grep fallback) | `find_symbol` — name → definitions + kind + owner + file:line, fuzzy-tail match |
| Locate | "Who calls / uses X?" | ❌ file-level `graph_neighbors` only | `symbol_usages` — inbound edges *at symbol granularity*, grouped by caller file |
| Plan | "What breaks if I change X?" | ❌ | `blast_radius` — transitive inbound closure with depth cap + contract hits (routes/config keys that reach X) |
| Plan | "What is the sanctioned way to do Y here?" | doctrine registry is a raw artifact | `convention_for` — given a file path + concern (config/dispatch/persistence/http), return the doctrine clause + one in-repo exemplar |
| Change | — (agent edits via its own tools) | — | — |
| Verify | "Did I make it worse?" | `guard_decision` + `convergence_report` ✅ (good shape) | keep; add diff-scoped `verify_paths([files])` so the agent asks about *its* edit, not the whole repo |
| Verify | "Did my delete leave second-order dead code?" | re-run analyze + orphans ✅ (proven today: deleting 16 orphans exposed 3 more) | keep; document the loop |

The Locate/Plan rows are the gap that matters most. Symbol-level graph queries
are what separates "static-analysis report reader" from "colleague who knows
the codebase".

## Findings from Dogfooding (2026-07-03)

Each observed through the MCP server itself, against this repository and draivix.

* **`repo_overview` returns 89.7 KB** — inlines the full contract inventory
  and every artifact path. → Split: `repo_brief` (budgeted prose+numbers) and
  keep the artifact map behind a dedicated `artifact_paths` call.
* **`show_hotspots` is good** (7.7 KB, ranked, honest flags) but
  `inbound_edges: 1527` counts edge *instances*, not distinct dependents —
  an agent reads that as "1527 files depend on this". Report both:
  `inbound_edges` and `inbound_files`.
* **`show_cycles` reported one 34-file, 6 225-edge "cycle"** — the whole crate
  as one SCC. For an intra-crate Rust module graph this is the *normal
  condition*, not a finding. Whole-corpus SCCs (≥ N% of analyzed files) should
  be reported as topology ("this crate is one mutually-referencing unit"),
  not as a cycle finding with a dominant-relation list.
* **`coverage_report`: "17 818 unresolved reference sites"** —
  undifferentiated. Split into: external/stdlib targets (expected, fine),
  same-repo candidates that failed to resolve (parser/resolver gaps — actionable),
  and dynamic/opaque (string dispatch). One number that mixes all three
  teaches the agent to ignore it.
* **`quality_evaluation` is the right *shape*** (4 KB, dimensions + suspects +
  recommendations) but: suspects list contained a duplicate row
  (same file, same reason twice) and severity bands ignore proof tiers
  (573 repeated literals ⇒ "hardwiring: high").
* **`guard_decision` is the best tool on the surface** — small, verdict-first,
  doctrine-ref'd, with explicit obligations. It should be the template the
  other tools converge on.
* **Orphan cascade works end-to-end** and matches the human workflow: the
  first sweep found 16 dead frontend modules; deleting them exposed 3
  second-order orphans on the next run. This delete→re-analyze→next-layer
  loop should be first-class in the triage prompt.

## Tool Surface (target)

Orientation (small, prose-leading):
- `repo_brief` — ≤ 3 KB. What it is, zones, entries, language mix, top-3
  pressures, doctrine headline, freshness.
- `quality_evaluation` — keep; add proof-tier breakdown, dedupe suspects.
- `guard_decision` — keep as-is (the template).

Locate (symbol-granular, the new core):
- `find_symbol(name)` — definitions with kind/owner/file:line.
- `symbol_usages(symbol)` — inbound references grouped by file, with lines.
- `blast_radius(symbol|file, depth)` — transitive dependents + reached
  contracts (routes/env/config), capped and explicit about truncation.

Convention:
- `convention_for(path, concern)` — doctrine clause + sanctioned mechanism +
  one exemplar file in this repo that does it right.

Verify:
- `verify_paths(files)` — diff-scoped guard: findings delta, new bypasses,
  orphan-cascade check restricted to the touched neighborhood.

Existing query tools (`list_findings`, `explain_finding`, `graph_neighbors`,
`graph_trace`, `list_graph_packets`, `repository_topology`, `cypher_query`)
stay; they get budgets and cursors.

## Non-Goals

- The MCP does not edit code. It informs the editor.
- No LLM calls inside the server; every answer is derived from the graph and
  artifacts deterministically.
- No per-repo special cases: every improvement above is generic engine work.

## Sequencing

1. ✅ `repo_brief` + budgets on existing tools (shipped 2026-07-03; runtime
   entries are directory-diversified so one migrations folder cannot flood
   the brief).
2. ✅ `find_symbol` / `symbol_usages` (shipped 2026-07-03; ambiguous bare
   names return capped candidates instead of guessing).
3. ✅ Unresolved-reference discrimination in `coverage_report` (shipped
   2026-07-03; primitives classed external even when colliding with repo
   file stems).
4. ✅ Whole-corpus SCC reclassification in `show_cycles` (shipped 2026-07-03;
   `corpus_scale_units`, cycle file lists capped at 20). Also shipped from the
   dogfooding list: hotspot `inbound_files`/`outbound_files`, quality-suspect
   dedupe, complexity findings routed to the matching obligation template.
5. `blast_radius`, `convention_for`, `verify_paths`.
