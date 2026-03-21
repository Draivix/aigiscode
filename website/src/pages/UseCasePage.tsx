import { useMemo } from 'react';
import { useParams } from 'react-router-dom';
import { LocalizedLink as Link } from '@/lib/i18n-routing';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import {
  ArrowRight,
  CheckCircle,
  ArrowLeft,
  WarningCircle,
} from '@phosphor-icons/react';
import SEO from '@/components/SEO';
import Breadcrumbs from '@/components/Breadcrumbs';
import UseCaseCard from '@/components/UseCaseCard';
import { getUseCaseBySlug, getRelatedUseCases } from '@/content/use-cases';
import {
  SITE_URL,
  buildGraphJsonLd,
  articleEntity,
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
          Use Case Not Found
        </h1>
        <p className="text-zinc-500 dark:text-zinc-400 mb-8">
          The use case you are looking for does not exist or has been moved.
        </p>
        <Link
          to="/use-cases"
          className="inline-flex items-center gap-2 px-6 py-3 rounded-full bg-zinc-900 dark:bg-white text-white dark:text-black text-sm font-semibold hover:scale-105 transition-transform"
        >
          <ArrowLeft size={16} weight="bold" />
          Back to Use Cases
        </Link>
      </div>
    </section>
  );
}

/* -------------------------------------------------------------------------- */
/*  UseCasePage — Individual use case                                        */
/* -------------------------------------------------------------------------- */

export default function UseCasePage() {
  const { slug } = useParams<{ slug: string }>();
  const { i18n } = useTranslation();
  const lang = i18n.language?.slice(0, 2) || 'en';

  const useCase = slug ? getUseCaseBySlug(slug) : undefined;

  const title = useCase ? useCase.title[lang] || useCase.title.en : '';
  const content = useCase ? useCase.content[lang] || useCase.content.en : '';
  const description = useCase
    ? useCase.metaDescription[lang] || useCase.metaDescription.en
    : '';

  const toc = useMemo(() => extractToc(content), [content]);
  const related = useMemo(
    () => (useCase ? getRelatedUseCases(useCase.relatedSlugs) : []),
    [useCase],
  );

  if (!useCase) {
    return <NotFound />;
  }

  const canonical = `${SITE_URL}/use-cases/${useCase.slug}`;

  const graphJsonLd = buildGraphJsonLd([
    webPageEntity(canonical, `${useCase.title.en} - AigisCode`, useCase.metaDescription.en),
    articleEntity({
      url: canonical,
      headline: useCase.title.en,
      description: useCase.metaDescription.en,
      datePublished: '2026-03-09',
      dateModified: '2026-03-09',
      keywords: useCase.features,
    }),
  ]);

  const breadcrumbItems = [
    { label: 'Use Cases', href: '/use-cases' },
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
          tags: useCase.features,
        }}
        breadcrumbs={[
          { name: 'Home', url: SITE_URL },
          { name: 'Use Cases', url: `${SITE_URL}/use-cases` },
          { name: title, url: canonical },
        ]}
        jsonLd={graphJsonLd}
      />

      {/* Hero */}
      <section className="pt-12 pb-8 md:pt-20 md:pb-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <Breadcrumbs items={breadcrumbItems} />

          <motion.div
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6 }}
          >
            {/* Category badge */}
            <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 mb-4">
              {useCase.category}
            </span>

            <h1 className="text-3xl md:text-4xl lg:text-5xl font-display font-bold tracking-tight text-zinc-900 dark:text-white">
              {title}
            </h1>

            <p className="mt-4 text-lg text-zinc-600 dark:text-zinc-400 leading-relaxed max-w-3xl">
              {useCase.shortDescription[lang] || useCase.shortDescription.en}
            </p>
          </motion.div>
        </div>
      </section>

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

              {/* Features list */}
              <div className="mt-12 p-6 rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02]">
                <h3 className="font-display font-semibold text-lg text-zinc-900 dark:text-white mb-4">
                  Key Features
                </h3>
                <ul className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  {useCase.features.map((feature) => (
                    <li key={feature} className="flex items-start gap-2">
                      <CheckCircle
                        size={18}
                        weight="fill"
                        className="text-emerald-500 flex-shrink-0 mt-0.5"
                      />
                      <span className="text-sm text-zinc-700 dark:text-zinc-300">
                        {feature}
                      </span>
                    </li>
                  ))}
                </ul>
              </div>

              {/* Code example */}
              {useCase.codeExample && (
                <div className="mt-8">
                  <h3 className="font-display font-semibold text-lg text-zinc-900 dark:text-white mb-4">
                    Quick Start
                  </h3>
                  <pre className="p-5 rounded-2xl bg-zinc-900 dark:bg-zinc-950 border border-zinc-800 dark:border-zinc-800/60 text-zinc-300 font-mono text-sm leading-relaxed overflow-x-auto">
                    <code>{useCase.codeExample}</code>
                  </pre>
                </div>
              )}
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

      {/* Related Use Cases */}
      {related.length > 0 && (
        <section className="py-16 md:py-24 border-t border-zinc-200/50 dark:border-white/5">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <h2 className="text-2xl md:text-3xl font-display font-bold tracking-tight text-zinc-900 dark:text-white mb-8">
              Related Use Cases
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {related.map((uc) => (
                <UseCaseCard key={uc.slug} useCase={uc} />
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
              to="/use-cases"
              className="inline-flex items-center gap-2 px-7 py-3.5 rounded-full border border-white/10 text-zinc-300 text-sm font-semibold hover:bg-white/5 transition-colors"
            >
              <ArrowLeft size={16} weight="bold" />
              All Use Cases
            </Link>
          </div>
        </div>
      </section>
    </>
  );
}
