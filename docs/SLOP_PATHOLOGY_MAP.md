# AI Code Slop — Pathology → AigisCode Capability Map

> Ground-truth mapping between the AI-generated "code slop" pathology catalog and
> what the AigisCode engine can actually detect **today**, with file:line
> evidence. Companion documents:
> `SLOP_ELIMINATION_ROADMAP.md` (what to build) and
> `SLOP_GUARDIAN_LOOP.md` (how agents use it operationally).
>
> Audited: full `rust/crates/aigiscore/src` tree (~56k LOC) as of 2026-07-02.
> Grades are honest. "Absent" means no machinery exists, not "bad product".

## The Core Pathology, Restated as an Engineering Problem

AI coding agents optimize locally because they cannot hold the global picture:
partial file reads, no persistent symbol table, reward for visible action over
restraint, context truncation, pattern-matching to training distribution instead
of the repo's conventions. The damage is emergent: every commit passes review,
the quarter loses the architecture.

This is exactly the problem `GOAL.md` and `ZEUS_SHIELD.md` define. AigisCode's
answer has three legs:

1. **Canonical understanding** — a whole-repo semantic graph an agent never has
   to reconstruct from grep fragments (`semantic-graph.json`,
   `dependency-graph.json`, `evidence-graph.json`, `contract-inventory.json`).
2. **Deterministic entropy detectors** — dead code, hardwiring, duplicate
   mechanisms, sanctioned-path bypass, abstraction sprawl, hand-rolled
   reinvention, complexity hotspots (`detectors/`, `assessment/`, `graph/`).
3. **Change governance + convergence** — run-over-run regression detection,
   allow/warn/block guard verdicts, policy/rules/doctrine memory
   (`artifacts.rs` convergence + guard, `policy/`, `doctrine/`).

The counter-tooling the pathology calls for — "tools a smarter model uses to
hold the global picture the coding agent can't" — is this product's literal
thesis. The map below shows how much of it is already real.

## Grading Legend

| Grade | Meaning |
|---|---|
| **STRONG** | Real algorithm, graph- or AST-backed, tested, low false-positive design (proof tiers / provenance / policy overlay) |
| **PARTIAL** | Real machinery exists but narrow (few families/frameworks), lexical/heuristic, or detects only a sub-shape of the symptom |
| **SUBSTRATE** | The data needed is already produced, but no detector consumes it for this symptom yet |
| **ABSENT** | Neither detector nor data exists; parser/pipeline extension needed |

## Symptom-by-Symptom Map

### 1. Duplication / divergent implementations (semantic clones)

**Grade: ABSENT (detector) / SUBSTRATE (identifiers only)** — the single biggest gap.

- What exists: `RepeatedLiteral` catches *identical string literals* appearing
  ≥2 times (`detectors/hardwiring.rs:122-138`), with contract-aware noise
  suppression (`hardwiring.rs:177-272`). The contract inventory counts duplicate
  route/hook/env/config values with locations (`contracts/mod.rs:79-90`).
- What is missing: no function-body clone detection of any kind — textual,
  token-shape, or behavioral. Root cause is structural: language adapters emit
  symbols + references but **discard statement bodies**, so there is nothing to
  fingerprint. Two divergent `validateEmail` implementations are invisible.
- Fix path: roadmap item **S8** (function shape fingerprints + near-duplicate
  clustering), **S14** (AI behavioral adjudication of clusters).

### 2. Overengineering / speculative generality

**Grade: PARTIAL**

- `AbstractionSprawl` (`assessment/mod.rs:1764-1936`): flags concepts
  surrounded by many role-named files (manager/handler/service/factory/…),
  i.e. abstraction layers accreting around one concern. Doctrine refs:
  `guardian.overengineering`, `guardian.minimal-mechanism`.
- `HandRolledParsing` + specialized stack detectors (scheduler DSLs
  `assessment/mod.rs:2123`, filesystem page resolution `:2248`, manifest-backed
  policy engines `:2392`): catch the "built a framework nobody asked for"
  flavor, mapped to `guardian.avoid-homegrown-*` doctrine clauses
  (`doctrine/mod.rs:95-160`).
- Missing sub-shapes: interface/trait with exactly one implementer (the
  `Extends`/`Implements` edges to compute this already exist in
  `resolve/mod.rs`), factory for a single product, config machinery for values
  that never vary. Roadmap **S6**.

### 3. Overfactoring / premature decomposition

**Grade: SUBSTRATE**

- No detector, but the call graph already carries everything needed: resolved
  `Call` edges with source/target symbols (`graph/mod.rs`), symbol line spans,
  parameter counts. "Six 8-line functions each called exactly once, scattered
  across five files" is a pure graph query: single-caller chains + cross-file
  scatter + small spans. Roadmap **S7**.

### 4. Not using existing libraries / hand-rolling stdlib

**Grade: PARTIAL**

- Real today: `HandRolledParsing` (parsing-primitive density + parsing role
  names, `assessment/mod.rs:1938-2093`) — "you wrote a parser instead of using
  one". Doctrine names the principle (`guardian.native-vs-library`,
  `doctrine/mod.rs:252-262`) with `preferred_mechanism =
  battle_tested_parser_or_native_contract`.
- Honest limitation: judgment is inferred from *code shape* ("this file owns a
  custom parsing stack"), not from a knowledge base of "the framework/stdlib
  already provides Y". The inverse failure (heavy dependency pulled in for
  something trivial) is not detected at all — see symptom 13.
- Fix path: **S9** (dependency/capability catalog), **S5** (repo-owned
  sanctioned mechanisms naming concrete APIs).

### 5. Dead code accumulation

**Grade: STRONG** — best-covered symptom.

- File level: orphan detection with entry-point patterns and — critically —
  `boundary_truncated` honesty on scoped scans so cropped slices don't fake
  orphan debt (`graph/analysis.rs:649-691`).
- Symbol level: unused private functions/methods via absent incoming `Call`
  edges, unused imports cross-checked against receiver bindings
  (`detectors/dead_code.rs:39-216`), with `Certain/Strong/Heuristic` proof
  tiers and language-aware suppressions (Python dunders, `__init__` re-exports,
  decorator-bound methods).
- Gap: only *private* symbols are flagged; public-API dead code needs
  entry-point reachability closure — the BFS machinery to do this already
  exists in the security module (`security/mod.rs:1003-1245`). Roadmap **S10**.

### 6. Wrapper / indirection sprawl

**Grade: SUBSTRATE**

- No pass-through/delegation-chain metric. Betweenness centrality
  (`graph/analysis.rs:733-790`, real Brandes implementation) measures
  connectivity, not thinness. But pass-through candidates are computable from
  existing data: symbol span ≤ ~3 lines + exactly one outgoing `Call` edge +
  matching arity. Chains of such candidates = ceremonial indirection depth.
  Roadmap **S7** (same detector family as overfactoring).

### 7. Inconsistent conventions / convention drift

**Grade: PARTIAL**

- `SplitIdentityModel` (`assessment/mod.rs:1210-1317`) catches one drift
  flavor: the same domain concept living under diverging identifier families.
  `CompatibilityScar` (`:1319-1415`) catches accumulated back-compat cruft in
  one file. Doctrine: `guardian.single-canonical-representation`.
- Missing: naming-style drift (`getUserById` vs `fetch_user` vs
  `retrieveUserRecord`), error-handling/async/logging pattern drift. The symbol
  table has every name + kind + language — distribution analysis is pure
  post-processing. Roadmap **S11**.

### 8. Copy-paste-mutate

**Grade: ABSENT** — same root cause as symptom 1 (no body-level data). Solved by
the same fingerprinting: near-identical shape hashes with small deltas are
precisely the copy-paste-mutate signature, *more* detectable than exact clones.
Roadmap **S8**.

### 9. Defensive-code bloat

**Grade: ABSENT**

- Requires statement-level extraction (guard/null-check/try-catch density,
  repeated validation along a call chain) that parsers do not emit. Cheapest
  real version rides on **S8** body extraction + call-graph chain analysis;
  full version is dataflow-shaped. Roadmap **S15** (late tier).

### 10. Comment noise

**Grade: ABSENT** — parsers discard comments entirely (verified across all five
adapters). Needs comment capture at parse time, then code-comment similarity
adjudication (cheap LLM pass over deterministic candidates). Roadmap **S15**.

### 11. Configuration / constant / state duplication

**Grade: STRONG-PARTIAL**

- `RepeatedLiteral`, `MagicString`, `HardcodedNetwork`, `EnvOutsideConfig`
  (`detectors/hardwiring.rs:9-15`), policy-tunable
  (`repeated_literal_min_occurrences`, `allowed_literals` in
  `policy/mod.rs`). Contract inventory tracks env keys, config keys, routes,
  hooks with occurrence counts + locations across PHP/Python/Ruby/JS/TS/Vue/Rust
  (`contracts/mod.rs:155-197,607-648`).
- Gap: inventory counts duplicates but nothing *flags* "same env key defaulted
  differently in three places" or "same route declared twice". Comparator is
  missing, substrate complete. Roadmap **S4** — one of the cheapest wins.

### 12. Test slop

**Grade: ABSENT (by design, mostly out of scope)**

- Test-like paths are deliberately excluded from contract inventory
  (`contracts/mod.rs:525-541`). Duplicate test-setup detection falls out of
  **S8** clone clustering (test files included); assertion-quality judgment is
  AI-adjudication territory (**S14** pattern), not deterministic core.

### 13. Dependency cancer / redundant new dependency

**Grade: ABSENT** — second biggest gap, and doctrinally central.

- `ZEUS_SHIELD.md`'s canonical example ("adding a large library to WordPress
  for sending email when native functions already exist") has **no detector**.
  Manifests (`Cargo.toml`/`package.json`/`composer.json`) are only checked for
  *existence* to gate external tools (`external/mod.rs:564,632,705,763`) —
  never parsed, never diffed, never capability-tagged.
- Closest existing signals, all reactive: composer-audit `abandoned_dependency`
  (`external/mod.rs:1734-1755`), SCA vulnerability findings (trivy/grype/
  osv-scanner/pip-audit/npm-audit/cargo-deny), cargo-deny `bans` (config-driven).
- Fix path: **S9** — manifest ingestion into the contract inventory +
  capability catalog ("this dep provides: http-client") + convergence delta on
  the dependency set. Turns `guardian.minimal-mechanism` and
  `guardian.native-vs-library` from inert clauses into enforced ones.

### 14. Parallel mechanisms for one concern (code-level path multiplicity)

**Grade: PARTIAL — and the most differentiated capability in the repo.**

- `DuplicateMechanism` (`assessment/mod.rs:1435-1609`): classifies files into
  mechanism families via `detect_mechanism_families` (`:2593-2640`), extracts
  concept tokens from paths + identifiers, and flags a concept implemented by
  ≥2 files through ≥2 *different* families. This is semantic
  path-multiplicity detection, not textual duplication — exactly the "two ways
  to send a notification" pathology. Benign pairs are excluded
  (notifications+queue, `:1485-1490`). Doctrine: `guardian.single-solution-path`,
  `guardian.mechanism-coherence`.
- Honest limitation: only **4 mechanism families** exist today
  (`lifecycle_hooks`, `event_bus`, `queue_jobs`, `direct_notifications`),
  recognized by substring/regex, Laravel/WordPress-flavored. The engine
  design is right; the catalog is thin. Roadmap **S3** (family expansion is
  data work, not architecture work).

### 15. UX-level dual-path (product-facing route multiplicity with divergent behavior)

**Grade: SUBSTRATE — all three ingredients exist, the composition doesn't.**

The "swipe soft-deletes, menu hard-deletes" problem decomposes into exactly
what the engine already produces:

1. **Entry surface enumeration** — contract inventory routes/hooks/commands
   (`contracts/mod.rs:566-605`), semantic model packs (Django routes/signals,
   WP REST routes, `semantic_models/mod.rs:43-76`), runtime entry candidates
   (`graph/analysis.rs`), route-declared runtime-entry promotion in topology.
2. **Path tracing** — BFS reachability over Call/Dispatch/ContainerResolution/
   EventPublish edges, already built for security sinks
   (`security/mod.rs:1003-1245`), plus typed multi-path `graph_traces` and
   `source_sink_paths` in agentic packets.
3. **Sink/side-effect anchoring** — semantic state-flow evidence for mutable
   carriers, dangerous-API sinks, runtime plugin edges (queue dispatch, hooks).

What's missing is the composition: cluster entry points by the shared
state-mutation they reach, then diff the side-effect sets along each path
("route A passes through audit-log call, route B doesn't"). Roadmap **S13** —
hardest deterministic item, highest product distinctiveness.

### 16. Cross-cutting: architectural entropy (cycles, god modules, hotspots)

**Grade: STRONG** — foundation layer, not in the user-facing symptom list but
load-bearing for everything above.

- Kosaraju SCC cycles with structural/runtime/framework/artifact
  classification (`graph/analysis.rs:457-558`), Martin coupling/instability
  (`:569-624`), hub-like and unstable-dependency smells (`:234-328`), Brandes
  betweenness bottlenecks (`:733-790`), `AlgorithmicComplexityHotspot` with
  caller-pressure paths and ast-grep operation provenance
  (`assessment/mod.rs:685-1208`).

## The Meta-Problem Mapping

The pathology's signature — *"the agent cannot see the whole, so it cannot
maintain coherence of the whole"* — maps to three delivery mechanisms, in
ascending order of leverage:

| Mechanism | Status | Evidence |
|---|---|---|
| **Whole-picture on demand** (agent queries the graph instead of grepping) | **Shipped** | 14 MCP tools incl. `graph_neighbors`, `graph_trace`, `repository_topology`, `cypher_query` (`mcp/mod.rs:280-628`); bounded `graph-packets.json` neighborhoods |
| **Live whole-picture mid-edit** (no architectural amnesia between turns) | **Shipped, Phase 1** | `mcp --watch`: notify watcher → full re-analysis → atomic republish with honest `Freshness` contract (`mcp/watch.rs`, `mcp/live.rs`). Non-incremental by design; see `ONLINE_CODE_GRAPH_ARCHITECTURE.md` Phase 2 |
| **Change governance** (entropy delta gets judged, not just reported) | **Shipped, with two wiring gaps** | Guard verdict from convergence regression flags (`artifacts.rs:3685-4226`): Block on high-sev security / new strong cycle, Warn on duplicate-mechanism/sprawl/bypass/contract deltas. Gaps: verdict is run-over-run repo-global, not diff-scoped (**S1**); `--watch` serves stale guard verdicts (**S2**) |

## Honest Structural Limits (what the current pipeline cannot see)

1. **No statement bodies, no comments** — parsers emit symbols + references
   only. Blocks symptoms 1, 8, 9, 10 until parse-layer extension (**S8**).
2. **Doctrine is inert data** — 32 clauses (`doctrine/mod.rs:71-410`) are
   attached to findings as refs; no code matches `preferred_mechanism` against
   actual repo APIs. Enforcement lives entirely in the heuristic detectors.
3. **Policy is suppression-only** — `.aigiscode/policy.json` can say "stop
   reporting this", never "prefer this instead" (`policy/mod.rs:46-203`).
4. **Guard is not diff-aware** — baseline = previous artifact set in the output
   dir (`artifacts.rs:884-889`); no git integration in the guard path; first
   run classifies everything `New`.
5. **Framework catalog asymmetry** — ast-grep catalogs: Laravel/Django/Rails;
   semantic packs: Django/WordPress/PHP-hooks only; Ruby lacks a complexity
   ruleset (`scanners/ast_grep.rs:1196`), Rust lacks a security ruleset
   (`:545`).

## Coverage Scoreboard

| # | Symptom | Grade | Primary machinery | Roadmap |
|---|---|---|---|---|
| 1 | Semantic duplication / divergent impls | ABSENT | — | S8, S14 |
| 2 | Overengineering / speculative generality | PARTIAL | AbstractionSprawl, HandRolled* | S6 |
| 3 | Overfactoring / premature decomposition | SUBSTRATE | call graph | S7 |
| 4 | Not using existing libraries | PARTIAL | HandRolledParsing, doctrine | S5, S9 |
| 5 | Dead code accumulation | STRONG | dead_code.rs, orphans | S10 |
| 6 | Wrapper / indirection sprawl | SUBSTRATE | call graph + spans | S7 |
| 7 | Inconsistent conventions | PARTIAL | SplitIdentityModel | S11 |
| 8 | Copy-paste-mutate | ABSENT | — | S8 |
| 9 | Defensive-code bloat | ABSENT | — | S15 |
| 10 | Comment noise | ABSENT | — | S15 |
| 11 | Config/constant duplication | STRONG-PARTIAL | hardwiring + contracts | S4 |
| 12 | Test slop | ABSENT | — | S8, S14 |
| 13 | Redundant dependencies | ABSENT | — | S9 |
| 14 | Parallel mechanisms (code) | PARTIAL | DuplicateMechanism | S3 |
| 15 | UX dual-path divergence | SUBSTRATE | contracts + reachability + state flows | S13 |
| 16 | Cycles / god modules / hotspots | STRONG | graph/analysis.rs | — |
| — | Guard on change (governance) | PARTIAL | convergence + guard | S1, S2 |
| — | Advise before writing | PARTIAL | MCP + packets + doctrine | S5, S12 |

Score today: 3 STRONG, 6 PARTIAL, 4 SUBSTRATE, 5 ABSENT. Every ABSENT and
SUBSTRATE item has a concrete implementation path inside the existing
architecture — none requires abandoning the typed-artifact / Rust-core /
plugin-layer contract. See `SLOP_ELIMINATION_ROADMAP.md`.
