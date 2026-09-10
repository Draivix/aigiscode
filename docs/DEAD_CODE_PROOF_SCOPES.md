# Dead-code evidence scopes

Native absence checks now depend on the evidence scope they require. A parse gap
elsewhere no longer removes a complete-file PHP unused-binding candidate.

| Scope | Required evidence | Limit |
| --- | --- | --- |
| `local_binding` | Complete source extraction for the importing file; binding use checked in captured source. PHP aliases can be checked without resolving their external target. | Preserve import initialization effects in languages where imports execute code. Detected dynamic namespace inspection defers the file. |
| `class_private_dispatch` | Complete PHP class and resolved trait sources, with named/lexical use checks. | Unresolved trait scope and detected dynamic dispatch defer the claim. Reflection, generated code and externally bound closures remain runtime-review obligations. |
| `module_reachability` | Complete native input coverage and an untruncated analysis boundary. | External consumers and dynamic entrypoints remain possible; module candidates are heuristic. |
| `runtime_registration` | Module scope plus complete supplemental registration evidence. | Computed or external registration and intended replacement wiring are not disproved. |

The parser preserves PHP `protected` visibility separately from `private`.
Protected dispatch cannot be justified by a complete class file alone. Trait
uses participate in the private method's required source scope. PHP lexical
checks account for case-insensitive aliases and methods and for use after an
import on the same line.

`deterministic-findings.json.dead_code` contains `scope_coverage` and each finding's
typed `proof`. The architecture surface, report summary, convergence record and
MCP coverage expose `dead_code_scope_coverage`. These counts describe eligible
or deferred proof scopes, not numbers of unused symbols. Dedicated dead-code
guardian packets carry typed proofs into both the agent contract and graph
packets. Reviews distinguish runtime entries, test/support code, incomplete
migrations, scoped unreachability and unknowns. Runtime-entry conclusions must
retain their implementation.

## September 10 Draivix observation

The live repository's existing `.aigiscode/scan.json` selects `app`, `routes`,
`config` and `resources/js`, with test/fixture and other directory exclusions.
The observation therefore covers a **truncated slice**, not the earlier broad
17,067-source corpus. The analyzer read Draivix and wrote artifacts only under
AigisCode's ignored `.tmp/dead-code-scopes` directory.

The configured run parsed 9,662 supported sources and produced ten dead-code
candidates. It retained all four previously reviewed cases:

- `Project.fields.php:4` — `ProjectStatusGroups`, local binding.
- `CompleteFieldServiceTripAction.php:22` — `RuntimeMessage`, local binding.
- `ServiceTripRouteOptimizationService.php:15` — `RuntimeMessage`, local binding.
- `EntityViewHandler.php:1297` — `hydrateManyToManyForShow`, class-private scope
  including `AssertsRecordAccess.php`; runtime obligations remain explicit.

It reported 8,910 eligible local-binding files and 752 deferred files, with
16,264 eligible private-dispatch symbols and 3,921 deferred symbols. Module and
registration checks were deferred and no orphan candidates were emitted. Native
and secondary-scanner coverage remained incomplete; the CLI returned status 1.
These results do not establish that a provider or intended replacement is dead.

When an orphan candidate can be emitted, its advisory action is now `investigate`,
replacing `probably_delete`. An unwired alternative needs a wiring/ownership
decision before it can become a removal proposal.

A subsequent CLI read of the real Project entity directory verified the emitted
dead-code task packet, its `local_binding` proof, binding location and removal
obligation. The boundary remained explicitly truncated. The configured run was
observed before the final case-folding and dedicated-packet refinements; it is
evidence for scope separation, not a final whole-repository acceptance result.

A separate native analysis of the four-file Entity handler directory omitted the
trait source. It reported 63 deferred private-dispatch symbols and no private
dead-code candidates. Thus the same method is reviewable when its trait scope is
captured and deferred when that required source is missing.

Production builds succeeded. Regression cases for unrelated parse gaps, trait
callbacks, dynamic dispatch and protected visibility were added but not run.
Tests and CI remain paused. No application workflow or deletion was executed.
