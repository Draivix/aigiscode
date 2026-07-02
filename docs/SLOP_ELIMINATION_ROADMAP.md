# Slop Elimination Roadmap

> Implementation plan for closing the gaps identified in
> `SLOP_PATHOLOGY_MAP.md`. Every item is placed inside the existing
> architecture contract (parsing → resolve → graph → detectors/assessment →
> surface → artifacts; framework knowledge in plugins/packs/catalogs;
> repo truth in policy/rules/doctrine). No item requires a new runtime, a
> sidecar language, or a second source of findings truth.
>
> Ordering principle: **wiring gaps first** (existing machinery, missing
> connections), **graph-query detectors second** (data exists, detector
> doesn't), **parse-layer extensions third** (new data), **composition and AI
> adjudication last** (hardest, highest distinctiveness).

## Tier 1 — Governance Wiring (days each; multiplies value of everything that exists)

### S1. Diff-scoped guard: judge the change, not just the snapshot delta

**Eliminates:** the biggest credibility gap in the after-loop. Today the guard
verdict is a repo-global run-over-run regression count
(`artifacts.rs:3685-4226`, baseline read from previous artifacts at
`:884-889`); it cannot say "*your* edit introduced this".

**How:**
- Add a `ChangedFileSet` input: `git diff --name-only <merge-base>` +
  `git status --porcelain` for working tree (git is already shelled for
  topology owner hints, `artifacts.rs:3336`; and for repo-root detection,
  `ingestion/scan.rs:320-345` — reuse, don't duplicate).
- Scope convergence attention items and guard triggers: a regression trigger
  fires at full level when its finding's `file_path` (or one-hop graph
  neighborhood — reuse `build_convergence_required_radius`,
  `artifacts.rs:3648-3683`) intersects the changed set; otherwise downgrade to
  an `inherited_debt` observation. New findings on changed files = the agent's
  responsibility; pre-existing findings elsewhere = backlog, not a gate.
- First-run behavior (**baseline bootstrap**): when
  `topology_baseline_observation = baseline_empty` (`artifacts.rs:3314`), emit
  `Allow` with verdict reason `baseline_captured` instead of the current
  spurious warn/block where every finding classifies as `New`
  (`artifacts.rs:4233`).

**Where:** `artifacts.rs` guard/convergence builders + a small
`ingestion/vcs.rs` helper. **Effort:** 2–4 days. **Difficulty:** low —
fingerprints, radius machinery, and per-smell regression flags already exist.

### S2. Live guard under `mcp --watch`

**Eliminates:** the stale-verdict trap: watch rebuilds pass
`write_artifacts=false` (`mcp/watch.rs:129-134`), and `McpState::new` reads
guard/convergence from *persisted* artifacts (`mcp/mod.rs:1013-1035`) — so the
daemon serves a live graph but a frozen allow/warn/block.

**How:**
- On each watch rebuild, recompute convergence in-memory against the last
  *persisted* baseline (keep the disk baseline stable so the session-long
  delta stays "since the agent started editing", which is exactly the diff
  semantics S1 wants) and rebuild the guard verdict from it.
- Stamp the `Freshness` contract on **every** MCP tool response, not only
  `repo_overview` (`mcp/mod.rs:286-307`); `guard_decision` and
  `convergence_report` must carry `indexed_revision`/`is_stale` — an agent
  acting on a stale verdict is the exact "silent staleness" failure
  `ONLINE_CODE_GRAPH_ARCHITECTURE.md` names as the design enemy.

**Where:** `mcp/watch.rs`, `mcp/mod.rs`, `mcp/contracts.rs`. **Effort:** 2–3
days. **Difficulty:** low.

### S3. Expand the mechanism-family catalog (DuplicateMechanism fuel)

**Eliminates more of:** parallel mechanisms (symptom 14). The detector design
is right; the catalog is 4 families (`detect_mechanism_families`,
`assessment/mod.rs:2593-2640`), Laravel/WordPress-flavored.

**How:** add families as data, per the existing pattern (path shapes + content
markers, provenance-tagged where framework-specific — follow the
`framework_catalogs.rs` precedent):
- `http_client` (curl/Guzzle/reqwest/fetch/axios/requests/Net::HTTP) — the
  ZEUS_SHIELD "second HTTP client" canonical case
- `validation` (framework validator vs schema lib vs hand-rolled checks)
- `serialization` (json_encode vs serde vs manual string building)
- `db_access` (ORM vs query builder vs raw SQL)
- `mail_dispatch`, `cache_access`, `auth_check`, `logging`, `retry_backoff`,
  `date_time`, `uuid_generation`, `feature_flags`, `cli_output`
- Consider promoting the family list to a typed catalog module (like
  `scanners/framework_catalogs.rs`) so packs can contribute families without
  touching the core detector.

**Where:** `assessment/mod.rs` + optionally a new `assessment/mechanisms.rs`
catalog. **Effort:** 3–5 days incl. FP-tuning fixtures. **Difficulty:** low —
data work; the concept-token grouping, ranking, severity, and guard regression
flag (`duplicate_mechanism_regression`) all exist.

### S4. Contract-duplicate comparator

**Eliminates:** config/state duplication blind spot (symptom 11's remaining
half) and a slice of UX dual-path (symptom 15). The inventory already stores
every route/hook/env/config value with occurrence counts + locations
(`contracts/mod.rs:79-90`); nothing flags collisions.

**How:** post-process the inventory: same route value declared in ≥2 files;
same env key read with different defaults; same config key written from
multiple sites; same hook name registered by parallel handlers in different
mechanism families (join with S3). Emit as a new assessment kind
(`ContractCollision`) with the standard fingerprint/severity/doctrine-ref
shape so convergence + guard pick it up for free.

**Where:** `assessment/` (consumes `ContractInventory`), surfaced via existing
pipelines. **Effort:** 2–3 days. **Difficulty:** low.

### S5. Repo-owned sanctioned mechanisms — make doctrine executable

**Eliminates:** the "generic slogan" advise gap. `preferred_mechanism` today is
a string like `single_sanctioned_configuration_path`
(`artifacts.rs:6105-6156`) — it never names the concrete repo API an agent
should call. Policy can only suppress, never prefer (`policy/mod.rs:46-203`).
`PLUGIN_STACK.md` already names this as the missing top layer.

**How:**
- Extend `.aigiscode/doctrine.json` (loader exists,
  `doctrine/mod.rs:425-456`) with `sanctioned_mechanisms`: concern →
  `{ symbol/module path, example call, forbidden alternatives, waiver owner,
  expiry }`.
- Validate sanctioned symbol paths against the semantic graph at load time
  (dangling doctrine = explicit warning, not silent decoration).
- Thread concrete mechanisms into guardian packets and MCP `explain_finding` /
  handoff output: "use `App\Support\Http\Client` (sanctioned), not a new
  Guzzle instance at app/Services/FooClient.php:12".
- `SanctionedPathBypass` and `DuplicateMechanism` findings should cite the
  sanctioned entry when one is declared for the concern.

**Where:** `doctrine/`, `artifacts.rs` packet builders, `mcp/contracts.rs`.
**Effort:** ~1 week. **Difficulty:** medium — schema + threading, no new
analysis.

## Tier 2 — New Detectors on the Existing Graph (no parser changes)

### S6. Speculative-generality detector

**Eliminates:** overengineering sub-shapes (symptom 2's gap).

**How:** pure graph queries over resolved edges + symbols:
- interface/trait/abstract class with exactly one implementer
  (`Extends`/`Implements` edges) and no test-double implementer;
- role-named file (factory/strategy/provider — reuse `is_abstraction_role`,
  `assessment/mod.rs:2663-2670`) whose product set has cardinality 1;
- config/registry symbols whose value set never varies (join with contract
  inventory `registered_keys`).
Emit as `SpeculativeGenerality` assessment kind → automatic convergence
regression flag + guard warn trigger, per the existing per-kind pattern.

**Effort:** 3–5 days. **Difficulty:** medium-low; main risk is FP tuning
(plugin-style architectures legitimately have single implementers early —
proof-tier as `heuristic` so the guard only warns).

### S7. Overfactoring + wrapper/indirection detector

**Eliminates:** symptoms 3 and 6 (both SUBSTRATE today).

**How:** from `resolved_edges` + symbol spans/arity (all already parsed):
- **pass-through candidate**: function span ≤ N lines, exactly one outgoing
  `Call`, arity ≈ callee arity, no other reference kinds → wrapper;
- **indirection chain**: path of pass-through candidates length ≥ 2 →
  ceremonial indirection depth finding;
- **fragmentation cluster**: group of small single-caller functions whose only
  caller chain reconstructs one logical flow across ≥ 3 files → overfactoring.
Rank by betweenness of the touched files so hot-path indirection outranks
leaf noise.

**Where:** new `detectors/indirection.rs` (fits the detectors contract) or
assessment kind. **Effort:** ~1 week. **Difficulty:** medium. Ruby caveat:
thinnest adapter (no return types, `parsing/ruby.rs:225`) → lower proof tier
there.

### S10. Public dead-code via entry-point reachability closure

**Eliminates:** the public-API half of symptom 5.

**How:** reuse the security module's BFS reachability
(`security/mod.rs:1003-1245`) generalized over Call/Dispatch/EventPublish/
ContainerResolution edges, seeded from runtime entry candidates + contract
routes/hooks + exported package surfaces. Symbols unreachable from any seed →
`UnreachablePublicSymbol` at `Heuristic` tier (dynamic dispatch honesty),
`boundary_truncated`-aware so scoped scans never lie. Runtime plugins already
recover framework-implicit edges (queue/container/signals/WP hooks,
`plugins/*`), which is what makes this viable without drowning in FPs.

**Effort:** ~1 week. **Difficulty:** medium — the risk is dynamic-dispatch
false positives; proof tiers + policy entry patterns are the mitigation.

### S11. Convention-drift detector (naming + idiom distributions)

**Eliminates:** symptom 7's remaining bulk.

**How:** per repo × language × symbol-kind, compute naming distributions
(casing, verb prefixes, accessor styles) from the symbol table; flag outliers
against the dominant local convention (the repo is its own spec — no global
style opinions, per GOAL.md's "project-specific accepted patterns" layer).
Verb-synonym clustering (`get/fetch/retrieve` on the same concept token —
reuse `split_identifier_words` + concept extraction from
`assessment/mod.rs:2642-2702`) catches divergent names for one concern.
Feed `SplitIdentityModel` rather than inventing a parallel kind where they
overlap.

**Effort:** ~1 week. **Difficulty:** medium; FP control via
occurrence-threshold + policy overlay.

### S12. The advise query: `find_existing` (before-loop keystone)

**Eliminates:** the root cause feeding symptoms 1, 4, 13, 14 — the agent
reimplements because asking "does this already exist?" is expensive. Today no
MCP tool answers it (`mcp/mod.rs:280-628` has navigation, not lookup-by-intent).

**How:** new MCP tool + CLI verb: input = concern description + optional
proposed symbol name; output = ranked existing candidates with evidence:
- symbol-name/concept-token match over the global symbol index
  (`resolve/mod.rs` global index already exists);
- mechanism-family match (S3 catalog): "notification concern → repo already
  uses queue_jobs at these 3 files";
- sanctioned mechanism (S5) if declared — authoritative answer;
- contract inventory hit (route/hook/env/config already declared);
- dependency capability hit (S9): "date math → chrono already in Cargo.toml".
Deterministic retrieval; optional embedding rerank later (fastembed is already
scoped feature-gated in `IMPROVEMENT-PLAN-2026-03-24.md` §6.2 — not required
for v1).

**Where:** `mcp/`, `cli.rs` orchestration, logic in a new `advise/` module.
**Effort:** 1–2 weeks. **Difficulty:** medium. This single tool converts
AigisCode from "reviewer after the fact" to "memory before the act" — the
highest-leverage item in the roadmap relative to the pathology's root causes.

## Tier 3 — Parse-Layer Extension + Catalogs (the two big unlocks)

### S8. Function shape fingerprints → semantic-duplication clustering

**Eliminates:** symptoms 1 and 8 (both ABSENT), improves 12; feeds S14.

**How:**
- Parse layer: for each function/method body, emit (a) a **normalized token
  stream hash** (identifiers → kind placeholders, literals → type placeholders)
  and (b) a **k-gram winnowing sketch / MinHash signature** for near-match.
  Tree-sitter already parses full bodies in all five adapters — the data is
  currently thrown away, so this is emission, not new parsing. Store as
  compact fields on `SymbolNode` (hash + ~64-byte sketch), keeping
  `semantic-graph.json` size sane.
- Detector: exact-hash groups = clones; high-Jaccard sketch pairs with
  differing hashes = **copy-paste-mutate candidates** (the divergent-bug-fix
  time bomb — flag these *higher* than exact clones); cluster cross-file, rank
  by span size × occurrence × bottleneck centrality. New
  `DuplicateImplementation` assessment kind → convergence/guard integration
  free, `duplicate_implementation_regression` blocks the #1 slop vector at
  edit time once S1/S2 land.
- Policy: `allowed_clone_patterns` for legitimate parallels (per-language SDK
  ports, generated code) via the existing suppression machinery.

**Effort:** 2–3 weeks (5 adapters + detector + fixtures). **Difficulty:**
medium-high — the algorithms are classical (winnowing/MinHash); the work is
per-language normalization quality. **Highest detection ROI in the roadmap.**

### S9. Dependency ingestion + capability catalog (dependency-cancer detector)

**Eliminates:** symptom 13 (ABSENT), completes symptom 4.

**How:**
- Parse manifests + lockfiles (`Cargo.toml`, `package.json`, `composer.json`,
  `pyproject.toml`, `Gemfile`) into a typed `DependencyInventory` artifact —
  a new contract-inventory category, same shape discipline.
- Ship a curated **capability catalog** (data, not code — plugin-stack layer):
  package → capabilities (`guzzle → http_client`, `moment/dayjs/date-fns →
  date_time`, `lodash.clonedeep → deep_clone`…) + framework-native capability
  map (`laravel → mail, queue, validation, http_client…`; `wordpress →
  mail(wp_mail), http(wp_remote_get)…`), versioned like the ast-grep catalogs.
- Detectors: **capability collision** (two deps providing the same capability
  = second-HTTP-client case); **native shadowing** (dep whose capability the
  detected framework provides = the WordPress-mail canonical case, keyed off
  the framework gates that already exist in
  `scanners/framework_catalogs.rs:285-352`); **new-dependency delta** via
  convergence set-diff (mirrors `contract_value_delta`,
  `artifacts.rs:4330`) → guard trigger `dependency_expansion` with doctrine
  ref `guardian.minimal-mechanism`.
- Join with the existing external plane: abandoned (composer-audit) or
  vulnerable (SCA tools) + *redundant* = strongest possible removal case,
  auto-cited in packets.

**Effort:** 2–3 weeks (manifest parsers are small; the catalog is the work —
seed with ~100 highest-frequency packages per ecosystem, grow as data).
**Difficulty:** medium. Turns ZEUS_SHIELD §4 from prose into a detector.

## Tier 4 — Hardest: Composition + AI Adjudication

> Tier 4 items are sketched here at roadmap granularity; the full mechanism
> design for the human-dominant classes (library-replacement recognition,
> dual-path elimination, dead-code-that-looks-alive) lives in
> `SLOP_HARD_PROBLEMS_DESIGN.md` — evidence ledger, probe protocol, runtime
> evidence adapters, tombstone lifecycle.

### S13. Outcome-path multiplicity (code + UX dual-path detector)

**Eliminates:** symptom 15 — the pathology's product-level endgame.

**How (composition of shipped parts):**
1. Enumerate entry points: contract routes/hooks/commands + runtime entries +
   semantic-pack routes (all exist).
2. For each entry, compute reachable **state-mutation sinks** via the
   generalized BFS from S10 over call/dispatch/event/container edges; label
   sinks using semantic state-flow evidence (mutable-carrier machinery in
   `agentic.rs`) + write-shaped calls (ORM writes, file writes, cache writes —
   mechanism families from S3 double as sink labels).
3. Cluster entries sharing a dominant sink: ≥2 entries → same mutation = a
   **path-multiplicity group**.
4. Diff the per-path side-effect sets (which other sinks each path touches):
   divergence = the "swipe soft-deletes / menu hard-deletes; only one route
   audit-logs" finding, with both full paths as evidence chains (trace
   format already exists in packets: `graph_traces`, `code_flows`,
   `source_sink_paths`).
5. Emit `DivergentOutcomePaths` with proof tier by resolution strength
   (`exact_resolved` / `receiver_typed` / `heuristic` — taxonomy already
   exists, `agentic.rs:1943`).

**Effort:** 3–5 weeks after S3+S10. **Difficulty:** high — sink labeling
precision and path explosion control (reuse the 8-hop cap + causal scoring
from `security/mod.rs`). Worth it: no mainstream analyzer ships this; it is
the most direct machine expression of "architectural entropy escaping into
the interface".

### S14. AI adjudication plane for semantic equivalence

**Eliminates:** the precision ceiling on S8/S11/S13 — deterministic analysis
can cluster candidates but cannot *prove* "these two functions implement the
same behavior" or "these comments restate the code".

**How:** this is what the agent execution layer already exists for
(`agent-run`/`agent-spider`, adapter boundary in `cli.rs:1206-1291`):
- Detector emits candidate clusters with bounded packets (both bodies, callers,
  traces — the packet format already carries exactly this).
- A structured adjudication task per cluster: "same behavior? divergences?
  which is canonical? merge plan?" against the JSON schema contract
  (`agent-output-schema.json` machinery exists).
- Verdicts persist as `rules.json`/policy entries (confirmed-distinct →
  permanent suppression; confirmed-duplicate → actionable finding at
  `modeled` precision), so the system **converges**: each adjudication makes
  the deterministic layer quieter — GOAL.md principle 5 applied to AI verdicts.
- Same pattern later adjudicates comment noise, test-assertion quality, and
  UX-divergence intent ("is dual-path deliberate?").

**Effort:** 1–2 weeks after S8 (infrastructure exists; work = task packet
shape + verdict persistence). **Difficulty:** medium, but *dependent* on S8.
Division of labor is the point: deterministic engine narrows O(n²) candidate
space to dozens; AI judges only the shortlist; policy remembers the answers.

### S15. Defensive-code bloat + comment noise

**Eliminates:** symptoms 9 and 10 (both ABSENT; lowest severity in the
catalog, hence last).

**How:** rides on S8's body extraction: emit guard-clause/try-catch/null-check
counts and comment blocks per function at parse time. Deterministic layer
flags outliers (validation repeated along a call chain — needs the call graph
plus per-function check signatures; comment-density anomalies); S14
adjudicates "is this comment restating the code / stale". Full repeated-
validation proof is dataflow-shaped — accept `heuristic` tier rather than
building a dataflow engine for the lowest-value symptom.

**Effort:** 1–2 weeks incremental after S8+S14. **Difficulty:** medium.

### S16. Incremental engine (Phase 2 of `ONLINE_CODE_GRAPH_ARCHITECTURE.md`)

Not a detector — the scaling enabler. Full re-analysis per watch event caps
the live guardian at small/medium repos. The phased plan (per-file resolution
isolation, salsa spine, impact-based invalidation) is already specified in
that document, including its risks. Sequence it by pain, not by roadmap
position: every Tier 1–3 item works correctly on the batch path today.

## Sequencing & Dependency Graph

```
Tier 1 (parallel, ~2-3 weeks total):  S1  S2  S3  S4  S5
Tier 2 (parallel, ~4-6 weeks):        S6  S7  S10  S11  S12(←S3,S5, S9 optional)
Tier 3 (~4-6 weeks):                  S8            S9
Tier 4:                               S14(←S8)  S13(←S3,S10)  S15(←S8,S14)  S16(when repo size demands)
```

## What This Buys, Cumulatively

| After | Symptom coverage |
|---|---|
| Tier 1 | Guard becomes honest per-change gate; parallel-mechanism detection scales past 4 families; config duplication fully closed; doctrine names real APIs |
| Tier 2 | Overfactoring, wrappers, speculative generality, public dead code, naming drift all detectable; agents can ask "does this exist?" before writing |
| Tier 3 | Semantic duplication + copy-paste-mutate (the #1 pathology) and dependency cancer (the ZEUS_SHIELD canonical) detectable |
| Tier 4 | UX dual-path divergence; AI-verified precision on all clustering detectors; system converges toward quiet |

End state vs the scoreboard in `SLOP_PATHOLOGY_MAP.md`: every ABSENT and
SUBSTRATE row becomes at least PARTIAL-with-AI-adjudication; nothing in the
pathology catalog remains structurally undetectable. The residual hard core —
proving behavioral equivalence and intent — is exactly the part delegated to
the AI plane on top of deterministic candidate generation, which is the
product's stated design (`GOAL.md`: explainable analysis narrows, AI + policy
converge).
