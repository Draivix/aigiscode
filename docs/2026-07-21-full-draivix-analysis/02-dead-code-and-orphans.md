# 02 — Dead Code & Orphans

Detector pool: 18 DeadCode findings + 4 orphan modules. All verified by full-repo
grep (including the excluded `Commands/` and `Tests/` dirs) + in-file call checks.

## Orphan modules (whole files)

| File | Verdict | Evidence |
|---|---|---|
| `app/Modules/Accounting/Services/AutomaticInvoiceRecalculationService.php` | **DELETE** | Zero refs anywhere in `app/`, `routes/`, `config/` (commands included). Truly dead. |
| `app/Services/Spreadsheet/XlsxExportWriter.php` | **DELETE** | Zero refs anywhere. Truly dead. |
| `app/Modules/ProductEngineering/Services/EngineeringDefinitionSelector.php` | **DELETE WITH ITS TEST** | Referenced only by `app/Modules/ProductEngineering/Tests/Integration/EngineeringDefinitionSelectorTest.php`. Dead in production; alive only to its own test. Delete both, or owner decides the feature returns. |
| `app/Modules/AiChat/Support/PromptTesting/AiChatVoiceSmokeScenarioCatalog.php` | **owner-decision** | Referenced only by `app/Modules/AiChat/Tests/Support/playwright_voice_smoke_scenarios.php` (test tooling, out of slice). Not dead the way the others are — it's voice-smoke test infrastructure. Keep if voice smoke runs; otherwise delete with the scenario. |

## Unused private functions (14)

**Pohoda provider cluster (10)** — `app/Modules/Pohoda/Tools/Pohoda/Providers/`
Copy-paste inheritance leftovers; zero call sites for every one (name-grep over
the whole repo, including `static::`/`$this->` forms):

| Method | Files |
|---|---|
| `getItemEntityType` | `Abstract/AbstractProvider.php:506`, `Abstract/InvoiceLikeProvider.php:371` |
| `getParentFieldNameForItems` | `Abstract/AbstractProvider.php:515`, `Abstract/InvoiceLikeProvider.php:380` |
| `getDefaultStatus` | `ProformaInvoiceProvider.php:76`, `ReceivedProformaInvoiceProvider.php:86` |
| `getPohodaOrderType` | `PurchaseOrderProvider.php:22`, `SalesOrderProvider.php:22` |
| `getPohodaOrderTypeStatic` | `PurchaseOrderProvider.php:30`, `SalesOrderProvider.php:30` |
| `createWarehouseItemsForProducts` | `WarehouseProvider.php:147` |

**Verdict: DELETE all 10.** Same shape as the 11 dead Pohoda methods removed in
the 2026-07-05 round — the provider family was cloned per document type and
the unused parts never got pruned.

**Others (3)**

| Method | Verdict | Evidence |
|---|---|---|
| `VacationRequestController::getUserRoleIds` (`:117`) | **DELETE** | Never called in-file. (Three *other* classes have their own same-named live methods — name collision, not usage.) |
| `ShipmentWorkflowService::tableColumns` (`:702`) | **DELETE** | Zero in-file calls. |
| `ShipmentWorkflowService::toFloat` (`:837`) | **DELETE** | Zero in-file calls. |

## Notes for the ledger

- 2 files + 2 test-coupled files + 13 methods = small but 100% verified deletions.
- The pool is this small because prior rounds (2026-07-03/04/05) already swept
  dead code hard — what remains is the tail, and it is high-precision.
- Tool note (details in `10`): the `getUserRoleIds` name collision was resolved
  correctly by the detector (file-scoped proof), and in-file grep confirms it.
