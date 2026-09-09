# Analytical artifact generations

A full analytical run writes its 20 native artifacts into a fresh directory under
`<output-root>/.generations/<id>/`. It seals every member's size and xxh3 hash in
`generation-manifest.json`, then atomically replaces
`<output-root>/current-generation.json`. That marker is the commit point.
No reader following this protocol sees an unfinished generation.

`analyze` returns paths within its committed generation. `info` selects the marker
once and reports both `artifact_generation` and that generation's directory.
Native baseline and fast-load readers use the same pinning contract. A caller
reading several files must keep the selected directory rather than resolving the
current marker again between files.

## Publication and retention

- An OS file lock serializes publishers to one output root. It is held through
  baseline selection, generation writing, compatibility refresh and commit.
  Native readers of existing generations do not hold this lock.
- The generation directory is private while being assembled. Existing regular
  artifact-file permissions are preserved before exposing the completed directory.
- All analytical members must exist before commit. The completion manifest covers
  reports, graphs, scanner output, doctrine, guard and agent context, including
  `scan-manifest.json`; it is not limited to the previous three-file baseline seal.
- Files are flushed before publication; directory metadata is also synced on Unix.
  After the pointer rename, a sync failure may report failure but must not delete
  the generation that a reader could already have selected.
- A failed uncommitted writer cleans up its own generation. Abrupt process death
  can leave an unreferenced staging directory, which readers ignore. Existing
  committed generations are retained so an earlier pin remains valid.
- No automatic generation pruning is provided. Removing a generation can invalidate
  external readers holding its paths; retention must account for those consumers.

Version 1 covers the fixed 20-member analytical family. A changed required member
inventory needs an explicit publication-version/migration decision. Kuzu databases,
agent execution results and policy suggestions are separate outputs. External raw
reports already use unique run directories and remain referenced from normalized
external analysis; they are not rewritten as shared flat raw-report files.

## Flat paths and legacy migration

The documented flat artifact paths are preserved as independent compatibility
copies, updated with per-file atomic writes before the commit marker. They are
useful for a standalone report or older sequential workflows, but they do not
provide a coherent multi-file view during publication. Copies, rather than hard
links, prevent a compatibility-file edit from altering a committed member.

Legacy directories without a generation marker remain readable with explicitly
absent generation identity. Existing baseline hash checks still apply. These old
flat directories do not gain a retrospective family-atomicity guarantee. During
an unfinished first publication, readers report that no complete generation has
been committed; a writing analysis can recover and publish a complete family.

Scan, policy, rules and doctrine configuration stay under `<project>/.aigiscode/`,
independently of the selected output root. Passing a
published generation as the output root of another full publication is rejected,
and native atomic member writers refuse to modify a sealed generation. Do not edit
generation members manually. Corrupt or unsupported commit records, traversal IDs,
symlinked generation members, missing files and checksum mismatches are explicit
errors; readers never silently substitute the flat copies for a damaged commit.

## MCP index and stored artifacts

MCP pins one disk generation before cache and baseline reads. Every tool/resource
response also carries `_meta["aigiscode/artifact_generation"]` with its directory,
generation ID and `inputs_match_index`:

- `true`: captured source, scan inventory, effective policy/rules, doctrine,
  resolver, engine and requested external-check identity match the served index;
- `false`: the disk generation describes different captured inputs;
- `null`: no generation identity is available, including legacy output.

A `--no-write` server may have a fresh in-memory index and older disk artifacts.
Use its in-memory tools/resources for current derived results. Matching inputs
does not mean identical report timestamps, Git diff context, convergence history
or baseline-dependent decisions. This status is separate from live freshness and
from native/secondary/external analysis completeness.

Input inventory and assessment-configuration fingerprints use already captured
scan and parsed configuration values. They do not re-read configuration while
serializing the result. Hashes detect consistency errors; they are not signatures
or authentication against someone able to rewrite the store.

A complete publication can contain an explicitly incomplete audit. Continue to
inspect `input_coverage`, `ast_grep_coverage`, external tool status and the guard;
publication alone is not an approval or security verdict.

The `fs4` dependency supplies cross-platform OS locks while retaining Rust 1.88
support; its synchronous API documents an MSRV of 1.75.
See the [maintainer's API documentation](https://docs.rs/fs4/latest/fs4/).
Production observations for this revision are on Linux. Windows behavior,
power-loss durability and adversarial interruption/concurrency scenarios still
require approved CI verification; added regressions are not a claim that they ran.
