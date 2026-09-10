# Architectural review contract

The `2026-09-10` agent response schema replaces the free-text action and
model-authored Markdown with typed architectural decisions. Each claim names its
task packet, concern, conclusion, action, compared implementations, surviving
owner, consumer changes, preserved behavior, missing evidence and verification
steps. Behavior comparisons describe every implementation under the same input.

The adapter must echo `source_snapshot_id` from `agentic-review.json`. Native
validation checks this against the captured analysis, verifies required packet
coverage, validates source paths and symbol ownership, and matches quoted source
against inclusive line spans. It rechecks captured inputs before publication.
Unknown conclusions require `investigate` and explicit missing evidence.
Consolidation and migration require multiple implementations, a survivor,
behavior comparisons and consumer changes. Change proposals require preserved
behavior and verification steps.

The HTTP adapter receives bounded captured excerpts because it has no filesystem
tool. Excerpt boundaries and omissions are explicit. Missing context is grounds
for investigation, never proof of absence. The local adapter can read source;
the native validator still checks its answer against the captured inputs. Its
complete prompt is retained privately in `agent-input.txt` and supplied through
stdin, avoiding command-line length limits. The adapter catalog advertises only
the implemented native HTTP and local CLI paths; no sidecar or HTTP filesystem
tool support is implied.

`agent-review.raw.json` retains the unvalidated adapter response. Invalid or stale
responses do not replace a previously published review. The authoritative
`agent-review.json` is a native envelope containing `schema_version`, `review_id`,
`source_snapshot_id` inside `proposal`, and native packet-to-finding bindings.
The review ID covers the proposal and its bindings. It detects accidental
modification; it is not an authentication signature.

`agent-review.md` is generated from that record and displays the same review ID.
Each file is published atomically, with JSON first. If Markdown publication
fails, JSON remains authoritative; compare IDs before treating exports as a
pair. On Unix, the new review exports and raw-response file use private file
permissions because citations can contain source code. Structured responses and
published records are bounded to 4 MiB; an oversized record is rejected.

`review-surface.json` carries a source-anchored proposal, stale-review or invalid
record status. Proposals alone preserve native findings and their policy
disposition. Changed review scopes require another review. Invalid records are
reported explicitly. MCP overview includes the review identity, status and
proposal count; the full typed decisions remain in the review artifact.

Matching source references does not prove semantic equivalence, production
reachability, runtime correctness or business approval. The native Markdown
labels conclusions as proposals and states that runtime behavior was not
verified. Automated tests and CI remain paused; new regression cases are not
reported as executed verification.

## Native MCP review and repository adoption

`prepare_architectural_review` selects exact implementation names/IDs, native
finding IDs or parsed behavior-comparison IDs and returns bounded source excerpts,
execution-path evidence, a focused task packet and the typed output schema.
`submit_architectural_review` validates the response against that captured context
and publishes the native record. Neither call runs another model. Symbol-only
reviews do not implicitly bind unrelated findings in the same file.

Published records are retained in `architectural-reviews/<review_id>.json` under
the configured output directory, in addition to the latest `agent-review.json`
and native Markdown. All exports use the artifact owner and bounded private
publication. A failure exporting the latest Markdown can leave valid JSON already
published; JSON and its identity remain authoritative.

`adopt_architectural_decision` takes `review_id`, `claim_index`, an explicit
`finding_ids` list, `disposition`, `reason` and `apply` (default `false`). Preview
returns a concrete policy entry. `apply: true` adopts a repository-approved
conclusion into `.aigiscode/policy.json` under `reviewed_decisions`, preserving
other policy fields and serializing concurrent adoption. `--no-write` rejects
both review publication and policy adoption. Adoption invalidates the live MCP
snapshot until reindexing; it never serves the old policy as fresh.

- `accepted_pattern` requires an explicit keep decision for a justified variation,
  runtime entry or test/support contract. Only the selected finding fingerprints
  become accepted; raw native findings remain available.
- `source_confirmed_concern` records an approved source-supported violation,
  incomplete migration or scoped unreachable candidate. It stays visible even
  when a broader policy would suppress the finding, and receives a focused agent
  packet. Confirmed security remains ahead of general simplification.
- An empty `finding_ids` list records architectural intent without suppressing any
  diagnostic. AI proposals alone never acquire an adopted disposition.

`tune` returns valid proposal drafts in its `suggestions` output, with the approval
requirement and no automatically selected finding fingerprints. These drafts are
excluded from `suggested_policy`; use the MCP adoption preview to bind the exact
findings. Custom output directories select the corresponding latest review.
Stale or invalid reviews produce explicit diagnostics instead of adoptable drafts.

## Scope, drift and acceptance

A review scope fingerprints captured implementations, consumers and quoted files,
their resolved one-hop callers/dependencies, relevant runtime registrations,
analysis boundary, engine, resolver, doctrine, coverage and non-review policy
configuration. Changed source or registration evidence makes the adopted decision
stale and reopens its affected findings. Unrelated source outside the neighborhood
and adoption metadata alone do not invalidate a decision. This is a bounded source
scope, not proof that all dynamic execution paths are known.

Convergence keeps repository review counts and IDs of unresolved source-confirmed
concerns whose original native observation was new/worsened against a verified
comparable baseline. Each policy entry retains that baseline identity and delta;
repeated unchanged scans do not erase the unresolved concern. First observations
and incomparable or incomplete baselines do not establish new confirmed drift.
Stale decisions require re-review, rather than becoming evidence of a runtime fix.

Policy visibility changes alone are not improvements. Guard regressions require
visible native findings as well as comparable graph deltas; accepted patterns do
not leak back into handoff action packets. Source-confirmed concerns and stale
review scopes remain part of the required investigation radius. The consolidated
JSON, native Markdown and MCP convergence contract report this distinction.

An `unreachable_within_scope` proposal additionally requires a selected native
local-binding or private-dispatch finding, complete evidence for that scope, and
a citation at the candidate declaration. Zero callers or a module-level orphan
signal cannot justify the same conclusion. Source-supported proposals may include
pending runtime checks; missing evidence essential to the conclusion requires
`unknown` / `investigate`.
