# Captured inputs and publication validation

The scanner records each admitted file's path, size and content hash. Both cold
parsing and fast graph loading read supported sources through the same helper:
the actual UTF-8 bytes must match that scanned size/hash before they can contribute
parsed facts or accompany a cached graph. A disappeared or changed source produces
`ProjectAnalysisError::InputChanged`; other read failures retain their IO error.
No analysis artifacts are published from that failed capture.

The watch indexer treats this particular error as its own observation of change,
advances the dirty revision and schedules a debounced fresh capture. It does not
depend on a filesystem event eventually arriving. The failed attempt remains
diagnostic evidence while the newer queued revision can wait normally. A CLI
invocation fails explicitly and can be rerun on stable inputs.

`SnapshotIdentity.input_inventory_fingerprint` uses the shared scan fingerprint
over every admitted path and content hash, including data files. The new
`semantic_env_fingerprint` records the declared root-level build/workspace inputs,
including known hidden configuration paths. Missing legacy environment identity
deserializes as empty and is insufficient for fast loading.

Fast loading requires matching root, engine, scope, complete admitted inventory,
semantic environment, resolver configuration and parsed-source manifests, plus the
existing cached graph checksum. Mismatch declines to ordinary analysis. This is
conservative: even a data-file edit that does not change graph meaning invalidates
the graph cache until finer dependency-specific reuse can be justified. Scope and
the candidate environment list define these inputs; this is not a fingerprint of
every file outside the configured analysis boundary or every process environment
variable. Fingerprints detect changes; they are not authentication.

The scan, semantic environment, resolver, policy and doctrine now share one
configuration reader for an analysis. It caches bytes or absence by normalized
absolute path; subsequent readers use the same captured value. Resolver package
directory checks and Python/Ruby source-root directory checks are captured too.
Their existence contributes to resolver identity, including empty directories.
The scan-scope fingerprint now includes `skip_hidden` explicitly.

Configuration that also appears in the admitted inventory must match the exact
inventoried size/content hash. Final validation rescans with the original caller's
base scan configuration and compares the effective scope, semantic environment,
all admitted paths, sizes and content hashes. It then compares every captured
configuration value/absence and directory predicate with the filesystem. Mtimes
are not identity. A detected change produces `InputChanged`; other reachable IO
or configuration errors remain explicit failures.

This validation runs before a graph-only result or full analysis returns, before
staged analysis artifacts become a published generation, and before MCP state or
an in-memory CLI agent context is returned. `VerifyInputs` reports analysis-stage
validation separately from `Analyze`/`LoadAnalysis`. Publication/MCP-context checks
remain part of their surrounding work. An artifact validation failure removes the
unpublished staging directory and leaves the previous generation selected. The
watch indexer also recognizes an `InputChanged` wrapped by artifact writing and
queues a fresh capture. Native-analysis restoration uses the same validation.

Artifact output must be outside the admitted input set (the default hidden
`.aigiscode` directory already is under default scan settings). If custom scan
settings admit the output files, their creation changes the inventory and fails
validation; configure an excluded output location rather than certifying a
self-changing input set.

Declared semantic configuration is either absent or a readable regular file.
Read/metadata failures and non-file inputs now propagate as `ScanError::SemanticConfig`,
rather than becoming a zero hash or an absent input. This fingerprints bytes; it
does not add syntax validation for every configuration format. The watcher uses
the same candidate list to register nearest existing parent directories, including
hidden parents such as `.cargo`, independently of source-directory exclusion.

Runtime plugins receive `RepoContext::new(root, &parsed_sources)`. WordPress hook
and signal callback expansion use those same borrowed bytes, including Python
receiver decorators. They perform no later source-file reads. A shared lazy line
index borrows each line without copying source strings; snippet bounds reject
zero/out-of-range lines and clamp context at EOF without integer overflow.
Native callers constructing a plugin context must supply the sources from their
graph capture. Missing paths produce no snippet; there is no filesystem fallback.
Semantic revision 12 invalidates graphs built under the earlier plugin contract.

These checks do not establish an atomic whole-repository snapshot. Each directory
walk and final comparison takes time; changes after an input's last check or a
change/revert wholly between checks may escape observation. Watcher freshness and
edit receipts still govern live revisions. The supplemental dead-code sweep and
external executables are not readers of this immutable configuration/source set;
their wider input and error coverage remains a separate limitation. Do not turn
validated admitted inputs into a claim that every downstream reader used an
atomic filesystem snapshot.

The [Draivix observation](2026-09-10-capture-identity.md) records stable analysis and
a real admitted JSON change. Race/error and hidden-configuration regressions have
been written but not run while automated tests and CI remain paused.
The [shared capture observation](2026-09-10-input-stability.md) records the later
configuration/publication changes and their bounded runtime evidence.
