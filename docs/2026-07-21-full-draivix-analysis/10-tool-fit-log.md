# 10 — Tool-Fit Log: aigiscode as the instrument

How the tool actually fit in hand for this analysis. Same standard as the rest
of the ledger: only what I measured or hit myself.

## What worked (and got used hard)

- **Dead-code family: 100% precision on the tail.** All 18 findings survived
  full-repo verification. On a repo cleaned by prior rounds, the remaining
  pool being this trustworthy is what makes a deletion ledger possible at all.
- **God-class findings post accessor-discount.** The survivor list
  (`EntityManager` 643 dependents, `EmailService`, `PermissionChecker`)
  matches what hand audits said before — the day-one entity noise is gone.
- **Determinism.** Multiple full analyzes during this session: byte-identical
  artifacts, and convergence deltas (99 new / 84 resolved) are real repo
  changes, not phantom churn. Trustworthy enough to diff.
- **Honesty labels.** 49% of findings being scan-config echo was *visible*
  (boundary-truncated). Knowing why beats hiding it — it re-scoped the whole
  ledger correctly.
- **Contract inventory.** Every spot-check landed on the right line
  (`Shipment.php:209` DPD URL day one; carrier array; channel/http_call
  buckets for cross-service reading).
- **Speed.** Full pipeline ≈54s on 7.4k files (was 3:02 at session start).
  Analysis-as-a-loop is viable; I ran it 15+ times without thinking about it.

## What was noise (and needed my judgment, every time)

- **"Abstraction sprawl" on `*ServiceProvider.php`** — Laravel idiom flagged
  as smell (10+ findings). Needs family suppression, not code review.
- **Guard "homegrown definition engine" on `FieldNormalizer`** — the docblock
  documents it as the audit-recommended extraction. The heuristic can't read
  intent; suppression with the docblock as the recorded reason is the answer.
- **"Duplicate mechanism" fan-out** — deliberate hooks+events+jobs
  architecture, 14 findings. Batch-suppress once; stop re-triaging.
- **Raw finding counts.** 2,544 is not a workable number. The interface that
  made it workable was `precision` + `family` + `severity` filtering into
  compact briefs (this session's P0 work — used constantly).

## Gaps hit during this analysis

- **No class-inventory query.** The normalizer/converter census (42 classes)
  was `find` + `grep` by hand. A `class_inventory(name_pattern)` tool (or a
  guaranteed-fresh `cypher_query`) answers that in one call.
- **No clone/similarity detection.** Dashlet-vs-Report alias normalizers were
  found by reading bodies. Near-duplicate detection (CBM's MinHash/LSH idea)
  is the honest next feature for the "shit code" use case.
- **Benchmark hygiene.** The watch daemon re-analyzes on edits and competes
  with benchmark runs — I killed daemons mid-session to measure cleanly. A
  `--quiet-watch` or benchmark mode would prevent that.
- **Out-of-slice blindness is explicit but manual.** Commands/Tests exclusion
  meant supplemental greps for verification. The boundary-truncated label
  made this safe; it didn't make it automatic.

## Dogfooding proof for this session's own changes

- Compact briefs + precision filters: the difference between "2,544 findings"
  and a working triage queue.
- Property resolution (`$this->prop`): `EmailService` edges at 1170/1247 are
  in the graph now; they weren't at session start. Coverage visibly improved.
- Accessor discount: killed exactly the false god-classes flagged on day one.
- The determinism purge paid off *during* this analysis — two of the leaks
  were found by my own seq/par artifact diff.

## Method honesty

I did **not** re-verify all 2,544 findings — nobody can. The method was
axis-driven mining with **100% source verification on every removal
candidate** (A-section of `09`) and family-level judgment on the rest. Where
judgment could be wrong, the doc says so (suppressible families are marked as
such, not silently dropped).

## Retrospective (post-review, same day)

The top-30 list (`11`) was independently double-checked (see
`12-verified-priority-list.md`): **17/30 clean, 11 right-thesis-wrong-numbers,
2 recommendations that would have caused damage.** Re-verified the reviewer's
counter-evidence myself; it holds. The failures, honestly owned:

1. **"Zero refs" is not a dead-code proof.** `AutomaticInvoiceRecalculationService`
   was called verified-dead — it is alive via convention dispatch
   (`AccountingRecomputeAggregatesCommand.php:283` builds
   `'…Services\'.$entityType.'RecalculationService'` and `app()`s it; entity
   list comes from `HasAggregates` convention discovery). My grep checked
   *literal references only*. The correct checklist for "dead": literal refs +
   **class-string construction, `app($var)`, `call_user_func`, scandir/
   convention discovery, module manifests, hook wiring**. Draivix's own
   `ErpAnalyzeDeadCodeCommand.php:319` encodes exactly this convention — I
   never found it. The tool's orphan heuristic made the same miss first
   (heuristic tier, no dynamic-dispatch channel); my human check rubber-stamped
   it. Tool backlog: generic dynamic-dispatch evidence as a dead-code
   *suppression* channel (suppression-only, cannot fabricate findings).
2. **Slice-scoped numbers presented as repo-wide.** Every dependent count I
   published was undercounted by the analyzed slice (EntityManager 643 vs 789,
   EmailService 52/24 vs 111/~30, Microsoft URLs ×5 vs ×35). Rule now: every
   aggregate carries its scope and method, or it does not get published.
3. **"Duplicate" verdicts need body diffs, not size contrast.**
   `EmailViewStatePreferenceNormalizer` (292 lines of bespoke schema migration)
   vs the 32-line sibling: zero shared logic. Dropped from the list.
4. **Check the carrier is alive before designing its config home.**
   `getTrackingLink()` had zero consumers — the right action was delete, not
   `config/carriers.php` (which also violated their platform layering law).
5. **Unreproducible aggregates need methodology attached** — and prior repo
   docs cited. The knot's "18 files / 848 edges" sits next to the repo's own
   `docs/old` analysis showing an earlier SCC inflated 151→3 under per-edge
   verification; both belong in the same sentence.

Reviewer verdict on the rest: 17/30 fully confirmed. The corrected,
re-prioritized list is `12-verified-priority-list.md` — it supersedes `11`.

