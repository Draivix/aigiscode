# Native resolver reuse and MCP graph memory

The native resolver now borrows candidate definitions instead of copying their
strings for every lookup. On the saved Draivix corpus its observed resolution
phase changed from 17.54 to 11.91 seconds, and the entire 752 MiB semantic graph
remained byte-identical by SHA-256. This is a paired observation on a shared host;
the earlier run enabled diagnostic tracing. It is not a statistical benchmark.

An opt-in, process-local resolver cache now reuses native results per
reference-bearing file. Start it with:

```sh
AIGISCORE_INCREMENTAL_RESOLVE=1 aigiscode mcp /repo --watch --no-write
```

`repo_overview.resolution_work` reports whether the complete lookup context
changed and how many files/references were processed or reused. Processing counts
resolution attempts, including unresolved sites; it does not count successful
bindings. These counters describe the indexed snapshot returned by that request,
including when the caller explicitly requests a stale snapshot.

The cache fingerprints every map/set in the freshly rebuilt resolution context
and the complete ordered reference sequence of each file. Any context change
invalidates the entire cache, including negative lookups. This is deliberately
conservative: import-reference positions are part of the context, so even some
location-only changes trigger full resolution. A stable context permits reuse
only for files with identical reference inputs. Sparse reference offsets and a
final ordering step preserve batch edge order and same-line occurrence identity.
New context fields require an explicit fingerprint update through exhaustive
struct destructuring. These are in-process non-cryptographic fingerprints, not
an authentication mechanism or durable cache format.

Only native results before graph normalization, parser-recovery downgrades and
runtime plugins are cached. Normalization and override discovery run again, as
do runtime plugins, parsing, scanning, detectors, assessment and report assembly.
A failed downstream assessment does not publish its snapshot; a panicked worker
drops its cache. File removal or loss of all references removes the cached entry.
There is no tree-sitter edit reuse or dependency-specific symbol invalidation yet.

MCP also stops retaining complete dependency and evidence copies alongside its
semantic graph. Those two resource views are built from the request's pinned
semantic graph when requested, on a blocking worker. Their URIs and complete
contents are preserved. Full-resource reads now do more work and still materialize
large JSON strings; they are not streaming server responses or bounded payloads.
This change reduces retained duplication rather than imposing silent output caps.

## Real source refresh observations

The observations use two separate working copies of the preserved 23,166-file
Draivix capture. The original Draivix checkout and the earlier reference capture
are only read. Each working copy starts from that capture and then receives the
same two captured real changes from `clients/mitel`:

- `Entities/Account/Account.fields.php`: adds a presentation setting without
  changing native reference inputs. All 1,086,615 reference results are reused.
- `Entities/ServiceTrip/ServiceTrip.hooks.php`: introduces imports and executable
  hook logic. The complete context changes, and all 1,086,643 references are
  processed again.

These are controlled source-refresh states, not claims that either copy is the
complete current live checkout. The live checkout had 53 changed captured paths
at the preceding observation; only the two named changes are replayed here.
Native and secondary input coverage remain incomplete.

Both runs first return the prior revision with `is_stale=true` when requested,
then return a fresh indexed revision after rebuilding. Four filesystem events
were observed per refresh, so the published revisions advance 1 → 5 → 9; a
revision is not an edit count.

The pre-memory-change run and the final run are compared using their actual MCP
responses. Complete dependency and evidence resource payloads are compared to
independent cold CLI output after each state, with recursive object-key sorting
and SHA-256. Array order and all exported records/properties are retained in this
comparison. This proves those two exported views on these inputs, not every
possible graph state or the entire semantic contract through MCP.

All six final view comparisons match the independent cold outputs. The first
refresh leaves both views unchanged; the second changes both hashes and matches
the cold results for the changed hook.

| Observation | Retained graph copies | Views built on request |
| --- | --- | --- |
| First useful overview from process start | 62.27 s | 60.04 s |
| Wait for first refreshed index | 56.43 s | 51.98 s |
| RSS sampled after first refresh | 6,478,360 KiB | 4,002,552 KiB |
| Wait for second refreshed index | 67.67 s | 62.13 s |
| RSS sampled after second refresh | 7,337,992 KiB | 5,777,124 KiB |
| Peak process RSS over the session | 7,962,884 KiB | 6,507,864 KiB |
| Complete graph resource response times | 0.84–1.68 s | 2.57–3.43 s |

The final run uses less retained memory but makes complete resource exports
slower. These are sequential, single observations with the same captured edits
on a shared host, not isolated trials. The source/output directory names differ.
Matching states in the two runs return identical complete payload data. The
final peak remains about 6.21 GiB; substantial memory remains necessary.

After both refreshes, all 23,166 files in the final working copy match the
captured inventory with exactly the two declared substitutions. The original
reference capture matches its original inventory without changes. Both selected
live files still match the captured update bytes at the final comparison.

## Acceptance limits

The [machine receipt](2026-09-09-incremental-resolution-evidence.json) records
commands, binary hashes, source refresh hashes, response revisions, work counters,
resource hashes, timings and memory observations. Session elapsed times include
manual inspection and are not startup or rebuild latency measurements.

The resolver cache remains opt-in under the architecture plan's requirement for
differential regression gates before default activation. CI cases cover changed
bodies, signatures, additions/deletions, negative lookups, aliases, interleaved
references, occurrence identity and downstream edge transformations. These tests
have been written but have not run; automated tests and CI remain paused under
David's instruction. The real refresh observations do not exercise the narrower
case where changed references in one file are processed while other files reuse
results; that case still needs the approved regression gate.

Whole-analysis rebuild latency and memory remain substantial. Symbol changes
still trigger global resolution, and parsed facts and later analyses are not yet
incremental. Multi-client load, adversarial watcher/worker failures, broader
source evidence and approved CI remain open. This does not close Q11–Q12, the
full acceptance goal or the separate work lock.
