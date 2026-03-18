# NewERP God Classes: Non-Service/Controller/Console Analysis

**Source:** `.aigiscode/aigiscode-report.json`
**Date:** 2026-03-15
**Scope:** All god classes excluding files containing `Service`, `Controller`, or `Console/Commands` in path.

---

## Executive Summary

Of 284 total god classes, 183 belong to Services, Controllers, or Console/Commands. The remaining **101 god classes** are analyzed here. The overwhelming majority (75 of 101, or 74%) are **Entities** -- the custom ORM/domain-object layer. Several sub-categories (Seeders, DevOps tooling, Pohoda providers) are architectural patterns where high method counts are expected and should be excluded via policy rather than refactored.

---

## 1. Summary by Category

| Category | Count | Avg Methods | Avg Deps | Avg Lines | Total Lines |
|---|---|---|---|---|---|
| Entities | 75 | 32.4 | 4.1 | 405.7 | 30,426 |
| Actions | 6 | 15.5 | 13.0 | 373.7 | 2,242 |
| Providers (Pohoda) | 5 | 24.6 | 7.8 | 683.2 | 3,416 |
| DevOps/Tooling | 4 | 15.2 | 14.5 | 369.8 | 1,479 |
| Frontend (TypeScript) | 3 | 24.7 | 4.7 | 618.0 | 1,854 |
| Support | 2 | 15.5 | 9.5 | 521.5 | 1,043 |
| Seeders | 2 | 14.0 | 12.5 | 647.5 | 1,295 |
| Tools/Integrations | 1 | 28.0 | 23.0 | 1,086.0 | 1,086 |
| Core Infrastructure | 1 | 28.0 | 2.0 | 579.0 | 579 |
| Middleware | 1 | 20.0 | 21.0 | 817.0 | 817 |
| Models | 1 | 16.0 | 15.0 | 296.0 | 296 |
| **Total** | **101** | **29.9** | **5.4** | **425.3** | **42,533** |

---

## 2. Top 10 Worst Offenders

| Rank | Class | Category | Methods | Deps | Lines | File |
|---|---|---|---|---|---|---|
| 1 | Email | Entity | 109 | 4 | 986 | `app/Entities/Email/Email.php` |
| 2 | SalesOrder | Entity | 68 | 3 | 578 | `app/Modules/Accounting/Entities/SalesOrder/SalesOrder.php` |
| 3 | ImportRun | Entity | 65 | 2 | 596 | `app/Modules/ETL/Entities/ImportRun/ImportRun.php` |
| 4 | EntityRegistry | Entity (Core) | 59 | 8 | 1,565 | `app/Entities/_Core/EntityRegistry.php` |
| 5 | EmailAccount | Entity | 56 | 2 | 570 | `app/Entities/EmailAccount/EmailAccount.php` |
| 6 | ImportDefinition | Entity | 53 | 2 | 473 | `app/Modules/ETL/Entities/ImportDefinition/ImportDefinition.php` |
| 7 | WebsiteContactProfile | Entity | 53 | 4 | 594 | `app/Modules/Website/Entities/WebsiteContactProfile/WebsiteContactProfile.php` |
| 8 | Template | Entity | 49 | 2 | 508 | `app/Modules/Documents/Entities/Template/Template.php` |
| 9 | Cart | Entity | 47 | 3 | 417 | `app/Modules/Website/Entities/Cart/Cart.php` |
| 10 | Shipment | Entity | 44 | 3 | 434 | `app/Modules/Accounting/Entities/Shipment/Shipment.php` |

All top 10 are Entities. Most have low dependency counts (2-4), indicating they are wide but not deeply coupled. The method inflation is driven by getters, setters, relationship accessors, and domain logic colocated on the entity.

---

## 3. Category Breakdown

### 3.1 Entities (75 classes -- 30,426 total lines)

By far the dominant category. These are the custom ORM domain objects under `app/Entities/` and `app/Modules/*/Entities/`.

**Entity sub-groups:**

| Sub-group | Count | Avg Methods | Avg Lines | Worst Offender |
|---|---|---|---|---|
| Core (`_Core`) | 14 | 30.8 | 820.1 | EntityRegistry (59m, 1565L) |
| Email-related | 9 | 43.9 | 439.0 | Email (109m, 986L) |
| Website | 12 | 35.4 | 347.5 | WebsiteContactProfile (53m, 594L) |
| Accounting | 4 | 45.2 | 406.0 | SalesOrder (68m, 578L) |
| ETL | 4 | 43.2 | 396.0 | ImportRun (65m, 596L) |
| CRM/CRMFoundation | 5 | 25.2 | 226.6 | Lead (37m, 328L) |
| Other modules | 27 | 26.0 | 240.1 | Template (49m, 508L) |

**Method-count distribution across entities:**

| Bracket | Count | Notes |
|---|---|---|
| 50+ methods | 7 | Genuine god classes, need refactoring |
| 30-49 methods | 33 | Borderline; many are getter/setter heavy |
| 20-29 methods | 22 | Near threshold; likely acceptable for entities |
| 15-19 methods | 13 | Barely flagged; policy-excludable |

**Key observations:**
- Most entities have very low dependency counts (2-4), meaning they are wide (many methods) but shallow (not deeply coupled). This is a hallmark of data-heavy domain objects with accessor methods.
- The high-density entities (12-13 methods per 100 lines) like OAuthClient, VaultShareKey, and WorkflowLog are almost entirely accessor-driven and are false positives for the god class detector.
- The Core `_Core` entities (EntityRegistry, EntityManager, PermissionLoader, UILoader, FieldLoader, etc.) are genuinely complex infrastructure with high line counts (800-1500 lines) and should be reviewed for possible decomposition.

### 3.2 Actions (6 classes -- 2,242 total lines)

| Class | Methods | Deps | Lines | File |
|---|---|---|---|---|
| DataBoxMessageActionHandler | 26 | 11 | 333 | `app/Modules/DataBox/Actions/...` |
| LeadBulkActionHandler | 25 | 19 | 588 | `app/Modules/CRM/Actions/...` |
| IssueInvoiceAction | 15 | 13 | 455 | `app/Modules/ProjectManagement/Actions/...` |
| RunListPreview | 10 | 12 | 353 | `app/Modules/Advanced/Actions/...` |
| ExportWorkTimeEnumToExcel | 9 | 11 | 297 | `app/Modules/HumanResources/Actions/...` |
| ConnectRevolutOpenBankingAction | 8 | 12 | 216 | `app/Modules/Banking/Actions/...` |

Actions have moderate method counts but high dependency counts (11-19). The LeadBulkActionHandler (19 deps) and IssueInvoiceAction (13 deps, 455 lines) are candidates for decomposition. The lower-method ones are flagged primarily due to dependency fan-out.

### 3.3 Providers -- Pohoda Integration (5 classes -- 3,416 total lines)

All five are part of the Pohoda accounting system integration (`app/Modules/Pohoda/Tools/Pohoda/Providers/`). This is an abstract provider hierarchy for XML-based data exchange with an external system. The method count is structural -- each provider maps fields between internal entities and Pohoda XML.

### 3.4 DevOps/Tooling (4 classes -- 1,479 total lines)

| Class | Methods | Deps | Lines | Location |
|---|---|---|---|---|
| SatelliteClient | 23 | 17 | 534 | `tools/satellite/` (Python) |
| LocalQueue | 17 | 11 | 367 | `tools/satellite/` (Python) |
| GoogleAdsAuth | 12 | 18 | 290 | `docker/agent/mcp/` (Python) |
| OAuthCallbackHandler | 9 | 12 | 288 | `docker/agent/mcp/` (Python) |

These are outside the main application. The satellite tool and MCP auth handlers are standalone utilities.

### 3.5 Frontend (3 classes -- 1,854 total lines)

| Class | Methods | Deps | Lines | File |
|---|---|---|---|---|
| CallsClient | 34 | 9 | 1,115 | `Mattermost/resources/js/calls/CallsClient.ts` |
| FormulaPlugin | 20 | 2 | 389 | `MagicOffice/resources/js/.../FormulaPlugin.ts` |
| TourRegistryClass | 20 | 3 | 350 | `Onboarding/resources/js/services/TourRegistry.ts` |

CallsClient at 1,115 lines is a genuine god class managing WebRTC calls. The other two are moderate.

### 3.6 Remaining Categories

| Category | Class | Methods | Deps | Lines | Notes |
|---|---|---|---|---|---|
| Tools/Integrations | PohodaTool | 28 | 23 | 1,086 | Pohoda XML orchestrator; 23 deps is very high |
| Middleware | HandleInertiaRequests | 20 | 21 | 817 | Inertia.js middleware sharing data to frontend |
| Models | User (Model) | 16 | 15 | 296 | Laravel User model; traits inflate method count |
| Core Infra | ModuleRegistry | 28 | 2 | 579 | Module discovery/registration |
| Seeders | CzechCRMFullSeeder | 15 | 8 | 706 | Test data; inherently wide |
| Seeders | PerformanceSeeder | 13 | 17 | 589 | Benchmark data; inherently wide |
| Support | AddressCompositeField | 16 | 2 | 420 | Composite field handler |
| Support | AiChatToolConversationFixtureFactory | 15 | 17 | 623 | Test fixture builder |

---

## 4. Recommendations

### Exclude via Policy (low refactoring value)

These categories produce false or low-value god class findings due to their inherent architecture. Suppress them in `.aigiscode/policy.json`:

| Category | Reason | Suggested Pattern |
|---|---|---|
| Entity accessor classes (15-29 methods, deps <= 4) | Getter/setter-heavy domain objects; width is structural, not a design flaw | `app/Entities/*/` and `app/Modules/*/Entities/*/` with method threshold raised to 30 for these paths |
| Pohoda Providers | External integration mapping; method count tracks field count, not complexity | `app/Modules/Pohoda/Tools/Pohoda/Providers/**` |
| Seeders | Test/benchmark data generators; inherently wide by design | `database/seeders/**` |
| DevOps/Tooling | Standalone utilities outside the application; different quality bar | `tools/**`, `docker/**` |
| Models (Laravel) | Trait-inflated method counts from framework conventions | `app/Models/**` |

### Needs Actual Refactoring (high value)

| Priority | Class | Why | Suggested Action |
|---|---|---|---|
| **Critical** | `Email` (109 methods, 986 lines) | 3-4x the entity average; clear SRP violation | Extract EmailComposition, EmailRouting, EmailMetadata value objects |
| **Critical** | `EntityRegistry` (59m, 1565L) | Core infrastructure with god-level responsibility | Split registration, lookup, and schema concerns |
| **Critical** | `EntityManager` (44m, 27 deps, 1019L) | Highest dependency count in entities; orchestrates too much | Decompose into focused managers per concern |
| **High** | `PermissionLoader` (41m, 1260L) | Extremely long for a single-concern loader | Break into per-entity-type permission resolvers |
| **High** | `UILoader` (32m, 1215L) | Similar pattern -- too much in one loader | Split by UI concern (fields, layouts, actions) |
| **High** | `SalesOrder` (68m, 578L) | Business entity doing too much domain logic inline | Extract calculation/validation to dedicated classes |
| **High** | `ImportRun` (65m, 596L) | ETL orchestration mixed with entity state | Separate run-state entity from import execution logic |
| **High** | `CallsClient` (34m, 1115L) | WebRTC client with signaling + media + UI state | Extract signaling, media management, UI state |
| **High** | `PohodaTool` (28m, 23 deps) | Highest dependency count overall; orchestration hub | Facade pattern -- delegate to providers, reduce direct deps |
| **Medium** | `HandleInertiaRequests` (20m, 21 deps, 817L) | Middleware sharing too much data to frontend | Break shared data into per-module data providers |
| **Medium** | `HookAwareMapper` (27m, 24 deps) | ORM mapper with excessive coupling | Review whether hooks can be loaded lazily or via events |
| **Medium** | `CustomEntityFactory` (16m, 25 deps) | Low methods but 25 deps is a coupling red flag | Dependency injection audit; likely instantiating too much directly |

### Summary Prioritization

- **71 entities below 50 methods with deps <= 4**: Exclude via policy. These are structural width, not design flaws.
- **14 Core `_Core` entities**: Review individually. The loaders and registries (EntityRegistry, EntityManager, PermissionLoader, UILoader, FieldLoader, HookLoader) are the real architectural debt.
- **7 entities with 50+ methods**: Refactor. These are genuine god classes regardless of category.
- **High-dep outliers** (EntityManager 27, HookAwareMapper 24, CustomEntityFactory 25, PohodaTool 23, HandleInertiaRequests 21): These are coupling hotspots independent of method count and deserve attention.
