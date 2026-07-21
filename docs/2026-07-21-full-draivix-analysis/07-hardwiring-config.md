# 07 — Hardwiring & Config Drift

Family: 134 findings (109 repeated literals + 22 hardcoded networks + env/bypass pressure).

## The one clean mechanical batch: integration base URLs → `config/services.php`

22 "Hardcoded network" findings, and they cluster beautifully — each
integration hardcodes its base URL in source:

| Cluster | Sites | Fix |
|---|---|---|
| Carrier tracking URLs (`Shipment.php:209-215`) | 7 URLs in one array (dpd, ppl, zasilkovna, postaonline, gls, dhl, ups) | One `config/carriers.php` map |
| `login.microsoftonline.com` (`OutlookOAuthService`) | 5 | `services.outlook.base_url` |
| `fioapi.fio.cz` (`FioClient`, `FioStatementClient`) | 3 | `services.fio.base_url` |
| `api.signi.com` (`SigniClient`) | 3 | `services.signi.base_url` |
| `graph.facebook.com`, `api.linkedin.com`, `api.telegram.org` (Marketing providers) | 3 | per-provider config entries |

**Verdict:** one mechanical config-centralization batch (~22 findings, zero
behavior change, testable per provider). Best cost/benefit ratio in the whole
hardwiring family. Bonus: EU/dev/prod base-URL switching becomes possible.

## Repeated literals (109) — mostly small-fry

Top values: XML declarations (`<?xml encoding…` ×7), XLSX package paths
(`xl/workbook.xml` etc.), DOM selectors in the PasswordVault addon, small HTML
fragments. Constant-worthy at best, config-worthy nowhere. **Verdict:** skip;
fix opportunistically when touching those files.

## Sanctioned path bypass (155) — structural, not a sweep

Raw `env()`/container lookups outside config/bootstrap boundaries. The
injection recipe exists from prior rounds (bypass-injection doctrine); this is
a per-module migration habit, not a batch cleanup. Keep the ratchet (no new
bypasses) rather than a big-bang rewrite.

## Ledger line

- **1 mechanical batch** (22 findings, integration URLs → config)
- skip repeated literals; ratchet bypass family
