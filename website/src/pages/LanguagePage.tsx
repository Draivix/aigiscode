import { useMemo, useEffect, useRef } from 'react';
import { useParams } from 'react-router-dom';
import { LocalizedLink as Link } from '@/lib/i18n-routing';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { animate, stagger } from 'animejs';
import { prefersReducedMotion } from '@/hooks/useAnime';
import {
  ArrowRight,
  ArrowLeft,
  CheckCircle,
  XCircle,
  WarningCircle,
  Code,
  Plugs,
  TreeStructure,
} from '@phosphor-icons/react';
import SEO from '@/components/SEO';
import Breadcrumbs from '@/components/Breadcrumbs';
import { getLanguageBySlug, getRelatedLanguages } from '@/content/languages';
import type { Language } from '@/content/languages';
import {
  SITE_URL,
  buildGraphJsonLd,
  techArticleEntity,
  webPageEntity,
} from '@/content/seo-schema';

/* -------------------------------------------------------------------------- */
/*  Table of Contents parser                                                  */
/* -------------------------------------------------------------------------- */

interface TocEntry {
  id: string;
  text: string;
  level: number;
}

function extractToc(html: string): TocEntry[] {
  const entries: TocEntry[] = [];
  const regex = /<h([2-3])\s+id="([^"]+)"[^>]*>(.*?)<\/h[2-3]>/g;
  let match: RegExpExecArray | null;
  while ((match = regex.exec(html)) !== null) {
    entries.push({
      level: parseInt(match[1], 10),
      id: match[2],
      text: match[3].replace(/<[^>]+>/g, ''),
    });
  }
  return entries;
}

/* -------------------------------------------------------------------------- */
/*  Not Found State                                                           */
/* -------------------------------------------------------------------------- */

function NotFound() {
  return (
    <section className="py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
        <WarningCircle size={48} className="mx-auto text-zinc-400 dark:text-zinc-600 mb-4" />
        <h1 className="text-2xl font-display font-bold text-zinc-900 dark:text-white mb-2">
          Language Not Found
        </h1>
        <p className="text-zinc-500 dark:text-zinc-400 mb-8">
          The language you are looking for does not exist or has been moved.
        </p>
        <Link
          to="/languages"
          className="inline-flex items-center gap-2 px-6 py-3 rounded-full bg-zinc-900 dark:bg-white text-white dark:text-black text-sm font-semibold hover:scale-105 transition-transform"
        >
          <ArrowLeft size={16} weight="bold" />
          Back to Languages
        </Link>
      </div>
    </section>
  );
}

/* -------------------------------------------------------------------------- */
/*  DetectorRow — used in the feature support table                           */
/* -------------------------------------------------------------------------- */

function DetectorRow({ label, supported, description }: { label: string; supported: boolean; description: string }) {
  return (
    <tr className="border-b border-zinc-200/50 dark:border-white/5 last:border-0">
      <td className="py-3 pr-4">
        <span className="font-medium text-sm text-zinc-900 dark:text-white">{label}</span>
      </td>
      <td className="py-3 px-4 text-center">
        {supported ? (
          <CheckCircle size={20} weight="fill" className="text-emerald-500 inline-block" />
        ) : (
          <XCircle size={20} weight="fill" className="text-zinc-300 dark:text-zinc-600 inline-block" />
        )}
      </td>
      <td className="py-3 pl-4">
        <span className="text-sm text-zinc-500 dark:text-zinc-400">{description}</span>
      </td>
    </tr>
  );
}

/* -------------------------------------------------------------------------- */
/*  RelatedLanguageCard — compact card for related languages                  */
/* -------------------------------------------------------------------------- */

function RelatedLanguageCard({ language }: { language: Language }) {
  const { i18n } = useTranslation();
  const lang = i18n.language?.slice(0, 2) || 'en';
  const title = language.title[lang] || language.title.en;

  return (
    <Link
      to={`/languages/${language.slug}`}
      className="group relative flex flex-col rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-6 overflow-hidden transition-all duration-300 hover:shadow-lg hover:shadow-indigo-500/5 hover:border-indigo-500/20 dark:hover:border-indigo-500/20"
    >
      <div className="absolute -inset-1 rounded-2xl bg-gradient-to-br from-indigo-500/20 via-purple-500/20 to-pink-500/20 opacity-0 group-hover:opacity-100 blur-xl transition-opacity duration-500 pointer-events-none" />

      <div className="relative">
        <div className="flex items-center gap-3 mb-3">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-indigo-500/10 to-purple-500/10 border border-indigo-500/10 flex items-center justify-center">
            <Code size={18} weight="duotone" className="text-indigo-500 dark:text-indigo-400" />
          </div>
          <span className="text-xs font-mono text-zinc-400 dark:text-zinc-500">{language.extensions}</span>
        </div>
        <h3 className="font-display font-semibold text-base text-zinc-900 dark:text-white group-hover:text-indigo-600 dark:group-hover:text-indigo-400 transition-colors mb-2">
          {title}
        </h3>
        <p className="text-sm text-zinc-500 dark:text-zinc-400 line-clamp-2">
          {language.shortDescription[lang] || language.shortDescription.en}
        </p>
        <div className="mt-3 flex items-center gap-1.5 text-sm font-medium text-indigo-600 dark:text-indigo-400 opacity-0 group-hover:opacity-100 transition-opacity">
          Learn more
          <ArrowRight size={14} weight="bold" />
        </div>
      </div>
    </Link>
  );
}

/* -------------------------------------------------------------------------- */
/*  LanguagePage — Individual language detail                                 */
/* -------------------------------------------------------------------------- */

export default function LanguagePage() {
  const { slug } = useParams<{ slug: string }>();
  const { i18n } = useTranslation();
  const lang = i18n.language?.slice(0, 2) || 'en';

  const language = slug ? getLanguageBySlug(slug) : undefined;

  const title = language ? language.title[lang] || language.title.en : '';
  const content = language ? language.content[lang] || language.content.en : '';
  const description = language
    ? language.metaDescription[lang] || language.metaDescription.en
    : '';

  const toc = useMemo(() => extractToc(content), [content]);
  const related = useMemo(
    () => (language ? getRelatedLanguages(language.relatedSlugs) : []),
    [language],
  );

  /* ---- anime.js hero entrance ---- */
  const heroRef = useRef<HTMLDivElement>(null);
  const featuresRef = useRef<HTMLUListElement>(null);

  useEffect(() => {
    if (!heroRef.current) return;
    if (prefersReducedMotion()) {
      heroRef.current.style.opacity = '1';
      heroRef.current.style.transform = 'none';
      return;
    }

    const anim = animate(heroRef.current, {
      opacity: [0, 1],
      translateY: [30, 0],
      duration: 600,
      ease: 'outQuart',
    });

    return () => { anim.pause(); };
  }, [slug]);

  /* ---- anime.js feature list stagger ---- */
  useEffect(() => {
    if (!featuresRef.current) return;
    const items = featuresRef.current.querySelectorAll('[data-feature-item]');
    if (items.length === 0) return;

    if (prefersReducedMotion()) {
      items.forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      return;
    }

    const anim = animate(items, {
      opacity: [0, 1],
      translateX: [-20, 0],
      duration: 400,
      delay: stagger(60),
      ease: 'outQuart',
    });

    return () => { anim.pause(); };
  }, [slug]);

  if (!language) {
    return <NotFound />;
  }

  const canonical = `${SITE_URL}/languages/${language.slug}`;

  const graphJsonLd = buildGraphJsonLd([
    webPageEntity(canonical, `${language.title.en} Analysis - AigisCode`, language.metaDescription.en),
    techArticleEntity({
      url: canonical,
      headline: language.title.en,
      description: language.metaDescription.en,
      datePublished: '2026-03-09',
      dateModified: '2026-03-09',
      keywords: language.features,
    }),
    {
      '@type': 'ComputerLanguage',
      '@id': `${canonical}#language`,
      name: language.title.en,
      url: canonical,
      alternateName: language.extensions,
    },
  ]);

  const breadcrumbItems = [
    { label: 'Languages', href: '/languages' },
    { label: title },
  ];

  return (
    <>
      <SEO
        title={`${title} — AigisCode`}
        description={description}
        canonical={canonical}
        article={{
          publishedTime: '2026-03-09',
          modifiedTime: '2026-03-09',
          author: 'AigisCode',
          tags: language.features,
        }}
        breadcrumbs={[
          { name: 'Home', url: SITE_URL },
          { name: 'Languages', url: `${SITE_URL}/languages` },
          { name: title, url: canonical },
        ]}
        jsonLd={graphJsonLd}
      />

      {/* Hero */}
      <section className="pt-12 pb-8 md:pt-20 md:pb-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <Breadcrumbs items={breadcrumbItems} />

          <div
            ref={heroRef}
            className="opacity-0"
          >
            <div className="flex items-start gap-5 mb-6">
              {/* Language icon — prominent */}
              <div className="w-16 h-16 md:w-20 md:h-20 rounded-2xl bg-gradient-to-br from-indigo-500/10 via-purple-500/10 to-pink-500/10 border border-indigo-500/10 flex items-center justify-center flex-shrink-0">
                <Code size={36} weight="duotone" className="text-indigo-500 dark:text-indigo-400 md:hidden" />
                <Code size={44} weight="duotone" className="text-indigo-500 dark:text-indigo-400 hidden md:block" />
              </div>

              <div className="flex-1 min-w-0">
                {/* Parser + Extensions badges */}
                <div className="flex flex-wrap items-center gap-2 mb-3">
                  <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/10">
                    {language.parser}
                  </span>
                  <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-mono bg-zinc-100 dark:bg-white/5 text-zinc-500 dark:text-zinc-400">
                    {language.extensions}
                  </span>
                </div>

                <h1 className="text-3xl md:text-4xl lg:text-5xl font-display font-bold tracking-tight text-zinc-900 dark:text-white">
                  {title}
                </h1>
              </div>
            </div>

            <p className="text-lg text-zinc-600 dark:text-zinc-400 leading-relaxed max-w-3xl">
              {language.shortDescription[lang] || language.shortDescription.en}
            </p>
          </div>
        </div>
      </section>

      {/* Parser Info + Feature Support Table */}
      <section className="pb-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {/* Extracted Features */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.1 }}
              className="p-6 rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02]"
            >
              <div className="flex items-center gap-2 mb-4">
                <TreeStructure size={20} weight="duotone" className="text-indigo-500 dark:text-indigo-400" />
                <h2 className="font-display font-semibold text-lg text-zinc-900 dark:text-white">
                  Extracted Symbols
                </h2>
              </div>
              <ul ref={featuresRef} className="space-y-2">
                {language.features.map((feature) => (
                  <li key={feature} data-feature-item className="flex items-start gap-2 opacity-0">
                    <CheckCircle
                      size={16}
                      weight="fill"
                      className="text-emerald-500 flex-shrink-0 mt-0.5"
                    />
                    <span className="text-sm text-zinc-700 dark:text-zinc-300">
                      {feature}
                    </span>
                  </li>
                ))}
              </ul>
            </motion.div>

            {/* Detector Support Table */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.2 }}
              className="p-6 rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02]"
            >
              <div className="flex items-center gap-2 mb-4">
                <Code size={20} weight="duotone" className="text-indigo-500 dark:text-indigo-400" />
                <h2 className="font-display font-semibold text-lg text-zinc-900 dark:text-white">
                  Detector Support
                </h2>
              </div>
              <table className="w-full">
                <thead>
                  <tr className="border-b border-zinc-200/50 dark:border-white/5">
                    <th className="text-left text-xs uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-medium pb-2 pr-4">
                      Detector
                    </th>
                    <th className="text-center text-xs uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-medium pb-2 px-4">
                      Status
                    </th>
                    <th className="text-left text-xs uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-medium pb-2 pl-4">
                      Description
                    </th>
                  </tr>
                </thead>
                <tbody>
                  <DetectorRow
                    label="Dead Code"
                    supported={language.detectorSupport.deadCode}
                    description="Unused imports, unreferenced methods, orphaned classes"
                  />
                  <DetectorRow
                    label="Hardwiring"
                    supported={language.detectorSupport.hardwiring}
                    description="Magic strings, repeated literals, hardcoded URLs and IPs"
                  />
                  <DetectorRow
                    label="Graph Analysis"
                    supported={language.detectorSupport.graphAnalysis}
                    description="Circular dependencies, god classes, bottleneck files"
                  />
                </tbody>
              </table>
            </motion.div>
          </div>
        </div>
      </section>

      {/* Framework Plugins */}
      {language.frameworkPlugins.length > 0 && (
        <section className="pb-12">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-60px' }}
              transition={{ duration: 0.5 }}
              className="p-6 rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02]"
            >
              <div className="flex items-center gap-2 mb-4">
                <Plugs size={20} weight="duotone" className="text-purple-500 dark:text-purple-400" />
                <h2 className="font-display font-semibold text-lg text-zinc-900 dark:text-white">
                  Framework Plugins
                </h2>
              </div>
              <p className="text-sm text-zinc-500 dark:text-zinc-400 mb-4">
                AigisCode ships with built-in runtime plugins that understand framework conventions,
                preventing false positives from framework-specific patterns.
              </p>
              <div className="flex flex-wrap gap-3">
                {language.frameworkPlugins.map((fw) => (
                  <div
                    key={fw}
                    className="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl border border-purple-500/10 bg-purple-500/5"
                  >
                    <Plugs size={16} weight="duotone" className="text-purple-500 dark:text-purple-400" />
                    <span className="text-sm font-medium text-purple-700 dark:text-purple-300">
                      {fw}
                    </span>
                  </div>
                ))}
              </div>
            </motion.div>
          </div>
        </section>
      )}

      {/* Content + Sidebar */}
      <section className="pb-20 md:pb-32">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex gap-12">
            {/* Main Content */}
            <motion.article
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5, delay: 0.15 }}
              className="flex-1 min-w-0"
            >
              {/* Article body */}
              <div
                className="prose prose-zinc dark:prose-invert prose-headings:font-display prose-headings:tracking-tight prose-h2:text-2xl prose-h2:mt-10 prose-h2:mb-4 prose-h3:text-xl prose-h3:mt-8 prose-h3:mb-3 prose-p:leading-relaxed prose-li:leading-relaxed prose-pre:bg-zinc-900 prose-pre:dark:bg-zinc-950 prose-pre:border prose-pre:border-zinc-800 prose-pre:dark:border-zinc-800/60 prose-pre:rounded-xl prose-code:text-indigo-600 prose-code:dark:text-indigo-400 prose-code:before:content-none prose-code:after:content-none prose-a:text-indigo-600 prose-a:dark:text-indigo-400 max-w-none"
                dangerouslySetInnerHTML={{ __html: content }}
              />
            </motion.article>

            {/* Desktop Sidebar — Table of Contents */}
            {toc.length > 0 && (
              <aside className="hidden xl:block w-64 flex-shrink-0">
                <div className="sticky top-24">
                  <h4 className="text-xs uppercase tracking-widest text-zinc-400 dark:text-zinc-500 font-medium mb-4">
                    On this page
                  </h4>
                  <nav aria-label="Table of contents">
                    <ul className="space-y-2 border-l border-zinc-200 dark:border-white/5">
                      {toc.map((entry) => (
                        <li key={entry.id}>
                          <a
                            href={`#${entry.id}`}
                            className={`block text-sm text-zinc-500 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-white transition-colors ${
                              entry.level === 3 ? 'pl-6' : 'pl-4'
                            } -ml-px border-l border-transparent hover:border-indigo-500`}
                          >
                            {entry.text}
                          </a>
                        </li>
                      ))}
                    </ul>
                  </nav>

                  {/* Sidebar CTA */}
                  <div className="mt-8 p-4 rounded-xl bg-gradient-to-br from-indigo-500/5 to-purple-500/5 border border-indigo-500/10">
                    <p className="text-sm font-medium text-zinc-900 dark:text-white mb-2">
                      Try AigisCode
                    </p>
                    <p className="text-xs text-zinc-500 dark:text-zinc-400 mb-3">
                      One command to analyze your codebase.
                    </p>
                    <code className="block text-xs bg-zinc-900 dark:bg-zinc-950 text-zinc-300 px-3 py-2 rounded-lg font-mono">
                      cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode
                    </code>
                  </div>
                </div>
              </aside>
            )}
          </div>
        </div>
      </section>

      {/* Related Languages */}
      {related.length > 0 && (
        <section className="py-16 md:py-24 border-t border-zinc-200/50 dark:border-white/5">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <h2 className="text-2xl md:text-3xl font-display font-bold tracking-tight text-zinc-900 dark:text-white mb-8">
              Other Supported Languages
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {related.map((rl) => (
                <RelatedLanguageCard key={rl.slug} language={rl} />
              ))}
            </div>
          </div>
        </section>
      )}

      {/* Bottom CTA */}
      <section className="py-16 md:py-24 bg-zinc-900 dark:bg-zinc-950 text-white">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <h2 className="text-2xl md:text-3xl font-display font-bold tracking-tight mb-4">
            Ready to Get Started?
          </h2>
          <p className="text-zinc-400 mb-8 max-w-lg mx-auto">
            Install AigisCode and run your first analysis in under a minute.
          </p>
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <a
              href="https://github.com/Draivix/aigiscode"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 px-7 py-3.5 rounded-full bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500 text-white font-semibold text-sm hover:scale-105 active:scale-[0.98] transition-transform shadow-xl"
            >
              View on GitHub
              <ArrowRight size={16} weight="bold" />
            </a>
            <Link
              to="/languages"
              className="inline-flex items-center gap-2 px-7 py-3.5 rounded-full border border-white/10 text-zinc-300 text-sm font-semibold hover:bg-white/5 transition-colors"
            >
              <ArrowLeft size={16} weight="bold" />
              All Languages
            </Link>
          </div>
        </div>
      </section>
    </>
  );
}
