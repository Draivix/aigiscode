import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import {
  Copy,
  Check,
  Terminal,
  GearSix,
  Code,
  BookOpen,
  ArrowRight,
  GithubLogo,
  MagnifyingGlass,
  ChartBar,
  Funnel,
  Robot,
  FileText,
  Database,
  Lightning,
  TreeStructure,
} from '@phosphor-icons/react';
import SEO from '@/components/SEO';
import Breadcrumbs from '@/components/Breadcrumbs';
import {
  SITE_URL,
  buildGraphJsonLd,
  howToEntity,
  webPageEntity,
} from '@/content/seo-schema';

const CLI_COMMANDS = [
  {
    command: 'aigiscode analyze <path>',
    key: 'index',
    icon: Database,
    descriptionFallback:
      'Run the native Rust scan, parse, resolve, and analysis pipeline for the target repository. This is the primary entrypoint and writes the canonical artifact family under .aigiscode/.',
  },
  {
    command: 'aigiscode analyze <path>',
    key: 'analyze',
    icon: MagnifyingGlass,
    descriptionFallback:
      'Emit deterministic analysis JSON for graph metrics, resolved edges, dead-code candidates, and hardwiring findings. The current product surface is Rust-only and deterministic by default.',
  },
  {
    command: 'aigiscode report <path>',
    key: 'report',
    icon: FileText,
    descriptionFallback:
      'Write the consolidated aigiscode-report.json artifact from the native Rust analyzer, including review and external-analysis overlays when enabled.',
  },
  {
    command: 'aigiscode surface <path>',
    key: 'tune',
    icon: Funnel,
    descriptionFallback:
      'Build the architecture-surface contract used by UI, atlas, and finding-first review workflows.',
  },
  {
    command: 'aigiscode --help',
    key: 'info',
    icon: ChartBar,
      descriptionFallback:
      'Display the supported Rust CLI commands and flags, including external-tool execution.',
  },
  {
    command: 'aigiscode surface',
    key: 'plugins',
    icon: Lightning,
    descriptionFallback:
      'Generate the architecture surface without reformatting or post-processing through a separate host runtime.',
  },
] as const;

const SUPPORTED_LANGUAGES = [
  {
    language: 'PHP',
    parser: 'tree-sitter',
    extensions: '.php',
    features: 'Classes, functions, imports, namespaces, traits',
  },
  {
    language: 'Python',
    parser: 'tree-sitter-python',
    extensions: '.py',
    features: 'Classes, functions, imports, decorators, type hints',
  },
  {
    language: 'TypeScript',
    parser: 'tree-sitter',
    extensions: '.ts, .tsx',
    features: 'Classes, functions, imports, interfaces, type aliases',
  },
  {
    language: 'JavaScript',
    parser: 'tree-sitter',
    extensions: '.js, .jsx, .mjs',
    features: 'Classes, functions, imports, CommonJS require',
  },
  {
    language: 'Vue',
    parser: 'tree-sitter',
    extensions: '.vue',
    features: 'SFC parsing, script setup, composables, template refs',
  },
  {
    language: 'Ruby',
    parser: 'tree-sitter',
    extensions: '.rb',
    features: 'Classes, modules, methods, requires',
  },
] as const;

const POLICY_EXAMPLE = `{
  "graph": {
    "orphan_entry_patterns": ["src/bootstrap/**", "bin/*"]
  },
  "dead_code": {
    "abandoned_entry_patterns": ["src/contracts/**"]
  },
  "hardwiring": {
    "repeated_literal_min_occurrences": 4,
    "skip_path_patterns": ["src/console/**"],
    "allowed_literals": ["localhost"]
  }
}`;

/* -------------------------------------------------------------------------- */
/*  Sub-components                                                            */
/* -------------------------------------------------------------------------- */

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      /* clipboard not available */
    }
  };

  return (
    <button
      onClick={handleCopy}
      className="p-1.5 rounded-md hover:bg-white/10 transition-colors"
      aria-label="Copy to clipboard"
    >
      {copied ? (
        <Check size={14} className="text-emerald-400" weight="bold" />
      ) : (
        <Copy size={14} className="text-zinc-500" />
      )}
    </button>
  );
}

function CodeBlock({ code, language }: { code: string; language?: string }) {
  return (
    <div className="relative rounded-xl bg-zinc-900 dark:bg-zinc-950 border border-zinc-800 dark:border-zinc-800/60 overflow-hidden">
      <div className="flex items-center justify-between px-4 py-2 border-b border-zinc-800 dark:border-zinc-800/60">
        <span className="text-xs text-zinc-500 font-mono">{language || 'shell'}</span>
        <CopyButton text={code} />
      </div>
      <pre className="p-4 overflow-x-auto text-sm font-mono text-zinc-300 leading-relaxed">
        <code>{code}</code>
      </pre>
    </div>
  );
}

function SectionHeading({
  id,
  icon: Icon,
  children,
}: {
  id: string;
  icon: React.ElementType;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center gap-3 mb-6">
      <div className="inline-flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500/10 to-purple-500/10 dark:from-indigo-500/20 dark:to-purple-500/20 text-indigo-500 flex-shrink-0">
        <Icon size={22} weight="duotone" />
      </div>
      <h2
        id={id}
        className="text-2xl md:text-3xl font-display font-bold tracking-tight text-zinc-900 dark:text-white"
      >
        {children}
      </h2>
    </div>
  );
}

/* -------------------------------------------------------------------------- */
/*  Docs Page                                                                 */
/* -------------------------------------------------------------------------- */

export default function DocsPage() {
  const { t } = useTranslation();

  const breadcrumbItems = [
    { label: t('docs.title', 'Documentation') },
  ];

  const breadcrumbSchema = [
    { name: t('nav.home', 'Home'), url: `${SITE_URL}/` },
    { name: t('docs.title', 'Documentation'), url: `${SITE_URL}/docs` },
  ];

  const graphJsonLd = buildGraphJsonLd([
    webPageEntity(
      `${SITE_URL}/docs`,
      'AigisCode Documentation',
      'Complete documentation for AigisCode: installation, CLI commands, configuration, supported languages, and integration guides.',
    ),
    howToEntity({
      name: 'How to Analyze a Codebase with AigisCode',
      description:
        'Step-by-step guide to installing AigisCode and running your first code analysis to detect circular dependencies, dead code, and hardwired values.',
      steps: [
        {
          name: 'Install AigisCode',
          text: 'Run cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode to install the native Rust CLI from GitHub.',
          url: `${SITE_URL}/docs#quick-start`,
        },
        {
          name: 'Run Analysis',
          text: 'Execute aigiscode analyze /path/to/project to run the full six-stage pipeline.',
          url: `${SITE_URL}/docs#cli-commands`,
        },
        {
          name: 'Review Results',
          text: 'Open .aigiscode/aigiscode-report.json or .aigiscode/review-surface.json for the consolidated Rust artifact contracts.',
          url: `${SITE_URL}/docs#cli-commands`,
        },
        {
          name: 'Customize Policy',
          text: 'Create .aigiscode/policy.json to configure detector sensitivity, import aliases, and framework-specific behavior.',
          url: `${SITE_URL}/docs#configuration`,
        },
      ],
    }),
  ]);

  return (
    <>
      <SEO
        title={t('docs.meta.title', 'Documentation - AigisCode')}
        description={t(
          'docs.meta.description',
          'Complete documentation for AigisCode: installation, CLI commands, configuration, supported languages, and integration guides. Get started analyzing your codebase in minutes.'
        )}
        canonical={`${SITE_URL}/docs`}
        breadcrumbs={breadcrumbSchema}
        jsonLd={graphJsonLd}
      />

      {/* Hero */}
      <section className="relative py-16 md:py-24 overflow-hidden">
        {/* Background glow */}
        <div className="absolute inset-0 pointer-events-none" aria-hidden="true">
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[300px] bg-gradient-to-r from-indigo-500/15 via-purple-500/10 to-pink-500/15 blur-[120px] rounded-full" />
        </div>

        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 relative z-10">
          {/* Breadcrumbs */}
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.4 }}
            className="mb-8"
          >
            <Breadcrumbs items={breadcrumbItems} />
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6 }}
          >
            <h1 className="text-4xl md:text-5xl lg:text-6xl font-display font-bold tracking-tighter text-zinc-900 dark:text-white">
              {t('docs.title', 'Documentation')}
            </h1>
            <p className="mt-4 text-lg md:text-xl text-zinc-600 dark:text-zinc-400 max-w-2xl leading-relaxed">
              {t(
                'docs.subtitle',
                'Everything you need to install, configure, and run AigisCode. From quick start to advanced policy customization, this guide covers the entire workflow for analyzing your codebase.'
              )}
            </p>
          </motion.div>

          {/* Table of Contents */}
          <motion.nav
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.15 }}
            className="mt-10 p-5 rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl"
            aria-label="Table of contents"
          >
            <h2 className="text-sm font-semibold text-zinc-900 dark:text-white mb-3 uppercase tracking-wider">
              {t('docs.toc', 'On this page')}
            </h2>
            <ul className="space-y-2 text-sm">
              {[
                { id: 'quick-start', label: t('docs.quickStart.title', 'Quick Start') },
                { id: 'cli-commands', label: t('docs.cliCommands.title', 'CLI Commands') },
                { id: 'configuration', label: t('docs.configuration.title', 'Configuration') },
                { id: 'supported-languages', label: t('docs.languages.title', 'Supported Languages') },
                { id: 'report-structure', label: t('docs.reportStructure.title', 'Report Structure') },
                { id: 'ai-integration', label: t('docs.aiIntegration.title', 'AI Integration') },
              ].map((item) => (
                <li key={item.id}>
                  <a
                    href={`#${item.id}`}
                    className="text-zinc-500 hover:text-indigo-500 dark:hover:text-indigo-400 transition-colors"
                  >
                    {item.label}
                  </a>
                </li>
              ))}
            </ul>
          </motion.nav>
        </div>
      </section>

      {/* Content Sections */}
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 pb-20 md:pb-32 space-y-20 md:space-y-28">
        {/* ---------------------------------------------------------------- */}
        {/*  Quick Start                                                     */}
        {/* ---------------------------------------------------------------- */}
        <motion.section
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.5 }}
          aria-labelledby="quick-start"
        >
          <SectionHeading id="quick-start" icon={Terminal}>
            {t('docs.quickStart.title', 'Quick Start')}
          </SectionHeading>

          <div className="prose-zinc space-y-5">
            <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">
              {t(
                'docs.quickStart.intro',
                'AigisCode is a native Rust CLI. Install it with the project installers or directly from GitHub with cargo, then analyze any codebase immediately. No repository configuration is required for the first run.'
              )}
            </p>

            <h3 className="text-lg font-display font-semibold text-zinc-900 dark:text-white mt-8 mb-3">
              {t('docs.quickStart.installTitle', '1. Install AigisCode')}
            </h3>
            <p className="text-xs text-zinc-500 dark:text-zinc-400 mb-2 font-medium uppercase tracking-wider">{t('docs.quickStart.macLinux', 'macOS / Linux')}</p>
            <CodeBlock code="curl -fsSL https://raw.githubusercontent.com/Draivix/aigiscode/main/install.sh | bash" />
            <p className="text-xs text-zinc-500 dark:text-zinc-400 mb-2 mt-4 font-medium uppercase tracking-wider">{t('docs.quickStart.windows', 'Windows (PowerShell)')}</p>
            <CodeBlock code="irm https://raw.githubusercontent.com/Draivix/aigiscode/main/install.ps1 | iex" />
            <p className="text-xs text-zinc-500 dark:text-zinc-400 mb-2 mt-4 font-medium uppercase tracking-wider">{t('docs.quickStart.pip', 'Or with cargo')}</p>
            <CodeBlock code="cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode" />

            <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mt-4">
              {t(
                'docs.quickStart.installNote',
                'The project installers fetch the native Rust binary layout for this repository. Direct cargo installation works anywhere a standard Rust toolchain is available.'
              )}
            </p>

            <h3 className="text-lg font-display font-semibold text-zinc-900 dark:text-white mt-8 mb-3">
              {t('docs.quickStart.analyzeTitle', '2. Analyze your project')}
            </h3>
            <CodeBlock code="aigiscode analyze /path/to/your/project" />

            <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mt-4">
              {t(
                'docs.quickStart.analyzeNote',
                'This runs the full six-stage pipeline: Index, Graph, Detect, Rules, AI Review, and Report. The analysis typically completes in under a minute for projects with fewer than 500 files. Results are written to the .aigiscode/ directory inside your project.'
              )}
            </p>

            <h3 className="text-lg font-display font-semibold text-zinc-900 dark:text-white mt-8 mb-3">
              {t('docs.quickStart.resultsTitle', '3. Review results')}
            </h3>
            <CodeBlock code="jq . .aigiscode/aigiscode-report.json" language="shell" />

            <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mt-4">
              {t(
                'docs.quickStart.resultsNote',
                'The artifact family contains low-noise dependency and evidence graph views, consolidated findings, the review surface, and an agent handoff JSON contract suitable for automation and MCP-driven code understanding.'
              )}
            </p>

            <h3 className="text-lg font-display font-semibold text-zinc-900 dark:text-white mt-8 mb-3">
              {t('docs.quickStart.skipAiTitle', '4. Run without AI (deterministic mode)')}
            </h3>
            <CodeBlock code="aigiscode analyze /path/to/project" />

            <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mt-4">
              {t(
                'docs.quickStart.skipAiNote',
                'The Rust CLI is deterministic by default. AI review and external tools are additive layers; the core scan, graph, detector, and report pipeline does not require an AI backend.'
              )}
            </p>
          </div>
        </motion.section>

        {/* ---------------------------------------------------------------- */}
        {/*  CLI Commands                                                    */}
        {/* ---------------------------------------------------------------- */}
        <motion.section
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.5 }}
          aria-labelledby="cli-commands"
        >
          <SectionHeading id="cli-commands" icon={Code}>
            {t('docs.cliCommands.title', 'CLI Commands')}
          </SectionHeading>

          <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mb-8">
            {t(
              'docs.cliCommands.intro',
              'AigisCode provides six commands that cover the entire analysis workflow. Each command can be run independently, giving you fine-grained control over the pipeline. The analyze command runs all stages in sequence, while individual commands let you re-run specific stages after configuration changes.'
            )}
          </p>

          <div className="space-y-5">
            {CLI_COMMANDS.map(({ command, key, icon: Icon, descriptionFallback }) => (
              <div
                key={key}
                className="rounded-xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-5 transition-colors hover:border-indigo-500/20 dark:hover:border-indigo-500/20"
              >
                <div className="flex items-start gap-3">
                  <div className="mt-0.5 inline-flex items-center justify-center w-8 h-8 rounded-lg bg-gradient-to-br from-indigo-500/10 to-purple-500/10 dark:from-indigo-500/20 dark:to-purple-500/20 text-indigo-500 flex-shrink-0">
                    <Icon size={18} weight="duotone" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-2">
                      <code className="text-sm font-mono font-medium text-zinc-900 dark:text-white bg-zinc-100 dark:bg-white/5 px-2 py-0.5 rounded-md">
                        {command}
                      </code>
                      <CopyButton text={command} />
                    </div>
                    <p className="text-sm text-zinc-600 dark:text-zinc-400 leading-relaxed">
                      {t(`docs.cliCommands.${key}.description`, descriptionFallback)}
                    </p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </motion.section>

        {/* ---------------------------------------------------------------- */}
        {/*  Configuration                                                   */}
        {/* ---------------------------------------------------------------- */}
        <motion.section
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.5 }}
          aria-labelledby="configuration"
        >
          <SectionHeading id="configuration" icon={GearSix}>
            {t('docs.configuration.title', 'Configuration')}
          </SectionHeading>

          <div className="space-y-5">
            <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">
              {t(
                'docs.configuration.intro',
                'AigisCode uses a policy-driven architecture. All project-specific behavior is expressed through a policy.json file placed in the .aigiscode/ directory. No configuration is required for the first run — AigisCode uses sensible defaults. As you review findings and identify patterns specific to your project, you can refine the policy to reduce false positives and focus on the issues that matter.'
              )}
            </p>

            <h3 className="text-lg font-display font-semibold text-zinc-900 dark:text-white mt-8 mb-3">
              {t('docs.configuration.policyTitle', 'Policy File: .aigiscode/policy.json')}
            </h3>
            <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mb-4">
              {t(
                'docs.configuration.policyDescription',
                'The policy file controls three main detector families: graph analysis, dead code detection, and hardwiring detection. Each section accepts glob patterns for path matching, threshold values for sensitivity tuning, and allowlists for known exceptions. Here is an example showing common configuration options:'
              )}
            </p>
            <CodeBlock code={POLICY_EXAMPLE} language="json" />

            <h3 className="text-lg font-display font-semibold text-zinc-900 dark:text-white mt-8 mb-3">
              {t('docs.configuration.graphTitle', 'Graph Configuration')}
            </h3>
            <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">
              {t(
                'docs.configuration.graphDescription',
                'The graph section controls how AigisCode resolves imports and builds the dependency graph. Use js_import_aliases to map path aliases like @/ to actual directories. The orphan_entry_patterns field tells AigisCode which files are intentional entry points (bootstrapping files, CLI scripts, test fixtures) so they are not flagged as orphan modules. The layer_map defines your architectural layers for layer violation detection.'
              )}
            </p>

            <h3 className="text-lg font-display font-semibold text-zinc-900 dark:text-white mt-8 mb-3">
              {t('docs.configuration.deadCodeTitle', 'Dead Code Configuration')}
            </h3>
            <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">
              {t(
                'docs.configuration.deadCodeDescription',
                'The dead_code section adjusts how the dead code detector identifies unused symbols. The abandoned_entry_patterns field suppresses false positives for files that are loaded dynamically (service containers, plugin systems, contract interfaces). You can also set the minimum confidence level that a finding must reach before it appears in the report.'
              )}
            </p>

            <h3 className="text-lg font-display font-semibold text-zinc-900 dark:text-white mt-8 mb-3">
              {t('docs.configuration.hardwiringTitle', 'Hardwiring Configuration')}
            </h3>
            <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">
              {t(
                'docs.configuration.hardwiringDescription',
                'The hardwiring section controls detection of magic strings, repeated literals, and hardcoded values. Increase repeated_literal_min_occurrences to reduce noise in projects that intentionally repeat certain strings. Use skip_path_patterns to exclude directories like console commands or seeders. The allowed_literals list lets you whitelist strings that are not truly hardwired, such as localhost addresses used only in development.'
              )}
            </p>

            <h3 className="text-lg font-display font-semibold text-zinc-900 dark:text-white mt-8 mb-3">
              {t('docs.configuration.rulesTitle', 'Exclusion Rules: .aigiscode/rules.json')}
            </h3>
            <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">
              {t(
                'docs.configuration.rulesDescription',
                'In addition to the policy file, you can create per-finding exclusion rules in rules.json. Each rule specifies a finding type, a file pattern, and optionally a symbol name. When a finding matches a rule, it is excluded from the report. This is useful for suppressing individual false positives that do not fit a broader policy pattern. Rules are applied during the Rules stage of the pipeline.'
              )}
            </p>
          </div>
        </motion.section>

        {/* ---------------------------------------------------------------- */}
        {/*  Supported Languages                                             */}
        {/* ---------------------------------------------------------------- */}
        <motion.section
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.5 }}
          aria-labelledby="supported-languages"
        >
          <SectionHeading id="supported-languages" icon={TreeStructure}>
            {t('docs.languages.title', 'Supported Languages')}
          </SectionHeading>

          <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mb-8">
            {t(
              'docs.languages.intro',
              'AigisCode supports six programming languages and frameworks. All languages are analyzed in a single pass with a unified dependency graph, meaning cross-language dependencies (such as a TypeScript frontend importing types generated from a Python backend) are tracked in the same graph. The parser column indicates whether the language uses tree-sitter or tree-sitter-python for source code parsing.'
            )}
          </p>

          {/* Table — responsive with horizontal scroll on mobile */}
          <div className="rounded-xl border border-zinc-200/50 dark:border-white/5 overflow-hidden">
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="bg-zinc-100/80 dark:bg-white/[0.03] border-b border-zinc-200/50 dark:border-white/5">
                    <th className="text-left px-5 py-3 font-display font-semibold text-zinc-900 dark:text-white">
                      {t('docs.languages.colLanguage', 'Language')}
                    </th>
                    <th className="text-left px-5 py-3 font-display font-semibold text-zinc-900 dark:text-white">
                      {t('docs.languages.colParser', 'Parser')}
                    </th>
                    <th className="text-left px-5 py-3 font-display font-semibold text-zinc-900 dark:text-white">
                      {t('docs.languages.colExtensions', 'Extensions')}
                    </th>
                    <th className="text-left px-5 py-3 font-display font-semibold text-zinc-900 dark:text-white hidden md:table-cell">
                      {t('docs.languages.colFeatures', 'Extracted Symbols')}
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-zinc-200/30 dark:divide-white/5">
                  {SUPPORTED_LANGUAGES.map((lang) => (
                    <tr
                      key={lang.language}
                      className="hover:bg-zinc-50/50 dark:hover:bg-white/[0.01] transition-colors"
                    >
                      <td className="px-5 py-3.5 font-medium text-zinc-900 dark:text-white font-mono text-sm">
                        {lang.language}
                      </td>
                      <td className="px-5 py-3.5 text-zinc-600 dark:text-zinc-400">
                        <span className="inline-block px-2 py-0.5 text-xs font-medium rounded-full bg-indigo-500/10 text-indigo-600 dark:text-indigo-400">
                          {lang.parser}
                        </span>
                      </td>
                      <td className="px-5 py-3.5 text-zinc-600 dark:text-zinc-400 font-mono text-xs">
                        {lang.extensions}
                      </td>
                      <td className="px-5 py-3.5 text-zinc-500 dark:text-zinc-500 hidden md:table-cell text-xs">
                        {lang.features}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          <p className="mt-4 text-sm text-zinc-500 dark:text-zinc-500">
            {t(
              'docs.languages.note',
              'Additional languages can be added by contributing a tree-sitter grammar binding and a symbol extraction module. See the contributing guide on GitHub for details.'
            )}
          </p>
        </motion.section>

        {/* ---------------------------------------------------------------- */}
        {/*  Report Structure                                                */}
        {/* ---------------------------------------------------------------- */}
        <motion.section
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.5 }}
          aria-labelledby="report-structure"
        >
          <SectionHeading id="report-structure" icon={BookOpen}>
            {t('docs.reportStructure.title', 'Report Structure')}
          </SectionHeading>

          <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mb-6">
            {t(
              'docs.reportStructure.intro',
              'The JSON report at .aigiscode/deterministic-analysis.json is the primary machine-readable output. AI coding agents, CI pipelines, and IDE integrations should parse this file. The report is organized into top-level sections that map directly to the analysis stages.'
            )}
          </p>

          <div className="space-y-4">
            {[
              {
                path: 'graph_analysis.strong_circular_dependencies',
                descKey: 'docs.reportStructure.strongCircular',
                descFallback:
                  'Architectural cycles between modules where both directions carry production imports. These are the highest-priority findings — they indicate modules that cannot be separated without refactoring.',
              },
              {
                path: 'graph_analysis.circular_dependencies',
                descKey: 'docs.reportStructure.allCircular',
                descFallback:
                  'All detected cycles, including runtime-only and load-time dependencies. Use this for a complete picture of circular coupling in your project.',
              },
              {
                path: 'graph_analysis.god_classes',
                descKey: 'docs.reportStructure.godClasses',
                descFallback:
                  'Classes with excessive responsibility, identified by high method count, high fan-in/fan-out, or both. These are candidates for decomposition into smaller, focused modules.',
              },
              {
                path: 'graph_analysis.bottleneck_files',
                descKey: 'docs.reportStructure.bottlenecks',
                descFallback:
                  'Files with unusually high coupling — many inbound and outbound dependencies. Changes to bottleneck files carry high blast radius and deserve careful review.',
              },
              {
                path: 'dead_code',
                descKey: 'docs.reportStructure.deadCode',
                descFallback:
                  'Unused imports, unreferenced methods, orphaned properties, and abandoned classes. Each finding includes the symbol name, file path, line number, and confidence level.',
              },
              {
                path: 'hardwiring',
                descKey: 'docs.reportStructure.hardwiring',
                descFallback:
                  'Magic strings, repeated literals, hardcoded IP addresses and URLs, and environment variable access outside config files. Findings include the literal value and all locations where it appears.',
              },
              {
                path: 'summary.detector_coverage',
                descKey: 'docs.reportStructure.coverage',
                descFallback:
                  'A breakdown of which detectors ran on which file types, so you can verify that your project received full coverage. If a language is not listed here, it was not analyzed.',
              },
            ].map(({ path, descKey, descFallback }) => (
              <div
                key={path}
                className="flex items-start gap-3 rounded-lg border border-zinc-200/30 dark:border-white/5 bg-white/30 dark:bg-white/[0.01] p-4"
              >
                <code className="text-xs font-mono text-indigo-500 bg-indigo-500/10 px-2 py-0.5 rounded-md flex-shrink-0 mt-0.5">
                  {path}
                </code>
                <p className="text-sm text-zinc-600 dark:text-zinc-400 leading-relaxed">
                  {t(descKey, descFallback)}
                </p>
              </div>
            ))}
          </div>
        </motion.section>

        {/* ---------------------------------------------------------------- */}
        {/*  AI Integration                                                  */}
        {/* ---------------------------------------------------------------- */}
        <motion.section
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.5 }}
          aria-labelledby="ai-integration"
        >
          <SectionHeading id="ai-integration" icon={Robot}>
            {t('docs.aiIntegration.title', 'AI Integration')}
          </SectionHeading>

          <div className="space-y-5">
            <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">
              {t(
                'docs.aiIntegration.intro',
                'AigisCode is designed to work alongside AI coding agents. The AI review stage uses configurable backends (OpenAI, Anthropic) to classify findings as true positives, false positives, or uncertain. This classification reduces noise and helps both human developers and AI agents focus on real issues.'
              )}
            </p>

            <h3 className="text-lg font-display font-semibold text-zinc-900 dark:text-white mt-8 mb-3">
              {t('docs.aiIntegration.backendsTitle', 'Configuring AI Backends')}
            </h3>
            <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mb-4">
              {t(
                'docs.aiIntegration.backendsDescription',
                'Set the appropriate environment variable for your preferred backend. AigisCode will automatically detect and use the available backend during the AI review stage.'
              )}
            </p>
            <CodeBlock
              code={`# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."

# Then run analysis with AI review
aigiscode analyze /path/to/project`}
              language="shell"
            />

            <h3 className="text-lg font-display font-semibold text-zinc-900 dark:text-white mt-8 mb-3">
              {t('docs.aiIntegration.agentWorkflowTitle', 'Recommended Workflow for AI Agents')}
            </h3>
            <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mb-4">
              {t(
                'docs.aiIntegration.agentWorkflowDescription',
                'AI coding agents should follow a structured workflow when using AigisCode. This workflow ensures findings are validated before fixes are applied, and that policy is updated to prevent repeat false positives.'
              )}
            </p>
            <ol className="space-y-3 text-sm text-zinc-600 dark:text-zinc-400">
              {[
                t('docs.aiIntegration.step1', 'Run aigiscode analyze to generate the baseline report.'),
                t('docs.aiIntegration.step2', 'Parse .aigiscode/deterministic-analysis.json for structured findings.'),
                t('docs.aiIntegration.step3', 'Triage findings by severity and confidence. Start with high confidence.'),
                t('docs.aiIntegration.step4', 'Sample findings and classify as true positive, false positive, or uncertain.'),
                t('docs.aiIntegration.step5', 'Apply fixes for confirmed true positives.'),
                t('docs.aiIntegration.step6', 'Add exclusion rules for false positives to .aigiscode/rules.json.'),
                t('docs.aiIntegration.step7', 'Encode repeated false positive patterns in .aigiscode/policy.json.'),
                t('docs.aiIntegration.step8', 'Run aigiscode analyze for fast re-evaluation after policy changes.'),
              ].map((step, i) => (
                <li key={i} className="flex items-start gap-3">
                  <span className="inline-flex items-center justify-center w-6 h-6 rounded-full bg-indigo-500/10 text-indigo-500 text-xs font-bold flex-shrink-0 mt-0.5">
                    {i + 1}
                  </span>
                  <span className="leading-relaxed">{step}</span>
                </li>
              ))}
            </ol>
          </div>
        </motion.section>

        {/* ---------------------------------------------------------------- */}
        {/*  CTA — GitHub + Further Resources                                */}
        {/* ---------------------------------------------------------------- */}
        <motion.section
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-80px' }}
          transition={{ duration: 0.5 }}
          className="rounded-2xl bg-zinc-900 dark:bg-zinc-950 border border-zinc-800 dark:border-zinc-800/60 p-8 md:p-12 text-center relative overflow-hidden"
        >
          {/* Glow */}
          <div className="absolute inset-0 pointer-events-none" aria-hidden="true">
            <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[400px] h-[200px] bg-gradient-to-r from-indigo-500/20 via-purple-500/15 to-pink-500/20 blur-[100px] rounded-full" />
          </div>

          <div className="relative z-10">
            <h2 className="text-2xl md:text-3xl font-display font-bold tracking-tight text-white">
              {t('docs.cta.title', 'Explore the Full Source')}
            </h2>
            <p className="mt-3 text-zinc-400 max-w-md mx-auto leading-relaxed">
              {t(
                'docs.cta.description',
                'AigisCode is open source under the MIT License. Browse the source code, read the contributing guide, report issues, and check the plugin documentation on GitHub.'
              )}
            </p>

            <div className="mt-8 flex flex-col sm:flex-row items-center justify-center gap-4">
              <a
                href="https://github.com/Draivix/aigiscode"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-2 px-7 py-3.5 rounded-full bg-white text-black text-sm font-semibold hover:scale-105 active:scale-[0.98] transition-transform"
              >
                <GithubLogo size={18} weight="bold" />
                {t('docs.cta.github', 'View on GitHub')}
              </a>
              <a
                href="https://github.com/Draivix/aigiscode/blob/main/docs/PLUGIN_SYSTEM.md"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-2 px-7 py-3.5 rounded-full border border-zinc-700 text-zinc-300 text-sm font-semibold hover:bg-white/5 transition-colors"
              >
                {t('docs.cta.plugins', 'Plugin Documentation')}
                <ArrowRight size={14} weight="bold" />
              </a>
            </div>
          </div>
        </motion.section>
      </div>
    </>
  );
}
