# Backend orphan evidence coverage

`DeadCodeResult.backend_orphan_coverage` records the supplemental PHP orphan
check independently of parser and secondary-scanner coverage. Its statuses are
`complete`, `incomplete`, `deferred_input_coverage`, `deferred_boundary`,
`not_applicable` and legacy/default `unknown`. It includes scanned file/byte
counts, total gaps, at most 20 gap previews and an input fingerprint.

The existing sweep's scope is preserved: eligible PHP/JSON/YAML/XML/NEON/INI/
shell/env/TS/JS text below the selected root, excluding parsed files, test paths,
hidden/dependency/build directories, generated prefixes, links and special files.
This is a declared lexical backstop, not every possible runtime caller.

Directory, entry, file-type and file-read failures, missing root, invalid UTF-8
and inputs over 1 MiB are explicit gaps. Reads are bounded to the limit plus one
byte, including growth after metadata lookup. Unix opens reject a final symlink
and use nonblocking mode before checking the opened file's regular-file type;
other platforms retain the ordinary file/type checks.

Any gap defers all backend orphan candidates while retaining independently
derived private-function, import and frontend findings. A truncated analysis
boundary also defers backend orphan candidates even if the selected supplemental
sweep has no gap. Incomplete native input coverage still defers all absence-based
checks before this sweep. No PHP input makes this backend check inapplicable.

The fingerprint describes successfully read content and observed gaps. It is
part of snapshot identity and is rechecked through the same bounded traversal at
analysis completion, artifact publication and MCP-state construction. A mismatch
returns the existing `InputChanged` error and uses the existing watcher retry
path. Fast-load still recomputes dead-code analysis before reusing native findings.
No new artifact path or dependency is introduced.

CLI/report summaries, architecture surface, convergence, MCP overview/coverage/
quality and MCP response metadata carry this status. Deferred, incomplete or
unknown evidence prevents a clean backend-orphan conclusion and a clean audit
exit; guard blocks with a missing-evidence obligation, and baseline comparison
records incomplete supplemental checks. Zero findings in a deferred check is
not proof that no orphan exists. This contract does not change the classification
of frontend orphan candidates at truncated boundaries.

Final rechecking is not an atomic filesystem snapshot. Changes between checks,
external executables and callers outside the declared scope remain limitations.
The detector's candidates remain heuristic even when this sweep is complete.
The new gap/boundary regression and updated real-filesystem orphan regressions
were written but not run or compiled as test targets; tests and CI remain paused.
