# Dead-code candidate ledger

Companion to [architecture review](README.md). All paths relative to Draivix.

Configured scan: **32 findings = 1 private method + 3 imports + 28 orphan modules**.
Every row was investigated with source and caller searches. This is not a list of
32 authorized deletions. Runtime class construction, metadata outside the snapshot,
and unfinished work remain material limits.

Private methods/imports were checked within their declaring file. Orphan names,
module paths and relevant exports were searched across app/resources/routes/config/
bootstrap/clients/tests/tools/database/apps. Test references are recorded separately;
negative assertions and fixtures are not production callers. Framework factories,
Pohoda provider mapping, widget manifests and the flagged replacement contracts
received additional inspection. PHPStan cache files are not runtime consumers.

## Local removals

| Finding | Source | Evidence |
|---|---|---|
| UnusedPrivateFunction: `hydrateManyToManyForShow` | `app/Http/Controllers/Entity/Handlers/EntityViewHandler.php:1297` | No production reference or same-class dynamic dispatch found; test mentions are source-contract checks. |
| UnusedImport: `ProjectStatusGroups` | `app/Modules/ProjectManagement/Entities/Project/Project.fields.php:4` | Only occurrence of imported binding in this file is the import line. |
| UnusedImport: `RuntimeMessage` | `app/Modules/ServiceManagement/Actions/ServiceTrip/CompleteFieldServiceTripAction.php:22` | Only occurrence of imported binding in this file is the import line. |
| UnusedImport: `RuntimeMessage` | `app/Modules/ServiceManagement/Services/ServiceTripRouteOptimizationService.php:15` | Only occurrence of imported binding in this file is the import line. |

## Orphan modules

File sizes include comments and declarations; they are not estimates of removable code.
Test mentions count matched source lines, not tests executed or coverage.
Test definitions were inspected; no tests were executed.

| Candidate and path | File lines | Test mentions | Review verdict |
|---|---:|---:|---|
| `app/Modules/Accounting/Http/Controllers/Api/AccountingBudgetController.php` | 32 | 1 | Retire with existing action migration; test explicitly requires file absence. Untracked. |
| `app/Modules/Accounting/Http/Controllers/Api/AccountingBusinessUnitController.php` | 108 | 1 | Retire with existing action migration; test explicitly requires file absence. Untracked. |
| `app/Modules/Accounting/Services/TaskInvoiceDraftService.php` | 326 | 3 | Wire or retire deliberately: workflow with integration tests but no production name references found. |
| `app/Modules/Accounting/Support/InvoiceLifecycleStatusGroups.php` | 60 | 3 | Review retirement after status consolidation; still exercised by tests. |
| `app/Modules/AccountingCompliance/Services/ArchiveHashChainService.php` | 63 | 5 | Untracked implementation with unit tests; decide integration before deletion. |
| `app/Modules/AiChat/Support/PromptTesting/AiChatVoiceSmokeScenarioCatalog.php` | 133 | 5 | Test/support catalog; production orphan does not make test tooling unnecessary. |
| `app/Modules/ETL/Services/SourceContractFingerprint.php` | 70 | 5 | Test-only utility by observed callers; decide whether contract fingerprinting still needs integration. |
| `app/Modules/ETL/Support/ImportRunProgressStep.php` | 20 | 0 | No callers found; review enum retirement. |
| `app/Modules/EspoIntegration/Services/RecordChangeClassifier.php` | 91 | 3 | Test-only by observed callers; decide whether classification still needs integration. |
| `app/Modules/Planning/Services/DemandCoverageReadModel.php` | 304 | 5 | Read model with tests but no production callers found; wire or retire at module boundary. |
| `app/Modules/Pohoda/Tools/Pohoda/Providers/PaymentOrderProvider.php` | 156 | 2 | Unregistered provider with test references; verify provider capability decision before removal. |
| `app/Modules/Pohoda/Tools/Pohoda/Providers/ProductCategoryProvider.php` | 203 | 0 | Keep pending owner decision: explicitly abstract and unregistered pending a port (config/pohoda.php:21–23). |
| `app/Modules/Pohoda/Tools/Pohoda/Providers/SupplierQuoteProvider.php` | 25 | 0 | Unregistered provider; review capability retirement rather than bulk provider deletion. |
| `app/Services/Action/ActionActorResolver.php` | 52 | 7 | Has tests but no production callers found. Distinct ActionContext contract; not a blind merge into array-context helper. |
| `app/Services/Filter/EntitySortTargetResolver.php` | 419 | 2 | Untracked replacement with tests; finish caller migration rather than delete. |
| `app/Services/View/ViewColumnSliceResolverInterface.php` | 19 | 5 | Untracked contract with no production implementation found; reconcile intended integration. |
| `app/Services/_Core/Filter/EffectiveFilterTypeResolver.php` | 64 | 9 | Untracked replacement with tests; finish caller migration rather than delete. |
| `app/Services/_Core/Permission/AclMembershipWriter.php` | 94 | 9 | Untracked effect handler; no production registration naming it found. Reconcile ACL migration. |
| `app/Services/_Core/StateMachine/OutboxTransitionReplayResolver.php` | 399 | 4 | Replay contract with tests but no production callers found; verify intended idempotency integration. |
| `app/Support/ActorContext.php` | 41 | 16 | Untracked consolidation target; live duplicate extraction remains. Wire, do not sweep away. |
| `resources/js/Components/Dashboard/configurators/ChartConfig.vue` | 279 | 12 | No module importer found; other ChartConfig occurrences are a local type in ChartView.vue. Review chart migration and tests. |
| `resources/js/Components/Dashboard/widgets/ChartWidget.vue` | 68 | 10 | No module importer found; WorkloadChartWidget is a different registered widget. Review chart migration and tests. |
| `resources/js/Components/Entity/EntityWorkspaceInlineTable.vue` | 508 | 0 | No import/path consumer found; modified file, so coordinate retirement with ongoing work. |
| `resources/js/Components/Entity/workspaceInlineEditing.ts` | 36 | 0 | Both exported functions have no external production references found; removal candidate. |
| `resources/js/composables/useActiveCurrencyOptions.ts` | 66 | 6 | No production callers found; review tests and current currency-options owner before removal. |
| `resources/js/composables/useAuthContextKey.ts` | 14 | 0 | No callers found; small unused wrapper over existing runtime-context helper. |
| `resources/js/composables/useEntityOptions.ts` | 94 | 3 | No production callers found; useEntityOptionsSearch is a different, live composable. Review tests before removal. |
| `resources/js/contracts/entity/actionCapabilities.ts` | 20 | 0 | No external production references found for exported resolver/types; removal candidate. |

## Why old deletion lists are unsafe

The July list is historical. Current scan has a different source corpus and
substantial concurrent changes. Finding counts and public interfaces changed.
AutomaticInvoiceRecalculationService remains a known dynamic-dispatch
lesson, not a deletion target. No source was removed during this review.
