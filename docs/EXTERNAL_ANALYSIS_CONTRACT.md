# External analysis integrity

External checks distinguish `not_requested`, `complete`, and `incomplete` in
`external_checks`. Completion means the requested tools returned usable reports;
it does not assert that every vulnerability or every dependency was covered.
Legacy artifacts without this summary retain an absent value, not a fabricated
complete result.

A failed/unavailable requested tool prevents a clean audit conclusion. The CLI
publishes available artifacts and exits nonzero. The guard emits an explicit
incomplete-check block, with a rerun obligation rather than claiming a source-code
defect. Valid findings from an otherwise incomplete report remain available.

Tool records preserve status, exit code, optional failure kind, raw stdout/stderr
paths, and a bounded stderr preview. The failure kinds distinguish invalid report,
process exit, process I/O, timeout, and unsupported tool selection. Raw output is
never replaced with a fabricated empty JSON document.

## Process ownership

Stdout and stderr stream directly into separate files. Normalization reads the
report afterward; JSONL reports are consumed line by line. This removes the
undrained-pipe deadlock and avoids retaining both complete streams in memory.

On Unix, each invocation receives its own process group. `waitid(WNOWAIT)` observes
leader completion without freeing its PID; remaining group members are terminated
before the leader is reaped. Timeout and error paths also own cleanup. The existing
transitive `libc` crate is now a direct Unix dependency because std exposes neither
this wait mode nor process-group termination. This is process-group ownership,
not a sandbox against a malicious scanner deliberately escaping its group.

Non-Unix execution currently reports `process_scope: direct_child`; it does not
claim equivalent descendant cleanup. Raw stream creation uses `create_new`; Unix
capture files are owner-only. Project and output paths are resolved before the
child changes working directory.

## Adapter semantics

- Ruff and Gitleaks request zero exit status on findings; abnormal exits still
  fail the run. See [Ruff exit codes](https://docs.astral.sh/ruff/linter/#exit-codes)
  and [Gitleaks flags](https://github.com/gitleaks/gitleaks/blob/master/README.md).
- SARIF parsing rejects malformed/empty documents, missing runs, unsupported
  versions, failed invocations, error notifications, and unmaterialized external
  fragments. A valid empty findings list and a malformed report are different
  outcomes.
- Cargo Clippy requires a successful `build-finished` record. cargo-deny JSON
  diagnostics and logs come from stderr, including operational errors. See
  [cargo-deny output](https://embarkstudios.github.io/cargo-deny/cli/check.html)
  and [its JSON logger](https://github.com/EmbarkStudios/cargo-deny/blob/main/src/cargo-deny/main.rs).
- Composer audits the lock file with plugins and scripts disabled. Both manifests
  must exist. Normalization includes advisories, abandoned packages, and dependency
  policy/filter entries; unreachable advisory repositories make coverage
  incomplete. See [Composer audit](https://getcomposer.org/doc/03-cli.md#audit)
  and [the report producer](https://github.com/composer/composer/blob/main/src/Composer/Advisory/Auditor.php).
- OSV-Scanner uses `--output-file`; no packages found is not a clean dependency
  audit. See [usage](https://google.github.io/osv-scanner/usage/)
  and [return codes](https://google.github.io/osv-scanner/output/#return-codes).
- pip-audit uses strict dependency collection; npm reports require the supported
  vulnerabilities-object shape. Valid result exit codes are accepted only with
  findings. See [pip-audit](https://pypi.org/project/pip-audit/)
  and [npm audit](https://docs.npmjs.com/cli/v9/commands/npm-audit/).

## Verification state

The production Rust binary builds with these changes. The last completed CI run
before this unit passed 408 tests at commit `2721fcb`. The user subsequently
stopped test execution; this unit has not run in CI. Process regressions written
before that instruction remain in the tree, unexecuted. No external scanner or
local test suite was run to present this unit as verified. The broader reliability
goal remains open.
