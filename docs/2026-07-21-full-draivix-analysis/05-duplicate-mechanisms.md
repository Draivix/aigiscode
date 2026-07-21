# 05 — Duplicate Mechanisms & Dual Paths

Detector pool: 14 "Duplicate mechanism" + 19 "Abstraction sprawl" findings,
plus manual overlap checks. Headline: **most of this axis is architectural
fact, not slop** — and two detector families need suppressions, not code edits.

## Deliberate dispatch fan-out (KEEP — accepted architecture)

The 14 "Duplicate mechanism" findings are all the same shape: one domain
change fanning out through lifecycle hooks + event bus + queue jobs
(`*.hooks.php` + `EntityChangedEvent` + listeners + `*Job`). This is the
codebase's documented "one change, many channels" design (realtime broadcast,
search reindex, notifications all need the same event). `HookAwareMapper`
must fan out by construction — it is the Cycle ORM event bridge.

**Verdict:** not dual paths. Batch-suppress with reason
"sanctioned fan-out architecture" rather than re-reading 14 findings per run.

## ServiceProvider/registrar "sprawl" (HEURISTIC NOISE — suppress)

10 of 19 "Abstraction sprawl" findings flag `*ServiceProvider.php` files for
"spreading one concern across too many abstraction roles" — that is what a
Laravel ServiceProvider **is** (the module's wiring index). Most of the rest
are registry/resolver idioms (`BankPaymentProviderRegistry`,
`ActivityCallProviderRegistry`, `PolicyDefinitionProvider`) — the same
plugin pattern the codebase uses *correctly* elsewhere (ProviderFolderMapper,
PreferenceValueNormalizer — see 04).

**Verdict:** my day-one suspicion confirmed — idiom, not sprawl. Suppress
family-wide for `*ServiceProvider.php`; keep the detector for hand-rolled
stacks. The one worth a look: `AiConversationModePolicyResolver` (860
confidence) — policy-resolution growing a registry habit; check when its
module is next touched, not now.

## Checked manually — NOT duplicates

- `EntityExportService` (398 lines, generic entity export) vs
  `ProductMovementSummaryExportService` (484 lines, "Legacy-compatible export
  of warehouse movement summary") — different concerns; the legacy one is a
  domain report with Espo output compatibility, not a second export engine.
  Their 16 findings each are hardwiring → `07`.
- `FieldNormalizer` (extraction, 04) / `LayoutNormalizer` (cycle, 08) — not duplicates.

## Real dual-path found: XLSX emission

`XlsxExportWriter` (dead shared streamer, `Services/Spreadsheet/`) vs **8+
direct PhpSpreadsheet users** (`AccountantPackageBuilder`,
`FinancialStatementSpreadsheetExporter`, `LedgerBookSpreadsheetExporter`,
`CsobSupplierInvoicesXlsxBuilder`, `ExportWorkTimeEnumToExcel`, …). The
shared writer died because it only streams flat rows; real exporters need
styling/multi-sheet.

**Verdict:** DELETE the dead writer (already in 02). The broader "every
exporter rolls its own XLSX" is a *future design decision* (rich shared
writer or per-exporter freedom) — recorded for the owner, not a cleanup item.

## Ledger line

- 1 batch suppression (dispatch fan-out, 14 findings)
- 1 family suppression (`*ServiceProvider.php` sprawl, 10+ findings)
- 1 deletion (dead XlsxExportWriter — already counted in 02)
- 1 owner note (XLSX strategy)
