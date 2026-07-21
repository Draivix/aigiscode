# 11 — Top 30 Shit Issues (ranked)

Ranked by impact × confidence, everything source-verified in docs 01–08.
Action codes: **D** delete · **C** config/mechanical · **S** suppress (honest) · **R** restructure · **O** owner decision

| # | Issue | Where | Action |
|---|---|---|---|
| 1 | The 18-file `_Core` knot — entity-definition engine mutually recursive with tenant DB layer (848 edges) | `Entities/_Core/*` + `Services/_Core/*` | R — existing redesign plan (`draivix/2026-07-04-core-knot-deep-analysis.md`) |
| 2 | 255 layer-contract violations — entities calling services, core calling modules | `Email.actions.php:3`, `User.hooks.php` ×4, `ActivityHistoryLoader`/`CalendarConfigLoader` → `ModuleRegistry` | R — per-module injection recipes exist from prior rounds |
| 3 | `AccountingProductionEvidenceService` — 8,511 lines, 149 methods, accretion factory | `Modules/AccountingCompliance/Services/` | R — split by evidence artifact type |
| 4 | `EntityManager` — 643 dependent files on one hub | `Entities/_Core/EntityManager.php` | R — design project, not a sweep |
| 5 | `ChatService` — 5,080 lines, 125 methods | `Modules/AiChat/Services/` | R — providers vs orchestration vs tools |
| 6 | `Email/Index.vue` (5,487) + `EmailPreview.vue` (5,312) god components | `resources/js/` | R — days each |
| 7 | `EmailService` — 52 methods, 24 dependents, two seams fused | `Services/Email/` | R — account-lifecycle vs sync |
| 8 | 155 sanctioned-path bypasses — raw env/container access outside boundaries | repo-wide | C/R — injection recipe per module, ratchet new ones |
| 9 | 633 complexity hotspots — incl. `json_decode` in loop in `ActivityHistoryLoader` reachable from `XmlTemplateRenderController` | `_Core`, controllers | C/R — hoist/parse-once per hot site |
| 10 | `CommandFieldWritePolicy` — `in_array` collection scans in loop | `Entities/_Core/` | C — precomputed set |
| 11 | 7 hardwired carrier tracking URLs in one entity | `Shipment.php:209-215` | C — `config/carriers.php` |
| 12 | Hardcoded base URLs: Microsoft ×5, Fio ×3, Signi ×3, FB/LinkedIn/Telegram ×3 | OAuth/banking/signing/marketing | C — `services.*.base_url` |
| 13 | `ActivityHostAccessResolver ⇄ PermissionChecker` structural cycle (79 edges) | `Services/` | R — interface for the callback direction |
| 14 | `FilterJsonbHandler ⇄ FilterQueryBuilder` cycle | `Services/_Core/Filter/` | R — same |
| 15 | `EmailSyncManager ⇄ SyncAccountJob/SyncFolderJob` (105 edges) | `Jobs/Email` | R — chaining via event/interface |
| 16 | Mattermost job⇄service cycles ×2 (58, 40 edges) | `Modules/Mattermost/` | R — same pattern |
| 17 | `SatelliteService ⇄ ServerSshKeyDeployment ⇄ SshKeyVaultMirror` cycle | `Modules/ServerManager/` | R |
| 18 | `AttachmentService ⇄ ThumbnailService` cycle | `Services/Attachment/` | R |
| 19 | `MigrationImportDataCommand` — 4,804 lines of migration-era command | `Console/Commands/` | O — archive with EspoMigration unit |
| 20 | EspoMigration shim layer in limbo — 807 lines + config + 2 migration commands | `Support/EspoMigration/` | O — set archive date or keep documented |
| 21 | `Email.php` god entity — 114 public methods, 128 dependents (accessor-discounted out of findings, width still real) | `Entities/Email/` | R — split read-model accessors from behavior |
| 22 | `PermissionChecker` — 27 real methods, 170 dependents | `Services/_Core/` | R — known extraction target |
| 23 | `MattermostClient` — 45 methods accreting domains | `Modules/Mattermost/` | R — channel/message/call/files split |
| 24 | `EmailViewStatePreferenceNormalizer` — 292 lines vs sibling's 32 for the same interface | `Modules/Email/Services/` | R — extract shared preference logic |
| 25 | XLSX dual path — dead shared `XlsxExportWriter` while 8+ exporters hand-roll PhpSpreadsheet | `Services/Spreadsheet/` + 8 files | D the dead writer + O on a rich shared writer |
| 26 | `ModuleRegistry` dual API — 160 static call sites, 0 instance; docblocks claim "backward compatibility" | `Modules/ModuleRegistry.php` | C — bless the static API or migrate; fix lying comments either way |
| 27 | `DashletFieldAliasNormalizer` vs `ReportFieldAliasNormalizer` — same algorithm, two vocabularies | Dashboard/Advanced `Support/` | C — extract shared walk (~100 lines) |
| 28 | Pohoda provider clone cluster — 10 dead copy-paste methods | `Modules/Pohoda/.../Providers/` | D — all verified zero-call |
| 29 | Dead weight: `XlsxExportWriter`, `AutomaticInvoiceRecalculationService`, `ResourceBookingService::confirm()`, `FIELD_YES/NO/READONLY` constants, `VacationRequestController::getUserRoleIds`, `ShipmentWorkflowService::tableColumns/toFloat` | various | D — all verified |
| 30 | `scan.json` excludes `Commands/` — 1,241 boundary-truncated findings (49% of all findings are config echo) and console commands invisible | `.aigiscode/scan.json` | C/O — drop the exclusion when console coverage wanted |

Not on the list (checked, not shit): ProviderFolderMapper and
PreferenceValueNormalizer polymorphic families, ConversionLoader machinery,
`unsafeBulkUpdateWithoutHooks` (documented escape hatch, 3 justified callers).
