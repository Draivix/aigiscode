# Online Code Graph Architecture

## Thesis

A code-intelligence tool for AI coding agents is only as good as how fast and
correctly it restructures its code graph while an agent is actively mid-edit.
A batch "run analyze, get a snapshot" tool is a report generator, not a
product an agent can trust turn-to-turn.

This document is the target architecture for making AigisCode genuinely
**online against code**: a persistent engine whose graph converges toward the
repository's true state in near-real-time as files change, with explicit,
honest freshness signals rather than silent staleness.

## Methodology

Produced by: a multi-agent workflow that read the actual source of two
competing tools (GitNexus — github.com/abhigyanpatwari/GitNexus; SocratiCode —
github.com/giancarloerra/socraticode) plus a self-audit of this repository,
followed by independent review and adversarial red-teaming by two frontier
models (Gemini 2.5 Pro via the `sage` MCP tool, GPT-5.5 Pro Extended via
chatgpt-gateway), each pushing back on and correcting both the original
critique and each other. Every claim below about GitNexus/SocratiCode is
sourced to a specific file and line; every architectural recommendation
survived at least one adversarial round.

## Where GitNexus fails

GitNexus (an embedded LadybugDB graph, 14-phase pipeline, MCP/CLI/HTTP
surfaces) fails nearly every dimension of "online" except "a process can stay
running":

- Zero filesystem watcher of any kind (exhaustive dependency and source grep:
  no chokidar/fs.watch/@parcel/watcher/inotify).
- Staleness is judged only by git commit-hash distance
  (`git rev-list --count lastCommit..HEAD`) — structurally blind to
  uncommitted working-tree edits an agent makes mid-session.
- The only automation is a Claude Code PostToolUse hook that fires solely on
  `git commit|merge|rebase|cherry-pick|pull` and just prints a suggestion to
  re-run `analyze` — it never reindexes itself, and there is no MCP tool to
  trigger analysis at all.
- Cross-file/graph-wide phases (scope resolution, MRO, Leiden communities,
  process tracing) are unconditionally whole-graph every run by design
  ("Leiden runs on the FULL graph" is stated as a correctness invariant). Only
  the final DB write-back is file-scoped.
- `detect_changes` reads a live `git diff` against **stored** symbol line
  ranges from the last analyze — it does not refresh the graph, and edits
  that shift line counts desync every downstream range with no drift warning.

Verdict: a sophisticated incremental *batch indexer* wrapped in daemon-shaped
processes that hold data warm but never become aware of live edits.

## Where SocratiCode fails

SocratiCode (stdio MCP server, Qdrant + Ollama backed) gets closer on watcher
mechanics but fails on the graph itself:

- Has a genuine native watcher (`@parcel/watcher`, one subscription per
  project root, ignore-globs at the native layer) with a real 2000ms trailing
  debounce and a concurrency-dedup guard — the strongest watcher story of the
  two competitors.
- But its own docs state outright: "Graph builds are always full
  reconstructions, not incremental." The one real incremental path (a
  symbol/call-graph patch) only fires below a hard-coded ≤50-changed-file
  cutoff; above that, or with no prior build, it falls back to a full
  rebuild, and cross-file resolution during the incremental path is
  best-effort/local only.
- There is no real graph database underneath: the file-import graph is
  `JSON.stringify()`'d wholesale into a single Qdrant point's payload,
  self-documented as "stored as metadata points, not real collections" — every
  read/write is all-or-nothing.
- Process lifetime is entirely host-controlled (it tears down every watcher
  and cache the instant its MCP host's stdio pipe closes); its own recon
  flags this as "genuinely ambiguous, not resolvable from source alone."
  Continuity across restarts is a fire-and-forget catch-up routine that can
  race a query into reading a stale index.

## Where AigisCode stands today

Confirmed batch-only. Every CLI verb (`analyze`, `report`, `agent`,
`agent-run`, `agent-spider`, `surface`, `graph`, `cypher`) funnels through one
shared pipeline that, on every invocation, re-walks the full tree, re-parses
every file, rebuilds every resolver index from zero, and recomputes all
detectors/artifacts over the complete graph. `ScannedFile` stores neither
`mtime` nor a content hash, so "unchanged since last run" is not even
representable in the data model yet.

The one asset already in place: a real, tested, native Rust MCP server on
the official `rmcp` SDK (12 tools / 16 resources / 3 prompts) — this repo's
own `CLAUDE.md` "Current CLI" list understates what already exists and should
be corrected alongside this document. But it changes nothing about
liveness: it calls `analyze_project()` once at boot, freezes the result into
an immutable `McpState`, and no code path refreshes it.

**Toolchain, fixed 2026-07-01**: all three review rounds independently
flagged a real, reproducible build blocker — the workspace declared
`rust-version = "1.82"` while the resolved lockfile (via the `globset` direct
dependency, and transitively via `darling 0.23.0`) actually required Rust
1.88, and this machine's distro-packaged `rustc` was 1.75.0 with no newer
`apt` candidate available. Fixed by installing `rustup` (the existing
`~/.rustup/settings.toml` shows it was configured before but not on `PATH` in
this shell), pinning `rust-toolchain.toml` to `1.88.0` (the verified floor —
`1.85.0` still failed on `darling`), and correcting `rust-version` in
`Cargo.toml` to match. `cargo check --workspace` and `cargo test --workspace`
now pass clean on the pinned toolchain. Anyone who clones this repo with
`rustup` installed will get the right compiler automatically.

## Target architecture

The prior-art survey (rust-analyzer/salsa, stack-graphs, tree-sitter
incremental reparse, clangd's dual index, SCIP, Cursor's Merkle-tree drift
detection, Continue.dev's content-hash catalog) converges on a layered,
file-keyed design assembled from several focused pieces rather than one
library. AigisCode already owns the two hardest prerequisites neither
competitor has natively in Rust: tree-sitter parsing across all six language
adapters, and a real MCP server on the official SDK.

The single most important correction from the adversarial rounds: **the
"single-writer" and "watcher/debounce" ideas that felt obviously right in
round one both had a latency-fatal reading that only surfaced under a
concrete burst scenario ("an agent makes 50 rapid multi-file edits across a
2000-file repo in 2 seconds").** Every recommendation below already bakes in
that correction.

1. **Revisioned consistency contract (this is the load-bearing piece the
   original critique underweighted).** Every graph answer is tied to a
   repository revision: `observed_revision`, `indexed_revision`,
   `dirty_paths`, `pending_paths`, `is_stale`, `semantic_env_revision`. A
   system that is usually fast enough to feel live is *more* dangerous when
   it silently falls behind than one that is honestly always-batch. Every
   MCP tool response carries these fields; graph-sensitive tools accept
   `min_revision` / `consistency: latest_available | wait_until_indexed |
   allow_stale` / `wait_ms`. Silent stale answers, not slowness, are the
   failure mode to design against.

2. **Durable file + semantic-environment identity.** Content hash and mtime
   alone are insufficient — a Rust file's meaning can change without its text
   changing (`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, feature/cfg
   flags, workspace layout). Extend `ScannedFile` with a content hash and
   thread a `(path, hash)` key through every derived fact, *and* track a
   `semantic_env_revision` fingerprint of the build/workspace configuration
   separately.

3. **Filesystem watcher as a hint stream, not a transaction log.** `notify` +
   `notify-debouncer-full` populate a dirty-path set and an append-only
   change journal — nothing parses directly from a watcher callback. Publish
   graph updates through a single **repository-level coalescing debouncer**
   (not per-path timers, which fragment one logical edit burst into dozens of
   near-simultaneous work items), but keep **per-path stable-read
   verification** so a truncate-then-write or temp-file-then-rename save is
   never parsed mid-write. Add explicit agent-facing hooks
   (`begin_edit_epoch` / `record_changed_paths` / `end_edit_epoch`) — an
   autonomous agent usually knows edit-transaction boundaries more precisely
   than filesystem events ever will.

4. **Parallel compute, serialized commit, atomic publish** — not a single
   task processing changed files one at a time. Read/hash/parse/extract runs
   in parallel (`rayon`) across an immutable base snapshot; only the final
   manifest/index update and revision bump are serialized and fast. Under a
   50-file burst, naive full serialization was independently shown (by
   adversarial red-teaming) to leave the graph 3-5 seconds stale exactly when
   the agent needs it — parallelizing compute while keeping the commit step
   serialized is the fix.

5. **Impact-based invalidation, not a file-count threshold.** Do not copy
   SocratiCode's ≤50-file cutoff — it is a documented performance cliff, not
   a safety valve (crossing it trades a multi-second problem for a
   20+ second one, per this repo's own measured `graph --kuzu` timing). Base
   the incremental-vs-broader decision on invalidation impact instead:
   public-surface fingerprint delta, import/export/module delta, semantic-env
   delta, reverse-dependency fanout. Process affected regions in topological
   waves (local facts → public surfaces → module graph → reverse dependents →
   resolved references → detectors → secondary indexes), collapsing cycles
   into SCCs.

6. **Stack-graphs-style per-file resolution.** Refactor each language
   adapter's output into an independent per-file subgraph, give
   `ResolutionContext` mutable upsert/delete-by-file operations instead of
   only "rebuild fresh," and stitch cross-file references lazily at
   query/report time as explicit "unresolved reference" facts — not silent
   best-effort guesses. This is the fix for what GitNexus treats as an
   unquestioned "correctness invariant": shrink the "must be global" set to
   what is provably non-local instead of defaulting everything cross-file to
   global.

7. **Salsa as the invalidation spine, adopted selectively.** Start with
   deterministic, file-keyed queries (`parse`, `file_facts`,
   `public_surface`) before moving cross-file detectors in. Keep filesystem
   I/O and watcher events entirely outside the query graph — stage everything
   into one coherent batch per edit epoch first. Budget memory from day one
   (rust-analyzer's own salsa 3.0 migration quadrupled memory before roughly
   a year of profiling closed most of the gap).

8. **Explicit partiality instead of silent quarantine.** Files mid-edit
   commonly produce tree-sitter `ERROR` nodes — this is normal, not
   exceptional, for an autonomous agent's edit cadence. Represent
   `Known / Unknown / Stale / Error / Approximate / LastKnownGood` states
   formally rather than deleting facts on a transient parse failure.

9. **Sharded immutable snapshots via ArcSwap**, not one monolithic graph
   clone per update. Shard by file facts / symbol index / reference index /
   module graph / call graph, so an edit only replaces the shards it
   actually touched, and track snapshot retention so long-running MCP calls
   cannot pin unbounded historical memory.

10. **Keep the Kuzu/Node.js bridge and embeddings off the daemon hot path.**
    `--kuzu`/Cypher export shells out to a Node helper today — fine as an
    opt-in batch/export feature, wrong as an always-on daemon dependency
    (contradicts the native-Rust-end-to-end goal). Give embeddings and
    natural-language summaries their own revision counter so a structural
    query never waits on embedding refresh.

Deliberately deferred: differential-dataflow/DBSP for genuinely global
queries (reachability, cycle detection, taint enumeration across the whole
graph). Every shipping system surveyed (stack-graphs, SCIP, clangd,
rust-analyzer) reaches sub-second freshness through file-level invalidation
plus query memoization, not a differential engine — revisit only if salsa
proves insufficient for a specific global query.

## Phased roadmap

### Phase 0 — Baselines (unblocks everything else)

- Toolchain/MSRV fix: **done** (see above).
- Correct `docs/BE_FE_MCP_ARCHITECTURE.md`'s framing of the Rust MCP server
  as future "Phase B" work, and this repo's `CLAUDE.md` "Current CLI" list,
  which both understate what is already implemented and tested today
  (`mcp`, `graph`, `cypher`).
- Measure real full-pipeline `analyze` timing and memory on a small, medium,
  and large repo. The only existing timing data point (22.78s for
  `graph --kuzu` on WordPress) covers a strict subset of the pipeline —
  not enough to size a daemon's resident memory or claim any speedup number.

### Phase 1 — Revisioned, honest daemon (no incrementality yet) — SHIPPED 2026-07-02

Status: implemented and verified (unit + integration + real-OS-watcher + a
binary-level `mcp --watch` smoke run). 284 tests pass; `cargo fmt`/`clippy`
clean.

- ✅ Content hash + mtime on `ScannedFile` (`src/revision.rs` identity types,
  `src/ingestion/hash.rs` xxh3) + a `semantic_env` fingerprint on `ScanResult`
  (config-change detection independent of source). Fact-level provenance
  threading (`(path, hash)` on every `ParsedFile`/`SymbolFact`) is Phase 2 prep
  and intentionally not required by the dumb-but-correct daemon.
- ✅ `notify` + `notify-debouncer-full` `mcp --watch` mode
  (`src/mcp/watch.rs`): native recursive watcher → ignore-filtered, access-event-
  suppressed → repository-level coalescer → single serialized rebuild task →
  `tokio::spawn_blocking(build_mcp_state)` → atomic publish. Re-runs the full
  `analyze_project()` on change (no incremental resolution yet). Rebuilds never
  write artifacts and `.aigiscode` is ignored, so the daemon never self-triggers.
- ✅ Live `ArcSwap`-backed state (`src/mcp/live.rs`, generic `LiveState<S>` so the
  revision logic is unit-testable): replaces the one-shot `McpState` clone. Every
  `repo_overview` response carries a `Freshness` contract
  (`revision`/`indexed_revision`/`observed_revision`/`is_stale`/`rebuilding`/
  `dirty_paths`/`generated_at_unix_ms`) plus optional
  `min_revision`/`consistency`/`wait_ms` params with a real `wait_until_indexed`.
  The two load-bearing honest-staleness tests (query-during-rebuild,
  changes-during-rebuild) pass.
- ⏳ Deferred (advisory, not correctness): MCP resource-subscription *push*
  (`notifications/resources/updated`). The freshness contract already covers
  correctness — a client polling `repo_overview` sees the revision change and the
  `is_stale` flag. Push is a latency optimization whose rmcp-1.2.0 peer/session
  bookkeeping needs verification first; tracked as the next small follow-up.

### Phase 2 — Incremental core

- Convert parsing to genuinely incremental where edit ranges are available
  (`Tree::edit` + `changed_ranges()`); full-parse the settled file otherwise
  — intra-file incremental parsing is not the main bottleneck in a
  filesystem-event-driven design, whole-repo re-resolution is.
- Give `ResolutionContext` mutable upsert/delete-by-file operations.
- Introduce the stack-graphs-style per-file isolation boundary behind a
  feature flag, validated by differential testing against the current
  full-batch path on the same corpus (`incremental_graph(final_state) ==
  full_batch_graph(final_state)`) before switching the daemon over. This is
  the single largest and highest-risk refactor — do not port-and-delete the
  old path on day one.
- Layer `salsa` over the now-incremental parse/resolve layer plus the
  cheapest, most-isolated detectors first.

### Phase 3 — Hardening for agent-driven load

- Split parallel compute (`rayon`) from a fast serialized commit step; make
  all parallel work deterministic against a specific base revision, with
  cancellation/supersession when a newer edit batch arrives mid-computation.
- Replace any file-count burst threshold with impact-based invalidation plus
  topological/SCC wave processing.
- Formally classify every existing detector/pass as file-local /
  crate-local / whole-graph — some (cycles, hotspots, convergence/
  guard-decision) may legitimately stay global; do not force a fake
  file-local model onto them.
- Build the adversarial test harness: 50 edits in 2 seconds, renames,
  deletes, temp-file writes, syntax errors mid-edit, `Cargo.toml`/feature
  changes, branch switches, watcher overflow/missed events — every case
  checked against the full-batch oracle.

### Phase 4 — Unification

- Fold `analyze`/`report`/`agent`/`agent-run`/`agent-spider` into "cold-start
  the same incremental engine from an empty cache" so the batch CLI and the
  daemon share one implementation instead of two that can drift apart.
- Design the lazy cross-file query layer explicitly (`find_references`,
  `callers/callees`, `impact_of_change`) with formal partiality states
  (`resolved / unresolved / ambiguous / stale / blocked_by_dirty_file /
  blocked_by_parse_error`) rather than leaving "how does a caller ask for
  all references" undesigned.

## Where the two reviewing models disagreed (kept visible, not smoothed over)

- **Single-writer wording.** The original proposal's "one dedicated async
  task owns all mutations" was read charitably in round one as safe; under
  adversarial pressure both reviewers converged that it is only safe if
  "writer" means the commit/publish authority, not literal serial per-file
  computation. Final position: parallel compute, serialized commit, atomic
  publish.
- **Debounce granularity.** Per-path debouncing (praised in round one) was
  reversed in round two: a repository-level coalescing debouncer is correct
  for *publication*, but per-path *stable-read verification* should stay —
  these are two different mechanisms, not one.
- **Burst threshold.** Full agreement across both models that a raw
  file-count cutoff (SocratiCode's precedent) is a performance-cliff
  anti-pattern, not a reasonable default to imitate.
- **Tree-sitter incremental parsing priority.** GPT-5.5 Pro pushed back
  harder than Gemini here: filesystem events don't carry edit ranges, so
  "true" incremental reparsing only pays off when an agent/editor supplies
  exact deltas; otherwise full-parsing the settled file is simpler and
  usually fast enough. The bigger win in both models' final view is avoiding
  whole-repo re-resolution, not intra-file parse speed.
- **What's missing from the original critique.** Both models independently
  flagged: no daemon-restart persistence story, vague "quarantine under
  ERROR subtrees" error handling instead of formal partiality states, and no
  agent-facing backpressure/staleness signal beyond internal queue metrics.
  GPT-5.5 Pro additionally pushed the revisioned-consistency-contract and
  semantic-environment-fingerprint framing harder than either the original
  critique or Gemini did — treated here as the single most load-bearing
  correction of the whole exercise.

## Risks

- Concurrent/in-flux edit correctness: an agent frequently leaves a repo in
  a transiently invalid state mid-multi-file-edit; any incremental graph
  computed during that window reflects a real-but-transient invalid state.
- Partial-parse error recovery is new, untested surface for every existing
  language adapter, all built against a "the file is finished being written"
  assumption.
- Memory growth is a previously observed cost, not a hypothetical — budget
  for it from day one rather than discovering it the way rust-analyzer did.
- The stack-graphs-style resolver refactor is the single largest
  implementation risk — reshapes how every language adapter emits facts;
  needs dedicated differential testing, not a confidence-based cutover.
- No empirical timing/memory data exists yet for AigisCode's own full
  pipeline at scale — any specific "N times faster than GitNexus/SocratiCode"
  claim before Phase 0's measurement step would be invented, not evidenced.

## Decision

AigisCode's differentiation is not "has a watcher" (SocratiCode already has
one) or "has incremental indexing" (GitNexus already has some). It is being
the first of the three that treats **freshness as an explicit, queryable
contract** rather than an implicit hope — an agent should never be able to
silently act on a stale graph, and should never be blocked behind a
synchronous full rebuild to get a fresh one.
