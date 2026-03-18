# AigisCode Report

**Project**: `/home/david/Work/Programming/newerp`
**Generated**: 2026-03-09 10:30:41
**aigiscode v0.1.0**

## Overview

| Metric | Value |
|--------|-------|
| Files indexed | 5392 |
| Symbols extracted | 27437 |
| Dependencies found | 21175 |
| Semantic envelopes | 0 |

### Language Breakdown

| Language | Files |
|----------|-------|
| php | 4314 |
| typescript | 596 |
| vue | 433 |
| python | 45 |
| javascript | 4 |

## Executive Summary

The codebase contains 5392 source files with 27437 symbols and 21175 dependencies. Analysis found: 22 circular dependencies, 290 god classes, 21 layer violations, 543 potentially dead files, 986 runtime entry candidates, 243 external analyzer findings. See the detailed sections below for specifics.

## Architecture Health

**Graph**: 5392 nodes, 9566 edges, density=0.000329

### Strong Circular Dependencies (22)

1. `app/Services/_Core/Audit/ActionHistoryService.php -> app/Services/_Core/ImpersonationService.php -> app/Models/User.php -> app/Services/_Core/TenantManager.php -> app/Entities/_Core/EntityManager.php -> app/Entities/_Core/Cycle/HookAwareMapper.php -> app/Services/_Core/Audit/ActionHistoryService.php`
2. `app/Services/_Core/Audit/ActionHistoryService.php -> app/Services/_Core/ImpersonationService.php -> app/Models/User.php -> app/Services/_Core/TenantManager.php -> app/Entities/_Core/EntityManager.php -> app/Services/_Core/Audit/ActionHistoryService.php`
3. `app/Services/_Core/PermissionCache.php -> app/Models/User.php -> app/Services/_Core/PermissionCache.php`
4. `app/Entities/_Core/EntityManager.php -> app/Entities/_Core/Cycle/HookAwareMapper.php -> app/Services/_Core/Events/OutboxPublisher.php -> app/Services/_Core/TenantManager.php -> app/Entities/_Core/EntityManager.php`
5. `app/Entities/_Core/EntityManager.php -> app/Entities/_Core/Cycle/HookAwareMapper.php -> app/Services/_Core/Realtime/EntityBroadcaster.php -> app/Services/_Core/TenantManager.php -> app/Entities/_Core/EntityManager.php`
6. `app/Entities/_Core/EntityManager.php -> app/Services/_Core/Events/OutboxPublisher.php -> app/Services/_Core/TenantManager.php -> app/Entities/_Core/EntityManager.php`
7. `app/Entities/_Core/EntityManager.php -> app/Services/_Core/Realtime/EntityBroadcaster.php -> app/Services/_Core/TenantManager.php -> app/Entities/_Core/EntityManager.php`
8. `app/Entities/_Core/EntityManager.php -> app/Services/_Core/RelationBatchLoader.php -> app/Entities/_Core/EntityManager.php`
9. `docker/agent/mcp/adsmcp/src/unified_ads_mcp/meta/insights.py -> docker/agent/mcp/adsmcp/src/unified_ads_mcp/server.py -> docker/agent/mcp/adsmcp/src/unified_ads_mcp/meta/insights.py`
10. `docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/ads.py -> docker/agent/mcp/adsmcp/src/unified_ads_mcp/server.py -> docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/ads.py`
11. `docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/campaigns.py -> docker/agent/mcp/adsmcp/src/unified_ads_mcp/server.py -> docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/campaigns.py`
12. `docker/agent/mcp/adsmcp/src/unified_ads_mcp/server.py -> docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/reporting.py -> docker/agent/mcp/adsmcp/src/unified_ads_mcp/server.py`
13. `docker/agent/mcp/adsmcp/src/unified_ads_mcp/server.py -> docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/ad_groups.py -> docker/agent/mcp/adsmcp/src/unified_ads_mcp/server.py`
14. `docker/agent/mcp/adsmcp/src/unified_ads_mcp/server.py -> docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/keywords.py -> docker/agent/mcp/adsmcp/src/unified_ads_mcp/server.py`
15. `docker/agent/mcp/adsmcp/src/unified_ads_mcp/server.py -> docker/agent/mcp/adsmcp/src/unified_ads_mcp/meta/campaigns.py -> docker/agent/mcp/adsmcp/src/unified_ads_mcp/server.py`
16. `app/Services/Email/EmailSyncManager.php -> app/Jobs/Email/SyncAccountJob.php -> app/Services/Email/EmailSyncManager.php`
17. `app/Services/Email/EmailSyncManager.php -> app/Jobs/Email/SyncFolderJob.php -> app/Services/Email/EmailSyncManager.php`
18. `app/Entities/_Core/Entity.php -> app/Entities/_Core/Concerns/HasEvolvableRelations.php -> app/Entities/_Core/Entity.php`
19. `app/Services/_Core/Permission/ScopeResolver.php -> app/Services/_Core/Permission/Policy/PolicyEvaluator.php -> app/Services/_Core/Permission/ScopeResolver.php`
20. `resources/js/composables/useInlineEdit.ts -> resources/js/composables/inlineEditExtensionRegistry.ts -> resources/js/composables/useInlineEdit.ts`

*... and 2 more cycles*

### Layer Violations (21)

| Source | Source Layer | Target | Target Layer | Violation |
|--------|-------------|--------|-------------|-----------|
| `app/Models/ScheduledJob.php` | Model | `app/Services/Settings/JobRegistry.php` | Service | Model -> Service (reversed dependency) |
| `app/Models/User.php` | Model | `app/Services/_Core/Concerns/DbalRowAccess.php` | Service | Model -> Service (reversed dependency) |
| `app/Models/User.php` | Model | `app/Services/_Core/PermissionCache.php` | Service | Model -> Service (reversed dependency) |
| `app/Models/User.php` | Model | `app/Services/_Core/TenantDb.php` | Service | Model -> Service (reversed dependency) |
| `app/Models/User.php` | Model | `app/Services/_Core/TenantManager.php` | Service | Model -> Service (reversed dependency) |
| `app/Modules/AiChat/Services/Tools/CountEntityTool.php` | Service | `app/Http/Controllers/Api/EntityApiController.php` | Controller | Service -> Controller (reversed dependency) |
| `app/Modules/AiChat/Services/Tools/DeleteEntityTool.php` | Service | `app/Http/Controllers/Api/EntityApiController.php` | Controller | Service -> Controller (reversed dependency) |
| `app/Modules/AiChat/Services/Tools/ExecuteEntityActionTool.php` | Service | `app/Http/Controllers/Api/EntityActionController.php` | Controller | Service -> Controller (reversed dependency) |
| `app/Modules/AiChat/Services/Tools/LinkRelationTool.php` | Service | `app/Http/Controllers/Api/EntityApiController.php` | Controller | Service -> Controller (reversed dependency) |
| `app/Modules/AiChat/Services/Tools/LookupFieldValuesTool.php` | Service | `app/Http/Controllers/Api/EntityLookupController.php` | Controller | Service -> Controller (reversed dependency) |
| `app/Modules/AiChat/Services/Tools/ReadRelationTool.php` | Service | `app/Http/Controllers/Api/EntityApiController.php` | Controller | Service -> Controller (reversed dependency) |
| `app/Modules/AiChat/Services/Tools/SyncRelationTool.php` | Service | `app/Http/Controllers/Api/EntityApiController.php` | Controller | Service -> Controller (reversed dependency) |
| `app/Modules/AiChat/Services/Tools/UnlinkRelationTool.php` | Service | `app/Http/Controllers/Api/EntityApiController.php` | Controller | Service -> Controller (reversed dependency) |
| `app/Services/Action/ActionRouteRegistry.php` | Service | `app/Http/Middleware/VerifyCsrfTokenUnlessBearerAuth.php` | Middleware | Service -> Middleware (reversed dependency) |
| `database/migrations/tenant/2026_03_08_000006_backfill_missing_detail_layouts.php` | Migration | `app/Services/_Core/LayoutNormalizer.php` | Service | Migration -> Service (reversed dependency) |
| `database/migrations/tenant/2026_03_08_000007_backfill_aicost_emailtemplate_detail_layouts.php` | Migration | `app/Services/_Core/LayoutNormalizer.php` | Service | Migration -> Service (reversed dependency) |
| `tests/Core/Feature/Services/Tenant/TenantResolverMiddlewareTest.php` | Service | `app/Http/Middleware/TenantResolver.php` | Middleware | Service -> Middleware (reversed dependency) |
| `tests/Unit/Services/AiChat/Tools/EntityApiBridgeToolsTest.php` | Service | `app/Http/Controllers/Api/EntityApiController.php` | Controller | Service -> Controller (reversed dependency) |
| `tests/Unit/Services/AiChat/Tools/EntityApiBridgeToolsTest.php` | Service | `app/Http/Controllers/Api/EntityActionController.php` | Controller | Service -> Controller (reversed dependency) |
| `tests/Unit/Services/AiChat/Tools/EntityApiBridgeToolsTest.php` | Service | `app/Http/Controllers/Api/EntityLookupController.php` | Controller | Service -> Controller (reversed dependency) |
| `tests/Unit/Services/Tenant/TenantResolverTest.php` | Service | `app/Http/Middleware/TenantResolver.php` | Middleware | Service -> Middleware (reversed dependency) |

### Module Coupling (top 15 most unstable)

| Module | Afferent (Ca) | Efferent (Ce) | Instability (I) |
|--------|---------------|---------------|-----------------|
| `app/Hooks` | 0 | 2 | 1.0 |
| `app/Providers` | 0 | 7 | 1.0 |
| `config` | 0 | 4 | 1.0 |
| `database/factories` | 0 | 1 | 1.0 |
| `database/seeders` | 0 | 3 | 1.0 |
| `scripts` | 0 | 4 | 1.0 |
| `server.php` | 0 | 1 | 1.0 |
| `tests/Core` | 0 | 8 | 1.0 |
| `tests/Fast` | 0 | 1 | 1.0 |
| `tests/Feature` | 0 | 11 | 1.0 |
| `tests/Integration` | 0 | 9 | 1.0 |
| `tests/Modules` | 0 | 5 | 1.0 |
| `tests/Performance` | 0 | 5 | 1.0 |
| `tests/Unit` | 0 | 16 | 1.0 |
| `app/Console` | 2 | 8 | 0.8 |

## Code Quality

### God Classes (290)

| Class | File | Methods | Dependencies | Lines |
|-------|------|---------|-------------|-------|
| `Email` | `app/Entities/Email/Email.php` | 109 | 4 | 986 |
| `SalesOrder` | `app/Modules/Accounting/Entities/SalesOrder/SalesOrder.php` | 68 | 3 | 577 |
| `ImportRun` | `app/Modules/ETL/Entities/ImportRun/ImportRun.php` | 65 | 2 | 596 |
| `MigrationImportDataCommand` | `app/Console/Commands/MigrationImportDataCommand.php` | 58 | 29 | 2616 |
| `EntityRegistry` | `app/Entities/_Core/EntityRegistry.php` | 58 | 11 | 1558 |
| `ChatService` | `app/Modules/AiChat/Services/ChatService.php` | 57 | 13 | 2244 |
| `EmailAccount` | `app/Entities/EmailAccount/EmailAccount.php` | 56 | 2 | 570 |
| `ImportDefinition` | `app/Modules/ETL/Entities/ImportDefinition/ImportDefinition.php` | 53 | 2 | 473 |
| `WebsiteContactProfile` | `app/Modules/Website/Entities/WebsiteContactProfile/WebsiteContactProfile.php` | 53 | 4 | 594 |
| `NativeImapService` | `app/Services/Email/NativeImapService.php` | 51 | 3 | 1308 |
| `Template` | `app/Modules/Documents/Entities/Template/Template.php` | 49 | 2 | 508 |
| `Cart` | `app/Modules/Website/Entities/Cart/Cart.php` | 47 | 3 | 417 |
| `Shipment` | `app/Modules/Accounting/Entities/Shipment/Shipment.php` | 44 | 3 | 434 |
| `Page` | `app/Modules/Website/Entities/Page/Page.php` | 44 | 4 | 403 |
| `EntityManager` | `app/Entities/_Core/EntityManager.php` | 43 | 27 | 1010 |
| `EmailDelegation` | `app/Entities/EmailDelegation/EmailDelegation.php` | 40 | 2 | 412 |
| `EmailService` | `app/Services/Email/EmailService.php` | 40 | 10 | 666 |
| `EmailActionQueue` | `app/Entities/EmailActionQueue/EmailActionQueue.php` | 39 | 2 | 452 |
| `EmailAccessService` | `app/Services/Email/EmailAccessService.php` | 39 | 12 | 1185 |
| `AbstractProvider` | `app/Modules/Pohoda/Tools/Pohoda/Providers/Abstract/AbstractProvider.php` | 38 | 19 | 939 |

### Bottleneck Files (top 10)

Files with highest betweenness centrality (changes have the widest blast radius):

| File | Centrality Score |
|------|-----------------|
| `tests/Support/EntityRegistryCacheWarmup.php` | 0.0011 |
| `bootstrap/app.php` | 0.0009 |
| `app/Entities/_Core/EntityManager.php` | 0.0009 |
| `tests/TestCase.php` | 0.0009 |
| `app/Entities/_Core/Entity.php` | 0.0005 |
| `app/Http/Middleware/HandleInertiaRequests.php` | 0.0005 |
| `tests/TestCase/TenantTestCase.php` | 0.0004 |
| `app/Entities/_Core/EntityRegistry.php` | 0.0003 |
| `app/Services/_Core/PermissionChecker.php` | 0.0003 |
| `app/Services/_Core/Audit/ActionHistoryService.php` | 0.0002 |

### Likely Orphan Files (543 files)

Files with outgoing dependencies but zero incoming dependencies, excluding known runtime entry candidates:

- `app/Entities/AppLogRecord/AppLogRecord.php`
- `app/Entities/AuthLogRecord/AuthLogRecord.php`
- `app/Entities/Calendar/Calendar.fields.php`
- `app/Entities/Calendar/Calendar.php`
- `app/Entities/CalendarPermission/CalendarPermission.php`
- `app/Entities/CalendarSubscription/CalendarSubscription.php`
- `app/Entities/RoleMenuDefault/RoleMenuDefault.php`
- `app/Entities/_Core/Cycle/CustomTypecastHandler.php`
- `app/Events/Chat/ToolComplete.php`
- `app/Events/Chat/ToolStart.php`
- `app/Modules/Accounting/Actions/PurchaseOrder/AddProducts.php`
- `app/Modules/Accounting/Actions/PurchaseOrder/GetWarehouseForItemsLegacyAction.php`
- `app/Modules/Accounting/Actions/PurchaseOrder/RemoveGoodsRestockItems.php`
- `app/Modules/Accounting/Actions/PurchaseOrder/RevertToOrderedLegacyAction.php`
- `app/Modules/Accounting/Entities/AutomaticInvoiceItem/AutomaticInvoiceItem.php`
- `app/Modules/Accounting/Entities/BusinessUnit/BusinessUnit.php`
- `app/Modules/Accounting/Entities/Commission/Commission.php`
- `app/Modules/Accounting/Entities/Discount/Discount.php`
- `app/Modules/Accounting/Entities/ExpenseReceipt/ExpenseReceipt.fields.php`
- `app/Modules/Accounting/Entities/ExpenseReceipt/ExpenseReceipt.php`
- `app/Modules/Accounting/Entities/OrderTransfer/OrderTransfer.php`
- `app/Modules/Accounting/Entities/Payment/Payment.relations.php`
- `app/Modules/Accounting/Entities/PerformanceRate/PerformanceRate.php`
- `app/Modules/Accounting/Entities/RevenueReceipt/RevenueReceipt.fields.php`
- `app/Modules/Accounting/Entities/RevenueReceipt/RevenueReceipt.php`
- `app/Modules/Accounting/Entities/SummaryVatRate/SummaryVatRate.php`
- `app/Modules/Accounting/Entities/SupplierQuote/SupplierQuote.php`
- `app/Modules/Accounting/Entities/SupplierQuoteItem/SupplierQuoteItem.php`
- `app/Modules/Accounting/Services/AccountingDuzpSettings.php`
- `app/Modules/Accounting/resources/js/Pages/Invoice/Create.vue`

*... and 513 more*

### Runtime Entry Candidates (986 files)

Files with zero incoming dependencies that appear to be loader-driven or configured entrypoints:

- `app/Actions/_Core/Kanban/PutOrderAction.php`
- `app/Console/Commands/AclCacheCommand.php`
- `app/Console/Commands/CacheWarmCommand.php`
- `app/Console/Commands/CleanupDeletedRecordsCommand.php`
- `app/Console/Commands/CleanupJobLogsCommand.php`
- `app/Console/Commands/ErpAnalyzeComplexityCommand.php`
- `app/Console/Commands/ErpAnalyzeCoverageCommand.php`
- `app/Console/Commands/ErpAnalyzeDependenciesCommand.php`
- `app/Console/Commands/ErpBuildCommand.php`
- `app/Console/Commands/ErpCreateDbEntityCommand.php`
- `app/Console/Commands/ErpCreateTenantCommand.php`
- `app/Console/Commands/ErpDebugAclCommand.php`
- `app/Console/Commands/ErpDebugConversionsCommand.php`
- `app/Console/Commands/ErpDebugEntityCommand.php`
- `app/Console/Commands/ErpDebugHooksCommand.php`
- `app/Console/Commands/ErpDebugTraceCommand.php`
- `app/Console/Commands/ErpDefinitionsCommand.php`
- `app/Console/Commands/ErpDevSeedCommand.php`
- `app/Console/Commands/ErpDocsApiCommand.php`
- `app/Console/Commands/ErpDocsEntityCommand.php`
- `app/Console/Commands/ErpDocsModuleCommand.php`
- `app/Console/Commands/ErpEntityRemoveCommand.php`
- `app/Console/Commands/ErpGenerateTypesCommand.php`
- `app/Console/Commands/ErpI18nAuditCommand.php`
- `app/Console/Commands/ErpI18nCoverageCommand.php`
- `app/Console/Commands/ErpI18nLeaksCommand.php`
- `app/Console/Commands/ErpLintEntityCommand.php`
- `app/Console/Commands/ErpLintPermissionsCommand.php`
- `app/Console/Commands/ErpLintRelationsCommand.php`
- `app/Console/Commands/ErpMakeFieldCommand.php`

*... and 956 more*

## Dead Code Analysis (0 findings)

No dead code detected.

## Hardwiring Analysis (0 findings)

No hardwiring issues detected.

## Extensions

### contract_inventory

```json
{
  "summary": {}
}
```

## External Analysis

| Tool | Status | Findings | Artifact |
|------|--------|----------|----------|
| ruff | findings | 243 | `reports/newerp/reports/20260309_103041/raw/ruff-security.json` |

### External Findings (243)

| Tool | Domain | Severity | File | Rule | Message |
|------|--------|----------|------|------|---------|
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/auth/google_auth.py:44` | `S105` | Possible hardcoded password assigned to: "GOOGLE_TOKEN_URL" |
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/auth/meta_auth.py:34` | `S105` | Possible hardcoded password assigned to: "META_TOKEN_URL" |
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/ad_groups.py:150` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/ads.py:161` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/asset_groups.py:70` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/asset_groups.py:145` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/asset_groups.py:183` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/asset_groups.py:663` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/campaigns.py:173` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/keywords.py:158` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/reporting.py:166` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/reporting.py:271` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/reporting.py:373` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/reporting.py:472` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/google/reporting.py:569` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | high | `docker/agent/mcp/adsmcp/src/unified_ads_mcp/meta/adsets.py:398` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | medium | `tests_fast/test_smoke.py:2` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tmp/mattermost-plugin-calls/test_4user_call.py:476` | `S110` | `try`-`except`-`pass` detected, consider logging the exception |
| ruff | security | high | `tmp/mattermost-plugin-calls/test_calls_backend.py:18` | `S105` | Possible hardcoded password assigned to: "USER1_TOKEN" |
| ruff | security | high | `tmp/mattermost-plugin-calls/test_calls_backend.py:19` | `S105` | Possible hardcoded password assigned to: "USER2_TOKEN" |
| ruff | security | medium | `tmp/mattermost-plugin-calls/test_calls_backend.py:38` | `S113` | Probable use of `requests` call without timeout |
| ruff | security | medium | `tmp/mattermost-plugin-calls/test_calls_backend.py:40` | `S113` | Probable use of `requests` call without timeout |
| ruff | security | medium | `tmp/mattermost-plugin-calls/test_calls_backend.py:218` | `S113` | Probable use of `requests` call without timeout |
| ruff | security | high | `tmp/mattermost-plugin-calls/test_webrtc_full.py:46` | `S105` | Possible hardcoded password assigned to: "USER1_TOKEN" |
| ruff | security | high | `tmp/mattermost-plugin-calls/test_webrtc_full.py:47` | `S105` | Possible hardcoded password assigned to: "USER2_TOKEN" |
| ruff | security | high | `tmp/mattermost-plugin-calls/test_ws_call.py:15` | `S105` | Possible hardcoded password assigned to: "USER1_TOKEN" |
| ruff | security | medium | `tools/_archive/server-satellite-v1/config.py:15` | `S104` | Possible binding to all interfaces |
| ruff | security | high | `tools/_archive/server-satellite-v1/domain_manager.py:44` | `S603` | `subprocess` call: check for execution of untrusted input |
| ruff | security | high | `tools/satellite/satellite/builtins/sync_domain.py:572` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | high | `tools/satellite/satellite/builtins/sync_domain.py:578` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | medium | `tools/satellite/satellite/builtins/sync_domain.py:657` | `S311` | Standard pseudo-random generators are not suitable for cryptographic purposes |
| ruff | security | high | `tools/satellite/satellite/queue.py:232` | `S608` | Possible SQL injection vector through string-based query construction |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:146` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:149` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:152` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:155` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:164` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:170` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:176` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:182` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:188` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:193` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:220` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:221` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:224` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:225` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:226` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:231` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:232` | `S101` | Use of `assert` detected |
| ruff | security | medium | `tools/satellite/tests/test_executor.py:233` | `S101` | Use of `assert` detected |

*... and 193 more*

## Recommendations

1. **Break Circular Dependencies**: Found 22 dependency cycle(s). Circular dependencies make the codebase hard to test, refactor, and reason about. Start by extracting shared interfaces or introducing an event system to decouple the most entangled modules.
2. **Refactor God Classes**: Found 290 oversized classes. The worst offender is `Email` in `app/Entities/Email/Email.php` with 109 methods. Consider extracting responsibilities into dedicated service classes.
3. **Fix Layer Violations**: Found 21 architectural layer violations. Lower layers (Models, Repositories) should not depend on higher layers (Controllers, Views). Introduce dependency inversion or event dispatching.
4. **Reduce Coupling on Bottleneck Files**: The file `tests/Support/EntityRegistryCacheWarmup.php` has the highest betweenness centrality (0.0011), meaning changes to it affect the most other files. Consider breaking it into smaller, more focused modules.
5. **Review Potentially Dead Code**: Found 543 files with no incoming dependencies. These exclude files already classified as runtime entry candidates, so the remaining list is a stronger signal for potentially unused code.
6. **Audit Runtime Entry Surfaces**: Found 986 files that look like runtime entrypoints or loader-driven modules. Consider encoding these patterns in policy or plugins if they represent stable framework conventions.
7. **Triage External Analyzer Findings**: Imported 243 findings from external tools: 243 security-oriented and 0 quality-oriented. Use the archived raw artifacts for tool-specific remediation details.

---
*Generated by aigiscode v0.1.0*