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
