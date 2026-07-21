# 03 — Legacy & Conversion Machinery

Surveyed: `@deprecated` markers, "backward compatibility" notes, EspoCRM migration
shims, conversion machinery. Verified by usage greps over the full repo.

## Verified dead legacy (removal candidates)

| Item | Verdict | Evidence |
|---|---|---|
| `ResourceBookingService::confirm()` (`app/Services/Calendar/ResourceBookingService.php:306`) | **DELETE** | `@deprecated Use approve() instead` — zero callers, zero routes. Dead alias. |
| Legacy permission constants `FIELD_YES`, `FIELD_NO`, `FIELD_READONLY` (`app/Entities/_Core/PermissionLoader.php:112-116`) | **DELETE** | "Legacy constants for backward compatibility" — zero usages anywhere (only declarations). The live vocabulary is `FIELD_SCOPE_*`. |

## Alive but documented (keep / owner-decision)

| Item | Verdict | Evidence |
|---|---|---|
| `EntityManager::unsafeBulkUpdateWithoutHooks` (`EntityManager.php:1051`) | **KEEP** | The WP4 escape hatch from the July fix round: `@deprecated for business writes`, callers must justify. Only 3 call sites (`NotificationController`, `ErpAccountingBackfillDuzpCommand`, +1). Audit note: verify each carries its justification comment. |
| `ModuleRegistry` static facade (6 methods) | **KEEP, fix comments** | "Static facade for backward compatibility" — but **160 call sites use the static API and 0 use the instance API**. The facade won; the DI path never happened. Not deletable — but the docblocks lie about which API is canonical. Either bless the static API in docs, or (big churn, low value) migrate to DI. Owner-decision on direction; the *comment debt* is real regardless. |
| `app/Support/EspoMigration/` (807 lines, 4 files) | **owner-decision** | EspoCRM source-migration infrastructure (PDO reader, profile resolver, connection factory), registered as a singleton in `AppServiceProvider:248`. Live import tooling, not dead code — but it's *finished-migration* infrastructure. Keep while data imports/fixes still run; then archive with `config/espocrm.php` + `ImportLegacyOutlookConnectionCommand` as one removal unit. |
| `app/Console/Commands/MigrationImportInvoiceAttachmentsCommand.php` | **owner-decision** | "Backward-compatible wrapper around the generic attachment importer" — a migration-era alias command (in the excluded Commands dir). Same unit as above. |

## Not legacy (checked, alive)

- `ConversionLoader` (602 lines) + `ConversionApiController` + `EntityConversionHandler` — the record-conversion feature (`onConvertTo*` hooks). Live business machinery, heavily wired. Its `json_decode`-in-loop hotspot belongs to the performance ledger, not removal.
- `RecurrenceService` custom format (`:266`) — documented compat with existing tests/behavior; used.
- `Contact.fields.php`/`Meeting.fields.php` "backward-compatible FK alias for sorting/filtering" — data-layer compat for existing DBs; these are schema reality, not code shit. Owner-domain decision, not a code deletion.

## Ledger line

Verified dead legacy here is small (1 method + 3 constants) precisely because
prior rounds cleaned aggressively. The bigger honest finding is **comment debt
claiming "backward compatibility" where the compat path is actually the only
path** (ModuleRegistry) — and a migration-tooling unit (EspoMigration) that
needs an owner decision with a date, not silent eternal life.
