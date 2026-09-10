# Parser-owned JavaScript and TypeScript bindings

Variable-bound arrow functions and function expressions are native function
symbols, including their parameters, parent symbol and source span. Scoped
function IDs include declaration positions so sibling blocks can contain
distinct same-named functions, even on one line. Vue uses the same parser over
its existing line-preserving script extraction; this does not expand template
coverage.

The parser collects declarations and then binds references through their AST
scope chain. This covers function/block scopes, parameters and destructuring,
catch bindings, lexical loop declarations, function-scoped `var`, and named
function expressions' private self-names. Ordinary local values and parameters
can shadow callable names without supplying a known function target.
Reassignments conservatively make the affected binding unknown throughout its
scope; this is not a control-flow or initialization-order analysis.

`semantic-graph.json.lexical_bindings` carries:

- `scoped_symbol_ids`: function definitions excluded from ordinary file/global
  name lookup and qualified-name fallback.
- `calls[].reference_index`: the index into this graph's complete, ordered
  `references` array.
- `calls[].argument_index`: `null` for the callee, `0` for a captured first
  identifier argument. This value binding does not itself assert invocation.
- `calls[].target_symbol_id`: the known function identity, or `null` when a
  local binding exists but its callable value is unknown.

A known callee binding resolves directly as `SameFile` with reason
`call:lexical-binding`. An unknown binding blocks name-based guessing. Missing
binding metadata does not mean a name is unbound; existing native import and
other resolution rules still apply. Scoped definitions cannot leak through those
fallback indexes. Signals consumes first-argument binding evidence before its
fallback target lookup; runtime plugins also exclude scoped JS/TS definitions
from module/global callback catalogs.

`SemanticGraph::append` adjusts reference indexes when combining parsed graphs.
Native callers must preserve this metadata and the reference order rather than
manually concatenating only symbols and references. Native resolver-cache identity
includes callee binding decisions and their symbol definitions, even when the
ordinary reference/symbol shapes have not changed; runtime overlays consume the
current argument facts after resolution. Semantic revision 14 prevents
reuse of graphs created before this contract.

Bindings prove supported lexical identities, not runtime execution, complete
JavaScript semantics or dynamic object/cross-module alias flow. Unknown values
within the collected bindings block guesses; unsupported expression shapes still
need further semantic work. Parser-recovered files retain the pipeline's evidence
downgrade. The [Draivix observation](2026-09-10-lexical-bindings.md)
records real CLI and cached MCP behavior; automated regression execution remains
paused.
