# Input identity before parsing and fast loading

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

These checks do not establish an atomic whole-repository snapshot. Configuration
changes later in the pipeline, new paths introduced after scanning and final
capture stability still require further work. Do not turn captured parser/plugin
bytes into a claim that every downstream reader used one immutable input set.

The [Draivix observation](2026-09-10-capture-identity.md) records stable analysis and
a real admitted JSON change. Race/error and hidden-configuration regressions have
been written but not run while automated tests and CI remain paused.
