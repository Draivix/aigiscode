# WordPress Graph Comparison

Date: 2026-03-17

Repositories:
- AigisCode: `/home/david/Work/Programming/aigiscode.com`
- WordPress target: `/home/david/Work/Programming/wordpress`
- GitNexus reference: `/home/david/Work/Programming/nexus/gitnexus`

## Commands

GitNexus:

```bash
cd /home/david/Work/Programming/nexus/gitnexus
node dist/cli/index.js analyze ../../wordpress
node dist/cli/index.js cypher "MATCH ()-[r:CodeRelation]->() RETURN r.type AS type, count(*) AS count ORDER BY count DESC" -r wordpress
```

AigisCode:

```bash
cd /home/david/Work/Programming/aigiscode.com
cargo run --release --quiet --bin aigiscode -- graph ../wordpress --kuzu
cargo run --release --quiet --bin aigiscode -- cypher ../wordpress "MATCH (n:CodeNode) RETURN count(*) AS node_count"
cargo run --release --quiet --bin aigiscode -- cypher ../wordpress "MATCH ()-[r:CodeRelation]->() RETURN count(*) AS edge_count"
cargo run --release --quiet --bin aigiscode -- cypher ../wordpress "MATCH ()-[r:CodeRelation]->() RETURN r.type AS type, count(*) AS count ORDER BY count DESC"
```

## Summary

This report started with the first raw Kuzu export. It is now superseded by the normalized dependency-view export and the WordPress resolver fixes from 2026-03-18.

GitNexus:
- wall clock: `22.4s`
- files: `2470`
- nodes: `19,692`
- edges: `64,453`

AigisCode, first raw Kuzu export:
- wall clock: `21.38s`
- scanned files: `5381`
- analyzed semantic files: `3340`
- semantic symbols: `32,876`
- semantic resolved edges: `123,174`
- Kuzu nodes: `36,202`
- Kuzu relationships: `156,050`

AigisCode, current normalized dependency view:
- wall clock: `22.78s` using the compiled release binary
- scanned files: `5381`
- analyzed semantic files: `3340`
- semantic symbols: `32,876`
- semantic resolved edges: `145,566`
- Kuzu nodes: `32,862`
- Kuzu relationships: `95,878`

Assessment:
- AigisCode is still on par on WordPress throughput for graph generation plus Kuzu materialization.
- The normalized Kuzu graph is far cleaner than the original export because it drops synthetic `MODULE` nodes, drops `CONTAINS`, and collapses repeated dependencies.
- AigisCode remains denser than GitNexus after normalization, but the remaining density is now much closer to semantic coverage than export noise.

## Relationship Breakdown

GitNexus:

| Type | Count |
| --- | ---: |
| `CALLS` | 26,584 |
| `DEFINES` | 15,486 |
| `HAS_METHOD` | 9,414 |
| `MEMBER_OF` | 7,173 |
| `CONTAINS` | 2,828 |
| `STEP_IN_PROCESS` | 1,678 |
| `IMPORTS` | 678 |
| `EXTENDS` | 307 |
| `OVERRIDES` | 229 |
| `IMPLEMENTS` | 76 |

AigisCode, first raw Kuzu export:

| Type | Count |
| --- | ---: |
| `CALL` | 112,014 |
| `CONTAINS` | 32,876 |
| `EVENTPUBLISH` | 4,010 |
| `EVENTSUBSCRIBE` | 2,192 |
| `OVERRIDES` | 1,946 |
| `EXTENDS` | 1,489 |
| `IMPORT` | 776 |
| `TYPEUSE` | 673 |
| `IMPLEMENTS` | 72 |

AigisCode, current normalized dependency view:

| Type | Count |
| --- | ---: |
| `CALL` | 85,451 |
| `EVENTPUBLISH` | 3,662 |
| `OVERRIDES` | 1,947 |
| `EVENTSUBSCRIBE` | 1,868 |
| `EXTENDS` | 1,489 |
| `IMPORT` | 764 |
| `TYPEUSE` | 625 |
| `IMPLEMENTS` | 72 |

## Important Differences

Why AigisCode is still denser after normalization:
- AigisCode persists a richer symbol-level dependency graph than GitNexus.
- AigisCode records more method/function dependency coverage, especially inbound call relationships on WordPress internals.
- AigisCode models WordPress hook wiring explicitly through `EVENTSUBSCRIBE` and `EVENTPUBLISH`.
- AigisCode now resolves more real global function calls after the PHP arity fix, which increased semantic resolved-edge counts while improving correctness.

Why GitNexus still has edges AigisCode does not:
- GitNexus currently materializes community membership and process-flow edges (`MEMBER_OF`, `STEP_IN_PROCESS`).
- AigisCode does not yet expose comparable community/process graph layers in the new Kuzu read model.

## Current Edge-Layer View In AigisCode

From the AigisCode Kuzu graph:

| Layer | Count |
| --- | ---: |
| `structural` | 149,846 |
| `runtime` | 4,010 |
| `framework` | 2,192 |

This is an advantage in graph interpretation:
- AigisCode can already distinguish structural edges from runtime/framework-expanded edges.
- GitNexus exposes a flatter relationship model in the current WordPress benchmark.

## Where AigisCode Is Better Right Now

- Typed graph layers: structural vs runtime vs framework.
- First-class WordPress hook publish/subscribe edges.
- Higher override and inheritance coverage.
- Native Rust semantic graph remains canonical; Kuzu is a query read model, not the source of truth.
- Native MCP server can now execute raw Cypher against the current repo graph for code understanding.

## Remaining Gaps

- Dynamic hook names are still under-modeled.
- Richer WordPress callback forms should be added.
- AigisCode still lacks GitNexus-style community and process overlays in the Kuzu view.
- The denser graph should be sampled continuously so density remains signal, not accidental inflation.

## Conclusion

The first raw Kuzu export overstated the gap because it mixed architectural dependencies with representational noise. After normalizing the Kuzu read model and fixing the sampled WordPress PHP resolution defects, AigisCode is still not smaller than GitNexus, but it is much cleaner than before and remains competitive on throughput. The next work is no longer "remove obvious export inflation"; it is to keep hand-checking whether the remaining extra call density is true architectural signal or still too broad in some PHP file-scope cases.
