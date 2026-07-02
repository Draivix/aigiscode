# Hard Problems Design — Mechanizing the Human-Dominant Slop Classes

> Deep design for the three slop classes where human engineers still dominate
> tooling, extending `SLOP_ELIMINATION_ROADMAP.md` past its Tier 4 sketches:
>
> - **H1** — "this custom code should be a library/native call" (recognition
>   *and* elimination)
> - **H2** — dual-path / N-path elimination (not just detection — safe
>   canonicalization and removal)
> - **H3** — dead code that looks alive (reachable, tested, compiled — and
>   dead anyway)
>
> Thesis: humans dominate these not through unmechanizable judgment but
> through three **information asymmetries** — memory of what exists,
> knowledge of production behavior, and permission to run experiments. Each
> asymmetry has a concrete mechanical counter. What remains genuinely
> human afterward is a thin layer of product intent, and even that gets a
> structured decision surface instead of a shrug.

## 0. Shared Substrate — Three Mechanisms All Designs Reuse

### 0.1 The Evidence Ledger

Every hard verdict below is a *fusion* of independent evidence classes, never
a single detector's opinion. New artifact family member:

```
.aigiscode/evidence-ledger.json
  { subject: symbol|file|path-pair|dependency,
    evidence: [ { class, source, proof_tier, payload, observed_at, window } ],
    verdict: { state, confidence_millis, missing_evidence: [...] } }
```

Design rules, consistent with the existing artifact contract:
- every evidence entry carries provenance + proof tier
  (`exact_resolved`/`modeled`/`heuristic` taxonomy already exists,
  `agentic.rs:1943`);
- a verdict lists **which evidence would change it** (`missing_evidence`) —
  this is what turns "the tool guesses" into "the tool states its proof
  obligations", and gives the AI plane a work queue instead of a debate;
- fusion is deterministic (rule table over evidence classes), so the same
  ledger always yields the same verdict — AI and runtime feeds add *evidence*,
  never override verdict logic.

### 0.2 The Probe Protocol — AI as experiment executor, not opinion source

The single most important upgrade over roadmap item S14. Instead of asking a
model "are these equivalent?", the engine emits a **probe specification** — a
falsifiable experiment with a machine-checkable outcome — and the agent
executes it. The result enters the ledger as evidence with `probe` provenance.

```
probe-task packet (extends the agentic-review task packet contract):
  { kind: differential_function | differential_endpoint | tombstone_tracer
        | flag_flip | migration_trial,
    subject, hypothesis,                      // e.g. "local slugify() ≡ sluggo::slugify for valid UTF-8"
    harness: { inputs_from: call_site_literals | property_gen | recorded_traffic,
               oracle: output_equality | state_delta_equality | sink_set_equality },
    safety: { purity_class, allowed_side_effects, sandbox_required },
    expected_artifacts: probe-evidence.json schema }
```

Infrastructure already in place for this: adapter execution boundary
(`agent-run`/`agent-spider`, `cli.rs:1206-1291`), generated JSON-schema output
contracts (`agent-output-schema.json`), bounded packets with both code bodies
and traces. The probe protocol is a new packet kind plus an ingest step —
not a new subsystem.

Purity gating: probes that execute code are only auto-eligible for
`purity_class: pure|read_only` subjects (inferred: no outbound
Dispatch/write-shaped sink edges from the subject — computable from the
existing graph). Impure subjects get shadow-mode or tombstone probes only
(H2/H3 below). The engine must never specify an experiment whose side effects
it can't bound.

### 0.3 Runtime Evidence Adapters — importing production knowledge

The `external/` plane already normalizes 11 external tools into typed findings
with raw-artifact archival (`external/mod.rs`). Extend the same pattern from
*scanners* to *runtime telemetry*:

```
aigiscode analyze . --runtime-evidence nginx-access:/var/log/nginx/access.log
aigiscode analyze . --runtime-evidence otel:spans.jsonl
aigiscode analyze . --runtime-evidence generic:hits.json   # {route|symbol|file, hit_count, window}
```

Adapters parse access logs / APM spans / coverage-in-production exports into
`(contract_value | file | symbol) → hit evidence` joined onto graph nodes via
the contract inventory (routes are already string-anchored to files). The
`generic` adapter is the escape hatch: any team can emit the trivial JSON from
whatever telemetry they have. Edge provenance gains `RuntimeTrace` /
`LogEvidence` variants — already sketched in
`IMPROVEMENT-PLAN-2026-03-24.md` §9.2 (PhaseSeed hybrid call graphs).

This is the direct counter to "the human knows nobody uses that route".
Without it, H2 election and H3 liveness stay static-only (still useful,
honestly tiered); with it, they reach the evidence class that today lives
only in a senior engineer's head.

### 0.4 Fingerprint-Keyed Persistence — verdicts that expire themselves

All accepted states (deliberate dual path, intentional custom implementation,
kept-alive code) persist to policy/rules keyed on a **behavior fingerprint**,
not just identity:

```
rules.json entry: { subject_pair, accepted: true, reason, owner,
                    behavior_fingerprint: <hash of side-effect set / probe result / signature> }
```

If the underlying behavior later diverges (fingerprint mismatch on a
subsequent run), the suppression auto-invalidates and the finding returns.
"Accepted as equivalent" stays silent only *while it stays equivalent*. This
prevents the meta-failure where yesterday's waiver silently blesses tomorrow's
divergence — the exact bug class ("swipe soft-deletes, menu hard-deletes")
re-emerging under an old suppression.

---

## H1 — "Should Be a Library": Capability Recognition + Replacement

### Problem shape

Recognizing that 80 lines of local code reimplement `p-retry`, `date-fns`,
`wp_mail()`, or `serde_json` requires matching **behavioral capability**, not
text — library internals share no tokens with hand-rolls. Roadmap S9 covers
the manifest side (redundant *dependency* added); H1 is the inverse and harder
direction: *code* → capability → sanctioned replacement.

### Layer A — Capability Signature Index (deterministic, Rust core)

A capability signature = co-occurring structural features that any
implementation of that capability must exhibit. Expressible in the existing
scanner plane (ast-grep patterns + graph facts), shipped as a **data catalog**
(`scanners/capability_signatures.rs` following the `framework_catalogs.rs`
precedent — versioned rules, explicit provenance, zero core changes per new
capability):

| Capability | Signature features (≥ threshold must co-occur) |
|---|---|
| `retry_backoff` | loop + sleep/delay call + multiplication/shift on the delay variable + error check inside loop + attempt counter |
| `debounce/throttle` | stored timer handle + clear-timer + re-arm with same callback |
| `deep_clone` | self-recursive function + type-switch on container kinds + per-key copy |
| `date_math` | arithmetic with 86400/3600/604800 + month-length table + leap-year `%4/%100/%400` |
| `uuid` | random hex + 8-4-4-4-12 formatting or version-bit masking |
| `csv/json/xml parse` | char-loop with quote-state variable, split-chain accumulation (extends `HandRolledParsing`'s primitive-density approach, `assessment/mod.rs:2716`) |
| `slugify`, `levenshtein`, `semver_compare`, `html_escape`, `query_string`, `currency_format`, `pluralize`, `mime_lookup`, `pagination_math`, `password_hash`, `csv_export`, `template_render` | analogous feature sets |

**Constant DNA** — the cheap high-precision second channel: hand-rolled
implementations of known algorithms carry unforgeable constants — CRC tables,
`0x5bd1e995` (murmur), `2166136261` (FNV), base64 alphabets, HTML entity
tables, month tables, TLD lists, RFC regex fragments. A constant→algorithm
index is a trivial scanner with near-zero false positives; a constant-DNA hit
plus a signature hit lifts the finding from `heuristic` to `modeled` on its
own. (The hardwiring detector already extracts literals repo-wide —
`detectors/hardwiring.rs` — the DNA index is a lookup layered on data that is
already collected.)

Fusion rule: **two independent evidence classes minimum** (signature +
constant DNA, or signature + probe) before a finding surfaces above
`heuristic`. Single-channel hits stay investigation prompts.

### Layer B — Resolution to a concrete replacement

A recognized capability is only actionable when the *replacement* is named:

1. **Already-sanctioned** — doctrine `sanctioned_mechanisms` (S5) names the
   repo's chosen implementation → finding says "replace with
   `App\Support\Retry::run()` (sanctioned)". Strongest case.
2. **Already-in-manifest** — dependency inventory (S9) shows a dep providing
   this capability → "you depend on `p-retry@6` and don't use it here".
   Second-strongest: no new dependency debate needed.
3. **Framework-native** — the framework capability map (S9) covers it →
   "WordPress provides `wp_mail()`" — the ZEUS_SHIELD canonical case
   (`ZEUS_SHIELD.md` §4), keyed off framework gates that already exist
   (`scanners/framework_catalogs.rs:285-352`).
4. **Ecosystem-known** — catalog suggests candidates, but *adding* a dep is
   itself governed (`guardian.minimal-mechanism`); finding proposes, guard
   treats the add as a dependency-expansion event. The tool must not become a
   dependency-cancer vector while fighting reinvention — these two detectors
   deliberately pull against each other, and doctrine arbitrates.

### Layer C — Probe-verified equivalence + migration

For `pure|read_only` subjects (most of this class: parsers, formatters,
validators, math), emit a `differential_function` probe: run local
implementation vs replacement on inputs harvested from **call-site literals**
(arity + literal arguments already in the reference records) plus
property-generated inputs. Outcomes:

- `behaviorally_equivalent` → migration is mechanical; agent rewrites call
  sites (bounded by the graph's exact caller list — no grep guessing),
  deletes the local implementation, convergence records `Resolved`.
- `equivalent_modulo_edge_cases(list)` → **the gold output**. The divergence
  list is precisely what a human reviewer could not enumerate: "local
  `validateEmail` rejects `+`-addressing; the library accepts it." Each
  divergence is adjudicated once (bug in local impl? deliberate policy?) and
  the verdict persists per §0.4.
- `divergent` → not a reimplementation; evidence ledger records the probe so
  the candidate never resurfaces (deterministic quieting).

### What stays hard, honestly

- Capabilities without crisp signatures (business-rule-adjacent logic).
  Mitigation: catalog grows by demand; S8 clone clustering catches the
  *internal* reinvention case (repo reimplements its own helper) without
  needing any catalog.
- Deliberate forks of library behavior. Mitigation: the probe's divergence
  list makes the deliberateness question *specific*, and §0.4 keeps the answer.
- Catalog maintenance cost. Mitigation: it is data (YAML-shaped rules), the
  KNighter pattern (`IMPROVEMENT-PLAN` §9.1) can draft signatures from known
  library test suites, and each ecosystem needs only its top ~50 capabilities
  to cover the overwhelming mass of real-world hand-rolling.

---

## H2 — Dual-Path / N-Path Elimination

### Problem shape

S13 detects "≥2 entries reach the same state mutation with divergent
side-effect sets". Elimination requires three decisions detection doesn't
make: **which path is canonical**, **how to converge without breakage**, and
**when a redundant entry may actually be removed**. Humans dominate via
product intent + production knowledge + fear management. Design each.

### Stage 1 — Full-stack path enumeration (extending S13)

Server-side entries come from the contract inventory + runtime-entry
promotion. The UX-level pathology needs the **frontend half**: the TS/JS
adapter already parses frontend code; add an entry-affordance extraction —
event handler → `fetch`/`axios`/form-action → route-string literal — and join
on the server route inventory (Stimulus `data-action` extraction already
exists, `contracts/mod.rs:346-385`, proving the pattern). Result: full-stack
paths *(button/gesture/menu-item) → route → handler → sink set*. Two UI
affordances hitting two different routes that reach the same mutation with
different side-effect sets **is** the "swipe soft-deletes / menu hard-deletes"
finding, end to end.

### Stage 2 — Canonical path election (deterministic scoring, not AI vibes)

Score every path in a multiplicity group:

| Signal | Rationale | Source |
|---|---|---|
| **Side-effect completeness** | The path performing the superset of cross-cutting obligations (audit log, cache invalidation, event emission, validation) is usually the sanctioned one | sink-set diff (S13) |
| **Doctrine** | A sanctioned mechanism naming one path decides the election outright | S5 |
| **Contract weight** | Framework-sanctioned surface (declared route/controller/command) beats ad-hoc wiring | contract inventory |
| **Maintenance signal** | Actively maintained vs fossil: last-touch recency, churn, author count | git (already shelled for owner hints, `artifacts.rs:3336`) |
| **Production usage** | Route hit counts over window — the human trump card, imported | runtime evidence adapters (§0.3) |

Election output = ranked paths + per-signal score breakdown. Ambiguous
elections (close scores, no doctrine) are **explicitly surfaced as product
decisions** with the evidence table attached — the tool's job is to reduce
"someone should look at this someday" to "answer this one scored question",
not to fake certainty.

### Stage 3 — Converge, then remove (strangler, mechanized)

The safety design: **never delete a divergent path; converge first, remove
second.** Divergent paths are load-bearing in unknown ways; identical paths
are safe to collapse.

1. Side-effect diff → generated obligations: "path B misses the audit-log
   call present at path A hop 3" (obligation machinery exists,
   `artifacts.rs:6166`).
2. Refactor both entries to route through one internal implementation — the
   canonical path's orchestration; the redundant entry becomes a thin
   forwarder. **Structurally verifiable**: after the refactor, both entries'
   traces pass through the same node and the sink-set diff is ∅ — the engine
   *proves* convergence from the graph, no judgment needed.
3. For endpoints, optional `differential_endpoint` probe before switching:
   shadow-replay recorded traffic (or agent-browser-driven UI flows for
   affordances) through both paths, compare state deltas + emitted events.
   Impure by definition → sandbox/staging only, per §0.2 safety gating.
4. Entry removal is now a *product* decision with usage data attached (Stage 2
   runtime evidence), decoupled from the *engineering* decision (behavior
   convergence) which is already done and proven. This decoupling is the
   design's core move: it splits the one scary decision humans agonize over
   into one provable step and one small, evidenced product choice.
5. Guard enforces monotonicity: `DivergentOutcomePaths` may never worsen
   (per-kind regression flags pattern, `artifacts.rs:3701-3757`); convergence
   history shows the funnel narrowing group by group.

Deliberate multi-path (mobile gesture + desktop menu, both wanted): accepted
with §0.4 fingerprint keying on the sink-set — both affordances stay silent
*only while behaviorally identical*. The moment one drifts, the finding
returns. That converts the UX dual-path pathology from "detect once" into a
standing invariant.

---

## H3 — Dead Code That Looks Alive

### Problem shape

Static reachability (S10) kills the easy class. The hard class *passes* every
static check: compiled, imported, tested, reachable — and dead. Sub-species:
reachable-but-never-invoked handlers; branches gated by flags that are never
on; code whose only consumers are its own tests; superseded implementations
kept "just in case". Humans catch these with production knowledge and
institutional memory. Both are importable.

### The Liveness Evidence Ledger — from binary to graded verdicts

Replace reachable/unreachable with a fused verdict over five cheap evidence
classes:

**E1 — Static reachability** from the entry closure (S10). Baseline.

**E2 — Test-only liveness** (purely static, high yield, and almost nobody
ships it): partition inbound paths by origin; a production symbol whose
*every* inbound path originates in test files is test-only-live — it
compiles, its tests pass, and it is dead. Test-path classifiers already exist
(`contracts/mod.rs:525-541`); this is a graph partition query. The single
best detector for "seems real" dead code, available with no runtime data at
all.

**E3 — Flag/config gating analysis**: extract guard conditions that reference
known config/env accessors (accessor patterns already cataloged in the
contract inventory, `contracts/mod.rs:607-648`) and join against declared
config values:
- guarded key absent from every config/env manifest → `config_dead (strong)`
- key statically false/off in every environment file → `config_dead (strong)`
- key externally controlled → `heuristic`, escalate to E4 for resolution

This mechanizes "that feature flag has been off since 2024".

**E4 — Production runtime evidence** (§0.3): reachable code with zero hits
across the observation window → `runtime_silent`. The window is part of the
evidence record — a 7-day window proves nothing about a year-end report
generator, and the ledger says so instead of hiding it (`missing_evidence:
longer window or seasonal-entry annotation`).

**E5 — History forensics** (git, computed per-candidate, not repo-wide):
callers deleted one by one over history (orphaned by attrition); last
substantive edit far beyond repo median; S8 clone-cluster sibling receiving
all new call edges while this member's shrink (the superseded-implementation
signature — the fossil twin).

Fusion ladder (deterministic, per §0.1):

```
provably_dead      unreachable | config_dead(strong)
test_only_live     E2 — production code consumed only by its tests
runtime_silent     reachable + zero production evidence over stated window
presumed_live      everything else (never surfaced as a finding)
```

### Elimination protocol — engineering the fear away

Deletion fear is rational: the cost of a wrong delete is unbounded, the cost
of keeping cruft is invisible. The protocol re-prices both sides:

1. **Tombstone probes** for `runtime_silent`/`test_only_live`: don't delete —
   the agent injects a one-line log-once beacon (a `tombstone_tracer` probe,
   §0.2), ships it, and the ledger waits. Beacon silent through the window →
   verdict upgrades to `provably_silent` → deletion is now evidence-backed.
   Beacon fires → candidate reclassifies `presumed_live` with the triggering
   call recorded — the system *learned a real entry point* it was blind to,
   which feeds back into E1's entry model. Tombstones carry owner + expiry in
   the ledger (the doctrine waiver expiry concept, `PLUGIN_STACK.md` §4,
   reused) so they can't themselves become cruft.
2. **Deletion execution**: per-cluster (a symbol plus its now-orphaned
   private helpers — the graph gives the closure), blast radius from the
   `required_radius` machinery (`artifacts.rs:3648`), tests run, one revert
   -friendly commit per cluster.
3. **Incentive inversion** — the pathology's root cause "deleting 200 lines
   looks like nothing happened" gets a direct product answer: convergence
   history and the report surface **deleted-LOC and resolved-finding counts
   as first-class wins**, and a guard `Allow` after a deletion-heavy session
   is an agent-reportable success. Restraint and garbage collection become
   visible work.

### What stays hard, honestly

- **External consumers the repo can't see** (public API clients, cron on
  another host, webhooks). No static analysis can close this; tombstones +
  runtime windows are the honest mitigation, and route-level findings say
  "externally exposed — tombstone before delete" (externally-reachable
  classification already exists, `security/mod.rs:260-276`).
- **Seasonal/rare paths**: window-stamped evidence + policy annotations for
  known-rare entries (`seasonal_entry_patterns`), not false confidence.
- **Reflection/metaprogramming call sites**: already handled philosophy-wise
  by proof tiers — dynamic-dispatch-heavy targets cap at `heuristic` and lean
  on E4/tombstones instead of static claims.

---

## Rollout & Module Placement

| Piece | Module home | Depends on | Order |
|---|---|---|---|
| Evidence ledger + fusion | new `evidence/` + `artifacts.rs` emission | — | 1 |
| Probe protocol packets + ingest | `agentic.rs` packet kinds + adapter loop | ledger | 2 |
| Runtime evidence adapters | `external/` plane (pattern exists) | ledger | 2 |
| E2 test-only liveness | `detectors/liveness.rs` | S10 closure | 2 (cheapest big win — ship first) |
| E3 flag/config gating | `detectors/liveness.rs` + contracts join | — | 3 |
| Constant-DNA index | scanner plane | — | 3 (cheap, high precision) |
| Capability signature catalog | `scanners/capability_signatures.rs` | S3/S9 catalogs | 4 |
| Differential-function probes | probe protocol | purity classifier | 4 |
| Full-stack entry join (frontend affordances) | `contracts/` + TS adapter | — | 4 |
| Path election + convergence proof | `assessment/` + guard flags | S13, runtime evidence | 5 |
| Tombstone lifecycle | ledger + probe + doctrine expiry | probes | 5 |

Two items in this table are disproportionately cheap for what they kill and
should jump the queue the moment S10 lands: **E2 test-only liveness** (pure
graph partition, no new data, directly targets "dead code that seems real")
and **constant DNA** (lookup over literals already extracted, near-zero FP
evidence of hand-rolled algorithms).

## Closing Position

None of the three problems is solved by "make the AI smarter". Each is solved
by giving a deterministic engine the three things the senior human actually
has — an index of what exists (capability catalog, doctrine, contract
inventory), production knowledge (runtime evidence adapters), and the ability
to run experiments (probe protocol) — then letting the AI plane do the two
things it is genuinely good at: executing bounded experiments and writing the
migration. The residue that stays human — "do we *want* two ways to delete an
email?" — arrives as a scored, evidenced, single-question decision surface
instead of an invisible entropy leak. That is the strongest claim this design
makes: not that humans leave the loop, but that nothing reaches them except
the decisions that are actually theirs.
