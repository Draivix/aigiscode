# NewERP God Classes: Controller Analysis

**Source:** `.aigiscode/aigiscode-report.json` (2026-03-15)
**Scope:** All god classes where `file` contains "Controller"

---

## 1. Summary

| Metric | Value |
|---|---|
| Total controller god classes | 55 |
| Total god classes in report | 284 |
| Controller share of god classes | 19.4% |
| Average methods per controller | 15.6 |
| Average dependencies per controller | 14.6 |
| Average lines per controller | 468 |
| Max methods | 40 (EntityApiController) |
| Max dependencies | 32 (EntityApiController) |
| Max lines | 1,072 (MenuController) |
| Total lines across all | 25,751 |

---

## 2. Top 10 Worst Offenders

Ranked by a composite score: `methods + dependencies + (lines / 50)`.

| Rank | Class | File | Methods | Deps | Lines | Score |
|---|---|---|---|---|---|---|
| 1 | EntityApiController | `app/Http/Controllers/Api/EntityApiController.php` | 40 | 32 | 938 | 90.8 |
| 2 | EntityFormHandler | `app/Http/Controllers/Entity/Handlers/EntityFormHandler.php` | 30 | 28 | 835 | 74.7 |
| 3 | EntityListHandler | `app/Http/Controllers/Entity/Handlers/EntityListHandler.php` | 26 | 28 | 1,028 | 74.6 |
| 4 | EmailController | `app/Modules/Email/Http/Controllers/Api/EmailController.php` | 35 | 24 | 857 | 76.1 |
| 5 | EntityViewHandler | `app/Http/Controllers/Entity/Handlers/EntityViewHandler.php` | 20 | 30 | 761 | 65.2 |
| 6 | ChatController | `app/Modules/AiChat/Http/Controllers/Api/ChatController.php` | 19 | 22 | 664 | 54.3 |
| 7 | EmailPageController | `app/Modules/Email/Http/Controllers/EmailPageController.php` | 20 | 23 | 598 | 55.0 |
| 8 | CalendarController | `app/Modules/Calendar/Http/Controllers/Api/CalendarController.php` | 34 | 10 | 222 | 48.4 |
| 9 | DashboardController | `app/Http/Controllers/DashboardController.php` | 27 | 12 | 500 | 49.0 |
| 10 | MattermostProxyController | `app/Modules/Mattermost/Http/Controllers/MattermostProxyController.php` | 18 | 17 | 954 | 54.1 |

---

## 3. Grouping by Module

### Core HTTP Controllers (`app/Http/Controllers/`)

| Subgroup | Count | Avg Methods | Avg Deps | Avg Lines |
|---|---|---|---|---|
| Admin | 18 | 14.4 | 12.8 | 417 |
| Api | 8 | 14.8 | 15.4 | 408 |
| Entity (Handlers) | 3 | 25.3 | 28.7 | 875 |
| Auth | 1 | 9.0 | 16.0 | 287 |
| Root-level | 3 | 19.7 | 17.3 | 465 |
| **Subtotal** | **33** | | | |

### Module Controllers (`app/Modules/*/`)

| Module | Count | Avg Methods | Avg Deps | Avg Lines |
|---|---|---|---|---|
| Email | 4 | 19.5 | 18.5 | 690 |
| Calendar | 3 | 19.7 | 9.7 | 371 |
| Storage | 3 | 11.0 | 12.3 | 348 |
| Mattermost | 2 | 14.5 | 13.5 | 636 |
| PasswordVault | 2 | 12.5 | 12.0 | 394 |
| AiChat | 1 | 19.0 | 22.0 | 664 |
| Documents | 1 | 22.0 | 11.0 | 567 |
| GitLabIntegration | 1 | 15.0 | 13.0 | 343 |
| GoogleIntegration | 1 | 9.0 | 13.0 | 238 |
| MagicOffice | 1 | 13.0 | 12.0 | 628 |
| OutlookIntegration | 1 | 12.0 | 13.0 | 327 |
| Banking | 1 | 7.0 | 11.0 | 228 |
| Website | 1 | 17.0 | 9.0 | 689 |
| **Subtotal** | **22** | | | |

---

## 4. All 55 Controllers (sorted by method count)

| Class | File | Methods | Deps | Lines |
|---|---|---|---|---|
| EntityApiController | `app/Http/Controllers/Api/EntityApiController.php` | 40 | 32 | 938 |
| EmailController | `app/Modules/Email/Http/Controllers/Api/EmailController.php` | 35 | 24 | 857 |
| CalendarController | `app/Modules/Calendar/Http/Controllers/Api/CalendarController.php` | 34 | 10 | 222 |
| EntityFormHandler | `app/Http/Controllers/Entity/Handlers/EntityFormHandler.php` | 30 | 28 | 835 |
| DashboardController | `app/Http/Controllers/DashboardController.php` | 27 | 12 | 500 |
| EntityListHandler | `app/Http/Controllers/Entity/Handlers/EntityListHandler.php` | 26 | 28 | 1,028 |
| ImportDefinitionController | `app/Http/Controllers/Admin/ImportDefinitionController.php` | 25 | 15 | 681 |
| MenuController | `app/Http/Controllers/Admin/MenuController.php` | 23 | 13 | 1,072 |
| TemplateBuilderController | `app/Modules/Documents/Http/Controllers/TemplateBuilderController.php` | 22 | 11 | 567 |
| EntityViewHandler | `app/Http/Controllers/Entity/Handlers/EntityViewHandler.php` | 20 | 30 | 761 |
| EmailPageController | `app/Modules/Email/Http/Controllers/EmailPageController.php` | 20 | 23 | 598 |
| ChatController | `app/Modules/AiChat/Http/Controllers/Api/ChatController.php` | 19 | 22 | 664 |
| MattermostProxyController | `app/Modules/Mattermost/Http/Controllers/MattermostProxyController.php` | 18 | 17 | 954 |
| AccountController | `app/Modules/Website/Http/Controllers/AccountController.php` | 17 | 9 | 689 |
| EntityRecordController | `app/Http/Controllers/Admin/EntityRecordController.php` | 17 | 18 | 192 |
| TemplateController | `app/Http/Controllers/Admin/TemplateController.php` | 17 | 9 | 516 |
| AttachmentController | `app/Http/Controllers/Api/AttachmentController.php` | 17 | 16 | 367 |
| ProfileController | `app/Http/Controllers/ProfileController.php` | 17 | 22 | 665 |
| RoleController | `app/Http/Controllers/Admin/RoleController.php` | 16 | 11 | 480 |
| WorkflowController | `app/Http/Controllers/Admin/WorkflowController.php` | 16 | 12 | 527 |
| SettingsLogoController | `app/Http/Controllers/Api/SettingsLogoController.php` | 15 | 8 | 244 |
| GenericEntityController | `app/Http/Controllers/GenericEntityController.php` | 15 | 18 | 231 |
| GitLabProxyController | `app/Modules/GitLabIntegration/Http/Controllers/GitLabProxyController.php` | 15 | 13 | 343 |
| ScheduledJobController | `app/Http/Controllers/Admin/ScheduledJobController.php` | 15 | 14 | 394 |
| EntityLookupController | `app/Http/Controllers/Api/EntityLookupController.php` | 15 | 17 | 512 |
| CalendarManagementHandler | `app/Modules/Calendar/Http/Controllers/Api/Calendar/CalendarManagementHandler.php` | 15 | 6 | 585 |
| ConversionController | `app/Http/Controllers/Admin/ConversionController.php` | 14 | 14 | 382 |
| ImportRunController | `app/Http/Controllers/Admin/ImportRunController.php` | 14 | 11 | 407 |
| VaultFolderController | `app/Modules/PasswordVault/Http/Controllers/Api/VaultFolderController.php` | 14 | 13 | 458 |
| EntityController | `app/Http/Controllers/Admin/EntityController.php` | 13 | 14 | 352 |
| FilterDefinitionController | `app/Http/Controllers/Admin/FilterDefinitionController.php` | 13 | 12 | 278 |
| EmailFolderController | `app/Modules/Email/Http/Controllers/Api/EmailFolderController.php` | 13 | 11 | 849 |
| SpreadsheetController | `app/Modules/MagicOffice/Http/Controllers/SpreadsheetController.php` | 13 | 12 | 628 |
| StorageFileController | `app/Modules/Storage/Http/Controllers/Api/StorageFileController.php` | 13 | 13 | 444 |
| WebhookController | `app/Http/Controllers/Admin/WebhookController.php` | 13 | 15 | 388 |
| FieldController | `app/Http/Controllers/Admin/FieldController.php` | 12 | 13 | 282 |
| OutlookIntegrationController | `app/Modules/OutlookIntegration/Controllers/OutlookIntegrationController.php` | 12 | 13 | 327 |
| StorageFolderController | `app/Modules/Storage/Http/Controllers/Api/StorageFolderController.php` | 12 | 10 | 365 |
| SettingsController | `app/Http/Controllers/Admin/SettingsController.php` | 12 | 18 | 427 |
| EntityUIController | `app/Http/Controllers/Admin/EntityUIController.php` | 11 | 11 | 322 |
| ChannelLinkController | `app/Modules/Mattermost/Http/Controllers/ChannelLinkController.php` | 11 | 10 | 318 |
| CredentialController | `app/Modules/PasswordVault/Http/Controllers/Api/CredentialController.php` | 11 | 11 | 331 |
| CurrencyController | `app/Http/Controllers/Admin/CurrencyController.php` | 11 | 10 | 351 |
| DebugController | `app/Http/Controllers/Api/DebugController.php` | 10 | 13 | 486 |
| CalendarPageController | `app/Modules/Calendar/Http/Controllers/CalendarPageController.php` | 10 | 13 | 306 |
| EmailAccountController | `app/Modules/Email/Http/Controllers/Api/EmailAccountController.php` | 10 | 16 | 457 |
| CalendarSettingsController | `app/Http/Controllers/Admin/CalendarSettingsController.php` | 9 | 10 | 250 |
| TenantController | `app/Http/Controllers/Admin/TenantController.php` | 9 | 11 | 213 |
| AuditController | `app/Http/Controllers/Api/AuditController.php` | 9 | 13 | 211 |
| LoginController | `app/Http/Controllers/Auth/LoginController.php` | 9 | 16 | 287 |
| GoogleIntegrationController | `app/Modules/GoogleIntegration/Controllers/GoogleIntegrationController.php` | 9 | 13 | 238 |
| FolderPermissionController | `app/Modules/Storage/Http/Controllers/Api/FolderPermissionController.php` | 8 | 14 | 234 |
| LayoutController | `app/Http/Controllers/Api/LayoutController.php` | 8 | 11 | 296 |
| RevolutOpenBankingAuthController | `app/Modules/Banking/Http/Controllers/RevolutOpenBankingAuthController.php` | 7 | 11 | 228 |
| SshTokenController | `app/Http/Controllers/Api/Auth/SshTokenController.php` | 4 | 13 | 214 |

---

## 5. Recommendations

### Truly problematic (refactor priority)

These controllers combine high method counts, high dependency counts, and high line counts -- indicating they are doing too much and coupling to too many subsystems.

| Class | Why it is problematic | Suggested action |
|---|---|---|
| **EntityApiController** (40m, 32d, 938L) | Highest on all three axes. A single API controller handling generic entity CRUD with 32 dependencies is a framework-within-a-framework. | Extract domain-specific action classes or use a service layer. The 32 dependencies suggest it is orchestrating logic that should live in services. |
| **EntityFormHandler** (30m, 28d, 835L) | Not a controller in name, but acts as one. 28 dependencies and 835 lines of form-handling logic. | Split into per-entity-type form strategies or move validation/persistence to dedicated services. |
| **EntityListHandler** (26m, 28d, 1028L) | Same pattern as EntityFormHandler. Over 1,000 lines of list-rendering logic with 28 dependencies. | Break into composable query/filter/sort strategies. |
| **EntityViewHandler** (20m, 30d, 761L) | 30 dependencies is the second highest in the set. View rendering should not need this much coupling. | Delegate to view-model builders; reduce direct service access. |
| **EmailController** (35m, 24d, 857L) | 35 methods across send, receive, search, folders, attachments, drafts -- essentially an entire email client API in one class. | Split into EmailSendController, EmailSearchController, EmailDraftController, etc. |
| **MattermostProxyController** (18m, 17d, 954L) | 954 lines for a "proxy" suggests it is doing transformation and business logic, not just proxying. | Extract message transformation and channel logic into services. |
| **MenuController** (23m, 13d, 1072L) | Highest line count (1,072). Likely contains menu tree manipulation, ordering, and rendering logic mixed together. | Extract menu tree operations into a MenuService. |

### Borderline (monitor, refactor if growing)

These have elevated numbers but may be acceptable for their domain scope:

- **ChatController** (19m, 22d, 664L) -- chat features naturally combine many concerns, but 22 deps is high.
- **EmailPageController** (20m, 23d, 598L) -- similar to EmailController, the Email module has too much in controllers overall.
- **DashboardController** (27m, 12d, 500L) -- dashboards aggregate many data sources; 27 methods is a lot but low dep count means it delegates well.
- **ImportDefinitionController** (25m, 15d, 681L) -- import workflows are inherently complex; watch for growth.
- **ProfileController** (17m, 22d, 665L) -- 22 deps for a profile page suggests it handles settings, avatar, 2FA, etc. Consider splitting.

### Normal CRUD controllers (acceptable)

Controllers with fewer than 15 methods, fewer than 15 dependencies, and fewer than 500 lines are within normal bounds for Laravel CRUD controllers. This includes 28 of the 55 controllers (51%). Examples:

- CalendarSettingsController (9m, 10d, 250L)
- TenantController (9m, 11d, 213L)
- FieldController (12m, 13d, 282L)
- FilterDefinitionController (13m, 12d, 278L)
- CurrencyController (11m, 10d, 351L)

These are flagged as god classes mainly because they exceed AigisCode's default thresholds, which are calibrated for general-purpose classes, not controllers. **Consider raising the god-class threshold for controller files in AigisCode policy**, or adding a policy annotation that controllers with standard CRUD verbs (index, show, create, store, edit, update, destroy) are expected to have 7+ methods by convention.

### Structural observation

The `app/Http/Controllers/Entity/Handlers/` trio (EntityFormHandler, EntityListHandler, EntityViewHandler) averages **28.7 dependencies** -- nearly double the overall average of 14.6. This is the single most tightly coupled area in the controller layer and the highest-priority refactoring target.

### Policy suggestion for AigisCode

To reduce noise in future runs, add to `.aigiscode/policy.json`:

```json
{
  "god_class": {
    "method_threshold_override": {
      "app/Http/Controllers/**": 20,
      "app/Modules/*/Http/Controllers/**": 20
    }
  }
}
```

This would reduce the 55 flagged controllers to approximately 17 -- the ones that genuinely warrant attention.
