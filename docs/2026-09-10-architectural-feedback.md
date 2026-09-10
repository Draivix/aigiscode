# Native architectural review and feedback observation

The six AigisCode implementation streams in
[the simplification plan](ARCHITECTURAL_SIMPLIFICATION_PLAN.md) now connect parsed
behavior and wiring evidence to typed proposals, source validation, native
publication, explicit repository policy and convergence. The
[review contract](ARCHITECTURAL_REVIEW_CONTRACT.md) documents the APIs and limits.

This observation uses the configured Draivix source slice and native Rust MCP.
No hosted model or delegated agent was invoked: the reviewing assistant authored
the typed proposals and submitted them through the product API. Draivix source
and repository policy were not edited. Runtime application behavior and the
regression suite were not executed; tests and CI remain paused.

## Reviewed decisions

| Case | Submitted action | Preserved contract / unresolved evidence |
| --- | --- | --- |
| Actor attribution | Propose consolidation into the existing `ActorContext` | Six same-input comparisons cover explicit IDs, invalid/non-positive IDs, user properties, `getId()` and ambient auth. Migrate persistence and notification consumers only after approving precedence and fallback. No approval or runtime equivalence is implied. |
| SmsService provider and invoice sender | Keep both runtime boundaries | The module-discovery source includes the provider convention; Accounting registers the sender against its dispatcher contract. Preserve boot callbacks, console registration and invoice/email preconditions. Bootstrap itself is outside this captured slice and was not executed. |
| ACL, sort-target, filter-type and view-column replacements | Investigate all four | Captured wiring leaves integration intent unresolved. Preserve ACL after-commit/transaction-outcome behavior, sort metadata/null ordering, tenant-specific physical types and the distinction between unsupported and dynamic-unsliced boards. Zero captured consumers does not establish deletion safety. |
| Three unused PHP imports | Delete only the selected local bindings | Native local-binding evidence is complete. Keep executable statements and the imported classes at their defining paths. Application/CI acceptance is pending. |
| Private relation hydration | Investigate | The native candidate remains present; reflection, generated code and externally bound closures still require runtime review. This is not a proven deletion. |
| Accounting production evidence | Propose a responsibility split | Separate artifact transport from evidence acceptance decisions while preserving the public facade, existing profile/input-normalization owners, tenant boundaries and missing/malformed-data behavior. Choose the extraction owner and execute acceptance checks before changing Draivix. |

## What the native workflow establishes

Five records contain twelve typed claims. Each accepted submission passed native
snapshot, task-binding, source-path and quoted-span validation and was archived
under its content identity. JSON is authoritative; the latest Markdown is rendered
from the same native record. The curated
[receipt](2026-09-10-architectural-feedback.json) retains identities, decisions,
source locations and the observation limits; full source quotations stay in the
private local review exports.

A provider adoption preview reads its archived record after later reviews have
replaced the latest export. It returns a reasoned `accepted_pattern` entry with
`applied: false`, empty finding fingerprints and `NotCompared`, so it neither
suppresses a diagnostic nor invents baseline-backed drift. Actual policy adoption
was not performed against Draivix.

The workflow rejected a citation to excluded `bootstrap/providers.php`; the
provider proposal was revised to cite captured discovery evidence and state the
missing bootstrap coverage. An invented quote was also rejected without replacing
the last valid review. These are native API observations on the real source slice,
not results from an automated test suite.

The configured slice still has incomplete input coverage: 16 recovered sources,
720 scope-limited Vue scripts and two unsupported Go files among 9,662 parsed
sources. Current observations therefore do not establish new confirmed drift or
a clean whole-application audit. Native validation anchors the reasoning; it does
not certify the architectural conclusion, authorize a business-contract change,
or verify runtime preservation.

A fresh MCP load restored the native graph/analysis and incorporated the latest
review as `SourceAnchoredProposal`: the three imports became
`SourceReviewedProposal`, while the private hydration candidate stayed
`Unreviewed`. All 4,571 native findings remained visible; no proposal silently
changed repository policy. The real `tune` command completed with three
approval-required drafts, no review-loading errors, and no `reviewed_decisions`
entry in the written suggested-policy patch.

## Regression coverage authored

The added cases cover adoption without self-invalidation, unrelated-source
stability, caller/helper/configuration/registration changes, exact finding
acceptance, stale reopening, confirmed-concern priority despite broad policy,
missing baseline evidence, archive retention, read-only publication rejection and
unproven absence claims. Existing convergence cases now distinguish visible
regressions from raw observations. These cases remain unexecuted under the current
pause, as do application acceptance checks and the broader Q01–Q12 obligations.
