# Plugin System

AigisCode uses a Rust-native plugin and overlay model.

## Current Plugin Scope

The current product already emits framework and runtime meaning through native
Rust plugins layered on top of the canonical semantic graph. Current shipped
slices include:

- queue and job dispatch runtime edges
- Laravel container-resolution edges
- WordPress hook and filter publish-subscribe edges
- signal callback registration, receiver decorators, and dispatch edges

These plugins enrich the graph without changing the language-truth layer in
core parsing and resolution.

## Design Boundary

Plugin responsibilities:

- framework and runtime conventions
- plugin-derived edges and facts
- non-structural overlays such as dispatch, container resolution, or hook
  publish-subscribe behavior

Core responsibilities:

- parsing and symbol extraction
- import, type, and call resolution
- canonical semantic graph truth

Policy and rules responsibilities:

- repository-specific accepted behavior
- suppressions, thresholds, and local doctrine
- convergence of reviewed false positives

## Public Contract

Plugin-derived behavior is visible through the normal Rust artifact family:

- `.aigiscode/semantic-graph.json`
- `.aigiscode/dependency-graph.json`
- `.aigiscode/evidence-graph.json`
- `.aigiscode/contract-inventory.json`
- `.aigiscode/architecture-surface.json`

Plugin-produced edges must stay typed, layered, and explainable. New plugin
work should build on Rust contracts, not on Python module loading or sidecar
runtimes.

## Signal Binding Evidence

Signal callbacks use the [captured parser sources](INPUT_CAPTURE_CONTRACT.md).
Parser-owned [lexical argument bindings](LEXICAL_BINDING_CONTRACT.md) take
precedence over callback-name lookup: an unknown local value blocks that
fallback, while a known local function remains available in its own scope.
The bounded call excerpt must match the actual reference receiver on the call's
starting line. A simple callable argument can be resolved; the result of a
callback factory call cannot be treated as the factory itself.

Bare callback names require a same-file function or resolved import. Qualified
callbacks require a declared/imported class, the enclosing class for `self`/`this`,
or a resolved module import containing the function. A missing member does not
fall back to an unrelated global function with the same final name. Class-method
lookups retain the class's defining file.

Signal registration and dispatch retain the full receiver expression. Imported
receivers retain their exported name and defining file, so aliases can meet while
unrelated same-named exports remain separate. Unresolved absolute imports retain
language and declared module; unresolved relative imports stay tied to the
importing file. These are file/import identities, not runtime object identities:
rebinding, nested-scope shadowing and dynamic instance aliasing are not proved.
Edges remain explicitly dynamic framework/runtime evidence; absence is not proof
that a callback cannot execute. Semantic revision 13 invalidates earlier graphs.

The [Draivix observation](2026-09-10-signal-bindings.md) records the exact removal
of five incorrect edges. Positive/negative regression fixtures were written but
remain unrun while automated tests and CI are paused.
