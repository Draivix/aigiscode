# Large-codebase audit quality contract

This is a release acceptance contract, not a claim of market leadership.
It implements GOAL.md and ZEUS_SHIELD.md. Native Rust remains the supported engine;
repository-specific policy must not become core heuristics.

## Required evidence

1. **Graph truth:** a resolved method belongs to its receiver; importing an unrelated
   class is not receiver evidence. Container targets come from exact parsed calls.
   Type-only dependencies remain distinguishable from runtime dependencies.
2. **Explainable cycles:** component membership and an ordered, closed witness are
   separate fields. Each witness edge carries source location and provenance and
   belongs to the graph view being reported. Classification uses that same view.
3. **Honest scope:** generated inputs, tests, excluded callers and unsupported source
   are explicit. A suppression sweep must honor explicit generated-source exclusions.
   Missing evidence never becomes an unconditional deletion recommendation.
4. **Cache correctness:** fast load must reject artifacts from incompatible parser,
   resolver or plugin semantics even when package version and source bytes match.
5. **Operational integrity:** reachable input and tool failures are surfaced, not
   translated into clean analysis. Security integrations preserve tool status and
   raw evidence. Partial analysis must remain distinguishable from complete analysis.
6. **Large-repository behavior:** avoid repeated whole-graph scans per finding or
   component. Measure wall time, peak resident memory, source count, edge count and
   artifact volume with one process and fixed inputs before claiming improvement.
7. **Actionable review:** rank evidence-backed problems, distinguish unfinished
   integration from obsolete code, and expose the source of uncertainty. Naming
   similarity and file size alone do not prove redundant mechanisms or bad design.

## Verification protocol

- Preserve immutable source fingerprints and analyzer revision for each run.
- Use the maintained mixed-language regressions plus a real large-source corpus.
- Compare graph changes at source anchors, not by preferring fewer findings.
- Include negative controls: legitimate inheritance, optional/variadic calls,
  ordinary imports, dynamic framework entrypoints, and distinct namespaced classes.
- Verify determinism under input-order changes and cold/warm analysis agreement.
- Add regression tests at the owning boundary and run automated gates in approved CI.
  Local production CLI builds and read-only audit runs are artifact verification,
  not substitutes for CI tests.
- Competitive claims require a shared labeled corpus and reproducible runs of the
  compared tools. Unmeasured capabilities and unexecuted checks remain explicit.

## Current implementation revision

The September 8 audit and its preserved source corpus establish the baseline.
The active revision addresses graph truth, cycle evidence, scope contamination,
cache compatibility and scaling before expanding heuristic detector coverage.
The separate trust-remediation work remains tracked by its existing work lock;
this document does not assert completion of that contract.
