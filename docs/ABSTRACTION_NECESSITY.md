# Abstraction necessity from captured behavior

The filename-role grouping that produced AbstractionSprawl candidates has been
removed. That finding family now proposes review of a chain containing at least
two private forwarding wrappers and a captured delegate. The arguments must be
forwarded unchanged to a unique strongly resolved callable, with compatible
declared returns and complete bounded body facts. Cyclic and cropped chains do
not produce this candidate.

Public/protected visibility, declared overrides, an owner's declared contract,
decorators/attributes, special calling conventions, defaults, branches,
mutations, error handling, loops and deferred callbacks prevent an automatic
private-forwarding interpretation. This does not prove that every retained
layer is necessary. It prevents a body-only shortcut from discarding a possible
contract. A reviewer still decides whether receiver dispatch and consumer
behavior permit inlining.

`implementation_context` and graph/agent packets expose
`execution_paths.implementations[].abstraction`: method responsibilities,
branch and return expressions, captured assignment expressions, strongly
resolved callees and existing dependency owners, boundary evidence and bounded
delegation chains. Public methods are previewed first. Missing bodies, weak or
unresolved calls, and cropped method/expression lists remain explicit.

The broad-interface detector now discounts only captured plain field reads and
unchanged field assignments, not `get`/`set` name prefixes. Its result is a
**broad public interface candidate**. The review must identify independent
reasons to change and existing owners to reuse before proposing a split.
Different callee names or a large method count do not establish those reasons.

## Native observations

The configured Draivix snapshot produced no strict multi-wrapper candidates;
that is not a whole-application overengineering verdict. The
[MCP profile excerpt](2026-09-10-abstraction-profiles.json) preserves concrete
boundary and dependency-owner observations:

- `ActorContext` retains its validation branches and public contract.
- `AclMembershipWriter` retains the effect interface, deferred callbacks,
  mutations, tenant dependency and publication/error-handling behavior.
- Email's sending-account, transfer and audit paths expose existing access,
  assignment and audit owners. Chat exposes conversation, tool-execution and
  streaming dependencies. These are inputs to a decomposition review.
- The evidence service exposes manifest/artifact operations alongside domain
  checks. Its unresolved calls remain gaps rather than invented owners.

Email, Chat and the evidence service contain 113, 120 and 150 captured method
symbols respectively. Each profile previews 32 methods and explicitly reports
incomplete body evidence. No split is justified merely by these counts.

The first sender profile exposed a capture gap when a PHP attribute precedes
the method name on another line. Capture now also matches the parser-owned
name location. A subsequent graph-only observation of Accounting Services
[captured the attributed method](2026-09-10-attributed-method-capture.json),
including its attribute, two validation branches and four calls. Its interface,
override attribute and behavior all require preservation.

Native builds and the listed CLI/MCP observations completed. Regression cases
use real parsed declarations for forwarding, attribute isolation, variadic
binding and field accessors; tests and CI remain paused and were not executed.
Draivix source files were not changed.
