# Python local class identity and binding

A class declared beneath a Python function has a declaration-position ID and
its enclosing symbol as lexical parent. Its methods retain that class as their
parent. Local classes and their descendants are excluded from the ordinary
file/global/qualified-name and callback candidate catalogs.

The Python parser contributes to the shared `SemanticGraph.lexical_bindings`
contract; there is no Python runtime or additional dependency. Collected free
calls, simple parameter types and superclass references whose names have a local
class declaration in the file follow the Python scope chain. Calls use `calls`;
types and heritage use `named_references`. Graph append rebases both index lists,
and the resolver/cache consume the existing typed binding contracts. Semantic
revision 17 invalidates earlier captures.

Class namespaces apply to the class body, not to nested function or class bodies.
Default arguments, annotations collected at the declaration and superclass
expressions use the surrounding definition context. Parameters, assignments,
loop/with/except bindings, match captures and deletions can mask a class name.
`global` selects the module namespace; `nonlocal` searches enclosing function
bindings. A write through either directive invalidates the destination's known
value. Missing or unknown local names block a foreign global-name guess.
Module imports retain native import resolution; scoped imports remain unknown
in this bounded class-binding pass. Decorated classes also have unknown values.

This is structural name binding, not execution or initialization-order analysis.
Multiple definitions/assignments make a value unknown across its scope. A
comprehension containing a relevant class-name reference is conservatively
unknown; evaluation order, full comprehension binding and version-specific
annotation scopes are not claimed. Namespace classes outside a function retain
the existing parser behavior. Function and method classification outside this
local-class boundary is not repaired by this change.

The existing receiver inference stores type-name strings and cannot prove which
local class a value carries through aliases, parameters or factories. Local
method candidates are therefore unavailable to name guesses, and a collected
member reference with an unresolved/local class type has an explicit unknown
callee. This pass does not establish own/inherited instance dispatch, metaclass
behavior, arbitrary dynamic rebinding or escaped local classes in other modules.
Those gaps must not be reported as clean or complete language coverage.

The scope distinction follows the [Python execution model](https://docs.python.org/3.11/reference/executionmodel.html).
Real source observations and verification limits are recorded in
[the Draivix local-class report](2026-09-10-python-classes.md).
