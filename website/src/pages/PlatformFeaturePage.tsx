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
  WarningCircle,
  GithubLogo,
  Plugs,
  Sliders,
  FileText,
  Graph,
  Terminal,
  MagnifyingGlass,
  PuzzlePiece,
} from '@phosphor-icons/react';
import SEO from '@/components/SEO';
import Breadcrumbs from '@/components/Breadcrumbs';
import {
  getPlatformFeatureBySlug,
  getRelatedPlatformFeatures,
} from '@/content/platform';
import type { PlatformFeature } from '@/content/platform';
import {
  SITE_URL,
  buildGraphJsonLd,
  techArticleEntity,
  softwareFeatureEntity,
  webPageEntity,
} from '@/content/seo-schema';

/* -------------------------------------------------------------------------- */
/*  Icon resolver                                                             */
/* -------------------------------------------------------------------------- */

const iconMap: Record<string, React.ElementType> = {
  GithubLogo,
  Plugs,
  Sliders,
  FileText,
  Graph,
  Terminal,
  MagnifyingGlass,
  Puzzle: PuzzlePiece,
};

function resolveIcon(name: string): React.ElementType {
  return iconMap[name] || PuzzlePiece;
}

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
/*  Related Feature Card (compact)                                            */
/* -------------------------------------------------------------------------- */

function RelatedFeatureCard({ feature }: { feature: PlatformFeature }) {
  const { i18n } = useTranslation();
  const lang = i18n.language?.slice(0, 2) || 'en';

  const title = feature.title[lang] || feature.title.en;
  const description =
    feature.shortDescription[lang] || feature.shortDescription.en;
  const Icon = resolveIcon(feature.icon);

  return (
    <Link
      to={`/platform/${feature.slug}`}
      className="group relative flex flex-col rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-6 overflow-hidden transition-all duration-300 hover:shadow-lg hover:shadow-indigo-500/5 hover:border-indigo-500/20 dark:hover:border-indigo-500/20"
    >
      <div className="absolute -inset-1 rounded-2xl bg-gradient-to-br from-indigo-500/20 via-purple-500/20 to-pink-500/20 opacity-0 group-hover:opacity-100 blur-xl transition-opacity duration-500 pointer-events-none" />

      <div className="relative flex flex-col flex-1">
        <div className="mb-4 inline-flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500/10 to-purple-500/10 dark:from-indigo-500/20 dark:to-purple-500/20 text-indigo-500">
          <Icon size={24} weight="duotone" />
        </div>
        <h3 className="font-display font-semibold text-lg mb-2 text-zinc-900 dark:text-white group-hover:text-indigo-600 dark:group-hover:text-indigo-400 transition-colors">
          {title}
        </h3>
        <p className="text-sm text-zinc-600 dark:text-zinc-400 leading-relaxed line-clamp-3">
          {description}
        </p>
        <div className="mt-4 pt-3 border-t border-zinc-200/50 dark:border-white/5 flex items-center gap-1.5 text-sm font-medium text-indigo-600 dark:text-indigo-400 opacity-0 group-hover:opacity-100 transition-opacity">
          Learn more
          <ArrowRight
            size={14}
            weight="bold"
            className="transition-transform group-hover:translate-x-0.5"
          />
        </div>
      </div>
    </Link>
  );
}

/* -------------------------------------------------------------------------- */
/*  Not Found State                                                           */
/* -------------------------------------------------------------------------- */

function NotFound() {
  return (
    <section className="py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
        <WarningCircle
          size={48}
          className="mx-auto text-zinc-400 dark:text-zinc-600 mb-4"
        />
        <h1 className="text-2xl font-display font-bold text-zinc-900 dark:text-white mb-2">
          Feature Not Found
        </h1>
        <p className="text-zinc-500 dark:text-zinc-400 mb-8">
          The platform feature you are looking for does not exist or has been
          moved.
        </p>
        <Link
          to="/platform"
          className="inline-flex items-center gap-2 px-6 py-3 rounded-full bg-zinc-900 dark:bg-white text-white dark:text-black text-sm font-semibold hover:scale-105 transition-transform"
        >
          <ArrowLeft size={16} weight="bold" />
          Back to Platform
        </Link>
      </div>
    </section>
  );
}

/* -------------------------------------------------------------------------- */
/*  PlatformFeaturePage — Individual platform feature                         */
/* -------------------------------------------------------------------------- */

export default function PlatformFeaturePage() {
  const { slug } = useParams<{ slug: string }>();
  const { i18n } = useTranslation();
  const lang = i18n.language?.slice(0, 2) || 'en';

  const feature = slug ? getPlatformFeatureBySlug(slug) : undefined;

  const title = feature ? feature.title[lang] || feature.title.en : '';
  const content = feature ? feature.content[lang] || feature.content.en : '';
  const description = feature
    ? feature.metaDescription[lang] || feature.metaDescription.en
    : '';

  const toc = useMemo(() => extractToc(content), [content]);
  const related = useMemo(
    () => (slug ? getRelatedPlatformFeatures(slug) : []),
    [slug],
  );

  /* Refs for anime.js animations */
  const heroRef = useRef<HTMLDivElement>(null);
  const highlightsRef = useRef<HTMLDivElement>(null);

  /* ---- Hero entrance animation ---- */
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

    return () => {
      anim.pause();
    };
  }, [slug]);

  /* ---- Highlights stagger animation ---- */
  useEffect(() => {
    if (!highlightsRef.current) return;

    const items = highlightsRef.current.querySelectorAll(
      '[data-highlight-item]',
    );
    if (items.length === 0) return;

    if (prefersReducedMotion()) {
      items.forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          animate(items, {
            opacity: [0, 1],
            translateX: [-20, 0],
            duration: 400,
            delay: stagger(60),
            ease: 'outQuart',
          });
          observer.disconnect();
        }
      },
      { threshold: 0.2, rootMargin: '-40px' },
    );

    observer.observe(highlightsRef.current);

    return () => {
      observer.disconnect();
    };
  }, [slug]);

  if (!feature) {
    return <NotFound />;
  }

  const Icon = resolveIcon(feature.icon);
  const canonical = `${SITE_URL}/platform/${feature.slug}`;

  const graphJsonLd = buildGraphJsonLd([
    webPageEntity(canonical, `${feature.title.en} - AigisCode Platform`, feature.metaDescription.en),
    techArticleEntity({
      url: canonical,
      headline: feature.title.en,
      description: feature.metaDescription.en,
      datePublished: '2026-03-09',
      dateModified: '2026-03-09',
      keywords: feature.highlights,
    }),
    softwareFeatureEntity({
      url: canonical,
      name: feature.title.en,
      description: feature.metaDescription.en,
      featureList: feature.highlights,
    }),
  ]);

  const breadcrumbItems = [
    { label: 'Platform', href: '/platform' },
    { label: title },
  ];

  return (
    <>
      <SEO
        title={`${title} — AigisCode Platform`}
        description={description}
        canonical={canonical}
        article={{
          publishedTime: '2026-03-09',
          modifiedTime: '2026-03-09',
          author: 'AigisCode',
          tags: feature.highlights,
        }}
        breadcrumbs={[
          { name: 'Home', url: SITE_URL },
          { name: 'Platform', url: `${SITE_URL}/platform` },
          { name: title, url: canonical },
        ]}
        jsonLd={graphJsonLd}
      />

      {/* Hero */}
      <section className="pt-12 pb-8 md:pt-20 md:pb-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <Breadcrumbs items={breadcrumbItems} />

          <div ref={heroRef} className="max-w-3xl opacity-0">
            {/* Icon */}
            <div className="mb-6 inline-flex items-center justify-center w-14 h-14 rounded-2xl bg-gradient-to-br from-indigo-500/10 to-purple-500/10 dark:from-indigo-500/20 dark:to-purple-500/20 text-indigo-500">
              <Icon size={32} weight="duotone" />
            </div>

            <h1 className="text-3xl md:text-4xl lg:text-5xl font-display font-bold tracking-tight">
              <span className="bg-clip-text text-transparent bg-gradient-to-r from-zinc-900 via-zinc-700 to-zinc-900 dark:from-white dark:via-zinc-300 dark:to-white">
                {title}
              </span>
            </h1>

            <p className="mt-4 text-lg text-zinc-600 dark:text-zinc-400 leading-relaxed">
              {feature.shortDescription[lang] || feature.shortDescription.en}
            </p>
          </div>
        </div>
      </section>

      {/* Highlights */}
      <section className="pb-8 md:pb-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div
            ref={highlightsRef}
            className="p-6 lg:p-8 rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02]"
          >
            <h2 className="font-display font-semibold text-lg text-zinc-900 dark:text-white mb-5">
              Key Highlights
            </h2>
            <ul className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              {feature.highlights.map((highlight) => (
                <li
                  key={highlight}
                  data-highlight-item
                  className="flex items-start gap-2.5 opacity-0"
                >
                  <CheckCircle
                    size={18}
                    weight="fill"
                    className="text-emerald-500 flex-shrink-0 mt-0.5"
                  />
                  <span className="text-sm text-zinc-700 dark:text-zinc-300">
                    {highlight}
                  </span>
                </li>
              ))}
            </ul>
          </div>
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

              {/* Code example */}
              {feature.codeExample && (
                <div className="mt-10">
                  <h3 className="font-display font-semibold text-lg text-zinc-900 dark:text-white mb-4">
                    Example
                  </h3>
                  <pre className="p-5 rounded-2xl bg-zinc-900 dark:bg-zinc-950 border border-zinc-800 dark:border-zinc-800/60 text-zinc-300 font-mono text-sm leading-relaxed overflow-x-auto">
                    <code>{feature.codeExample}</code>
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

      {/* Related Features */}
      {related.length > 0 && (
        <section className="py-16 md:py-24 border-t border-zinc-200/50 dark:border-white/5">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <h2 className="text-2xl md:text-3xl font-display font-bold tracking-tight text-zinc-900 dark:text-white mb-8">
              Related Platform Features
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {related.map((f) => (
                <RelatedFeatureCard key={f.slug} feature={f} />
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
              to="/platform"
              className="inline-flex items-center gap-2 px-7 py-3.5 rounded-full border border-white/10 text-zinc-300 text-sm font-semibold hover:bg-white/5 transition-colors"
            >
              <ArrowLeft size={16} weight="bold" />
              All Platform Features
            </Link>
          </div>
        </div>
      </section>
    </>
  );
}
