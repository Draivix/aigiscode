# Draivix architecture review — 2026-09-08

Analysis only for Draivix. AigisCode improvements are implemented separately.

The strongest current findings are **inconsistent actor attribution, incomplete
replacement wiring, and concentrated orchestration**. The metadata kernel itself
is justified by this application's contract. Replacing it with generic CRUD, or
splitting every dependency hub, would be the wrong response.

## Evidence and scope

Target HEAD: `95276a526f6c9e684f81ea4b83639894e2f02c38`, **plus the existing dirty
working tree**, not HEAD alone. File references below are relative to Draivix.
These findings describe the checkout, not a deployed system.

Three corpus profiles were inspected through repeated AigisCode runs:

| Corpus | Coverage | Use |
|---|---|---|
| Current configured checkout | `app`, `routes`, `config`, `resources/js`; 12,647 scanned files, 9,662 parsed sources | Production-oriented findings and all numbers below unless stated otherwise |
| Broader copied source tree | 17,748 parsed sources, including clients, bootstrap, database, tools and tests | Missing-caller investigation and detector scope comparison |
| Broader tree without generated cache/bundle sources | 17,013 parsed sources | Final supplementary review corpus; all 32 configured-scan candidates remain visible |

The normal scan excludes `Tests`, `tests`, `tests_fast`, `__tests__`, fixtures,
mock directories, `lang`, and `database`. **Commands are included now**; the July
warning about excluded commands is stale. The graph correctly marks this scan as
a truncated slice, with 1,486 boundary-truncated files. Those are not 1,486 dead files.

The broader corpus is a source snapshot, not a production-only census. It also
contains checked-in Android JavaScript assets. Hidden files, symlinks, dependency
trees, generated build directories, and most non-source assets were not copied.
Composer/package/TypeScript/Cargo manifests and the existing doctrine were copied;
the restrictive scan configuration was deliberately omitted. Generated PHPStan
caches and Android bundle sources were subsequently isolated outside the copied
root, with a matching snapshot-only exclusion profile. Parsing exclusions alone
did not stop the detector's supplementary sweep from reading those caches.
No target policy was edited. Aggregate counts from these corpora are not interchangeable.

Source verification combined AigisCode findings, semantic edges, literal caller
searches across production and test trees, registration/configuration reads, and
method-body inspection. All 17,748 copied source files matched the originals by
SHA-256 when compared. The before/after configured scan manifests matched exactly;
the original Git status was also unchanged. No Draivix command, test, database
operation, installation, or source edit was performed.

Local evidence is under `target/draivix-audit-2026-09-08/` in AigisCode:
`before/`, `after/`, `final/`, `self/`, `self-after/`, `full/`, `full-after/`,
`full-clean/`, `full-final/`, `dead-code-review.json`, and `source-integrity.json`. Generated graph artifacts and
the copied source tree remain untracked. The first scan took 67.71 seconds with a
maximum resident set of 1,585,280 KiB; a self-scan overlapped part of that run, so
this is an observed execution measurement, not an isolated benchmark.

## Prioritized findings

### 1. Actor identity has several live, disagreeing implementations

**High-confidence design defect; prioritize completion of the existing consolidation.**

| Consumer | Source | Current behavior |
|---|---|---|
| ORM side effects | `app/Entities/_Core/Cycle/HookAwareMapper.php:1140` | Explicit actor, then user property, then ambient `Auth::id()` |
| Deletion side effects | `app/Entities/_Core/EntityDeleteSideEffects.php:124` | Same fallback sequence |
| Realtime | `app/Services/_Core/Realtime/EntityBroadcaster.php:184` | Explicit actor or user property; no ambient fallback |
| Status publication | `app/Services/_Core/StateMachine/StatusTransitionPublisher.php:124` | Explicit actor, `Authenticatable`, public user properties, ambient fallback |
| Assignment notifications | `app/Services/Notifications/AssignmentNotificationService.php:120` | Only user property; ignores explicit `actorId`; requires positive ID |
| Existing user helper | `app/Services/_Core/Permission/UserContext.php:132` | Getter or property; numeric conversion without positive-ID requirement |

Static counterexample: context `['actorId' => 41]` yields 41 in the ORM/realtime
helpers but null in assignment notifications. The latter uses that result to skip
self-assignment notifications at lines 41–45 and 73–77. Negative numeric IDs also
receive inconsistent treatment. This conclusion follows from source branches;
no runtime notification was sent to reproduce it.

`app/Support/ActorContext.php:13` already defines a positive-ID boundary, but no
production caller was found. It is untracked and has tests specifying integration.
Do not delete it as an orphan. Finish one declared contract for explicit actor,
user shape, and ambient-auth fallback, then migrate the live callers and remove
their duplicate extraction logic. A different action-context adapter may still be
justified; the input types and missing-actor behavior differ.

### 2. Replacement contracts exist beside unchanged consumers

**High-confidence incomplete integration, not proof that the new abstractions are useless.**

The same pattern affects these untracked files:

- `app/Services/_Core/Permission/AclMembershipWriter.php:17`
- `app/Services/Filter/EntitySortTargetResolver.php:20`
- `app/Services/_Core/Filter/EffectiveFilterTypeResolver.php:12`
- `app/Services/View/ViewColumnSliceResolverInterface.php:7`

Names appear in declarations and tests, not in production consumers. For the ACL
writer, the effect interface/runner exist, but no production registration naming
this implementation was found. For the view interface, no implementation naming
the interface was found. These are wiring gaps to reconcile with the concurrent
work, not permission to sweep the files away. Acceptance must cover the actual
caller or registered runtime path; a test of the isolated helper is insufficient.

### 3. There is a small, concrete removal lane

`app/Http/Controllers/Entity/Handlers/EntityViewHandler.php:1297` still declares
private `hydrateManyToManyForShow`, with no production use found. Its body owns a
second relation/pivot derivation path and converts query failures into empty data.
No same-class variable-method/reflection dispatch was found. Tests mention its name
as source-contract checks, not runtime callers. Removing this dead path is preferable
to refactoring its query logic.

Three imports are also unused in their own files:

- `ProjectStatusGroups` — `app/Modules/ProjectManagement/Entities/Project/Project.fields.php:4`
- `RuntimeMessage` — `app/Modules/ServiceManagement/Actions/ServiceTrip/CompleteFieldServiceTripAction.php:22`
- `RuntimeMessage` — `app/Modules/ServiceManagement/Services/ServiceTripRouteOptimizationService.php:15`

Each imported name occurs only on its import line in that file. Other users of the
class are irrelevant to removing these bindings. See the [complete candidate ledger](dead-code.md).

### 4. Two legacy controllers contradict their own removal contract

`AccountingBudgetController.php` and `AccountingBusinessUnitController.php` under
`app/Modules/Accounting/Http/Controllers/Api/` remain as untracked files, with no
production name references found. `AccountingWriteActionsContractTest.php:61–62`
explicitly requires their absence. That test also names the replacement metadata
actions, `RecalculateBusinessUnitBudgetsAction` and `UpdateBusinessUnitAssignmentAction`.

This is unusually strong retirement evidence. Reconcile the untracked files with
their author and remove the obsolete endpoints as part of the existing action
migration. No test was executed; this is a static contradiction with the test definition.

### 5. Production-evidence orchestration is still one 8,551-line class

`app/Modules/AccountingCompliance/Services/AccountingProductionEvidenceService.php`
combines requirements (`:401`), evidence evaluation (`:424`), template construction
(`:580`), artifact attachment/staging (`:638`, `:769`), command generation (`:869`),
report/template persistence (`:1037`, `:1053`), and path validation (`:8308` onward).

These are concrete ownership boundaries, not a conclusion based only on length.
First separate artifact storage/path handling from domain evidence validation;
keep one orchestration entrypoint and preserve the existing public contract. Avoid
creating a new plugin framework for every check. Its location is AccountingCompliance,
as in the July review; its current size is 8,551 lines versus the previously recorded 8,511.

### 6. Email retains a broad facade after several useful extractions

`app/Services/Email/EmailService.php` is 2,655 lines, with 56 non-magic public methods
and 27 dependent files in the configured graph. Existing collaborators already own
send, sync, access, assignment, threading, and body fetching (`:38–65`). But the facade
also owns compose selection (`:132`), resend/draft workflows (`:288`, `:302`), sending
orchestration (`:618`), attachment operations (`:551`), and serialization further down.

Continue along existing service boundaries; make callers depend on the narrow
owner they use. Do not mechanically split every forwarding method into another
class. `NativeImapService` also has a wide API, but a protocol adapter can legitimately
be broad: API width alone is not enough to condemn it.

### 7. Email UI concentrates several workflows and request states

`resources/js/Pages/Email/Index.vue` has 5,649 lines;
`resources/js/Components/Email/EmailPreview.vue` has 5,365. The page combines compose,
detail navigation, selection, bulk send, transfer, and sync operations. It already
imports useful extracted components/composables at `Index.vue:75–95`, yet still owns
detail requests (`:3139`, `:3146`), a transfer-target promise cache (`:3359`), and
inline sync (`:4363`). Preview owns another header request at `:745`.

Move remaining workflow/request ownership into the existing seams and canonical
query layer. Keep tenant/auth query keys and mutation invalidation together. This
is a maintainability finding; no browser defect or stale-response race was reproduced.

### 8. Chat mixes CRUD, generation, voice, tools, and telemetry

`app/Modules/AiChat/Services/ChatService.php` is 4,640 lines. Its public responsibilities
include conversation creation (`:243`), synchronous messages (`:881`), voice bootstrap
(`:1297`), live tool execution (`:1618`), telemetry (`:1860`, `:1931`), and streaming
(`:2510`). These are independent change drivers behind one service.

Use the existing provider, turn-policy, and tool contracts to narrow ownership.
Preserve the queue/tool gates documented by the module. Do not replace them with a
second orchestration framework or interpret their necessary registrations as sprawl.

### 9. Core/module boundary needs a declared extension contract

`ActivityHistoryLoader.php:7,130`, `CalendarConfigLoader.php:5,112`, and
`EntityRegistry.php:7,627`, all under `app/Entities/_Core/`, depend directly on
`App\Modules\ModuleRegistry`. This contradicts the machine doctrine's core→support-only
rule. Module contributions are required product behavior, so deleting discovery is
not an option.

Either formalize module discovery as an approved kernel contract or move contribution
enumeration behind an existing extension boundary. Model that decision explicitly
in doctrine. Likewise, module HTTP controllers using shared HTTP utilities are
misclassified by the broad `app/Modules` layer prefix; this needs policy refinement,
not an engine exception for Draivix filenames.

### 10. One small runtime cycle is worth untangling; several others are not

`entityCreateSubmitRegistry.ts:2` imports its client loader, while
`entityCreateSubmitClientLoader.ts:4` imports the registry's runtime normalizer.
Both live under `resources/js/Pages/Entity/composables/`. Put shared normalization
and types in one independent module so the loader and registry need not import
each other's runtime implementation. No initialization failure was reproduced.

By contrast, `entityFormClientExtensionLoader.ts:2` imports only a **type** from its
registry. The analyzer still puts that pair in its strong dependency groups. That
is not evidence of a runtime cycle. Similarly, Email job→service→job dispatch groups
are ordinary asynchronous orchestration, and `Email.php:36` deliberately registers
its audit enricher through metadata. Do not spend effort eliminating those solely
because an SCC exists.

## What the raw findings do not prove

The configured scan reports 17 strong SCCs, 318 layer findings, 234 sanctioned-path
findings, 11 god-class candidates, and 1,053 complexity hotspots. These are leads,
not that many verified defects. The largest reported strong SCC contains 45 files
and 1,912 internal evidence edges; those are not 1,912 unique file dependencies.
Neither its full membership nor every edge was independently validated.

Concrete counterexamples found during review:

- `ConversionLoader.php:164–169` decodes a different definition in each iteration.
  Hoisting that decode outside the loop would change behavior, not improve it.
- `ActivityHistoryLoader.php:506` decodes each row's own configuration; its map is
  cached afterward. The scanner marker does not prove redundant repeated work.
- `ApiResponse.php:11–15` now explicitly owns success envelopes; errors belong to
  `bootstrap/app.php:170–171`. The controller AGENTS warning about competing error
  envelopes is stale for this implementation.
- `ProductCategoryProvider.php` is deliberately abstract and unregistered pending a
  port, as documented by `app/Modules/Pohoda/config/pohoda.php:21–23`. It is an owner
  decision, not an immediately disposable implementation.
- Registries, service providers, metadata loaders, and the generic entity manager
  have legitimate fan-in. AigisCode's naming/width heuristics cannot independently
  decide whether those abstractions are necessary.

No external security analyzers were run. The 38 native security findings were not
validated as vulnerabilities in this architecture review.

## AigisCode outcome and verification

See [AigisCode changes and remaining limits](aigiscode.md). Two detector defects
were addressed: cross-language launcher blindness and test references masking
unwired production code. Heuristic frontend findings no longer claim `safe_delete`.
Regression tests were added using parsed source fixtures and real graph resolution.

The release CLI was built and exercised on both repositories and the broader
snapshot. Automated tests and lint/type checks were not run locally. This checkout
has a GitHub origin and GitHub workflow; no accessible AigisCode GitLab project or
pipeline was found. **GitLab CI validation remains outstanding.** Existing unrelated
work and `.plan/WORK_LOCK` were preserved.
