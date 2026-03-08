<p align="center">
  <br />
  <strong>AigisCode</strong>
  <br />
  <em>AI-powered code guardian — static analysis that watches your entire codebase</em>
  <br />
  <br />
</p>

<p align="center">
  <a href="https://pypi.org/project/aigiscode/"><img src="https://img.shields.io/pypi/v/aigiscode?color=blue&label=PyPI" alt="PyPI version" /></a>
  <a href="https://pypi.org/project/aigiscode/"><img src="https://img.shields.io/pypi/pyversions/aigiscode" alt="Python 3.12+" /></a>
  <a href="https://github.com/david-strejc/aigiscode/actions"><img src="https://img.shields.io/github/actions/workflow/status/david-strejc/aigiscode/ci.yml?label=CI" alt="CI status" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green" alt="MIT License" /></a>
</p>

---

**Aigis** (Αἰγίς) is the ancient Greek word for _Aegis_ — the divine shield.
The first two letters happen to be **AI**.

AigisCode is a whole-codebase evaluator for large, mixed-language projects.
It combines deterministic static indexing with structural analysis, detector passes,
policy-driven adaptation, and AI-assisted review to find the problems that
single-file linters miss: circular dependencies, dead code, hardwired values,
layer violations, and architectural bottlenecks.

## Quick Start

```bash
pip install aigiscode
cd your-project
aigiscode analyze .
```

That's it. AigisCode indexes your source, builds a dependency graph, runs
detectors, applies policy rules, optionally asks an AI backend to review
the results, and generates both human-readable and machine-readable reports.

## What Does AigisCode Find?

| Category | Examples |
|---|---|
| **Circular dependencies** | Module A imports B, B imports C, C imports A — both strong (architectural) and total (runtime/load) cycles |
| **Dead code** | Unused imports, unreferenced private methods, orphaned properties, abandoned classes |
| **Hardwired values** | Magic strings, repeated literals, hardcoded IPs and URLs, env access outside config |
| **Layer violations** | A controller importing directly from a view, a model reaching into middleware |
| **Structural risks** | God classes, bottleneck files, orphan modules with no inbound dependencies |
| **Runtime contracts** | Routes, hooks, env vars, config keys — extracted and cross-referenced against findings |

Every finding includes file path, line number, category, confidence level, and a
suggested fix. False positive rates are driven down by contract-aware filtering
and optional AI review.

## How It Works

AigisCode runs a six-stage pipeline:

```
 Source Code
     |
     v
 +---------+     +----------+     +----------+     +---------+     +-----------+     +----------+
 |  Index  | --> |  Graph   | --> |  Detect  | --> |  Rules  | --> | AI Review | --> |  Report  |
 +---------+     +----------+     +----------+     +---------+     +-----------+     +----------+
  tree-sitter     dependency       dead code        saved rules     classify          JSON + MD
  Python AST      analysis         hardwiring       pre-filter      true_positive     contract
  SQLite store    cycles,          magic strings    false           false_positive    inventory
  symbols,        coupling,                         positives       needs_context     metrics
  dependencies    layers
```

**1. Index** — Parses source files with tree-sitter (PHP, TypeScript, JavaScript, Vue) and Python AST. Stores files, symbols, dependencies, and semantic envelopes in a local SQLite database. Supports incremental re-indexing.

**2. Graph** — Builds a file-level dependency graph with NetworkX. Computes circular dependencies (strong vs. total), coupling metrics, bottleneck files, layer violations, god classes, orphan files, and runtime entry candidates.

**3. Detect** — Runs generic detector passes for dead code and hardwiring. Detectors emit candidates with confidence levels; they do not encode project-specific logic.

**4. Rules** — Applies saved exclusion rules from `.aigiscode/rules.json` to pre-filter known false positives. Rules are the durable memory of prior audits.

**5. AI Review** — Sends a sample of remaining findings to an AI backend (OpenAI Codex or Anthropic Claude) for classification as `true_positive`, `false_positive`, or `needs_context`. Proposes new exclusion rules from confirmed false positives.

**6. Report** — Generates a structured JSON report and a human-readable Markdown summary. Includes a contract inventory (routes, hooks, env keys, config keys) and full metric breakdowns.

## Supported Languages

| Language | Index | Dead Code | Hardwiring | Parser |
|---|:---:|:---:|:---:|---|
| PHP | yes | yes | yes | tree-sitter |
| Python | yes | yes | yes | Python AST |
| TypeScript | yes | yes | yes | tree-sitter |
| JavaScript | yes | yes | yes | tree-sitter |
| Vue | yes | yes | yes | tree-sitter |
| Ruby | yes | -- | yes | tree-sitter |

Detector coverage is reported explicitly. When a language is indexed but a
detector does not yet support it, the report flags partial coverage instead of
silently treating it as fully analyzed.

## CLI Commands

```
aigiscode index <path>        Parse and store the codebase index
aigiscode analyze <path>      Full pipeline: index + graph + detect + review + report
aigiscode report <path>       Re-generate report from existing index (fast re-evaluation)
aigiscode tune <path>         AI-guided policy tuning with regression guards
aigiscode info <path>         Show index stats and detector coverage
aigiscode plugins             List available plugins and their policy fields
```

Key flags:

```
--skip-ai                     Run without AI backends (deterministic only)
--analytical-mode             Ask AI to propose a policy patch
--reset                       Full re-index (ignore incremental cache)
-P <plugin>                   Select a built-in plugin profile
--plugin-module <path.py>     Load an external Python plugin module
--policy-file <path.json>     Override policy from a JSON file
-v / --verbose                Enable debug logging
```

## Configuration

AigisCode is policy-driven. Instead of hard-coding project-specific behavior
into the analyzer, you express it through a JSON policy file with four sections:

```json
{
  "graph": {
    "js_import_aliases": { "@/": "src/" },
    "orphan_entry_patterns": ["src/bootstrap/**/*.ts"],
    "layer_violation_excludes": ["resources/js/**"]
  },
  "dead_code": {
    "abandoned_entry_patterns": ["/Contracts/"],
    "abandoned_languages": ["php"]
  },
  "hardwiring": {
    "repeated_literal_min_occurrences": 4,
    "skip_path_patterns": ["app/Console/*"],
    "js_env_allow_names": ["DEV", "PROD", "MODE"]
  },
  "ai": {
    "allow_claude_fallback": true
  }
}
```

Policy is merged in layers: built-in defaults, selected plugins, auto-detected
plugins, external plugin modules, project file (`.aigiscode/policy.json`),
and ad-hoc `--policy-file`. Later layers override earlier ones.

### Built-in Plugins

| Plugin | Description |
|---|---|
| `generic` | Safe defaults for mixed-language repositories (always loaded) |
| `django` | Django-aware runtime conventions and entry points |
| `wordpress` | WordPress admin and hook conventions |
| `laravel` | Laravel-specific entry points and dynamic contexts |

### External Plugins

Write a Python module with `build_policy_patch()` and optional runtime hooks:

```python
def build_policy_patch(project_path, selected_plugins):
    return {
        "dead_code": {
            "abandoned_entry_patterns": ["/app/Legacy/"]
        }
    }

# Optional: refine results at runtime
def refine_graph_result(graph_result, graph, store, project_path, policy):
    return graph_result

def refine_dead_code_result(dead_code_result, store, project_path, policy):
    return dead_code_result
```

```bash
aigiscode analyze /repo --plugin-module ./my_plugin.py
```

## For AI Agents

AigisCode is designed to be consumed by other AI agents, not just humans.

The primary machine interface is the JSON report:

```
.aigiscode/aigiscode-report.json
```

It contains structured data for every finding category, metric, and contract
inventory — ready for downstream planning, triage, and automated remediation
without parsing prose.

Recommended agent workflow:

1. `aigiscode analyze /repo` — generate baseline
2. Read `.aigiscode/aigiscode-report.json` — parse structured findings
3. Sample findings and classify (true positive / false positive / uncertain)
4. Encode narrow policy for repeated false positives
5. `aigiscode report /repo` — fast re-evaluation after policy changes
6. `aigiscode tune /repo -i 2` — optional AI-guided policy refinement

Key JSON fields for agents:

- `graph_analysis.strong_circular_dependencies` — architectural cycle triage
- `graph_analysis.circular_dependencies` — broader runtime context
- `dead_code` — unused imports, methods, properties, classes
- `hardwiring` — magic strings, repeated literals, hardcoded network
- `extensions.contract_inventory` — routes, hooks, env keys, config keys

See [docs/AI_AGENT_USAGE.md](docs/AI_AGENT_USAGE.md) for the full agent integration guide.

## Architecture

The system separates generic analysis from project-specific interpretation
across four layers of responsibility:

1. **Index and graph construction** — generic, language-aware parsing
2. **Generic detectors** — emit candidates, not verdicts
3. **Policy and exclusion rules** — project-specific adaptation
4. **AI review and tuning** — final-stage classification

Design principles: decoupling over convenience, explainable heuristics over
opaque model-only decisions, partial but explicit coverage over false certainty.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full design document.

## Requirements

- Python 3.12+
- Optional: `OPENAI_API_KEY` or `ANTHROPIC_API_KEY` for AI-assisted review

Core dependencies: tree-sitter, NetworkX, Pydantic, Typer, Rich.

## Contributing

Contributions are welcome. Please see [CONTRIBUTING.md](CONTRIBUTING.md) for
guidelines on development setup, testing, and pull request conventions.

Before patching the analyzer core, consider whether the issue can be expressed
through policy or a plugin module. The design boundary is intentional:
generic analysis logic belongs in the core, project-specific behavior belongs
in policy.

## License

[MIT](LICENSE)

---

<p align="center">
  <sub>AigisCode — your codebase's shield against architectural decay.</sub>
</p>
