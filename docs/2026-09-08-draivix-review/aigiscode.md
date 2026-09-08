# AigisCode dogfooding results

Companion to the [Draivix review](README.md). Product decisions follow `GOAL.md`
and `ZEUS_SHIELD.md`: preserve evidence, keep language behavior generic, and treat
repository policy separately from graph truth.

## Implemented and pushed

### Cross-language launchers and honest deletion confidence — `23a1832`

The initial self-scan classified `tools/kuzu_bridge.mjs` as `safe_delete`.
`rust/crates/aigiscore/src/kuzu_index.rs:24` names that script and the Rust command
path invokes Node with it. The frontend orphan detector only considered path
literals from frontend source and required a `./`, `../`, or `@/` prefix.

The existing path-evidence channel now reads all analyzed languages and recognizes
bare relative paths such as `tools/worker.mjs`. This suppresses an orphan candidate;
it does not fabricate a call edge. No dependency or language sidecar was added.

Frontend orphan findings now use `probably_delete`, retain heuristic proof tier,
and disclose excluded-caller/dynamic-entrypoint uncertainty. Their fingerprint and
artifact location remain unchanged. Consumers can still deserialize old string
verdicts; new output no longer gives a deletion guarantee it cannot establish.

Observed outcomes:

- Self-analysis: the live helper disappears from the orphan list, 1 → 0.
- Configured Draivix analysis: all 32 candidate fingerprints remain; eight
  frontend `safe_delete` verdicts become qualified candidates.
- Regression added: Rust and PHP launchers keep their JavaScript helpers alive,
  while a genuinely unreferenced sibling remains a heuristic candidate.

### Test consumers no longer establish production reachability — `648fb3d`

Broadening the source scan initially hid 23 of the configured scan's candidates.
Some were referenced only in tests, including assertions that retired files must
not exist. The out-of-slice sweep already excluded tests, but the analyzed graph,
literal/import channels, and convention-factory scan did not. Thus including a
test in the graph could invert a production orphan verdict.

The orphan passes now exclude test-file evidence consistently across resolved
edges, imports, literals, glob patterns, naming conventions, and dynamic factory
suffixes. The existing hardwiring test-path classifier was moved to the shared
detector module and reused. Private-function analysis and the semantic graph are
unchanged. Test source files themselves are not new orphan candidates.

Finding evidence now says **non-test** sources and explicitly requires reviewing
test callers before deletion. A test-only helper may be legitimate test support,
or a replacement waiting for integration; the detector does not decide that.

On the identical 17,748-source snapshot, orphan findings increased from 55 to 74;
total dead-code findings increased from 162 to 181. Fourteen of the previously
missing configured-scan candidates returned. The remaining nine were named in
generated PHPStan cache source, not production callers. No non-test inbound graph
edge was found for those nine. This is recorded rather than represented as complete
scope invariance.

After isolating generated cache/bundle copies outside the source root, the final
broader run covered 17,013 sources and retained **all 32** configured-scan
candidates. It reported 198 dead-code candidates, including 91 orphan modules;
the additional broad-corpus findings were not all individually reviewed. This
last comparison includes an explicit corpus change, not just the detector fix.

Regression added: parse and resolve a mixed PHP/TypeScript corpus both with and
without tests; test imports, a test glob, and a test-only dynamic factory must not
hide four unwired production files, while ordinary production consumers keep their
targets alive. The fixture also checks that the test→production import was actually
resolved, so it exercises the graph-evidence channel.

## What self-analysis and source inspection still reveal

The initial self-scan covered 102 source files, with 6,255 resolved edges, zero
strong SCCs, and two total SCCs. Zero strong SCCs is not an architectural clean bill.
Several important issues require further work:

1. **Receiver truth can still manufacture coupling.** In Draivix,
   `StructureApplier.php:114` calls `$operation->isNoop()` with no arguments. The
   emitted finding attributes it to `LocaleFileWriter::isNoop`, whose declaration
   at `LocaleFileWriter.php:109` requires two arguments. `FileOperation.php:37`
   declares the actual zero-argument operation method. The same wrong target
   appears at `StructureApplier.php:183`; the call at `:328` really does use the
   locale writer. In `resolve/mod.rs`, the arity filter is retained only if it
   leaves candidates, and imported candidates can survive without receiver proof.
   Fix receiver ownership with language-aware call evidence; do not blindly reject
   all arity mismatches without modeling variadics and unpacked arguments.

2. **Container evidence is reconstructed from a nearby text window.**
   `plugins/container.rs:148–150` joins five lines before regex extraction.
   A neighboring call can therefore supply the binding. Exact parser-owned call
   facts are the appropriate owning layer. This is already recorded in the
   separate trust-remediation work; this review did not claim to complete it.

3. **SCC membership is not a directed cycle witness.** The report may join stored
   component members with arrows. Sorted member order does not prove those edges.
   Preserve member sets and render only graph-verified witnesses. Likewise,
   TypeScript type imports need to remain distinguishable from runtime imports
   when advising about initialization cycles.

4. **Generated source can contaminate reachability.** The broad snapshot included
   `phpstan/resultCache.php` and `phpstan/cache/`, plus Android bundled assets.
   Cache entries mention source paths without executing them. An explicit parsing
   exclusion still left the supplementary sweep reading the caches. The generated
   copies were therefore moved outside the disposable source root for the final
   run. These paths were not hardcoded into core or written into Draivix policy.
   A general generated-source contract shared by parsing and supplementary
   reachability evidence remains a precision gap.

5. **Large Rust ownership units remain.** At the initial checkout, `artifacts.rs`
   had 9,129 lines, `assessment/mod.rs` 6,964, `agentic.rs` 5,558, and `mcp/mod.rs`
   4,000, including their embedded tests. In particular, artifact emission and
   substantial derived-report construction share one large module. Split along
   existing contracts when changing those responsibilities, rather than doing a
   bulk file-size refactor during a detector fix.

6. **The optional Kuzu path is still Node-backed.** This is existing implementation,
   not the native Rust end-to-end target. Hiding or deleting its live bridge would
   break the optional feature; migration needs an explicit product decision.

7. **Heuristic scores are not calibrated defect verdicts.** Per-row decoding,
   service-provider naming, and wide platform APIs all produced misleading review
   leads. Function-body clone detection and ownership-aware caller grouping would
   help with the actor-drift example; naming similarity alone would not prove it.

## Validation and delivery limits

`cargo build --release --bin aigiscode` succeeded. Rebuilt CLI executions completed
against AigisCode, the live configured Draivix tree, and the broader copied tree.
No local test suite, linter, type checker, or external security adapter was run.
Regression tests are written, **not reported as passing**.

Origin is `git@github.com:Draivix/aigiscode.git`; the repository has
`.github/workflows/ci.yml`, not a GitLab pipeline definition. The accessible GitLab
project search returned no AigisCode project. No new pipeline was added and no
GitHub test workflow was deliberately triggered as a substitute. GitLab validation
is outstanding. Changes are pushed on `fix/draivix-audit-precision`.

The separate trust-remediation worktree and `.plan/WORK_LOCK` remain untouched.
These two fixes do not close that larger contract.
