# Draivix code quality — current assessment

**Draivix has a coherent intended architecture, but uneven implementation quality
and significant maintenance risk in shared workflows.** The most actionable
problems are inconsistent behavior across parallel implementations, unfinished
integration of replacement contracts, and concentration of unrelated workflow
responsibilities. The findings below are sufficient to prioritize repairs now.

This assessment combines the broad preserved-source audit with a fresh read of
19 relevant files in `/home/david/Work/Programming/draivix`. All 19 matched the
audited copy by SHA-256 on September 10. That verifies these findings against
the current checkout; it does not recertify every file in the live working tree
or a deployed application. No Draivix source, application command or database
was changed. Source identities are recorded in
[the accompanying evidence](2026-09-10-draivix-quality-evidence.json).

| Priority | Finding and evidence | Recommended outcome |
| --- | --- | --- |
| 1 | **Actor attribution disagrees between live paths.** `HookAwareMapper.php:1140` accepts explicit `actorId` and falls back to ambient authentication; `AssignmentNotificationService.php:120` reads only `context.user`. For `['actorId' => 41]`, these branches return 41 and null respectively. Assignment self-notification suppression uses that result at lines 41–45. | Agree on one actor contract, including explicit ID precedence, user shape, positive-ID validation and ambient fallback; migrate the existing consumers. `app/Support/ActorContext.php:13` already offers a starting point but the inspected consumers still use their own implementations. |
| 2 | **Replacement contracts are not integrated into the inspected production paths.** Searches under `app/` found the declarations of `AclMembershipWriter`, `EntitySortTargetResolver`, `EffectiveFilterTypeResolver` and `ViewColumnSliceResolverInterface`, without named production consumers of those exact contracts. The concurrent working tree contains unfinished work. | Complete caller/registration integration or explicitly retire the replacement. Accept the actual runtime path, not only an isolated helper. A name search cannot exclude every dynamic registration, so these are integration review targets rather than automatic deletion candidates. |
| 3 | **Large workflow owners concentrate independent change drivers.** `AccountingProductionEvidenceService.php` has 8,551 lines and owns evidence requirements, evaluation, templates, artifact handling and persistence. `ChatService.php` has 4,640 lines covering conversation CRUD, synchronous/streaming generation, voice and tools. `EmailService.php` has 2,655 lines despite existing specialized collaborators. | Extract along existing ownership boundaries. Begin with evidence artifact storage versus domain validation, and finish Email/Chat workflow separation using their existing collaborators. Preserve public contracts and avoid creating another orchestration framework. |
| 4 | **Email UI has concentrated workflow and request-state ownership.** `Email/Index.vue` has 5,649 lines; `EmailPreview.vue` has 5,365. Compose, selection, detail navigation, transfer, sync and request state still share these components despite existing extractions. | Move remaining workflows into existing components/composables and the canonical query layer; keep tenant/auth keys and invalidation together. This is a maintainability risk, not a reproduced browser race. |
| 5 | **One small runtime dependency cycle is directly actionable.** `entityCreateSubmitRegistry.ts:2` imports its loader; `entityCreateSubmitClientLoader.ts:4` imports the registry's runtime normalizer. | Put shared normalization/types in an independent module so registry and loader no longer import each other's runtime implementation. No initialization failure has been reproduced. |
| 6 | **A concrete repeated-query pattern merits measurement.** `clients/mitel/Hooks/Task/TaskWorkflowHook.php:315–329` calls `findBy` for each trip, then checks its protocols. | Measure SQL count and trip cardinality in the real workflow; batch protocol loading if material, preserving validation behavior. The query-in-loop is confirmed; production latency and business impact are not measured. |

There is also a small cleanup lane: the private
`EntityViewHandler::hydrateManyToManyForShow` at line 1297 has no named production
caller in the current `app/` and `clients/` search, consistent with the earlier
method-body and dispatch review. Its implementation duplicates relation/pivot
hydration and converts query failures into empty data. Treat removal as a bounded
review item, with dynamic dispatch and concurrent work checked before deletion.

The architecture has useful foundations: a declared metadata kernel, explicit
module/provider discovery, and specialized collaborators already extracted from
Email and Chat. Shared registries, entity managers and protocol adapters can
legitimately have many dependents. Required metadata registration and ordinary
job/service orchestration should be preserved. Core use of `ModuleRegistry`
needs an explicit extension contract consistent with doctrine; counting every
core-to-module reference as a defect does not make that design decision.

The provider example illustrates the need for context. A scan of SmsService alone
previously proposed deleting `SmsServiceServiceProvider`. Outside that slice,
`bootstrap/providers.php:47–49` calls module discovery and includes its providers;
`app/Modules/ModuleProviderDiscovery.php:94–97` constructs the provider class from
the module directory name. The provider matches that rule. The analyzer now
defers backend orphan judgments at truncated boundaries. A zero-use result in a
slice is insufficient evidence to delete it.

The latest broad audit covers 17,067 supported sources, 132,657 symbols and
331,060 resolved edges; it reports 19 strong and 22 total cyclic components.
Those are graph observations, not counts of production failures. Type-only links,
framework registration and asynchronous dispatch require different treatment.
The audit retains 80 parser recoveries, 792 Vue files with script-only coverage
and 146 unsupported sources. Broad dead-code checks are consequently deferred.

**What remains:** implement the prioritized Draivix repairs, verify their actual
caller/workflow behavior, and run approved regression and security checks.
Tests and CI remain paused by instruction; external security scanners were not
run. Production reliability, exploitability and user-visible performance therefore
remain unverified. AigisCode still needs broader parser/resolver validation,
watcher/concurrency failure coverage and incremental-performance acceptance.
These limitations constrain confidence; they do not prevent acting on the
source-confirmed contract and ownership problems above.
