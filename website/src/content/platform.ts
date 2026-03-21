/* -------------------------------------------------------------------------- */
/*  Platform Feature Data — AigisCode Marketing Website                       */
/* -------------------------------------------------------------------------- */

export interface PlatformFeature {
  slug: string;
  icon: string;
  title: Record<string, string>;
  shortDescription: Record<string, string>;
  metaDescription: Record<string, string>;
  content: Record<string, string>;
  highlights: string[];
  codeExample?: string;
  relatedSlugs: string[];
}

type LocaleKey =
  | 'en'
  | 'cs'
  | 'fr'
  | 'es'
  | 'zh'
  | 'hi'
  | 'pt'
  | 'ar'
  | 'pl'
  | 'bn';

const LOCALES: LocaleKey[] = ['en', 'cs', 'fr', 'es', 'zh', 'hi', 'pt', 'ar', 'pl', 'bn'];

function localized(text: string): Record<string, string> {
  return Object.fromEntries(LOCALES.map((locale) => [locale, text]));
}

function feature(
  slug: string,
  icon: string,
  title: string,
  shortDescription: string,
  metaDescription: string,
  content: string,
  highlights: string[],
  relatedSlugs: string[],
  codeExample?: string,
): PlatformFeature {
  return {
    slug,
    icon,
    title: localized(title),
    shortDescription: localized(shortDescription),
    metaDescription: localized(metaDescription),
    content: localized(content),
    highlights,
    relatedSlugs,
    codeExample,
  };
}

export const platformFeatures: PlatformFeature[] = [
  feature(
    'open-source',
    'GithubLogo',
    'Open Source',
    'AigisCode is MIT-licensed, inspectable, and built in the open around a native Rust analysis core.',
    'Open-source Rust codebase analysis with machine-readable artifacts, guard-mode direction, and GitHub-first development.',
    `<h2 id="open-source">Open Source Rust Core</h2>
<p>AigisCode is developed in the open on GitHub. The product runtime is a native Rust analyzer, not a hosted black box and not a sidecar Python shell. The source, artifacts, and architecture contract are all inspectable.</p>
<h2 id="how-it-ships">How It Ships</h2>
<p>The canonical product surface is the Rust CLI plus the machine-readable artifact family it writes into <code>.aigiscode/</code>. That gives teams a portable code-understanding engine they can run locally, in CI, or inside agent workflows.</p>`,
    [
      'MIT-licensed Rust product surface',
      'Inspectable artifact contracts',
      'GitHub-first development workflow',
      'No hosted-only black box',
    ],
    ['cli-tools', 'architecture-analysis', 'report-generator'],
    `cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode`,
  ),
  feature(
    'plugin-system',
    'PuzzlePiece',
    'Plugin System',
    'Framework semantics live in Rust-native overlays that enrich the graph without mutating core language truth.',
    'Runtime plugins add framework-aware edges, entry points, and non-structural evidence to the canonical graph.',
    `<h2 id="plugin-system">Runtime Plugins</h2>
<p>AigisCode keeps parsing and resolution generic, then layers framework and runtime meaning on top through Rust-native plugins. This is how queue dispatch, Laravel container wiring, and WordPress hooks become visible without flattening everything into ordinary structural imports.</p>
<h2 id="design-boundary">Design Boundary</h2>
<p>Language truth belongs in core. Framework behavior belongs in plugins. Repository-specific accepted behavior belongs in policy and rules. That boundary is what keeps the graph explainable and reusable across repositories.</p>`,
    [
      'Rust-native framework overlays',
      'Typed runtime edges',
      'Explainable plugin-origin evidence',
      'Separate from repo policy and rules',
    ],
    ['architecture-analysis', 'extension-api', 'policy-engine'],
    `{
  "relationKind": "EventSubscribe",
  "layer": "Framework",
  "origin": "Plugin(wordpress_hooks)"
}`,
  ),
  feature(
    'policy-engine',
    'Sliders',
    'Policy Engine',
    'Policy and rules adapt the analyzer to each repository without turning core detectors into repo-specific hacks.',
    'Project doctrine, suppressions, thresholds, and accepted patterns are encoded in policy and rules.',
    `<h2 id="policy-engine">Policy And Rules</h2>
<p>The analyzer should learn repositories, not be rewritten for them. Policy and rules encode accepted entry points, skip paths, literal allowances, and reviewed suppressions while keeping core graph logic generic.</p>
<h2 id="convergence">Convergence</h2>
<p>This is part of the Zeus Shield model: repeated false positives move into policy or rules, while real structural or security findings remain visible and actionable.</p>`,
    [
      'Repository-specific doctrine layer',
      'Suppressions without detector hacks',
      'Convergence-oriented workflow',
      'Feeds review and handoff artifacts',
    ],
    ['custom-detectors', 'report-generator', 'plugin-system'],
    `{
  "hardwiring": {
    "skip_path_patterns": ["app/Console/**"]
  }
}`,
  ),
  feature(
    'report-generator',
    'FileText',
    'Report Generator',
    'The Rust pipeline emits consolidated machine-readable reports, review surfaces, and handoff contracts for humans and agents.',
    'Structured JSON artifacts expose findings, graph summaries, review state, feedback loop metrics, and handoff context.',
    `<h2 id="report-generator">Artifact Family</h2>
<p>AigisCode does not stop at one flat report. It emits layered artifacts such as <code>semantic-graph.json</code>, <code>dependency-graph.json</code>, <code>evidence-graph.json</code>, <code>review-surface.json</code>, <code>aigiscode-handoff.json</code>, and <code>aigiscode-report.json</code>.</p>
<h2 id="why-it-matters">Why It Matters</h2>
<p>Different consumers need different levels of detail. Architecture views want low-noise dependencies. Agents and reviewers need evidence, feedback-loop state, and next-step handoff context.</p>`,
    [
      'Consolidated Rust artifact family',
      'Feedback-loop metrics',
      'Agent handoff JSON contract',
      'MCP-friendly machine surfaces',
    ],
    ['architecture-analysis', 'cli-tools', 'policy-engine'],
    `jq '.feedback_loop' .aigiscode/aigiscode-report.json`,
  ),
  feature(
    'architecture-analysis',
    'Graph',
    'Architecture Analysis',
    'AigisCode builds a layered graph so structural, runtime, and framework relationships can be analyzed without collapsing into noise.',
    'Canonical semantic graph plus dependency and evidence projections for architecture, debugging, and agent workflows.',
    `<h2 id="architecture-analysis">Layered Graphing</h2>
<p>The canonical semantic graph preserves language truth and edge evidence. On top of that, AigisCode emits a normalized <code>dependency_view</code> for low-noise architecture work and an <code>evidence_view</code> for call-site and runtime detail.</p>
<h2 id="why-layered">Why Layered</h2>
<p>One flat graph is not enough. Structural imports, runtime dispatch, and framework overlays have different meanings. The product gets better when it preserves those distinctions instead of hiding them behind one big edge count.</p>`,
    [
      'Canonical semantic graph',
      'Normalized dependency view',
      'Detailed evidence view',
      'Typed cycle classification',
    ],
    ['plugin-system', 'report-generator', 'extension-api'],
    `aigiscode graph /path/to/repo --kuzu`,
  ),
  feature(
    'cli-tools',
    'Terminal',
    'CLI Tools',
    'The native Rust CLI is the supported runtime surface for analysis, reporting, graph export, MCP access, and artifact generation.',
    'Analyze repositories, materialize graph views, generate reports, and expose MCP resources from one Rust command surface.',
    `<h2 id="cli-tools">Native Rust CLI</h2>
<p>The CLI is intentionally small. It orchestrates scan, resolve, analysis, graph projection, review surfaces, and artifacts, but it does not own detector logic or report semantics. Those stay in typed Rust modules.</p>
<h2 id="supported-commands">Supported Commands</h2>
<p>Use <code>aigiscode analyze</code> for the full pipeline, <code>aigiscode report</code> for fast re-evaluation, <code>aigiscode graph</code> for graph projections and Kuzu materialization, and <code>aigiscode mcp</code> for the native MCP server.</p>`,
    [
      'Single Rust runtime surface',
      'Analyze, report, graph, and MCP commands',
      'Artifact-first workflow',
      'CI-friendly command shape',
    ],
    ['report-generator', 'architecture-analysis', 'open-source'],
    `aigiscode analyze ../wordpress`,
  ),
  feature(
    'custom-detectors',
    'MagnifyingGlass',
    'Rule Packs',
    'Repository-specific guardrails are expressed through rule packs, policy, and reviewed suppressions instead of custom sidecar runtimes.',
    'Encode project doctrine without patching the Rust core or reintroducing a second runtime.',
    `<h2 id="rule-packs">Rule Packs</h2>
<p>Rule packs are how a repository teaches AigisCode about its own accepted patterns. They define tolerated entry points, review-backed suppressions, and local doctrine while leaving core graph truth untouched.</p>
<h2 id="authoring">Authoring Surface</h2>
<p>Today the practical authoring surfaces are <code>.aigiscode/policy.json</code> and <code>.aigiscode/rules.json</code>. They are explicit, reviewable, versioned with the repository, and fit naturally into the feedback loop.</p>`,
    [
      'Repository doctrine without patching core',
      'Policy and rules stay versioned in the repo',
      'Same lifecycle as native findings',
      'Supports convergence over time',
    ],
    ['policy-engine', 'extension-api', 'plugin-system'],
    `[
  {
    "finding_type": "dead_code.unused_import",
    "file_pattern": "app/Legacy/**"
  }
]`,
  ),
  feature(
    'extension-api',
    'Puzzle',
    'Runtime Plugins',
    'Framework and runtime semantics are expressed through typed Rust-native plugins, not Python module loading.',
    'Plugin overlays add framework-aware edges and runtime evidence while preserving a single graph contract.',
    `<h2 id="runtime-plugins">Runtime Plugins</h2>
<p>Runtime plugins enrich the graph with framework-specific meaning such as queue dispatch, WordPress hook publish-subscribe behavior, and Laravel container resolution. They are part of the Rust analyzer surface, not an external module loader.</p>
<h2 id="public-contract">Public Contract</h2>
<p>Plugin-derived edges appear in the same artifact family as structural edges. That means MCP, the cockpit, and future guard mode reason over one graph family instead of disconnected subsystems.</p>`,
    [
      'Rust-native plugin overlays',
      'Framework-aware typed edges',
      'Shared artifact contract',
      'No Python sidecar runtime',
    ],
    ['plugin-system', 'architecture-analysis', 'policy-engine'],
    `{
  "relationKind": "Dispatch",
  "layer": "Runtime",
  "origin": "Plugin(queue)"
}`,
  ),
];

/* -------------------------------------------------------------------------- */
/*  Helpers                                                                   */
/* -------------------------------------------------------------------------- */

export function getPlatformFeatureBySlug(slug: string): PlatformFeature | undefined {
  return platformFeatures.find((f) => f.slug === slug);
}

export function getRelatedPlatformFeatures(slug: string): PlatformFeature[] {
  const feature = getPlatformFeatureBySlug(slug);
  if (!feature) return [];
  return feature.relatedSlugs
    .map((s) => platformFeatures.find((f) => f.slug === s))
    .filter(Boolean) as PlatformFeature[];
}
