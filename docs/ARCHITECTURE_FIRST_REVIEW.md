# Architecture-First Review — Evaluator Gap Analysis & Roadmap

> Written 2026-07-04, after two days of dogfooding AigisCode against draivix and
> against itself (MCP tool build-out, the 151-file-knot forensics). This document
> evaluates the evaluator against the product's intended review model and sets the
> sequence for closing the gap.

## The review model this tool must serve

A senior architect reviews a codebase in two passes:

1. **Architecture pass — signatures and edges, no bodies.** What exists (classes,
   methods, their shapes), where it lives (layers, modules), and who talks to whom
   (direction, fan-in/out, cycles, ownership). Design flaws are visible here:
   wrong-direction dependencies, god classes, dual mechanisms, leaky visibility,
   layer skipping, cyclic knowledge. Function bodies are irrelevant to this pass —
   they are brick and mortar.
2. **Implementation pass — bodies, prioritized by pass 1.** Complexity, security
   sinks, hardwiring, hygiene — evaluated *in the order the architecture pass says
   matters* (load-bearing symbols first), not alphabetically.

The tool's job: make pass 1 first-class, machine-checkable, and cheap for an AI agent
to consume; then feed pass 2 its priority order.

## Honest inventory: where the evaluator stands today

**Already strong (pass-1 substrate):**
- Symbol graph carries the skeleton already: kind, qualified name, owner type,
  visibility, parameter counts, return type (71% of draivix symbols have
  `return_type_name`), line spans — plus symbol-granular resolved edges with
  receiver-proof discipline (post `d140a7f`/`9899b98`, edges are trustworthy; the
  fake-SCC era is over).
- File-level architecture views: cycles (now honest), bottlenecks, zones/topology,
  orphans, boundary truth, corpus-scale reclassification.
- Doctrine: machine-readable intent (sanctioned mechanisms, blocking clauses) +
  guard verdicts + convergence deltas.
- MCP: budgeted orientation (`repo_brief`), symbol lookup (`find_symbol`,
  `symbol_usages`), discriminated coverage.

**Misaligned (exists, but wrong altitude):**
- The finding mix is dominated by pass-2 material: on draivix, complexity (568) +
  hardwiring (131) + security (25) ≈ 76% of actionable findings are body-level, and
  the report interleaves them with the ~20 genuinely architectural ones. An
  architect reading `aigiscode-report.md` gets mortar before walls.
- The architectural detectors that do exist (duplicate mechanism, abstraction
  sprawl, split identity, sanctioned-path bypass) judge **file contents by
  heuristic**, not the **signature graph by structure**. They are good smell
  detectors, but they are not "the design is wrong here" proofs.
- Cycles/SCC analysis is **file-granular**. Two unrelated methods in one file merge
  cycles; symbol-level layering violations hide inside file-level edges.

**Missing (the actual gap):**

| # | Missing capability | Why it blocks pass 1 |
|---|---|---|
| G1 | **Design skeleton view** — "show me module X as signatures + cross-module edges, no bodies" | The architect's primary read. All data exists in the graph; no view/tool renders it. An agent today must read raw source (bodies included) to see a module's shape — the exact token waste the MCP exists to prevent. |
| G2 | **Layer contract + direction enforcement** — user-declared layers (paths → rank + allowed dependencies) with deterministic violations | Zones are derived, not declared; nothing says "definitions must not depend on runtime." The draivix `_Core` redesign (L0–L4) has no enforcement mechanism except re-reading reports. This is THE architecture detector and it is absent. |
| G3 | **Signature-level design findings** — fan-in/fan-out per symbol, god-class evidence (public-method count × distinct dependent modules × disjoint internal clusters), visibility leaks (public used only locally), layer-skipping calls, sibling-contract asymmetry | We compute none of these despite having every input. These are the findings an architect states from the skeleton alone. |
| G4 | **Public API surface as a contract** — per module: exported/public symbols consumed from outside; churn tracked by convergence | Contract inventory tracks routes/env/hooks but not code-level API. Breaking-change detection at the design level is impossible today. |
| G5 | **Symbol-granular SCC/cycle + cut analysis** — cycles between methods/classes, with weighted feedback-edge cut lists (the knot forensics I ran by hand in python) | File granularity both hides and exaggerates. The cut-list analysis proved decisive twice this week and lives outside the product. |
| G6 | **Two-phase review surface** — explicit `architecture` vs `implementation` finding phases; report and guard lead with phase 1; phase 2 ordered by phase-1 centrality | The user's review model, encoded. Today's report shape actively fights it. |
| G7 | **Signature completeness in parsers** — parameter *types* (PHP type hints, TS annotations), `implements`/trait edges, `instanceof`/`::class` refs | Pass 1 quality is bounded by signature truth. We carry parameter counts but not types; coupling-via-signature is invisible. Long-standing backlog item; it now has a concrete consumer. |

## Sequencing (next steps, in order)

Each step is generic engine work, validated on draivix + self, gated by tests.

1. ✅ **`module_design` MCP tool (G1)** — shipped 2026-07-04 (`bf27030`). Also fixed
   en route: the PHP parser hardcoded every method Public; visibility is now real,
   which G3's VisibilityLeak will need. First live read of draivix `_Core` surfaced
   EntityRegistry=57 public methods, EntityManager=46 in one 9 KB call.
2. ✅ **Layer contract + violation detector (G2)** — shipped 2026-07-04 (`a695b8b`).
   Doctrine `layers` section + deterministic `LayerContractViolation` findings with
   guardian packets. First draivix contract yields 263 verified violations (111 =
   module controllers extending the http-layer base Controller). Dogfooding it
   exposed and fixed two further resolver fake-edge classes (bare free calls
   binding methods; unproven `$this->` calls binding globally) — another 2,746
   fabricated edges gone.
3. **Design finding family (G3)** — new detector family `design` computed from the
   symbol graph only. First detector shipped 2026-07-04: ✅ **GodClass** — fires only
   when BOTH signals hold (≥25 public non-magic methods AND ≥10 distinct external
   dependent files; width without consumers is a big helper, consumers without
   width is a healthy hub). Evidence carries per-method consumption
   (`used:find@338files`). Draivix: 15 findings (EntityManager 31/620,
   EntityRegistry 38/290, PermissionChecker 30/163, Email 114 public methods);
   self-repo: zero. Remaining in family: `VisibilityLeak` (FP-prone — must exempt
   magic methods, runtime entries, route-declared controllers, overrides),
   `SiblingContractDrift`, `HighFanInWithoutInterface`. `LayerSkip` dropped —
   the explicit layer contract from #2 already encodes direction. These are
   pass-1 findings by construction — no body reads anywhere in the family.
   *Effort: medium, incremental per detector.*
4. **Two-phase surface (G6)** — every finding gets `phase: architecture |
   implementation`. `aigiscode-report.md` restructured: Architecture first,
   Implementation second with priority = centrality of the enclosing symbol's
   file. MCP `list_findings` gains a `phase` filter; guard weighs architecture
   regressions above implementation ones. *Effort: small-medium, mostly plumbing.*
5. **Symbol-level SCC + cut analysis as `scc_breakdown` MCP tool (G5)** — port the
   knot-forensics notebook into the graph crate: Tarjan at symbol granularity,
   weighted ELS feedback-edge cut list, edge-kind breakdown, resulting layer order.
   Already proven decisive in the field. *Effort: medium.*
6. **API-surface contract (G4)** — extend contract inventory with per-zone public
   symbol surface + convergence tracking (added/removed/signature-changed).
   *Effort: medium.*
7. **Parser signature completeness (G7)** — parameter types, `implements`/trait
   edges, type-position references (PHP first, TS second). Feeds every item above;
   scheduled last only because items 1–5 already work on today's 71%-complete
   signatures. *Effort: large, incremental.*

## What pass 2 (function bodies) looks like after this

Nothing about body-level detection needs to be built — it exists (complexity,
security, hardwiring, dead code). What changes is **ordering and framing**: the
implementation report stops being a flat list and becomes "the bodies inside your
load-bearing design elements, worst-first" — complexity in `EntityManager` outranks
identical complexity in a leaf exporter because pass 1 says so. That is the "brick
and mortar last" model, and it falls out of step 4 for free.

## Non-goals

- No LLM-in-the-loop scoring; every pass-1 finding must be a deterministic graph
  fact with quotable evidence (the knot episode is the cautionary tale for trusting
  derived judgments — and the receipt that verification pays).
- No per-repo special cases: layers are user-declared config; detectors stay
  generic.
