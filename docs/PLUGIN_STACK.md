# Plugin Stack

This document defines the plugin/model stack AigisCode should grow into.

The product should not treat every extension as just "a plugin".
Different layers solve different problems and have different trust levels.

## Core Principle

Language truth belongs as low as possible.
Framework/runtime semantics belong above that.
Repository doctrine belongs above both.

So the stack is:

1. Language frontends
2. Semantic model packs
3. Framework/runtime plugins
4. Doctrine, policy, and rules

## 1. Language Frontends

These are not optional repo plugins.
They are product-native analyzers that own syntax and symbol truth.

Responsibilities:
- parsing
- symbol extraction
- import/reference discovery
- type/member resolution support
- exact file/span anchors

Examples:
- PHP parser and resolver
- Python parser and resolver
- TypeScript parser and resolver
- Ruby parser and resolver
- Rust parser and resolver

This layer should produce:
- canonical symbols
- canonical references
- exact spans where available
- language-native evidence for downstream findings

This is the high-trust layer.

## 2. Semantic Model Packs

This layer captures language- or ecosystem-specific meaning that is still more
"semantic model" than "runtime convention".

Examples:
- Laravel container helper resolution model
- Django route/signal model
- WordPress REST route registration model
- PHP hook-map declaration model
- Python settings access model

These packs should:
- enrich the graph with typed meaning
- declare provenance and precision
- provide evidence anchors
- stay deterministic

The right mental model is similar to CodeQL model packs:
- core extractor stays generic
- model layer adds framework/library semantics

## 3. Framework/Runtime Plugins

This is the current `plugins/` territory, but it should be explicitly described
as runtime/framework expansion, not generic code truth.

Examples:
- queue dispatch edges
- signal publish/subscribe edges
- WordPress hook registration and dispatch
- container resolution edges

These plugins should:
- emit runtime/framework edges
- attach evidence anchors and confidence
- never override language truth
- remain explainable

They are allowed to be convention-aware, but they should not be repo hacks.

## 4. Doctrine, Policy, and Rules

This is the top layer.

The final codebase should absolutely have rules, but rules are not enough on
their own.

The stack should end with:

- Doctrine registry
  - sanctioned mechanisms
  - forbidden boundaries
  - preferred pathways
  - ownership and expiry-backed exceptions

- Policy
  - project-wide tolerances
  - thresholds
  - accepted categories
  - skip patterns

- Rules
  - narrow audited exclusions
  - finding-specific suppressions
  - per-tool or per-family exceptions

Current state:
- `policy.json` exists
- `rules.json` exists
- built-in doctrine registry now exists as first-class typed data via `.aigiscode/doctrine-registry.json`
- repo-owned doctrine extensions, waivers, owners, and expiry are still missing

So the current answer is:
- yes, we already have policy/rules
- yes, the final layer now exists in product form
- no, it is not complete until doctrine becomes repo-owned and enforceable beyond built-in guardian clauses

## Recommended Product Shape

AigisCode should converge toward this contract:

- Core language frontends: exact / modeled truth
- Semantic model packs: framework/library meaning
- Runtime plugins: dynamic edge expansion
- Doctrine registry: organizational law
- Policy/rules: reviewed local tolerance

That gives us:
- exact parsing and spans
- explainable framework/runtime behavior
- repo-specific accepted truth
- diff-aware guard decisions

## Immediate Next Steps

1. Keep language parsing/resolution in core.
2. Introduce explicit plugin/model categories in the Rust contracts.
3. Add provenance/precision/evidence anchors to runtime/model outputs uniformly.
4. Extend the doctrine registry from built-in guardian clauses to repo-owned sanctioned mechanisms, waivers, and expiry.
5. Use the combined stack for:
   - dependency-overkill detection
   - native-vs-library misuse
   - framework bypass detection
   - overengineering judgment
   - guard-mode obligations
