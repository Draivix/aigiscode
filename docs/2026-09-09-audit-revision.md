# Audit trust and scale revision — September 9, 2026

This revision fixes source identity and evidence defects exposed by auditing the
preserved Draivix corpus. It is a step toward the acceptance contract in
[AUDIT_QUALITY_CONTRACT.md](AUDIT_QUALITY_CONTRACT.md), not a verified market ranking.
Draivix itself was not edited.

## Delivered behavior

- **Cycles:** each component has an ordered, closed `directed_witness` of real edges
  with source locations. Strong-cycle classification uses the strong view. Edge
  partitioning replaces repeated whole-graph scans per component. Component members
  are no longer rendered as if alphabetical order were a call chain.
- **Receiver ownership:** an unknown explicit receiver cannot acquire methods from
  unrelated imports or classes in its file. PHP imports use declared qualified
  identity, including aliases and namespace blocks. Same-line imports retain their
  own reference identity.
- **Container targets:** the PHP parser supplies an exact first positional class
  literal for each call. Container resolution no longer reads nearby source lines
  or guesses a target from a class basename or directory. Grouped class imports,
  namespace blocks, and trait/interface/enum members preserve identity.
- **Dependency meaning:** TypeScript type-only imports and re-exports remain in the
  full graph but cannot close strong cycles. Production references to targets that
  match test-source conventions retain explicit uncertainty instead of becoming
  hard dependencies through test stubs. Ordinary names such as `Contest.php` do not
  match the PHP test suffix convention.
- **Scope:** explicit `generated_path_prefixes` exclude generated files from both
  parsing and supplemental orphan discovery. Scope is exposed in the architecture
  overview. Supplemental scans skip symlinks and special files. Exclusion prefixes
  reject paths that escape the repository.
- **Cache and publication:** semantic revision 6 invalidates older graph semantics.
  The manifest binds the exact serialized graph through `semantic_graph_xxh3`;
  fast load decodes and hashes one buffered stream. Individual JSON and Markdown
  artifacts use temporary files and rename, preserving old files on write failure.

Implementation commits: `8229f15`, `36a7f75`, `ef21719`, `84698bd`.

## Fixed-corpus measurement

The baseline (`e3ce102`) and revised engine (`ef21719`, semantic revision 6) each
analyzed the same preserved 17,013 supported sources with release binaries. The
source-manifest array has the same SHA-256 in both runs:
`74a49f531eca27773944f9bedd833652c7c84859ecf7dde84e9b9110acd2afb0`.
This fingerprints the ordered path/xxh3 records, not a cryptographic source archive.
Generated copies were physically outside the snapshot for both measurements so
the baseline, which predates explicit generated exclusions, saw identical inputs.

| Metric | Baseline | Revised |
| --- | ---: | ---: |
| End-to-end wall time | 214.68 s | 79.90 s |
| Peak RSS | 3,498,788 KiB | 3,471,724 KiB |
| Resolve phase | 145,616 ms | 16,249 ms |
| Analyze phase | 39,314 ms | 33,520 ms |
| Artifact bytes | 1,714,653,376 | 1,685,767,571 |
| Symbols | 123,380 | 124,053 |
| Resolved edges | 343,619 | 323,371 |
| Strong cyclic components | 17 | 18 |

This is one sequential pair on a shared host with warm filesystem caches and
default parallelism, measured with `/usr/bin/time -v`; it is not a statistically
controlled benchmark. Observed elapsed time improved 2.69×, while peak memory
remains approximately 3.3 GiB. Reduced namespace/container search work dominates
the measured resolution improvement. Artifact volume remains large.

Raw summaries, timing logs, pinned binaries, and artifacts are retained locally in
`target/audit-trust-2026-09-09/`; [compact receipts](2026-09-09-audit-revision-evidence.json) accompany this document. To repeat
the comparison, build each revision in a separate worktree **and separate Cargo
target directory**, preserve the same source snapshot, and run:

```sh
/usr/bin/time -v /path/to/baseline/aigiscode analyze /path/to/snapshot --output-dir /path/to/baseline-output
/usr/bin/time -v /path/to/revised/aigiscode analyze /path/to/snapshot --output-dir /path/to/revised-output
```

## Verification and release limits

Artifact inspection found 24 full-view and 18 strong-view components. All 97
witness steps belong to the emitted dependency projection, all paths close, and
no strong witness uses an inferred or type-only edge. The largest strong
component shrank from 105 to 46 files after source identity and test-stub repairs.
This does not establish that every remaining dependency is correct.

The `StructureApplier.php:114` and `:183` false `LocaleFileWriter::isNoop` bindings
are absent; the genuine `:328` call remains. The TypeScript
`entityFormClientExtensionLoader`/`entityFormExtensionRegistry` pair remains a
full-view component but is absent from the strong view. Earlier source review
and generated-input scope verification retained all 32 reviewed dead-code
candidates; their deletion still requires the qualifications in the original
ledger.

A real stdio MCP initialization and `repo_overview` call successfully loaded the
revision-6 cached graph, reporting 17,013 files, 124,053 symbols, and 323,371 edges.
The daemon explicitly confirmed that Parse+Resolve were skipped. Startup through
the first overview response took 69.65 seconds; it still decodes the large graph
and recomputes analysis. This is an API smoke observation, not a cold/warm speed
comparison. That observation also exposed missing scope/witness fields in MCP's
projection, which were subsequently added to `repo_overview` and `show_cycles`.
The final binary (`84698bd`) was then queried again: both tools succeeded in
68.08 seconds from process start, all three generated prefixes were visible,
and all 18 strong plus 24 full-view cycle witnesses arrived as closed paths.
This repeated API observation verifies the changed projection; the performance
pair above remains tied to `ef21719`.

Production Rust release builds succeeded. Automated regressions were added for
identity, source scope, type imports, cycle evidence, cache mismatch, and artifact
failure behavior. **They have not run.** Local suites and quality gates are
prohibited by the workstation contract. The repository exposes GitHub CI, while
the contract requires GitLab CI; no accessible AigisCode GitLab pipeline was found.
Approval to use the existing GitHub workflow is pending. Build and corpus evidence
do not replace these gates.

Dynamic receiver types remain incomplete: the Draivix `StructureApplier` foreach
receiver no longer resolves to an unrelated `LocaleFileWriter::isNoop`, but its
correct `FileOperation` ownership is not yet inferred. Named container arguments
are conservative and unsupported by this new literal fact. Test-path conventions
are uncertainty signals, not a complete framework source-set model.

Publication is atomic per file, not across the entire artifact family. The graph
fingerprint checks coherence, not malicious rewriting of both graph and manifest.
No power-loss durability claim is made for the parent directory. The separate
trust-remediation work lock remains open.

No shared labeled competitor corpus has been run. Fewer edges or smaller strongly
connected components alone do not prove better precision; the source-backed cases
and explicit unresolved evidence are what justify these changes.
