import { useEffect, useRef, useCallback } from 'react';
import { LocalizedLink as Link } from '@/lib/i18n-routing';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { animate, stagger } from 'animejs';
import { prefersReducedMotion } from '@/hooks/useAnime';
import {
  ArrowRight,
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
import AnimatedGradient from '@/components/AnimatedGradient';
import { platformFeatures } from '@/content/platform';
import type { PlatformFeature } from '@/content/platform';
import {
  SITE_URL,
  buildGraphJsonLd,
  collectionPageEntity,
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
/*  Platform Feature Card                                                     */
/* -------------------------------------------------------------------------- */

function PlatformFeatureCard({ feature }: { feature: PlatformFeature }) {
  const { i18n } = useTranslation();
  const lang = i18n.language?.slice(0, 2) || 'en';

  const title = feature.title[lang] || feature.title.en;
  const description =
    feature.shortDescription[lang] || feature.shortDescription.en;
  const Icon = resolveIcon(feature.icon);

  return (
    <Link
      to={`/platform/${feature.slug}`}
      className="group relative flex flex-col rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-6 lg:p-8 overflow-hidden transition-all duration-300 hover:shadow-lg hover:shadow-indigo-500/5 hover:border-indigo-500/20 dark:hover:border-indigo-500/20"
    >
      {/* Hover glow */}
      <div className="absolute -inset-1 rounded-2xl bg-gradient-to-br from-indigo-500/20 via-purple-500/20 to-pink-500/20 opacity-0 group-hover:opacity-100 blur-xl transition-opacity duration-500 pointer-events-none" />

      <div className="relative flex flex-col flex-1">
        {/* Icon */}
        <div className="mb-5 inline-flex items-center justify-center w-12 h-12 rounded-xl bg-gradient-to-br from-indigo-500/10 to-purple-500/10 dark:from-indigo-500/20 dark:to-purple-500/20 text-indigo-500">
          <Icon size={28} weight="duotone" />
        </div>

        {/* Title */}
        <h3 className="font-display font-semibold text-xl mb-3 text-zinc-900 dark:text-white group-hover:text-indigo-600 dark:group-hover:text-indigo-400 transition-colors">
          {title}
        </h3>

        {/* Description */}
        <p className="text-sm text-zinc-600 dark:text-zinc-400 leading-relaxed flex-1">
          {description}
        </p>

        {/* Highlights preview */}
        <div className="mt-5 flex flex-wrap gap-1.5">
          {feature.highlights.slice(0, 3).map((highlight) => (
            <span
              key={highlight}
              className="inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium bg-zinc-100 dark:bg-white/5 text-zinc-500 dark:text-zinc-500"
            >
              {highlight.length > 35 ? highlight.slice(0, 35) + '...' : highlight}
            </span>
          ))}
          {feature.highlights.length > 3 && (
            <span className="inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium bg-zinc-100 dark:bg-white/5 text-zinc-500 dark:text-zinc-500">
              +{feature.highlights.length - 3}
            </span>
          )}
        </div>

        {/* Read more */}
        <div className="mt-5 pt-4 border-t border-zinc-200/50 dark:border-white/5 flex items-center gap-1.5 text-sm font-medium text-indigo-600 dark:text-indigo-400 opacity-0 group-hover:opacity-100 transition-opacity">
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
/*  PlatformPage — Hub listing all platform features                          */
/* -------------------------------------------------------------------------- */

export default function PlatformPage() {
  const { i18n } = useTranslation();
  const lang = i18n.language?.slice(0, 2) || 'en';

  /* Refs for anime.js animations */
  const heroRef = useRef<HTMLDivElement>(null);
  const gridRef = useRef<HTMLDivElement>(null);
  const hasAnimatedGrid = useRef(false);
  const observerRef = useRef<IntersectionObserver | null>(null);

  const breadcrumbItems = [{ label: 'Platform' }];

  const jsonLd = buildGraphJsonLd([
    collectionPageEntity({
      url: `${SITE_URL}/platform`,
      name: 'AigisCode Platform',
      description:
        'Explore the AigisCode platform: open source MIT-licensed codebase analyzer with plugins, policy engine, architecture analysis, CLI tools, and extensible API.',
      items: platformFeatures.map((f) => ({
        name: f.title.en,
        url: `${SITE_URL}/platform/${f.slug}`,
      })),
    }),
  ]);

  /* ---- Hero fade-in animation ---- */
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
  }, []);

  /* ---- Grid stagger animation handler ---- */
  const animateGrid = useCallback(() => {
    if (!gridRef.current) return;

    const cards = gridRef.current.querySelectorAll('[data-platform-card]');
    if (cards.length === 0) return;

    if (prefersReducedMotion()) {
      cards.forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      return;
    }

    const rows = Math.ceil(cards.length / 2);
    animate(cards, {
      opacity: [0, 1],
      scale: [0.85, 1],
      translateY: [30, 0],
      duration: 500,
      delay: stagger(80, { grid: [2, rows], from: 'first' }),
      ease: 'outQuart',
    });
  }, []);

  /* ---- IntersectionObserver for grid ---- */
  useEffect(() => {
    if (!gridRef.current) return;

    if (prefersReducedMotion()) {
      gridRef.current
        .querySelectorAll('[data-platform-card]')
        .forEach((el) => {
          (el as HTMLElement).style.opacity = '1';
        });
      return;
    }

    observerRef.current = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !hasAnimatedGrid.current) {
          hasAnimatedGrid.current = true;
          animateGrid();
          observerRef.current?.disconnect();
        }
      },
      { threshold: 0.1, rootMargin: '-60px' },
    );

    observerRef.current.observe(gridRef.current);

    return () => {
      observerRef.current?.disconnect();
    };
  }, [animateGrid]);

  return (
    <>
      <SEO
        title="Platform — AigisCode"
        description="Explore the AigisCode platform: open-source Rust codebase analyzer with runtime plugins, policy and rules, architecture analysis, CLI tools, and machine-readable artifacts."
        canonical={`${SITE_URL}/platform`}
        breadcrumbs={[
          { name: 'Home', url: SITE_URL },
          { name: 'Platform', url: `${SITE_URL}/platform` },
        ]}
        jsonLd={jsonLd}
      />

      {/* Hero */}
      <section className="pt-12 pb-8 md:pt-20 md:pb-12 relative overflow-hidden">
        <AnimatedGradient className="absolute inset-0" />

        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative">
          <Breadcrumbs items={breadcrumbItems} />

          <div ref={heroRef} className="max-w-3xl opacity-0">
            <h1 className="text-3xl md:text-4xl lg:text-5xl font-display font-bold tracking-tight text-zinc-900 dark:text-white">
              Platform
            </h1>
            <p className="mt-4 text-lg text-zinc-600 dark:text-zinc-400 leading-relaxed">
              AigisCode is built as a generic, pluggable, rules-driven analysis
              system. Explore the components that make it work — from the open
              source Rust core to runtime plugins and rule packs.
            </p>
          </div>
        </div>
      </section>

      {/* Platform Features Grid */}
      <section className="pb-20 md:pb-32">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div
            ref={gridRef}
            className="grid grid-cols-1 md:grid-cols-2 gap-6"
          >
            {platformFeatures.map((feature) => (
              <div
                key={feature.slug}
                data-platform-card
                className="opacity-0"
              >
                <PlatformFeatureCard feature={feature} />
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="py-20 md:py-32 bg-zinc-900 dark:bg-zinc-950 text-white relative overflow-hidden">
        <div
          className="absolute inset-0 pointer-events-none"
          aria-hidden="true"
        >
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[300px] bg-gradient-to-r from-indigo-500/30 via-purple-500/20 to-pink-500/30 blur-[120px] rounded-full" />
        </div>

        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative">
          <motion.div
            initial={{ opacity: 0, y: 40 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-100px' }}
            transition={{ duration: 0.6 }}
            className="text-center"
          >
            <h2 className="text-3xl md:text-4xl font-display font-bold tracking-tight">
              Ready to Guard Your Codebase?
            </h2>
            <p className="mt-4 text-lg text-zinc-400 max-w-xl mx-auto">
              One command to analyze your entire project. Open source, forever
              free.
            </p>
            <div className="mt-10 flex flex-col sm:flex-row items-center justify-center gap-4">
              <a
                href="https://github.com/Draivix/aigiscode"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-2 px-7 py-3.5 rounded-full bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500 text-white font-semibold text-sm hover:scale-105 active:scale-[0.98] transition-transform shadow-xl"
              >
                Get Started
                <ArrowRight size={16} weight="bold" />
              </a>
              <div className="inline-flex items-center gap-3 bg-zinc-800/80 text-zinc-300 rounded-xl px-5 py-3 font-mono text-sm border border-zinc-700/50">
                <span className="text-zinc-500 select-none">$</span>
                <span className="select-all">cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</span>
              </div>
            </div>
          </motion.div>
        </div>
      </section>
    </>
  );
}
