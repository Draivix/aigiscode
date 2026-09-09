# Native Cypher and immutable query exports

Six real MCP Cypher queries return aggregate counts matching the saved Draivix
dependency graph: 129,375 nodes, 253,735 aggregated relationships and 325,573
relationship occurrences. The class census, four source-anchored dependencies and a bounded
class/method distinction also match. The [machine receipt](2026-09-09-native-cypher-evidence.json)
records the binary, database, input identity, actual responses and timing logs.

| Observed operation | Wall time / response latency | Peak process RSS |
| --- | --- | --- |
| Self CLI, only Git available on PATH | 0.77 s | 187,572 KiB |
| Draivix CLI, first database construction | 34.70 s | 2,331,816 KiB |
| Draivix CLI, graph reconstruction and database reuse | 31.32 s | 2,285,640 KiB |
| MCP, first useful overview from source | 76.44 s | 3,544,052 KiB over the session |
| Six subsequent Cypher requests | 34.91–91.22 ms each | Included in session measurement |

These are individual observations on a shared host. The first Draivix CLI run
briefly overlapped a self-reuse query; they are not isolated benchmarks. CLI and
MCP selected the same 95,363,072-byte database generation. The method query finds
`normalizeConfig` in both normalizers; the equivalent class query over those same
two files returns zero rows and retains the `name`/`path` columns. The two real
calls to `CanonicalFieldAliasResolver::resolve` retain lines 98 and 180, alongside
their imports at lines 9 and 8. `SUM(occurrenceCount)` returns Kuzu INT128 values,
so its JSON totals are decimal strings, including the aggregate `325573`.

All 23,166 captured files still match their saved SHA-256 inventory. At the final
comparison, the live original tree differs in 53 captured files, all under
`clients/mitel/`; HEAD remains `95276a526f6c9e684f81ea4b83639894e2f02c38`.
This work only read that original tree. The observations above refer to the
unchanged capture, not acceptance of the subsequently changed live tree. Both
CLI and MCP still expose incomplete native input coverage, and MCP also exposes
the 18 oversized / 792 unsupported-Vue secondary-scanner gaps.

The optional Cypher path now uses the official [Kuzu Rust bindings](https://docs.rs/kuzu/0.11.3/kuzu/) to the native
C++ engine. The Node bridge and sibling-checkout lookup are removed. Database
publication belongs to `artifacts/kuzu.rs`; `kuzu_index/native.rs` owns query
execution and value conversion, while the existing normalized graph projection
remains shared with JSON exports.

CLI queries reconstruct the semantic graph from current source, scan settings,
resolver settings and runtime plugins before considering reuse. The cache key
compares the exact normalized node/relation CSV contents, canonical source root
and engine fingerprint. A configuration change that leaves the projected graph
identical can reuse that graph. This is graph equality, not an assertion that
reports, policies or convergence baselines are identical. It is also not
incremental parsing or an atomic filesystem snapshot during concurrent edits.

A writer holds an OS lock, builds a separate database, checkpoints and closes it,
records its size/content hash, then atomically replaces `kuzu-current.json`.
Queries verify the immutable generation's database against its manifest and open
it read-only. Failed construction preserves the previously published database.
MCP holds a generation path for its indexed snapshot; another publisher cannot
replace that file through this API. Blocking native queries run on a blocking
worker, and their results include the exact database path and input coverage.

The export path is now `<output>/.kuzu-generations/<id>/graph.kuzu`. The old root
`graph.kuzu` remains untouched and is never reused. Existing readers that assumed
the old path must use the returned path. Query exports are separate from the
20-member analytical artifact family. Old generations are retained because
readers can hold their paths outside the publishing process. There is no automatic
pruning. Hashes detect content mismatch; they do not authenticate hostile edits.
Unix directory syncing is implemented; Windows and power-loss behavior have not
been exercised here. Read-only database access is not a Cypher filesystem sandbox.

The CLI releases its in-memory graph before querying. CSV emission streams rows
rather than retaining another entire CSV string. Each database instance uses a
512 MiB buffer pool limit and four execution threads. These are engine settings,
not a bound on the entire process: parsing, resolution and result materialization
have additional memory costs.

Query results preserve empty column schemas and reject multiple statements,
duplicate column aliases and non-finite JSON numbers. INT128/DECIMAL values use
strings; temporal values use the binding's string representation; INTERVAL uses
decimal-string nanoseconds. BLOB is a byte array, MAP is an array of key/value
objects, and node/relationship properties remain nested alongside identity
metadata. Upstream Rust iterator conversion panics become explicit query errors.

Builds require CMake and a C++20 compiler. Kuzu 0.11.3 pins `cxx` to 1.0.138 but
allows newer `cxx-build` versions. The first native link exposed incompatible
bridge symbols from cxx-build 1.0.200. The crate now explicitly pins the generator
to 1.0.138, including for unlocked dependency resolution; native extension symbols
are exported by the final executable. This is an upstream build compatibility
constraint, not a new runtime service.

Regression cases cover source-change cache invalidation through the CLI, invalid
source after a valid cache, reuse of an identical graph, retained older pins,
legacy preservation, malformed manifests, database corruption, failed relation
import, path quoting and query result semantics. They are written for approved
CI and have not been run. Automated tests and CI remain disabled under David's
instruction. Q11–Q12 still require incremental processing, scale/concurrency
measurements, remaining ownership work and approved regression gates.
The actual API observations above do not establish interruption recovery,
concurrent publication, all supported query value types, or universal graph
correctness. The overall goal and the separate work lock remain open.
