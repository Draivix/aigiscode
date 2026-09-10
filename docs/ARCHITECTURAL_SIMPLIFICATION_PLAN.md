# Architectural simplification: next AigisCode priorities

This plan addresses David's requested order of concern: dead code, DRY violations,
overengineering, dual paths and other architectural defects. It extends the
existing GOAL.md and Zeus Shield contracts. It is an implementation plan; the
capabilities below are not all implemented or verified today.

## What an architectural review must establish

Review one business capability at a time: actor identity, relation hydration,
permission publication, query normalization, email delivery or another concrete
operation. Establish its input/output contract, authoritative state, permitted
effects and ownership boundary. Identify its runtime entries and trace the paths
from those entries through implementation, state changes, errors and consumers.
Include routes, jobs, hooks, provider discovery, metadata, client extensions and
operational commands; distinguish test/support uses from production uses.

Compare paths under the same relevant inputs. Record disagreements in validation,
defaults, authorization, tenant context, transactions, caching, retries and side
effects. For every abstraction, identify the invariant, transformation, lifecycle
or external boundary it owns. A public contract or a deliberate adapter can justify
a thin layer or a single implementation.

The output must name the smallest justified action: delete, consolidate, migrate,
inline, split, keep or investigate. Name the surviving owner, affected consumers,
behavior that must survive, source evidence and missing proof. Similar names,
long files, many dependents and zero static callers are candidate signals.

## Current implementation gaps

- [Parsed implementation comparisons](BEHAVIOR_COMPARISON.md) now supplement
  token/marker grouping with bounded body, branch, selector and call evidence.
  The configured Draivix observation found the actor-ID disagreement and its
  existing helper; the distinct relation-schema adapters remained unverified
  input-contract evidence without a DRY finding. Capture and candidate search
  are incomplete, and concrete consolidation remains a source-review proposal.
- `abstraction_roles` and `abstraction_sprawl_concepts` derive evidence from
  names/paths; their content parameter is unused. This does not establish that
  the participating layers lack separate responsibilities.
- The [architectural review contract](ARCHITECTURAL_REVIEW_CONTRACT.md) now carries
  typed actions, survivors, comparisons, preserved behavior and missing evidence.
  Both adapters validate source identity, citations and packet coverage before
  publication. Source-reviewed proposals and stale/invalid review records are
  represented in the review surface. Semantic acceptance and regression execution
  remain open; reference validation alone does not prove the proposed conclusion.
- [Dead-code proof scopes](DEAD_CODE_PROOF_SCOPES.md) now separate local bindings,
  PHP class/trait dispatch, module reachability and registration requirements.
  The configured Draivix observation retained the four reviewed local candidates
  while broader checks stayed deferred. Complete local parsing still does not
  establish whole-application unreachability or runtime acceptance.

## Implementation sequence and acceptance

| Order | Improve AigisCode | Evidence required before calling it successful |
| --- | --- | --- |
| 1 | Extend the existing graph packets, agent response and review surface with a typed conclusion/action, implementations compared, surviving owner, consumer changes, preserved behavior and missing evidence. Validate locations and source identity. | An inference remains labelled as an inference. A stale or invented location cannot become a confirmed finding. The actor-attribution case produces an input/behavior comparison and a concrete migration; a framework provider is retained with its discovery evidence. |
| 2 | Separate dead-code proof by its required scope: local binding, class/private dispatch, module reachability and runtime registration. Classify unreachable candidates, dynamic entries, test/support artifacts, incomplete migrations and unknowns separately. | Preserve the three source-verified PHP unused-import candidates and the reviewed private-method candidate. Reject deletion of the discovered SmsService provider and an unwired but intended replacement. Incomplete coverage blocks the affected claim rather than being silently treated as no usage. |
| 3 | Find repeated business decisions using existing parsed bodies, symbols, calls and effects; use similarity to propose comparisons, then compare contracts and branches. | Detect disagreement among actor-ID extractors. Explain a shared normalization primitive where one exists. Retain adapters with different input contracts and normalizers with different schemas. A DRY finding includes the repeated rule, behavior differences, canonical owner and caller migration. |
| 4 | Model competing execution paths and unfinished replacement wiring. Identify selectors, registrations, active consumers, both-path execution, compatibility obligations and retirement conditions. | Show whether old and new paths both run, are selected conditionally, or whether the replacement is not wired. Review ActorContext and the four replacement-contract candidates against actual consumers. Intentional provider alternatives and version boundaries retain their justification. |
| 5 | Evaluate abstraction necessity through delegation chains, transformations, owned state/invariants and public boundaries. Separate independent change responsibilities in broad services. | An inline/delete proposal explains what the removed layer contributes and how its contract survives. A split proposal names responsibility boundaries and existing owners to reuse. Neither filename roles nor file length alone produce an overengineering verdict. |
| 6 | Feed reviewed conclusions back into existing policy/doctrine and compare eligible baselines, prioritizing new confirmed drift. | Necessary patterns stay accepted with reasons; changed source makes stale conclusions reviewable again. Success means unjustified paths removed with required behavior preserved, not merely fewer warnings or more deleted lines. |

Reuse native parsing/resolution, framework model packs, contract inventory,
bounded graph/state-flow packets, the existing agent execution adapters, policy
and convergence. Generic language facts belong in core; framework discovery in
model packs; repository-approved architecture in doctrine. AI should formulate
and compare architectural claims while the engine anchors them to captured facts.
Runtime validation must not be implied by a source-only argument.

## First Draivix remediation queue

The [current assessment](2026-09-10-draivix-quality.md) and
[reviewed candidate ledger](2026-09-08-draivix-review/dead-code.md) supply the cases.
These are proposed source changes, not changes applied to Draivix.
Confirmed security or data-integrity faults take priority over simplification.

1. Remove the three unused PHP import bindings, rechecked by same-file occurrence
   searches: `ProjectStatusGroups` in `Project.fields.php`, and `RuntimeMessage`
   in `CompleteFieldServiceTripAction.php` and `ServiceTripRouteOptimizationService.php`.
2. Complete the bounded deletion review for
   `EntityViewHandler::hydrateManyToManyForShow`. No named production caller or
   same-class dynamic dispatch was found in the prior review, and the current
   body is unchanged. This path also duplicates relation/pivot hydration and
   turns query failures into an empty relation. Verify dispatch assumptions before
   removal; do not spend effort redesigning an unused path.
3. Consolidate actor attribution. The explicit `actorId`-only context is accepted
   by persistence and ignored by assignment notifications. Agree on precedence,
   positive-ID rules and ambient-auth behavior, then migrate the existing callers.
   The proposed ActorContext helper is an integration target, not an orphan to delete.
4. Finish or explicitly retire the pending ACL, sort-target, filter-type and
   view-column contracts. Check registrations and consumers before deciding which
   implementation survives. The ACL effect writer owns after-commit behavior;
   its lifecycle must not be lost through a superficial merge.
5. Remove the small create-submit registry/loader runtime cycle by moving shared
   normalization to an independent owner. Keep type-only links and ordinary
   asynchronous job/service relationships correctly classified.
6. Separate evidence artifact handling from evidence-domain decisions, then
   continue the existing Email and Chat workflow extractions. Measure the Mitel
   per-trip query pattern before changing validation/loading behavior.

## Other violations to prioritize

| Violation | Why it matters | Current evidence posture |
| --- | --- | --- |
| Inconsistent validation, defaults or identity rules | Equivalent operations behave differently across callers. | Actor attribution is source-confirmed; broaden the same-contract review. |
| Swallowed errors presented as successful empty data | Users and callers cannot distinguish absence from failure. | Present in the private relation-hydration candidate; remove that dead path if confirmed, then inspect live equivalents. |
| Multiple owners of the same mutable state or cache | Updates and invalidation can disagree, especially across tenant/session boundaries. | Email's distributed request/state ownership is an audit target; no stale-response or tenant-leak incident was reproduced. |
| Hidden dependencies and boundary bypasses | Ambient auth, service location and direct persistence can bypass intended policy/lifecycle. | Actor fallback and core/module dependencies are observed; each boundary needs a contract-specific judgment. |
| Duplicate effects or unclear transaction/retry ownership | One action may notify, persist or publish twice, or publish before commit. | Next trace target; the ACL writer's after-commit contract is evidence of a boundary to preserve, not a demonstrated duplicate-delivery incident. |
| Unnecessary runtime cycles and broad public interfaces | Initialization and ordinary changes affect too many owners. | Create-submit cycle and Accounting/Email/Chat responsibility concentration are observed. |
| Business rules encoded independently in several places | The same policy drifts between UI, API, jobs and imports. | Apply the semantic DRY review; do not label protocol constants or legitimate schema adapters as business-rule duplication. |
| Per-item IO and unbounded work in request paths | Latency and resource use grow with real input cardinality. | Mitel query-in-loop is observed; production impact still needs measurement. |
| Hand-built mechanisms or unnecessary dependencies | Extra behavior and maintenance can duplicate an existing platform capability. | Review against actual requirements and existing owners; preserve justified metadata/runtime engines. |

Tests and CI remain paused under the current instruction. The regression cases
and runtime acceptance requirements stay explicit; source inspection does not
turn them into executed checks. The evidence/action contract and scoped dead-code
unit and parsed behavior comparisons are implemented with the verification limits documented above. Continue
competing-path wiring, abstraction necessity
and reviewed policy/convergence feedback. Broader Q01–Q12 correctness obligations
remain open.
