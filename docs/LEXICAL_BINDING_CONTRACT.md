# Parser-owned JavaScript and TypeScript bindings

Variable-bound arrow functions and function expressions are native function
symbols, including their parameters, parent symbol and source span. Scoped
function and local class/interface/enum IDs include declaration positions so sibling blocks can contain
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

- `scoped_symbol_ids`: local definitions and their descendants excluded from ordinary file/global
  name lookup and qualified-name fallback.
- `calls[].reference_index`: the index into this graph's complete, ordered
  `references` array.
- `calls[].argument_index`: `null` for the callee, `0` for a captured first
  identifier argument. This value binding does not itself assert invocation.
- `calls[].target_symbol_id`: a supported function, constructed class or direct
  method identity, or `null` when a collected binding has no supported target.
- `named_references`: reference indexes and nullable symbol IDs for collected
  type and heritage references. The type namespace is separate from value
  bindings; generic parameters and type aliases can shadow an outer type.

A known callee binding resolves directly as `SameFile` with reason
`call:lexical-binding`; named references use `reference:lexical-binding`. An unknown binding blocks name-based guessing. Missing
binding metadata does not mean a name is unbound; existing native import and
other resolution rules still apply. Scoped definitions cannot leak through those
fallback indexes. Signals consumes first-argument binding evidence before its
fallback target lookup; runtime plugins also exclude scoped JS/TS definitions
from module/global callback catalogs.

Named local classes bind `new Local()` through the value scope at construction.
Direct `new Local().method()`, `const value = new Local(); value.method()`, and
class-method `this.method()` can bind a unique own method. Instance and static
methods are separate. Construction scopes are retained: a later nested class
with the same name cannot change an already captured variable's constructor.
Arrow functions preserve the enclosing method's `this`; an ordinary nested
function has no such proof. Reassignment or duplicate declaration invalidates
instance evidence, and an unknown instance cannot recover a foreign method
through a name guess. The class body has its own self-name binding.

This direct method proof is deliberately bounded to scoped classes. Inherited
methods, class fields/accessor-returned callables, arbitrary aliases, factory
returns, global mutations, initialization order and dynamic method replacement
are not established by these facts. A known scoped receiver with no unique own
method remains unresolved. An unsupported value with a local type hint also
blocks a fallback guess; this does not prove all calls through typed parameters.
Ordinary imported and module-level receiver resolution continues through the
existing resolver. Ordinary function construction is not asserted by the new
class-construction binding.

`SemanticGraph::append` adjusts both call and named-reference indexes when combining parsed graphs.
Native callers must preserve this metadata and the reference order rather than
manually concatenating only symbols and references. Native resolver-cache identity
includes callee binding decisions and their symbol definitions, even when the
ordinary reference/symbol shapes have not changed; runtime overlays consume the
current argument facts after resolution. Semantic revision 16 prevents
reuse of graphs created before the scoped-class extension.

Bindings prove supported lexical identities, not runtime execution, complete
JavaScript semantics or dynamic object/cross-module alias flow. Unknown values
within the collected bindings block guesses; unsupported expression shapes still
need further semantic work. Parser-recovered files retain the pipeline's evidence
downgrade. The [Draivix observation](2026-09-10-lexical-bindings.md)
records real CLI and cached MCP behavior; automated regression execution remains
paused.

The lexical/class distinction follows the [ECMAScript specification](https://262.ecma-international.org/16.0/index.html#sec-class-definitions).
The [scoped-class observation](2026-09-10-scoped-classes.md) records the
subsequent Draivix CLI and MCP results and their verification limits.
