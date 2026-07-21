# 09 — Removal & Simplification Ledger (main deliverable)

Every entry below was verified against source (full-repo greps incl. excluded
`Commands/`/`Tests/` dirs, in-file call checks, body reads). Method per entry:
detector lead → hand verification → verdict. Nothing here is taken on trust
from a finding count.

## A. Verified deletions — zero behavior change, do these first

| # | What | Where | Proof |
|---|---|---|---|
| A1 | Delete file | `app/Services/Spreadsheet/XlsxExportWriter.php` | Zero refs repo-wide |
| A2 | Delete file | `app/Modules/Accounting/Services/AutomaticInvoiceRecalculationService.php` | Zero refs repo-wide |
| A3 | Delete 10 dead methods | Pohoda providers: `getItemEntityType` ×2, `getParentFieldNameForItems` ×2, `getDefaultStatus` ×2, `getPohodaOrderType` ×2, `getPohodaOrderTypeStatic` ×2, `createWarehouseItemsForProducts` ×1 | Zero call sites incl. `static::`/`$this->` forms |
| A4 | Delete 3 dead methods | `VacationRequestController::getUserRoleIds`, `ShipmentWorkflowService::tableColumns`, `ShipmentWorkflowService::toFloat` | Zero in-file calls |
| A5 | Delete deprecated alias | `ResourceBookingService::confirm()` | `@deprecated`, zero callers/routes |
| A6 | Delete 3 legacy constants | `FIELD_YES/FIELD_NO/FIELD_READONLY` in `PermissionLoader.php` | Zero usages outside declaration |
| A7 | Delete file + its only test | `EngineeringDefinitionSelector.php` + `EngineeringDefinitionSelectorTest.php` | Referenced only by its own test |

**Total A: 3 files + 13 methods + 1 alias + 3 constants + 1 test-coupled pair.**

## B. Mechanical batches — one PR each

| # | What | Scope |
|---|---|---|
| B1 | Integration base URLs → `config/services.php` / `config/carriers.php` | 22 findings: 7 carrier URLs (`Shipment.php:209-215`), 5 Microsoft, 3 Fio, 3 Signi, 3 social (Facebook/LinkedIn/Telegram) |
| B2 | Batch-suppress dispatch fan-out findings with reason "sanctioned fan-out architecture (hooks+events+jobs)" | 14 findings stop re-triaging every run |
| B3 | Family-suppress "Abstraction sprawl" on `*ServiceProvider.php` | 10+ findings; idiom, not sprawl |
| B4 | Suppress `FieldNormalizer` "homegrown definition engine" guard flag | False positive — docblock documents it as the audit-recommended FieldLoader extraction (481 lines, 29 users, pure) |

## C. Merge (small)

| # | What | Note |
|---|---|---|
| C1 | Extract shared field-alias normalization walk; keep per-module vocabularies | `DashletFieldAliasNormalizer` (138) + `ReportFieldAliasNormalizer` (242) — cousins, ~100 lines saved |

## D. Restructure candidates (real work, ranked)

| # | What | Why | Size |
|---|---|---|---|
| D1 | Split `AccountingProductionEvidenceService` by evidence artifact type | 8,511 lines, 149 methods, accretion factory | days |
| D2 | Break `⇄ PermissionChecker` and `⇄ FilterQueryBuilder` structural pairs | core-path coupling | 0.5–2d each |
| D3 | Split `EmailService` (account-lifecycle vs sync orchestration) | 52 methods, 24 dependents | 1–2d |
| D4 | Break 5 job↔service loops (Email sync, Mattermost ×2, Satellite chain, Attachment⇄Thumbnail) | chaining via interface/event | 0.5–2d each |
| D5 | Split `ChatService` (providers vs orchestration vs tools) | 5,080 lines, 125 methods | days |
| D6 | Break down `Email/Index.vue` (5,487) + `EmailPreview.vue` (5,312) | god components | days |
| D7 | `_Core` knot redesign (18 files) | **use the existing plan** (`draivix/2026-07-04-core-knot-deep-analysis.md`), don't re-analyze | project |

## E. Owner decisions needed (with a date, not eternal limbo)

| # | What | Question |
|---|---|---|
| E1 | `app/Support/EspoMigration/` (807 lines) + `config/espocrm.php` + `ImportLegacyOutlookConnectionCommand` + `MigrationImportInvoiceAttachmentsCommand` | Migration tooling is done or still used for data fixes? Pick an archive date. |
| E2 | `AiChatVoiceSmokeScenarioCatalog` + playwright voice smoke scenario | Voice smoke tests alive? Keep, or delete together. |
| E3 | `MigrationImportDataCommand.php` (4,804 lines) + migration command family | Same E1 question, bigger file. |

## F. Comment debt (not code, still debt)

- `ModuleRegistry` — 6 methods documented "Static facade for backward compatibility" while **160 call sites use the static API and 0 use the instance API**. The facade is the API; bless it or migrate — but stop documenting a winner as a shim.

## Scoreboard

- **Immediate deletions (A):** 3 files, 13 methods, 1 alias, 3 constants, 1 pair — all hand-verified
- **Findings removed by config batch (B1):** ~22
- **Findings removed by honest suppressions (B2-B4):** ~25+
- **Lines directly deletable (A):** ~1,000–1,500 plus C1's ~100
- **Restructure (D):** the real mass — but each item is a decision, not a sweep

The honest headline after full verification: **this repo is well-kept.** The
tail is small because prior rounds cleaned hard. What remains is a short list
of safe deletions, one mechanical config batch, three suppression decisions,
and a restructure lane that needs owner time, not an agent sweep.
