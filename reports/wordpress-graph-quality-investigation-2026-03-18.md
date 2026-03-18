# WordPress Graph Quality Investigation

Date: 2026-03-18

Scope:
- compare AigisCode and `../nexus` graph quality on the same `../wordpress` repository
- judge semantic value, not only raw node/edge totals
- sample random files and inspect concrete nodes and edges

## Headline

AigisCode does **not** simply have "more graph" because the resolver is globally broken. The current gap is mixed:

- AigisCode's canonical graph is stronger on cross-file structural coverage and WordPress runtime modeling.
- AigisCode's current Kuzu comparison view is noisier than Nexus because it includes:
  - one synthetic `MODULE` node per analyzed file
  - one `CONTAINS` edge per symbol
  - per-call-site multiplicity instead of only unique dependency pairs
- AigisCode still has real WordPress precision defects that must be fixed before claiming clear superiority.

## Update After Fixes

The main issues above were addressed in the same session:

- the Kuzu artifact is now a normalized `dependency_view`
  - synthetic `MODULE` nodes are omitted
  - `CONTAINS` edges are omitted
  - repeated dependencies are collapsed with `occurrenceCount`
- PHP call resolution now respects optional parameters and prefers free functions for free calls
- the sampled WordPress precision defects in `credits.php` are fixed
  - `wp_remote_get()` resolves
  - `esc_attr()` resolves
  - `translate()` now resolves to the global function instead of a method

Updated normalized totals on `../wordpress`:

- AigisCode dependency-view nodes: `32,862`
- AigisCode dependency-view edges: `95,878`
- AigisCode release-binary wall clock including Kuzu materialization: `22.78s`
- GitNexus wall clock: `22.4s`
- GitNexus nodes / edges: `19,692` / `64,453`

Updated relationship breakdown in AigisCode:

- `CALL`: `85,451`
- `EVENTPUBLISH`: `3,662`
- `OVERRIDES`: `1,947`
- `EVENTSUBSCRIBE`: `1,868`
- `EXTENDS`: `1,489`
- `IMPORT`: `764`
- `TYPEUSE`: `625`
- `IMPLEMENTS`: `72`

Updated conclusion:

- the low-noise Kuzu view is materially better than before and no longer polluted by synthetic module/containment modeling
- AigisCode is still denser than Nexus, but that remaining density is now much more likely to be semantic signal rather than export inflation
- the next remaining parity question is mostly about whether our call graph is still too broad in some file-scope PHP cases, not about obvious representation noise

## Aggregate Findings

Raw totals on `../wordpress`:

- AigisCode: `36,202` nodes / `156,050` edges
- Nexus: `19,692` nodes / `64,453` edges

Important decomposition of the AigisCode inflation:

- `3,340` synthetic `MODULE` nodes, one per analyzed file
- `32,876` `CONTAINS` edges, one per symbol
- `112,014` `CALL` edges, where repeated call sites between the same source and target are preserved

Relevant normalizations:

- AigisCode total `CALL` edges: `112,014`
- AigisCode distinct source->target `CALL` pairs: `75,221`
- AigisCode distinct source->target `CALL` pairs excluding `FILE` sources and `CLASS` targets: `62,711`
- Nexus total `CALLS` edges: `26,584`

Conclusion:

- raw Kuzu totals are not a fair parity metric
- AigisCode is richer, but the current read model mixes architectural dependencies with evidence/detail edges

## Random File Samples

### `src/wp-includes/class-wp-block-supports.php`

Nodes:

- AigisCode: `CLASS`, `FILE`, `FUNCTION`, `METHOD x5`, synthetic `MODULE`
- Nexus: `CLASS`, `FILE`, `FUNCTION`, `METHOD x5`, `PROPERTY x3`

Takeaway:

- AigisCode is not uniformly "fatter" on every file. Here Nexus actually exposes more semantic node kinds because it keeps properties while AigisCode currently does not.

Calls:

- AigisCode outgoing `CALL`: `7`
- Nexus outgoing `CALLS`: `8`
- AigisCode incoming `CALL`: `27`
- Nexus incoming `CALLS`: `6`

Why AigisCode looks better here:

- AigisCode resolves many real inbound usages from `src/wp-includes/block-supports/*.php` into `WP_Block_Supports::get_instance()`
- those edges are plausible and useful architectural signal, not obvious noise

Verdict:

- AigisCode better on inbound cross-file usage coverage for this file

### `src/wp-admin/includes/credits.php`

Calls touching the file:

- AigisCode: `28` `CALL` + `6` `CONTAINS`
- Nexus: `17` `CALLS` + `5` `DEFINES` + `3` `MEMBER_OF` + `1` `CONTAINS`

What AigisCode gets right:

- it captures repeated call sites like two `wp_sprintf()` uses and two `get_avatar_data()` uses
- that is useful evidence in the canonical graph

What AigisCode got wrong before the fix:

- missed real global function call `wp_remote_get()`
- missed real global function call `esc_attr()`
- resolved global `translate()` to a method target instead of the global function

Current status after the fix:

- `wp_remote_get()` resolves at line `42`
- `esc_attr()` resolves at line `147`
- `translate()` resolves as a `FUNCTION` at lines `104` and `156`

Verdict:

- the specific resolver defects found in the sample are fixed
- this file is now a positive example for AigisCode rather than a blocker

### `src/wp-content/themes/twentyten/search.php`

Nodes:

- AigisCode: `FILE` in the normalized dependency view
- Nexus: `FILE`

Calls from the file:

- AigisCode: `esc_html`, `get_template_part`, `_e` twice, `get_search_form`, `get_footer`
- Nexus: `__`, `_e`, `esc_html`

Verdict:

- AigisCode is better on concrete template call coverage here
- the synthetic `MODULE` node no longer pollutes the parity view

### `tests/phpunit/tests/oembed/wpOembed.php`

Nodes:

- AigisCode: class + methods + file + synthetic `MODULE`
- Nexus: no matching nodes found in the repo graph

Verdict:

- AigisCode currently covers WordPress test files that Nexus does not appear to expose in the compared graph
- this is useful coverage, but it also means aggregate totals are not directly comparable unless scope is normalized

## Judgment

Which graph is "better" right now depends on what is being judged.

Better canonical graph:

- AigisCode
- reason: broader cross-file coverage, explicit runtime/framework edges, better inbound usage signal on sampled WordPress internals, broader file coverage

Better low-noise dependency view:

- Nexus
- reason: slimmer entity model, likely closer to unique dependency pairs, less representational inflation in the graph exported for comparison

## Product Decision

The best solution is **not** to throw away AigisCode's richer graph.

The right product shape is:

1. keep the richer canonical graph
2. add a normalized `dependency_view` for parity, dashboards, and architectural reporting
3. keep an `evidence_view` with call-site multiplicity, runtime/plugin edges, and line-level evidence for deep analysis
4. fix the concrete WordPress global-function resolution defects before claiming superiority

## Immediate Work

- remove synthetic `MODULE` nodes and `CONTAINS` edges from benchmark/parity counts
- add unique dependency-pair projections alongside raw call-site evidence counts
- decide whether class-target calls should remain `CALL` or be split into a dedicated relation kind like `INSTANTIATE`
- fix WordPress global function misses in `credits.php` (`wp_remote_get`, `esc_attr`)
- stop misresolving global `translate()` to method targets
