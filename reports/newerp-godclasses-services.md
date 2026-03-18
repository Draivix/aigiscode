# NewERP God Classes: Services Analysis

Report generated from AigisCode analysis (`aigiscode-report.json`).

Scope: all god classes where the file path contains a `/Services/` directory segment.

## 1. Summary Statistics

| Metric | Value |
|---|---|
| Total god classes in project | 284 |
| God classes in `/Services/` | 126 (44.4%) |
| In `app/Services/` (core) | 61 |
| In `app/Modules/*/Services/` (module) | 65 |
| Average methods | 20.8 |
| Median methods | 18 |
| Average dependencies | 7.3 |
| Average lines | 594 |
| Median lines | 516 |
| Total lines across all | 74,901 |

Nearly half of all god classes live in Service directories. The email subsystem and AiChat module are the most concentrated areas.

## 2. Top 10 Worst Offenders

Ranked by composite score: `methods * (1 + ln(deps + 1)) * lines / 1000`. This weights method count, dependency fan-out, and file size together.

| Rank | Class | File | Methods | Deps | Lines | Score |
|---|---|---|---|---|---|---|
| 1 | ChatService | app/Modules/AiChat/Services/ChatService.php | 114 | 24 | 3933 | 1892 |
| 2 | EntityPanelService | app/Services/Entity/EntityPanelService.php | 42 | 13 | 1484 | 227 |
| 3 | EmailAccessService | app/Services/Email/EmailAccessService.php | 39 | 12 | 1185 | 165 |
| 4 | NativeImapService | app/Services/Email/NativeImapService.php | 51 | 3 | 1308 | 159 |
| 5 | FilterQueryBuilder | app/Services/_Core/Filter/FilterQueryBuilder.php | 39 | 10 | 1148 | 152 |
| 6 | EntityRelationService | app/Services/Entity/EntityRelationService.php | 32 | 10 | 1017 | 111 |
| 7 | EmailService | app/Services/Email/EmailService.php | 41 | 12 | 705 | 103 |
| 8 | MockDataSeeder | app/Services/_Core/MockDataSeeder.php | 25 | 28 | 912 | 100 |
| 9 | MenuService | app/Services/Menu/MenuService.php | 32 | 7 | 988 | 97 |
| 10 | EmailSendService | app/Services/Email/EmailSendService.php | 27 | 20 | 839 | 92 |

`ChatService` is an extreme outlier: 114 methods, 24 dependencies, 3933 lines. Its composite score is 8x higher than the second-place class.

## 3. Grouping by Domain

### 3.1 Core Services (`app/Services/`)

| Domain | Count | Avg Methods | Avg Deps | Avg Lines |
|---|---|---|---|---|
| _Core (top-level) | 9 | 24.7 | 12.3 | 575 |
| Email | 13 | 26.5 | 9.6 | 781 |
| Entity | 7 | 22.1 | 8.9 | 740 |
| _Core/Filter | 4 | 25.2 | 6.0 | 672 |
| _Core/Permission | 4 | 21.0 | 5.2 | 530 |
| Menu | 4 | 19.8 | 4.5 | 584 |
| Storage | 3 | 19.0 | 8.3 | 523 |
| Calendar | 3 | 16.7 | 6.0 | 572 |
| View | 4 | 15.2 | 1.2 | 312 |
| Other (5 domains, 1 each) | 5 | 14.8 | 6.2 | 449 |

The Email domain dominates: 13 god classes averaging 781 lines. The `_Core` top-level services have the highest average dependency count (12.3), indicating they act as integration hubs.

### 3.2 Module Services (`app/Modules/*/Services/`)

| Module | Count | Avg Methods | Avg Deps | Avg Lines |
|---|---|---|---|---|
| AiChat | 16 | 24.8 | 9.1 | 778 |
| Documents | 4 | 25.0 | 2.8 | 949 |
| Dashboard | 3 | 24.0 | 5.0 | 587 |
| Automation | 3 | 22.7 | 3.3 | 641 |
| Website | 3 | 21.7 | 7.7 | 600 |
| WarehouseMobile | 3 | 17.7 | 3.7 | 600 |
| Onboarding | 3 | 16.3 | 2.3 | 457 |
| DataBox | 3 | 18.0 | 10.7 | 401 |
| Mattermost | 3 | 18.3 | 4.7 | 446 |
| ETL | 3 | 16.7 | 9.0 | 483 |
| Other (16 modules, 1-2 each) | 18 | 14.6 | 6.8 | 415 |

AiChat has the largest concentration (16 god classes) driven by the ChatService mega-class plus 9 tool classes and 4 provider classes.

## 4. Classification: True God Classes vs Acceptable Aggregation

### 4.1 True God Classes (high confidence, refactoring warranted)

These have high method counts, high dependency fan-out, and large line counts. They violate single-responsibility and are structurally entangled.

| Class | Methods | Deps | Lines | Reasoning |
|---|---|---|---|---|
| ChatService | 114 | 24 | 3933 | Extreme outlier on every metric. Orchestrates AI providers, tools, message handling, streaming, context assembly, and error recovery in one class. Should split into at least ChatOrchestrator, MessageBuilder, ToolDispatcher, StreamHandler. |
| EntityPanelService | 42 | 13 | 1484 | General-purpose UI data assembly for all entity types. Mixes layout logic, permission checks, relation loading, and data formatting. |
| NativeImapService | 51 | 3 | 1308 | Low deps but 51 methods and 1308 lines: classic "do everything" IMAP wrapper. Protocol handling, folder sync, message parsing, and flag management in one class. |
| EmailAccessService | 39 | 12 | 1185 | Permission, delegation, folder access, and account resolution combined. |
| FilterQueryBuilder | 39 | 10 | 1148 | Query building for every filter type, operator, and data type in one class. |
| MockDataSeeder | 25+32 | 28 | 912+231 | Two classes in one file (MockDataSeeder + MockDataFallbackFaker), 28 deps. Seeder that knows about every entity type. |
| PdfHtmlRenderer | 30 | 0 | 1533 | Zero deps but 1533 lines. Monolithic template rendering with inline formatting logic. |
| SchemaAutoSync | 21 | 9 | 889 | Schema migration logic mixed with entity introspection. |

### 4.2 Borderline (likely needs monitoring, not urgent)

These have elevated metrics but may be justified as domain-aggregation points.

| Class | Methods | Deps | Lines | Reasoning |
|---|---|---|---|---|
| EmailService | 41 | 12 | 705 | Facade for email operations. High method count from many thin delegation methods. Acceptable if methods are truly thin. |
| MenuService | 32 | 7 | 988 | Menu building is inherently complex (permissions, modules, roles). May be acceptable. |
| EntityRelationService | 32 | 10 | 1017 | Relation loading/saving for a generic entity system. Complexity may be structural. |
| EmailSendService | 27 | 20 | 839 | 20 deps is high, but sending email legitimately touches SMTP, templates, attachments, queuing, permissions, logging. |
| TenantManager | 25 | 13 | 569 | Multi-tenancy hub. High deps expected for a cross-cutting concern. |
| CalendarAggregatorService | 17 | 10 | 825 | Aggregates multiple calendar sources. Long but scoped. |
| BankTransactionInvoiceAssigner | 25 | 12 | 891 | Complex matching logic. May be acceptable as a single responsibility. |

### 4.3 Acceptable Service Aggregation (low concern)

These are flagged as god classes but their metrics are moderate and the responsibility is focused.

| Pattern | Count | Reasoning |
|---|---|---|
| AiChat provider classes (ClaudeProvider, CerebrasProvider, DeepSeekProvider, GeminiProvider) | 4 | 17-20 methods, 4 deps. LLM API adapters with expected interface surface. Not god classes. |
| AiChat tool classes (CreateEntityTool, SearchTool, QueryEntityTool, etc.) | 9 | 12-28 methods, each scoped to a single AI tool. Method count is driven by parameter validation and response formatting. |
| View type classes (MapViewType, ChartViewType, KanbanViewType, TimelineViewType) | 4 | 15-16 methods, 1-2 deps. Strategy pattern implementations. Not god classes. |
| Dashboard services (DashletService, DashletRenderer) | 2 | 15-21 methods. Focused on dashlet rendering. |
| Onboarding services (AchievementService, ProgressTracker, TourGenerator) | 3 | 15-17 methods, 0-4 deps. Small, focused. |
| WhatsAppSettings | 1 | 15 methods, 2 deps, 140 lines. False positive. |

### 4.4 Summary Verdict

| Category | Count | % of Service God Classes |
|---|---|---|
| True god classes (refactor) | 8 | 6% |
| Borderline (monitor) | 7 | 6% |
| Acceptable aggregation (suppress) | ~23 | 18% |
| Low-signal remainder | ~88 | 70% |

The majority of the 126 entries are triggered by the god-class threshold but represent focused services with moderate complexity. The true architectural problems are concentrated in 8 classes, with ChatService being the clear priority.

## 5. Recommendations

1. **ChatService** is the single highest-impact refactoring target in the entire codebase. At 114 methods and 3933 lines, it is responsible for too many concerns. Decompose into focused collaborators.

2. **Email subsystem** (13 god classes, avg 781 lines) needs a domain review. The responsibilities of EmailService, EmailAccessService, EmailSendService, EmailSyncService, EmailSyncManager, EmailActionService, EmailActionQueueService, EmailAssignmentService, EmailDelegationService, EmailBodyFetchService, EmailSearchService, AttachmentCacheService, and NativeImapService overlap and form a tightly coupled cluster.

3. **Policy suppression**: The 23+ acceptable-aggregation entries (AI providers, AI tools, view types, small module services) should be encoded as accepted patterns in `.aigiscode/policy.json` to reduce noise in future runs.

4. **MockDataSeeder** (two classes in one file, 28 deps) is a maintenance hazard. It knows about every entity type and should be split into per-module seeders loaded via a registry.

5. **PdfHtmlRenderer** (1533 lines, 0 deps) is a standalone monolith. The zero-dependency count means it can be refactored in isolation without risk of cascading changes.
