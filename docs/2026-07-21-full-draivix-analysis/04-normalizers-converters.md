# 04 — Normalizers, Converters, Mappers

**42 classes** matching `*Normalizer*|*Converter*|*Mapper*|*Transformer*|*Serializer*`
outside tests. The name count is not the problem — overlap is. Verdict first:
most of this is *healthy*, two spots are merge candidates, one guard finding is
a false positive.

## Healthy polymorphic families (KEEP — this is the good pattern)

| Family | Shape |
|---|---|
| `ProviderFolderMapper` | interface + `Gmail`/`Outlook` impls + registry — email folder mapping per provider |
| `PreferenceValueNormalizer` | interface (10 lines) + `ViewConfig` (32 lines) + `EmailViewState` (**292 lines — fat, see 06**) impls |
| Banking sync mappers (`BankTransactionStatementMapper`, `FioSyncMapper`, `RevolutOpenBankingSyncMapper`) | per-provider sync mapping, one job each |

## Merge candidates (same algorithm, two vocabularies)

| Pair | Finding | Verdict |
|---|---|---|
| `DashletFieldAliasNormalizer` (Dashboard, 138 lines) vs `ReportFieldAliasNormalizer` (Advanced, 242 lines) | Both normalize legacy field aliases in stored configs (`normalizeConfig`), different key sets/shapes. Cousins, not clones. | **MERGE (small)** — extract shared alias-resolution walk into one service; each module keeps its key vocabulary. Not urgent. |
| `CurrencyNormalizer` (Accounting) vs `CurrencyMetadataNormalizer` (Services/Currency) | Different jobs (value normalization vs metadata building). | KEEP — rename one if names confuse. |
| Calendar trio: `CalendarScheduleNormalizer` (189) + `calendar_schedule_normalizer.hook.php` (14-line wiring) + `CalendarRecurrencePayloadNormalizer` (118) | Distinct concerns (save-time entity normalization vs recurrence payload shape) + thin hook wiring. | KEEP. |

## The `_Core` normalization stack (guard-flagged, mostly cleared)

- `FieldNormalizer` (481 lines, **29 users**) — the guard flags it as a
  "homegrown definition engine" (worsened attention item). **False positive:**
  the docblock states it is a deliberate, audit-recommended extraction from
  `FieldLoader` (pure functions, no static state, no I/O). Recorded in
  `10-tool-fit-log.md` as a heuristic overreach — suppression candidate via
  `suppress_finding` with reason, NOT a code change.
- `LayoutNormalizer` (`Services/_Core/LayoutNormalizer.php`) — in the 18-file
  cycle (see 08); cycle membership is the issue, not the class itself.

## Long tail (single-purpose, fine)

AdminMenuItem, AggregateTarget, AiToolFilter, AttachmentFieldValue,
BankTransactionAmount, CalendarRecurrencePayload, CustomFieldChange,
DocumentTemplateDesign, EmailSignatureHtml, EmailViewStatePreference,
EntityType, PaymentSymbol, SpreadsheetFieldMapping, SpreadsheetLoadRequest,
UrlField, ViewConfigPreference, XmlFeedFilter normalizers; AiParsing, EspoWorkflow
(migration tooling — see 03), Fio, Gmail/Outlook folder, InboundMessage, Matomo,
Merk, StreamNotePivot mappers; ETL `Transformer`; `UomConverter`;
`HookAwareMapper` (Cycle ORM bridge — framework wiring, see 08).

## Ledger line

- **1 merge action** (Dashlet/Report alias normalization, ~100 lines saved)
- **1 suppression action** (`FieldNormalizer` guard flag is wrong — suppress with the docblock as the reason)
- Everything else: keep. 42 names ≠ 42 problems.
