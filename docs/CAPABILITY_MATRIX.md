# AigisCode Capability Matrix (2026-07-04)

Dense inventory: what the tool does → what bad habit/failure it prevents.
Written for brainstorming coverage gaps. One line per capability.

## Substrate (what it sees)

| Capability | Enables / prevents |
|---|---|
| Native parsers: PHP, TS/JS, Vue SFC, Python, Ruby, Rust (tree-sitter) | Whole-codebase truth without runtime; no language silo blind spots |
| Symbol graph: kind, qualified name, owner type, visibility, params, return type, spans | Signature-level (body-free) architecture judgment |
| Resolved edge graph with receiver-proof discipline + confidence tiers (SameFile/ImportScoped/Global) | Fake edges / fabricated coupling; "confident wrong" beats honest unresolved — we choose honest unresolved |
| Deterministic resolution (sorted candidates, byte-identical reruns) | Flaky findings; diff noise between runs |
| Evidence graph: call sites, line anchors, runtime evidence | Unverifiable claims — every finding cites source locations |
| Contract inventory: routes, hooks, env keys, config keys, registered keys, symbolic literals | Invisible framework wiring; contract drift |
| Semantic model packs (framework/library knowledge as data, not core code) | Framework vocabulary leaking into generic engine |
| Scoped analysis with boundary truth (`boundary-truncated` vs orphan) | False accusations on partial scans |
| Live MCP daemon (`--watch`): re-analyze on change, freshness contract (revision/is_stale/wait) | Agents acting on stale graph silently |

## Architecture-phase detectors (design sins, signature graph only)

| Detector | Prevents |
|---|---|
| Strong cycles/SCC (file-level, corpus-scale reclassification) | Circular knowledge; modules that can't exist independently; also prevents mislabeling a whole-crate SCC as a "cycle finding" |
| Layer contract violations (user-declared layers in doctrine, longest-prefix, deterministic, `certain` precision) | Wrong-direction dependencies: core reaching up, modules grabbing http internals, layering rot |
| GodClass (≥25 public non-magic methods AND ≥10 dependent files; evidence = top used methods per consumer count) | Everything-classes whose every change ripples wide; names concrete segregation seams |
| Hub-like dependency smell | Unplanned coupling hubs |
| Unstable dependency smell | Depending on churny code from stable code |
| Bottleneck centrality | Unplanned load-bearing chokepoint files |
| Orphan modules — frontend (import/glob/worker/re-export suppression) + backend PHP (contracts/convention-suffix/convention-stem/lexical sweep incl. out-of-slice; test-only = dead) | Dead files kept "just in case"; AI-generated files nothing wired in |
| SplitIdentityModel | Same concept under two names (snake vs camel drift) |
| CompatibilityScar | Old + new mechanism coexisting "temporarily" forever |
| DuplicateMechanism (language-gated dispatch catalogs) | One concern dispatched via hooks + jobs + events simultaneously |
| AbstractionSprawl (layer-word stopwords) | One concept smeared across 4+ roles/dirs |
| SanctionedPathBypass (raw env outside config; container/service-locator outside providers; ast-grep reinforced) | Sneaking past sanctioned paths; hidden dependencies via service location |
| HandRolledParsing (incl. homegrown schema validation, scheduler DSL, policy engine, page resolution) | Reinventing infrastructure that libraries do better |
| WarningHeavyHotspot (pressure aggregation) | Slow rot concentration going unnoticed |

## Implementation-phase detectors (body sins)

| Detector | Prevents |
|---|---|
| AlgorithmicComplexityHotspot (IO-in-loop ranked: http > db > fs > cache > json > sort > regex > nested; reachability-boosted; string/comment masked) | N+1 queries, json_decode-in-loop, accidental O(n²) on hot paths |
| Dead code: UnusedImport (lexical-guarded), UnusedPrivateFunction | Import litter; dead private helpers; copy-paste residue |
| Hardwiring: HardcodedNetwork (endpoint-only, xmlns/schema/href filtered), RepeatedLiteral (constant-DNA, css/test filtered), magic strings | Config that can't be changed without code edits |
| Security: DangerousApi (eval/exec/shell_exec/unserialize/innerHTML/document.write/new Function; hardened-unserialize aware; prose/string-mention masked) | Injection surfaces; "eval mentioned in a blog post" false alarms |
| ast-grep structural scanner plane with provenance (`ast_grep.pattern.laravel` etc.) | Framework misuse clues without polluting graph truth |

## Judgment & process layer

| Capability | Prevents |
|---|---|
| Two-phase review surface (`phase: architecture\|implementation`; report + sort lead with architecture) | Mortar-before-walls review; complexity noise drowning design findings |
| Guard verdict (allow/warn/block + review radius + obligations) | Merging architectural/security regressions unnoticed |
| Convergence history (run-over-run deltas per fingerprint) | Slow drift; "is it getting better or worse" blindness |
| Doctrine registry (machine-readable clauses, blocking dispositions, layers) | Tribal-knowledge rules nobody enforces |
| Guardian packets (per-finding obligation + preferred mechanism + acceptance) | Findings without fix directives |
| Policy (`policy.json` + `tune`): suppression with explicit status (accepted_by_policy/excluded_by_rule) | Silent suppression; findings vanishing without record |
| Proof tiers (Certain/Strong/Heuristic) + precision labels (exact/modeled/heuristic) on every finding | Overclaiming; agents trusting weak evidence as fact |
| Severity via scaled families (no saturation at max) | "Everything is critical" = nothing is |
| Fingerprint stability across runs | Finding identity churn breaking ratchets |

## Agent interface (MCP + artifacts)

| Capability | Prevents |
|---|---|
| `repo_brief` ≤3KB orientation (language mix, entries, hotspots, doctrine headline) | Agent burning 100k tokens to orient |
| `find_symbol` / `symbol_usages` (tiered ranking, caller-grouped, never guesses ambiguous) | Grep-and-pray symbol lookup |
| `module_design` (signatures + cross-module edges, zero bodies) | Reading raw source to see a module's shape |
| `list_findings` (family/phase/severity/path/language filters) + `explain_finding` (evidence by id) | Aggregate dumps without discrimination |
| `show_hotspots` (inbound/outbound file counts), `show_cycles`, `graph_neighbors`, `graph_trace`, `cypher_query` | Instance-vs-dependents confusion; unanswerable "who talks to whom" |
| `coverage_report` with unresolved breakdown (same-repo vs external, per kind) | Hidden analysis blind spots presented as clean |
| Graph packets + repository topology (bounded doctrine-aware neighborhoods, zone briefs, triage steps) | Agents loading whole graph artifacts |
| Agent handoff artifact + agent-run/agent-spider executors (codex-exec, responses-http adapters) | Graph-blind AI review |
| Freshness params (`min_revision`/`consistency`/`wait_ms`) | Racing the indexer |

## Known gaps (current roadmap)

- VisibilityLeak (public-but-locally-used), SiblingContractDrift, HighFanInWithoutInterface (G3 remainder)
- API surface as contract + breaking-change tracking in convergence (G4)
- Symbol-granular SCC + weighted feedback-edge cut lists as a tool (G5 — done by hand twice, not productized)
- Parser signature completeness: param types, implements/trait edges, instanceof/::class refs (G7)
- Dead exported symbols (symbol-level orphans, not just files)
- Guard phase weighting (architecture regressions should outweigh implementation ones)
- Phase-2 body review ordered by phase-1 centrality (partially: report is split, ordering not centrality-driven)
- Generated-file flag as pipeline concept (two ad-hoc implementations exist)
- Cross-run severity calibration; test-coverage awareness; duplication (clone) detection beyond mechanism families; type-flow/nullability analysis; secrets scanning; dependency-manifest audit (outdated/vulnerable packages); commit-history signals (churn × complexity)
