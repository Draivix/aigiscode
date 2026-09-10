# PHP anonymous classes

The native PHP parser gives every `anonymous_class` AST node a `Class` symbol.
Its body owns its methods, trait uses, heritage references and `$this` type.
The surrounding function or method remains the class symbol's lexical parent.
An anonymous class at file scope has no lexical parent.

Synthetic names have the form `anonymous:<file-hash>:L<line>:B<byte-offset>`.
The hash is xxh3-128 of the graph-relative file path. Colons distinguish these
names from valid PHP declaration names; the file component separates equal
offsets in different files in the same PHP namespace. The qualified name also
includes that namespace. Names and IDs are snapshot identities: moving the
declaration or renaming the file changes them. Anonymous class symbols have
private visibility because PHP does not declare a publicly addressable name;
individual method visibility remains unchanged.

The containing `new` expression contributes a constructor call to the anonymous
class. Its argument count comes from the anonymous node's actual argument list.
Constructor arguments retain the enclosing scope: `$this->provide()` in
`new class($this->provide()) { ... }` belongs to the surrounding object. Calls
inside the class body belong to that class's methods. The parser also recognizes
anonymous types in its existing direct constructor receiver and assignment
inference paths. `$this` and `self` inside methods use the anonymous identity.

Property lookup stops at the nearest class, including an anonymous class, and
does not descend into another class's body. Local receiver inference can inspect
an anonymous class's constructor arguments but does not borrow parameter or
assignment types from its declarations. Existing handling of promoted,
declared and documented property types is reused.

Resolution and override generation consume the ordinary typed graph facts.
There is no repository-specific rule, post-parse ID deduplication, new dependency
or separate runtime. Semantic revision 15 invalidates graphs produced under the
old parser semantics.

This does not establish general flow-sensitive PHP object typing. Mutable
assignments, arbitrary aliases, closures, inherited dispatch and parser recovery
retain the limits of the existing analyzer. Constructor edges identify the
constructed class, as for named classes; they do not assert execution of every
method. Duplicate identities in other language parsers are a separate issue.

The source-based Draivix observation and the unrun regression are documented in
[the accompanying report](2026-09-10-anonymous-classes.md).
