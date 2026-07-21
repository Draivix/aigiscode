# 12 — Verified Priority List (doublecheck of doc 11)

Verified 2026-07-21 against the live draivix working tree by 6 parallel read-only
verification passes (per-claim `wc -l`, method greps, import-based dependent counts,
bidirectional cycle greps, adversarial dead-code caller hunts including dynamic dispatch).

**Score: 17 findings fully confirmed, 11 confirmed-with-corrections, 2 materially wrong
(#24 recommendation, #29a deletion target), 0 stale paths.** Every file exists where doc 11
says it does. Aggregate scan numbers (848 edges, 255 violations, 155 bypasses, 633 hotspots)
are reproducible only via the analyzer tooling; every concrete code example checked out.

## Corrections that change the action

| # | What doc 11 says | What's actually true |
|---|---|---|
| 29a | `AutomaticInvoiceRecalculationService` dead — delete | **ALIVE.** Reached by convention dispatch: `AccountingRecomputeAggregatesCommand::resolveRecalculationService()` builds `'App\Modules\Accounting\Services\'.$entityType.'RecalculationService'` (`AccountingRecomputeAggregatesCommand.php:283`, invoked `:151`); `AutomaticInvoiceAggregateService implements HasAggregates` puts `AutomaticInvoice` in the loop. Deleting breaks `accounting:recompute-aggregates`. |
| 24 | `EmailViewStatePreferenceNormalizer` 292 lines duplicates sibling's logic — extract shared preference code | **No shared logic exists.** The 292 lines are bespoke email view-state schema migration (v1→2, paneFilters, senderRules, filterModel); the 32-line sibling only normalizes pageSize. Nothing extractable. Size contrast real, issue not. **Drop.** |
| 26 | ModuleRegistry: 160 static call sites, **0 instance** | 160 static confirmed exactly, but **26 constructor-injection sites exist** (`ModuleGeneratedRouteServiceProvider.php:24`, `DashboardService.php:30`, `ModuleBridge.php:18`, …). Instance API is live; "bless static" is no longer the obvious call. Docblocks do lie ("Static facade for backward compatibility" ×7). |
| 20 | EspoMigration shim = 807 lines + config + **2 commands**, archivable | 807 lines exact, but **23 commands** reference it and it is **load-bearing in live runtime**: `MigrationSideEffectGuard` inside `HookAwareMapper.php:21`, `NotificationService.php:11`, `EntityDeleteSideEffects.php:10`; `CanonicalFieldAliasResolver` inside `EntityUiConfigService.php:14` + both alias normalizers; PDO reader bound in `AppServiceProvider.php:65-66`. Archival requires decoupling first. |
| 15, 16 | Email/Mattermost job⇄service cycles = restructure targets | Cycles real, but this is the standard Laravel shape (service `::dispatch`es job, job resolves service in `handle()`); `EmailSyncManager.php:113` comment even documents it. Near-zero payoff for restructuring. Deprioritize. |
| 11 | Fix = `config/carriers.php` | Platform law: tenant-variable business config belongs in module settings registry / metadata, not new PHP config. Tracking-URL templates are borderline (public carrier constants) — pick layer per doctrine before implementing, don't default to a new config file. |

## Number drift (thesis holds, magnitude worse than doc 11)

- #4 EntityManager dependents: **789 non-test / 941 with tests** (claim 643).
- #7 EmailService: **111 methods (54 public), ~30 importers** (claim 52 / 24).
- #21 Email.php: 113 public methods, **152 importers** (claim 128).
- #22 PermissionChecker: 30 public methods, **211 importers** (claim 170).
- #12 hardcoded base URLs: Microsoft **×35** non-test (claim ×5); FB/LI/Telegram+WhatsApp 17 across 6 files.
- #2 `User.hooks.php`: **5 services / ~6 call sites** (claim 4).
- #28 Pohoda: **11 dead method defs** across 6 names (claim 10) — live template-hook siblings (`getSpecificDateMappings`, `mapPohodaStatusToEspoStatus`, partner-link hooks) must NOT go with them.
- #8 env() outside config/: only **18 non-test** occurrences; the 155 is a curated analyzer subset dominated by container pulls (`app()` non-test ≈ 1040 repo-wide).
- #1 core knot: mutual recursion confirmed hard both directions (16 files Entities/_Core→Services/_Core, top targets `TenantManager`/`TenantDb` ×8 each; 53 files back). But "18 files / 848 edges" matches no reproducible measure, and the repo's own `docs/old/2026-07-04-core-knot-deep-analysis.md` found the earlier 151-file SCC collapsed to 3 files / 215 edges after per-edge verification. Re-scope before investing.

## Priority list

### P0 — do first (cheap, prevents damage / restores analysis trust)
1. **Correct doc 11 itself**: strike #24, mark #29a ALIVE, fix #26 "0 instance", re-scope #20. Prevents someone executing a breaking deletion. (This doc is that correction.)
2. **#30 scan.json**: `ignored_dir_names` drops `Commands`/`commands` — 1,241 boundary-truncated findings and all console code invisible. One-line config change, re-run scan, every future number gets trustworthy.

### P1 — verified quick wins (delete/config, hours each)
3. **#29 b–e**: delete `ResourceBookingService::confirm()` (`:306`), `FIELD_YES/NO/READONLY` (`PermissionLoader.php:112-116`), `VacationRequestController::getUserRoleIds` (`:117`), `ShipmentWorkflowService::tableColumns`/`toFloat` (`:702`,`:837`). All adversarially verified dead. **Not** `AutomaticInvoiceRecalculationService`.
4. **#28**: delete the 11 Pohoda clone methods (keep live sibling hooks).
5. **#25 (D half)**: delete dead `XlsxExportWriter` (zero callers; docs already flag it idle). The rich-shared-writer decision stays separate (→ P3 #25-O).
6. **#11**: move 7 carrier URLs out of `Shipment.php:209-215` — layer per doctrine (settings registry vs config), not automatic `config/carriers.php`.
7. **#26 (docblock half)**: fix the 7 lying "backward compatibility" docblocks now; API-direction decision → P3.

### P2 — high-value structural work (days each, start when scheduled)
8. **#2 layer violations**: confirmed pattern (`Email.actions.php:3`, `User.hooks.php` ×5, `ActivityHistoryLoader.php:7,130`, `CalendarConfigLoader.php:5,111`). Per-module injection recipes exist; ratchet new ones via lint.
9. **#12**: centralize provider base URLs (35 Microsoft occurrences is a real refactor, not a config tweak; batch per provider module).
10. **#6**: split `Email/Index.vue` (5,487) + `EmailPreview.vue` (5,312) — slimmer split components already exist beside them to grow into.
11. **#7**: split `EmailService` — worse than doc says (111 methods, 3+ seams: account lifecycle / sync+serialize / send+draft).
12. **#3**: split `AccountingProductionEvidenceService` (8,511 L, 12 identifiable concern clusters, 140 private methods).
13. **#5**: split `ChatService` (5,080 L, 28 public; coordination god-class — conversation CRUD / turn orchestration / live-voice / provider facade / telemetry).
14. **#23**: split `MattermostClient` (47 public, 5+ domains).
15. **#9**: hoist the confirmed hot sites (`ActivityHistoryLoader.php:496-507` json_decode-in-row-loop first); treat 633 as a worklist source, not a target.

### P3 — design projects / owner decisions (re-scope before investing)
16. **#21** Email.php read-model split (152 importers — big blast radius, plan needed).
17. **#22** PermissionChecker extraction (211 importers).
18. **#4** EntityManager hub (789 deps — design project, explicitly not a sweep).
19. **#1** _Core knot — re-run analyzer first; own history says SCC numbers deflate under per-edge verification. Follow existing phased plan in `docs/old/2026-07-04-core-knot-deep-analysis.md` only after re-scope.
20. **#8** sanctioned-path bypasses — define the boundary rules in the analyzer, then ratchet; raw env() portion is only ~18 non-test sites (fix those in-slice).
21. **#26 (direction half)** ModuleRegistry: decide static vs instance with the knowledge that both are live (160 vs 26 sites).
22. **#13, #17, #18** real cycles, all deliberately lazy-broken and stable — interface extraction when touching those areas anyway.
23. **#27**: extract only the genuinely shared alias-walk primitive (Dashlet 138 L vs Report 242 L; Report keeps its extra behavior).
24. **#19/#20** Espo migration unit: owner decision, corrected scope — 4,804-line command archivable with the unit, but the 807-line shim needs `MigrationSideEffectGuard` + `CanonicalFieldAliasResolver` decoupled from live runtime first (they're in the ORM mapper and delete/notification paths).
25. **#10** `CommandFieldWritePolicy` in_array (`:52`, `:181`) — real but micro; writer lists are tiny. Fix opportunistically.

### P4 — drop from the list
26. **#24** — non-issue (see corrections).
27. **#15, #16** — standard framework job⇄service pattern; not worth restructuring.
28. **#14** — back-edge is static string-helpers only (`FilterJsonbHandler.php:236-246`); cosmetic.
