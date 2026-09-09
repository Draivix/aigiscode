# Draivix triage now identifies its supporting evidence

The full audit no longer recommends dismantling the 41-line CRM metadata resolver
registry based on naming and the entire `app` zone's 32,635 relations. All 25
abstraction-role candidates remain present with unchanged IDs, now at low review
priority and with explicit instructions to validate responsibilities first.

The [machine receipt](2026-09-09-triage-priority-evidence.json) records the before
and after recommendation, source and engine identities, retained candidates,
bounded-preview inspection, baseline state and actual MCP results.

## Verified result

The full preserved Draivix snapshot was audited in 77.20 seconds with
3,510,828 KiB peak RSS. Native graph analysis, raw architectural assessment and
security results are unchanged. All 3,218 logical findings are `NotCompared`;
none is described as resolved or improved after these engine changes. Coverage
remains incomplete and the command returns 1 after writing its artifacts.

Every selected cluster has its priority evidence in the bounded preview, every
primary target belongs to its zone, and the existing presentation limits hold.
The real MCP topology tool returned the same recommendation and structured steps
as the artifact. Its cached session produced the first useful overview in 44.03
seconds and topology immediately afterward. These are single-run observations,
not statistical performance claims.

The new `priority_basis` binds priority and precision to one actual finding or
packet. Selection now considers all zone inputs before trimming previews; it
previously took ten IDs alphabetically. Text and structured steps derive from
the same selected clusters, and zone totals no longer determine local urgency.
The pure ordering logic moved into `artifacts/triage.rs`, separate from emission.
See the [contract](TRIAGE_PRIORITY_CONTRACT.md).

## What the first recommendation actually means

The first three `app` review targets now concern declared layer contracts:

| Source | Observed dependency |
| --- | --- |
| `app/Entities/Email/Email.actions.php:3,32` | Registers `EmailActionHandler` from the services layer |
| `app/Entities/Email/Email.php:36` | Declares `EmailAuditRecordEnricher` in entity metadata |
| `app/Entities/EmailFilter/EmailFilter.hooks.php:4` | References the existing Email filter service from lifecycle hooks |

The supplied doctrine allows the `entities` layer to depend on `core` and
`support`, not `services`. The references are real and violate that machine
contract. The detector's `certain` label describes that match; it does not prove
that these live registrations should be removed.

The actionable decision is to reconcile the sanctioned metadata and lifecycle
boundary with doctrine. Preserve the handlers and registrations while reviewing
that contract; do not introduce wrappers or move service behavior into entities
merely to silence a path rule. No runtime failure was reproduced here, and no
Draivix code or doctrine was changed.

The CRM registry remains a useful negative control. It owns a provider map,
serves live consumers and participates in tenant reset. Its shared naming concepts
do not establish redundant responsibility. The new packet language reflects that
limit instead of instructing an unconditional collapse.

## Verification limits and remaining work

Production Rust builds passed without warnings. Regressions for mixed evidence
on one file, foreign primary targets, deterministic ordering and consistent steps
were added but not run. Automated tests and CI remain disabled under David's
instruction. A separate real self-audit processed 114 sources in 2.35 seconds;
it still reports native and secondary coverage gaps.

This repairs demonstrated guidance defects, not every detector or priority.
Doctrinal intent, source-level semantics and business impact still require review.
Incremental processing, whole-family publication, remaining oversized modules,
secondary scanning limits, the optional Node path and broader Q01–Q12 acceptance
remain open.
