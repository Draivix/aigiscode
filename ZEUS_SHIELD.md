# ZEUS_SHIELD.md

## Purpose

Zeus Shield is the doctrine for AigisCode as a guardian system.

The product must not only detect issues in code.
It must help prevent architectural drift, dependency sprawl, security decay,
and AI-generated incoherence before those problems become normal.

The purpose of Zeus Shield is:

- protect architectonic quality
- protect security posture
- preserve coherence of evolving systems
- prevent local optimization from destroying global structure
- keep AI coding agents aligned with the real doctrine of a codebase

## The Problem

Software can be written in endlessly many ways.
That freedom is dangerous.

Human developers and AI agents both tend to drift toward:

- shortest-path implementations
- copy-paste evolution
- parallel patterns for the same concern
- unnecessary new libraries
- bypassing native framework capabilities
- hidden runtime coupling
- convenience over coherence
- security shortcuts
- “just one more abstraction”

This is how codebases decay.

Zeus Shield exists to resist that decay.

## Core Belief

The main idea of AigisCode is not only to analyze codebases after the fact.
Its deeper role is to act as a guardian for AI and human engineering work so
that changes remain understandable, aligned, and structurally healthy.

The guardian must prevent the same kinds of damage that weak engineering causes:

- dependency cancer
- architectural entropy
- duplicated mechanisms
- framework misuse
- runtime sprawl
- uncontrolled conventions
- insecure shortcuts

## Guardian Definition

Zeus Shield should become:

> a system that prevents local optimization from destroying global coherence

That means the product must judge code changes and codebases against
architectural invariants, not only syntax or isolated defects.

## What Must Be Protected

The guardian protects five things:

1. Structural coherence
   - clear dependency direction
   - understandable module boundaries
   - limited cycle formation
   - low mechanism duplication

2. Runtime coherence
   - dynamic behavior is explicit and explainable
   - framework conventions do not silently collapse the graph
   - runtime expansion is modeled instead of guessed

3. Pattern coherence
   - one concern should not acquire five parallel implementations
   - existing local patterns should be reused
   - native platform capabilities should be preferred over unnecessary
     dependencies

4. Security coherence
   - security findings must be normalized into the same decision pipeline
   - supply-chain and dependency risk must be treated as architecture risk
   - insecure convenience should not be allowed to masquerade as engineering

5. Historical coherence
   - the system should detect regressions between versions
   - a codebase should converge, not randomly mutate

## The Five Shields

Zeus Shield must operate through five layers of defense.

### 1. Canonical Understanding

The guardian needs an accurate model of the codebase:

- modules
- symbols
- dependencies
- runtime edges
- framework conventions
- hotspots
- security surfaces
- existing implementation patterns

Without this, the guardian is blind.

### 2. Architectural Doctrine

Every repository should be judged against explicit doctrine:

- approved patterns
- forbidden patterns
- allowed dependencies
- native-first rules
- boundary rules
- extension rules
- security rules
- cost rules

Examples:

- do not add a library for capability already provided by the platform
- do not introduce a second HTTP client
- do not bypass the internal mail gateway
- do not access ORM directly from controllers
- do not introduce a new configuration mechanism in parallel to the current one

### 3. Change Governance

The guardian must judge diffs, not only snapshots.

The critical questions are:

- did this change introduce a new cycle
- did it deepen an old dependency collapse
- did it add a new dependency category
- did it create a parallel mechanism
- did it bypass an existing internal path
- did it weaken boundaries

This is the difference between reporting mess and preventing new mess.

### 4. Cost And Redundancy Detection

Much bad engineering is not incorrect.
It is unjustifiably expensive, redundant, or incoherent.

Zeus Shield must detect:

- unnecessary new dependencies
- duplicate capabilities
- redundant abstraction layers
- local reinvention of existing solutions
- framework bypasses
- library-over-native misuse

The canonical example:

- adding a large library to WordPress for sending email when native functions
  already exist

This is not just a style issue.
It is architectural and operational waste.

### 5. AI Guidance Loop

The guardian must shape agent behavior before and after code is written.

Before:

- what patterns are already approved here
- how is this concern already solved in this repo
- what must not be introduced
- what dependencies are already sanctioned

After:

- did this change violate doctrine
- did it create drift
- did it increase entropy
- should a rule or policy entry be generated

## Product Modes

AigisCode should operate in four modes:

### Understand

Build an explainable model of the codebase.

### Guard

Evaluate changes against doctrine and architecture.

### Converge

Encode reviewed truth into rules and policy so the system gets quieter and more
precise over time.

### Advise

Give AI agents and humans guidance before they write code.

## Required Product Questions

For every important change, the guardian should be able to answer:

- what problem is this change solving
- is there already a native/framework/internal way to solve it
- is this introducing a second pattern for the same concern
- does this strengthen or weaken architecture
- does this increase dependency entropy
- does this introduce hidden runtime coupling
- does this worsen security posture
- should this be allowed, warned, or blocked

## Knowledge Classes

To perform its role, Zeus Shield needs five knowledge classes:

1. Structural knowledge
   - imports
   - calls
   - graph edges
   - SCCs
   - bottlenecks

2. Platform knowledge
   - what WordPress, Laravel, Django, Rails, Rust, and other ecosystems already
     provide

3. Repository knowledge
   - what this repo already standardized on
   - which internal services and patterns are canonical

4. Organizational knowledge
   - what the team allows, forbids, prefers, or treats as too expensive

5. Historical knowledge
   - what changed
   - what regressed
   - what converged

## Enforcement Model

Zeus Shield should support three levels of judgment:

- Allow
  - aligned with doctrine
  - no meaningful architectural or security regression

- Warn
  - questionable
  - increases complexity or divergence
  - needs human review

- Block
  - violates explicit doctrine
  - introduces unjustified dependency or mechanism sprawl
  - materially worsens architecture or security

## Architectural Doctrine For Implementation

To make Zeus Shield real inside AigisCode:

- generic language truth belongs in core
- runtime and framework conventions belong in plugins
- repo-specific accepted behavior belongs in rules and policy
- security tool findings must be normalized into the same typed review/report
  pipeline
- graph edges must preserve evidence and layer meaning
- the system must distinguish structural truth from runtime-expanded behavior

## Anti-Goals

Zeus Shield must not become:

- a vibes-based code reviewer
- a bag of repo-specific hacks
- a weak replacement for stronger analyzers
- a flat finding list without architectural meaning
- a system that optimizes for count instead of signal

## Definition Of Success

Zeus Shield succeeds when AigisCode can reliably say:

- this is a real structural defect
- this is a runtime/plugin-expanded effect
- this is a framework artifact
- this is unjustified dependency expansion
- this is a duplicate mechanism
- this is allowed locally and should not waste review time
- this security issue is real and tied to this architectural surface

And when AI agents use that guidance to stop introducing decay.

## Development Rule

When implementing AigisCode under Zeus Shield:

- prefer system memory over local improvisation
- prefer doctrine over convenience
- prefer native/internal mechanisms over redundant new dependencies
- prefer explicit architectural truth over flattened heuristics
- prefer convergence over noisy detection
- prefer enforceable, typed contracts over taste
