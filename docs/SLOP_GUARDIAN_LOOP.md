# The Guardian Loop — Using AigisCode + AI Agents Against Code Slop

> Operational companion to `SLOP_PATHOLOGY_MAP.md` (what is detectable) and
> `SLOP_ELIMINATION_ROADMAP.md` (what to build). This document describes how
> to run the loop **today**, with honest caveats where roadmap items are not
> yet landed, and how the loop tightens as they land.

## Why a Loop, Not a Scan

Slop is not a defect class; it is an entropy *rate*. A one-shot scan reports
mess; only a loop prevents new mess. ZEUS_SHIELD's four modes map onto a cycle
that runs per work session:

```
      ┌────────────► UNDERSTAND ─────────────┐
      │        (analyze: build graph truth)   │
      │                                       ▼
   CONVERGE                                ADVISE
 (policy/rules/doctrine:                (before writing:
  encode reviewed truth,                 what exists, what's
  system gets quieter)                   sanctioned, what's forbidden)
      ▲                                       │
      │                                       ▼
      └──────────────── GUARD ◄───────────────┘
              (after writing: did entropy increase?
               allow / warn / block + obligations)
```

The division of labor, which is the whole thesis:

| Layer | Job | Why this split |
|---|---|---|
| **Deterministic engine** (Rust) | Hold the global picture; generate candidates with evidence; measure deltas | Cheap, repeatable, no context window, no hallucination |
| **AI agents** | Adjudicate semantic equivalence, intent, fix execution | Judgment that resists mechanization |
| **Policy / rules / doctrine** | Persist verdicts so nothing is re-litigated | Converts AI+human judgment into permanent deterministic truth |

Each pathology root cause gets a specific counter:

| Root cause (pathology) | Counter |
|---|---|
| Partial file reads → reimplementing what exists | Graph + contract inventory served whole via MCP; `find_existing` query (S12) |
| No persistent symbol table between sessions | `.aigiscode/` artifact family **is** the persistent symbol table; `mcp --watch` keeps it live |
| Reward for action over restraint | Guard makes restraint visible: `Allow` verdict is a reportable success; new duplicate mechanism = `Warn/Block` with obligations |
| Context truncation losing early decisions | Doctrine registry + sanctioned mechanisms outlive every context window |
| Pattern-matching to training distribution | Convention/mechanism findings anchor agents to *this repo's* dominant patterns |

## Phase 1 — UNDERSTAND (session start)

```bash
aigiscode analyze /repo            # full artifact family → .aigiscode/
aigiscode agent /repo              # print the machine review contract
```

Agent reads, in priority order (never grep-reconstructs):

1. `aigiscode-report.json` — consolidated summary.
2. `repository-topology.json` — zones, runtime entries, recommended start
   slice, triage steps. The map the coding agent otherwise lacks.
3. `graph-packets.json` — bounded doctrine-aware neighborhoods; loads a
   working set without loading the repo.
4. `contract-inventory.json` — every declared route/hook/env/config key.
   **This is the anti-reimplementation index**: check before declaring
   anything runtime-shaped.
5. `doctrine-registry.json` — the law: sanctioned/forbidden mechanisms.

Cost note: run once per session; `mcp --watch` (Phase 3) keeps it current so
re-runs are not the agent's job.

## Phase 2 — ADVISE (before writing code)

The step AI coding agents skip because it is expensive for them and cheap for
the engine. Wire the MCP server into the coding agent:

```json
{ "mcpServers": { "aigiscode": {
    "command": "aigiscode", "args": ["mcp", "/repo", "--watch"] } } }
```

Pre-edit protocol for the coding agent (encode in CLAUDE.md/AGENTS.md of the
*target* repo):

1. **Placement** — `repository_topology` → which zone owns this concern.
2. **Prior art** — `repo_overview` contracts + `graph_neighbors` on candidate
   files: does a helper/route/hook/config key for this already exist?
   Today: navigate + inventory. After S12: one `find_existing` call.
   After S5: authoritative "use `App\Support\Http\Client`, adding a second
   client is a doctrine violation."
3. **Blast radius** — `graph_trace` between the files to be touched;
   `show_hotspots` — editing a high-centrality bottleneck ⇒ smaller, more
   careful change.
4. **Doctrine check** — `doctrine` resource: is the planned mechanism
   sanctioned? Is there a forbidden pattern adjacent?

This protocol directly attacks symptoms 1, 4, 13, 14 (duplication,
reinvention, dependency cancer, parallel mechanisms) at the only point they
are cheap to prevent — before the code exists.

## Phase 3 — Live picture mid-edit (`mcp --watch`)

The daemon re-analyzes on file change and atomically republishes; the
`Freshness` contract (`revision`/`indexed_revision`/`is_stale`/`dirty_paths`)
means an agent is never *silently* served a stale graph — it can pass
`consistency: wait_until_indexed` + `wait_ms` when it needs post-edit truth.

Honest caveats today (both are Tier-1 roadmap items):
- Freshness params are wired on `repo_overview` only; other tools return the
  latest snapshot without a staleness stamp (S2).
- `guard_decision`/`convergence_report` under `--watch` stay pinned to the
  last artifact-writing run — for a live verdict, re-run `aigiscode analyze`
  (S2 fixes).
- Rebuild is full re-analysis (Phase 1 of the online architecture): fine for
  small/medium repos, the incremental core (S16) is the large-repo enabler.

## Phase 4 — GUARD (after writing, before merging)

```bash
aigiscode analyze /repo    # rebuilds artifacts + convergence vs previous run
cat .aigiscode/guard-decision.json
```

Semantics today (be precise with agents about this):
- Verdict = regression vs the **previous analyze run in this output dir**:
  fingerprint set-diff (New/Worsened/Improved/Resolved) + per-smell count
  deltas + contract value deltas.
- `Block`: new/worsened high-severity security, new strong dependency cycle.
  `Warn`: duplicate mechanism, sanctioned-path bypass, abstraction sprawl,
  hand-rolled parsing, split identity, compatibility scar, complexity hotspot,
  contract deltas, radius pressure. Verdict = any-block → Block, else
  any-warn → Warn, else Allow.
- Each trigger carries doctrine refs + **obligations** (action + acceptance
  criteria) — feed these back to the coding agent as its punch list, and the
  `required_radius` files as its review set.

Session discipline that makes this diff-shaped *today* (until S1 lands):
run `analyze` at session start (baseline = pre-work state), then after the
change — the delta is then exactly "what this session introduced". Know the
first-run caveat: with no baseline everything classifies `New` (the artifact
marks `baseline_empty`).

CI shape:

```bash
aigiscode analyze . --output-dir "$CI_BASELINE_DIR"   # baseline persisted per branch
verdict=$(jq -r .verdict "$CI_BASELINE_DIR/guard-decision.json")
case "$verdict" in
  Block) exit 1 ;;                             # entropy gate failed
  Warn)  post_pr_comment_from guard-decision.json ;;  # triggers + obligations
  Allow) exit 0 ;;
esac
```

Hook shape (Claude Code, per-session gate): a `Stop`/pre-commit hook that runs
`analyze` against the session-start baseline and injects Warn/Block triggers
back into the conversation — the agent fixes its own entropy before claiming
done. This is the tightest available loop until S1+S2 make it per-edit.

## Phase 5 — Remediate the stock (spending down existing slop)

Prevention gates the flow; the stock needs paydown. Use the packet crawler so
cleanup stays bounded and evidence-backed instead of "refactor at will":

```bash
aigiscode agent-spider /repo --adapter codex-exec --limit 5
```

One report per top packet (duplicate-mechanism cluster, bypass site, hotspot…)
with graph traces and obligations. Effective per-symptom queues today:

| Slop symptom | Queue |
|---|---|
| Dead code | `deterministic-findings.json` dead-code findings, `Certain`/`Strong` tiers first — safe mechanical deletion |
| Config/constant duplication | hardwiring `RepeatedLiteral`/`EnvOutsideConfig` findings |
| Parallel mechanisms | `DuplicateMechanism` findings → consolidate to sanctioned family, then encode in doctrine (Phase 6) |
| Framework bypass | `SanctionedPathBypass` + ast-grep provenance clues |
| Reinvention | `HandRolledParsing` + stack detectors → replace with named library/native contract |
| Hotspots | `AlgorithmicComplexityHotspot` with caller-pressure paths |

After each bounded batch: `aigiscode analyze` → verify convergence shows
`Improved/Resolved`, not new `New`.

## Phase 6 — CONVERGE (encode verdicts, system gets quieter)

Every human/AI triage decision must land in a file, or it will be re-litigated
by the next context window:

- **False positive / accepted local pattern** → `.aigiscode/policy.json`
  (family-level patterns, thresholds) or `.aigiscode/rules.json` (narrow
  per-finding exclusions). `aigiscode tune` proposes conservative starters.
- **Confirmed pattern-to-follow** → `.aigiscode/doctrine.json` clause override;
  after S5, a `sanctioned_mechanisms` entry naming the concrete API.
- **Adjudicated duplicate/distinct verdicts** (after S8+S14) → persisted rules,
  so each AI judgment permanently upgrades deterministic precision.

Convergence is the metric that matters across sessions: visible-finding delta
trending negative + guard `Allow` streak = entropy rate beaten. That number —
not finding count — is what GOAL.md defines as success.

## Anti-Patterns (the loop's own failure modes)

1. **Suppression as slop** — bulk-adding policy patterns to silence the guard
   is the meta-pathology. Every suppression needs a reason; `rules.json`
   entries are narrow by design. Review policy diffs like code.
2. **Baseline gaming** — re-running `analyze` right before a check to flatten
   the delta. CI must own the baseline artifact dir, not the agent.
3. **Trusting heuristic tiers as proof** — `heuristic` findings gate at Warn
   for a reason; treat them as investigation prompts, not verdicts. The proof
   tiers exist so agents don't over-rotate on weak evidence.
4. **Running the loop only on big repos** — entropy compounds from the first
   duplicated helper. Cost of the loop on a small repo is seconds.

## Maturity Ladder (where any given repo stands)

| Level | Practice | Requires |
|---|---|---|
| 0 | One-shot `analyze` + human reads report | today |
| 1 | Session loop: baseline → work → guard, agent consumes obligations | today |
| 2 | MCP-wired agent with pre-edit ADVISE protocol + `--watch` | today |
| 3 | CI entropy gate on guard verdict + policy under review | today |
| 4 | Diff-scoped live guard per edit turn; `find_existing` before writing | S1, S2, S12 |
| 5 | Clone/dependency/outcome-path detectors + AI adjudication converging to quiet | S8, S9, S13, S14 |

Levels 0–3 need no new code — only adoption. That is the honest headline: the
majority of the pathology's *prevention* value is already shippable behavior;
the roadmap raises the *detection* ceiling.
