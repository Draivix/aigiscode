# Execution paths and replacement wiring

The native MCP `implementation_context` tool accepts one to eight exact symbol
IDs, qualified names or unambiguous names. It returns a source snapshot ID,
input coverage, bounded parsed bodies and comparisons, and an execution-path
assessment. Ambiguous names require an exact ID from `find_symbol`.
Agent task packets and graph packets carry the same wiring evidence for their
selected implementations or primary file; selection and detail limits are explicit.

For a class, the assessment includes calls to its methods and excludes calls
originating within that class. For a method, it retains its captured callers.
Test calls, non-call references, declared interfaces and implementation types
remain separate. A captured call is not proof that its caller is reachable from
an application entrypoint. Zero captured callers is a wiring question, not a
deletion recommendation.

Relationships distinguish implementations with captured consumers, shared
callers, conditional sites, opposing branches, and one or both sides without
captured consumers. Opposing branches describe source control flow only; that
classification is withheld for sites inside loops because separate iterations
can select different branches. Branch conditions retain source locations.
Unconditional call sites do not prove both calls complete: earlier returns,
errors and dispatch still require review. Deferred callbacks are excluded from
the enclosing method's immediate body calls and retain a separate count.

## Registration evidence

`semantic-graph.json.runtime_registrations` is emitted by runtime model packs.
The Laravel container model captures `bind`, `bindIf`, `singleton`,
`singletonIf`, `scoped` and `scopedIf` calls on recognized application-container
receivers when their first argument is a parser-resolved class literal. A
literal implementation in the second argument is retained too. PHP argument
facts preserve canonical class names in the first sixteen positional slots;
aliases are resolved by the parser, not by nearby-text matching.

Records retain model, mechanism, source location, parse coverage and optional
syntactic conditionality. Registered alternatives of the same contract stay
visible as alternatives; registration alone does not establish which binding
wins or executes. Factories, computed keys, named arguments, external discovery
and other unsupported registration forms remain missing evidence. The bounded
contract inventory's name matches are exposed separately as
`contract_name_mentions`, not as proof of binding this implementation.

## Required architectural decision

Review the same concrete input across the selected paths. Identify selectors,
registration precedence, consumers, transaction/retry ownership and compatibility
obligations. Name the surviving owner and actual caller migration, or retain an
intentional alternative with its reason. State the conditions for retiring the
old path. The [typed review contract](ARCHITECTURAL_REVIEW_CONTRACT.md) keeps
these conclusions as source-reviewed proposals until runtime acceptance exists.

Regression cases cover conditional calls, opposing branches with and without a
loop, an unreferenced replacement, deferred callbacks and registered interface
alternatives using real parsed PHP sources. Tests and CI remain paused; these
cases have not been executed.

## Native Draivix observations

The [configured-slice MCP observation](2026-09-10-execution-paths.json) used one
fresh published snapshot. The actor extractors retained three and two captured
call sites, while `ActorContext` had none. All four replacement contracts also
had no captured consumers: `AclMembershipWriter`, `EntitySortTargetResolver`,
`EffectiveFilterTypeResolver` and `ViewColumnSliceResolverInterface`. The ACL
writer retained its declared `RelationMutationEffect` interface. These results
identify integration review targets; input and runtime coverage remain incomplete.

The same run captured 162 literal container registrations. For the positive
registration case, `InvoiceThankYouEmailSender` had no direct captured call but
retained its dispatcher interface and the binding in
`AccountingServiceProvider.php:135`. An absent direct caller therefore cannot
justify deleting that implementation.

The first ACL body query exposed a PHP closure-node mismatch. After correcting
it against the installed grammar, a separate native MCP observation of the
cropped Permission directory retained
[two deferred callbacks](2026-09-10-deferred-callbacks.json) in `afterMutation`.
The callback's `publish` call no longer appears among immediate body calls;
the immediate conditional publication and both callback registration calls
remain. This smaller observation verifies callback capture, not application
wiring. The configured-slice receipt above contains consumer and registration
evidence and deliberately does not present the earlier body counters as current.

Native builds completed. No application transactions, notifications or regression
suites were executed, and Draivix source files were not changed.
