# Draivix audit coverage — 2026-09-10

A native AigisCode audit was performed on the current Draivix working tree,
combining repository-wide graph/scanner output with focused source review.
Detailed security findings and source evidence are retained privately in the
ignored local directory `.aigiscode/reviews/draivix-2026-09-10/`; they are not
published in this public repository.

The [machine coverage receipt](2026-09-10-draivix-aigiscode-audit.json) records
what was exercised without exposing unremediated security details.

| Coverage | Observation |
| --- | --- |
| Main configured slice | 9,662 parsed source files; 4,571 native findings |
| Native security screening | 38 sites across 26 source files |
| Dead-code screening | All 10 native candidates |
| Dependency screening | Witnesses for all 17 strong components; 20 total components |
| Focused native reviews | 11 source-anchored claims in 10 archived records, citing 26 files |
| Client companion graph | 2,435 parsed sources, including support/reference code |
| Bootstrap companion graph | Six parsed sources |
| Native MCP usage | 16 distinct successful methods |

Main coverage remains incomplete: 16 recovered sources, 720 scope-limited Vue
scripts and two unsupported Go files. The client graph also has coverage gaps
and returned exit status 1 while emitting its graph; bootstrap returned 0.
Companion slices do not prove cross-slice absence or runtime wiring.

The audit used native coverage, quality, findings, topology, module design,
implementation context, graph traces, impact radius, cycle, guard, convergence
and source-review publication capabilities. Source-reference validation does
not certify the semantic conclusion or reproduce a production incident.

Draivix source and policy were unchanged. Tests, CI, application runtime,
external SAST and dependency-advisory gates were not executed under the existing
pause. This audit is not an exhaustive manual review of every parsed line or a
production-readiness certification. Broader Q01–Q12 work remains open.
