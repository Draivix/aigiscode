# Plugin System

AigisCode uses a Rust-native plugin and overlay model.

## Current Plugin Scope

The current product already emits framework and runtime meaning through native
Rust plugins layered on top of the canonical semantic graph. Current shipped
slices include:

- queue and job dispatch runtime edges
- Laravel container-resolution edges
- WordPress hook and filter publish-subscribe edges

These plugins enrich the graph without changing the language-truth layer in
core parsing and resolution.

## Design Boundary

Plugin responsibilities:

- framework and runtime conventions
- plugin-derived edges and facts
- non-structural overlays such as dispatch, container resolution, or hook
  publish-subscribe behavior

Core responsibilities:

- parsing and symbol extraction
- import, type, and call resolution
- canonical semantic graph truth

Policy and rules responsibilities:

- repository-specific accepted behavior
- suppressions, thresholds, and local doctrine
- convergence of reviewed false positives

## Public Contract

Plugin-derived behavior is visible through the normal Rust artifact family:

- `.aigiscode/semantic-graph.json`
- `.aigiscode/dependency-graph.json`
- `.aigiscode/evidence-graph.json`
- `.aigiscode/architecture-surface.json`

Plugin-produced edges must stay typed, layered, and explainable. New plugin
work should build on Rust contracts, not on Python module loading or sidecar
runtimes.
