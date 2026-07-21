# 06 — God Classes & Oversized Files

## God-class findings (6, post accessor-discount)

This run is the first with framework-idiom accessors discounted — the old
entity noise (`Email.php` 114 methods, `User.php`, `Attachment`) is gone, and
what survives is genuine service-hub width:

| File | Real methods | Dependent files | Read |
|---|---|---|---|
| `Entities/_Core/EntityManager.php` | 29 (+4 accessors) | **643** | The entity engine's hub. Splitting it is a *design project* (already scoped in prior rounds), not a cleanup item. |
| `Entities/_Core/EntityRegistry.php` | 28 (+12) | 291 | Same family as above. |
| `Services/Email/EmailService.php` | 52 (+2) | 24 | Extraction candidate: account lifecycle vs sync orchestration vs status queries read as separate seams. |
| `Services/Email/NativeImapService.php` | 43 | 14 | IMAP protocol client — fat but coherent; split only if protocol surface grows. |
| `Mattermost/Services/MattermostClient.php` | 45 | 19 | API client accreting channel/message/call/files domains — natural split lines if Mattermost work continues. |
| `Services/_Core/PermissionChecker.php` | 27 (+3) | 170 | Known extraction target from prior rounds ("163/325"); a redesign, not a sweep. |

## Oversized files (non-test)

| File | Lines | Shape |
|---|---|---|
| `AccountingCompliance/Services/AccountingProductionEvidenceService.php` | **8,511** | One class, **149 methods** — evidence-template factory grown by accretion. Top split candidate: group by evidence artifact type. |
| `resources/js/Pages/Email/Index.vue` | 5,487 | Page-level god component. |
| `resources/js/Components/Email/EmailPreview.vue` | 5,312 | Same family. |
| `AiChat/Services/ChatService.php` | 5,080 | 125 methods — chat orchestration accretion. |
| `Console/Commands/MigrationImportDataCommand.php` | 4,804 | Migration tooling (owner-decision class, see 03). |
| `Mattermost/.../MessageItem.vue` | 4,527 | Chat UI accretion. |

`resources/js/types/entities.d.ts` (100,802 lines) is generated — excluded by
design, not a finding.

## Ledger line

- No safe deletions here; the entries feed `09` as **restructure candidates**:
  `AccountingProductionEvidenceService` (split by evidence type),
  `EmailService` (split account-lifecycle vs sync), `ChatService` (split
  providers vs orchestration vs tools), the two Email Vue components.
- God-class findings are now honest post-discount — record in `10` that the
  entity-class noise is gone and the survivor list matches prior hand audits.
