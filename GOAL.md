# GOAL.md

## Mission

`ZEUS_SHIELD.md` describes the guardian doctrine that operationalizes this
mission for AI and human engineering work.

AigisCode exists to help ensure the architectonic quality and security of
analyzed software.

The product is not only a linter and not only a graph visualizer.
It is a whole-codebase reasoning system that should reconstruct how software is
really wired, identify structural and runtime risks, classify what is truly
actionable, and get more precise over time.

## Product Goal

The primary goal of AigisCode is:

- ensure architectonic quality of analyzed software
- ensure security posture of analyzed software
- do this through explainable analysis, not opaque guesses

That means the system should use:

- native graph analysis
- language-aware semantic resolution
- well-known analyzers and ecosystem tools where they are stronger than bespoke
  in-house logic
- policy, rules, and review layers to reduce false positives over time

## What "Architectonic Quality" Means

AigisCode should help answer questions like:

- is the dependency structure clean and explainable
- are there real architectural cycles
- are there dangerous bottlenecks, god modules, or orphaned components
- are runtime conventions hiding coupling that the team does not see
- are boundaries between layers, modules, and responsibilities actually holding
- is the system becoming easier or harder to evolve

The product should distinguish:

- structural truth
- runtime/framework-expanded behavior
- project-specific accepted patterns
- actual defects
- likely artifacts or low-value noise

## What "Security" Means

AigisCode should not try to replace strong security tools with weaker homemade
logic.

Instead, the product should:

- integrate well-known SAST, secrets, dependency, license, and supply-chain
  analyzers
- normalize those results into one typed architecture/review/report pipeline
- preserve raw evidence
- connect security findings to the architecture graph where possible
- treat security as part of system quality, not a separate afterthought

## Core Product Principles

1. Explainability over magic.
   Every important finding should be traceable to graph edges, facts, tool
   output, or explicit rules/policy.

2. Better graphing than `../nexus`.
   AigisCode should preserve richer edge meaning, stronger evidence, better
   cycle classification, and clearer separation of structural vs runtime
   coupling.

3. Use the best analyzer for each job.
   If an ecosystem already has a strong maintained analyzer, AigisCode should
   integrate it instead of recreating it badly.

4. Core correctness stays generic.
   Language semantics belong in core, framework conventions belong in plugins,
   and repo-specific accepted patterns belong in rules/policy.

5. Convergence is a first-class feature.
   The system should get quieter and more precise as teams review findings and
   encode accepted patterns.

## Product Architecture Contract

The product should be built in layers:

- Core graph semantics
  - parsing
  - symbol extraction
  - resolution
  - typed graph edges
  - generic deterministic analysis

- Runtime and framework plugins
  - Laravel
  - Django
  - Rails
  - Inertia
  - ORMs
  - queues
  - event systems
  - template and page linkage

- Rules and policy
  - repo-specific accepted patterns
  - tolerated local conventions
  - allowed literals
  - entrypoints
  - suppressions with explicit reasons

- Review and reporting
  - actionable findings
  - accepted-by-policy findings
  - suppressed-by-rule findings
  - AI/human triage surfaces
  - historical convergence reporting

## Analyzer Strategy

AigisCode should combine:

- its own semantic graph engine
- its own architecture analysis
- ecosystem-native analyzers
- security scanners
- dependency analyzers
- framework/runtime plugins

The differentiated value of AigisCode should be:

- graph reconstruction
- architecture reasoning
- evidence-preserving normalization
- layered interpretation
- convergence through rules/policy/review

Not:

- rewriting every ecosystem analyzer from scratch
- stuffing repo-specific hacks into the core
- flattening all findings into one noisy list

## Success Criteria

The software is successful when it can reliably say:

- this is a real structural cycle
- this is a runtime/plugin-expanded cycle
- this is likely a framework artifact
- this is a real dead-code or hardwiring issue
- this is accepted locally and should not waste review time
- this security issue is real and how it relates to the system architecture

And when teams can use it to:

- improve architecture intentionally
- reduce security risk
- understand large codebases faster
- guide AI coding agents with trustworthy machine-readable outputs

## Non-Goals

AigisCode should not become:

- a bag of repo-specific regexes
- a weak clone of existing best-in-class analyzers
- a report generator without a rigorous graph model
- a system that optimizes only for finding count instead of signal quality

## Development Rule

When making product and architecture decisions:

- prefer improvements that strengthen generic graph truth
- push framework-specific behavior into plugins
- push repo-specific accepted behavior into rules or policy
- preserve evidence and layered meaning
- optimize for architectonic quality and security of analyzed software
