# Saved-edit receipts and change verification

Filesystem notifications are asynchronous. An immediate read after a save can
precede notification delivery and still describe the previous index as current
relative to observed events. Agents that know they saved changes should establish
an explicit revision floor through `record_changed_paths` under `mcp --watch`.

1. Save the whole edit batch through the agent's editing tools.
2. Call `record_changed_paths` with the saved repository-relative paths.
3. Pass the returned `paths` and `min_revision` to `verify_change`, with a suitable
   `wait_ms`, or use that floor with `repo_overview` and `wait_until_indexed`.
4. Inspect freshness, input coverage and baseline comparability before interpreting
   the returned counts. A receipt acknowledges invalidation, not completed analysis.

The receipt is available during initial indexing. It validates the whole path batch,
advances the observed revision and wakes the same single writer used by filesystem
events. A writer already processing an earlier target cannot publish that target
as satisfying the new receipt; a subsequent capture must reach the requested floor.
The writer's revision bookkeeping remains authoritative even if wakeups coalesce.

Receipts apply only to the running server instance. They are not durable edit
transactions, saved-content hashes or proofs that the listed paths are admitted by
scan policy. They do not change files, expand scan scope or suppress intermediate
captures during a multi-file save. Call after saving, not before it. Concurrent
external edits and failures still require the ordinary freshness/coverage checks.
The planned begin/end edit-epoch transaction protocol is not implemented here.

`paths` accepts one to 128 entries, each nonblank, NUL-free and at most 4096 UTF-8
bytes. Absolute paths, drive prefixes and parent traversal are rejected. Portable
backslash separators are normalized; duplicate normalized paths are removed.
New files, deletions and directories are valid without an existence check. Use
`.` to invalidate the repository as a whole, including when an external semantic
configuration input changed. Invalid input changes no revision or dirty-path state.
A one-shot server or an unavailable index writer rejects the receipt.

Tool-call parameter objects, assuming the receipt returned revision 7:

```jsonl
{"name":"record_changed_paths","arguments":{"paths":["app/Services/Example.php"]}}
{"name":"verify_change","arguments":{"paths":["app/Services/Example.php"],"min_revision":7,"wait_ms":120000}}
```

Use the actual returned revision rather than guessing the next number: filesystem
events and other agents can advance it too. `verify_change` waits at the shared
request boundary and pins one snapshot. Its wait defaults to zero once an index
exists, or 30 seconds while the first index is pending; the maximum is 120 seconds.
If no index exists when the wait expires, the response has explicit `indexing` or
`failed` state. If an older index exists, its delta carries unsatisfied/stale
freshness and an explicit caveat that post-edit verification is incomplete.
The response does not silently convert a timeout into a successful verification.

Analysis failures are associated with the revision of the failed attempt. A newer
queued receipt or an active retry remains eligible to wait; an older failure does
not make that request permanently non-retryable. The previous error stays visible
in freshness until a successful publication. An idle failed attempt with no newer
observations still returns failure promptly.

`verify_change.baseline` identifies the artifact baseline actually used. It is not
necessarily the previous watch revision. Without a comparable baseline, zero
regressions/fixes do not establish an unchanged or improved codebase. Incomplete
parser or secondary coverage also remains visible through MCP metadata and guard.

Scope matching respects path-component boundaries, so `src/main` is not a substring
selector for `src/main.rs`. Directory/ancestor relationships remain applicable.
An empty explicit scope uses all currently pending dirty paths, not just their
50-path freshness preview. The response displays at most 50 scope paths but reports
the complete `scope_path_count` and sets `truncated` when any returned list is capped.
After indexing, dirty paths may be cleared; pass the receipt's paths explicitly to
verify the edit batch instead of relying on that transient fallback.

The [Draivix observation](2026-09-10-edit-receipts.md) records actual startup and
post-save behavior. Automated regression and adversarial CI remain paused under
the user's instruction; this contract does not claim those gates have passed.
