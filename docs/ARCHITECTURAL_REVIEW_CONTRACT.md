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
record status. Source-reviewed findings remain deterministic findings and retain
their policy disposition. Changed inputs require another review. Invalid records
are reported explicitly. MCP overview includes the review identity, status and
proposal count; the full typed decisions remain in the review artifact.

Matching source references does not prove semantic equivalence, production
reachability, runtime correctness or business approval. The native Markdown
labels conclusions as proposals and states that runtime behavior was not
verified. Automated tests and CI remain paused; new regression cases are not
reported as executed verification.
