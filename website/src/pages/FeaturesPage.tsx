import { useState, useEffect, useRef, useCallback } from 'react';
import { LocalizedLink as Link } from '@/lib/i18n-routing';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { animate, stagger } from 'animejs';
import { prefersReducedMotion } from '@/hooks/useAnime';
import {
  ArrowRight,
  ArrowsClockwise,
  Brain,
  Bug,
  Crosshair,
  FileMagnifyingGlass,
  Funnel,
  Lightning,
  TreeStructure,
  Warning,
} from '@phosphor-icons/react';
import SEO from '@/components/SEO';
import Breadcrumbs from '@/components/Breadcrumbs';
import AnimatedGradient from '@/components/AnimatedGradient';
import FloatingIcons from '@/components/FloatingIcons';
import { features, categories } from '@/content/features';
import type { Feature } from '@/content/features';
import {
  SITE_URL,
  buildGraphJsonLd,
  collectionPageEntity,
} from '@/content/seo-schema';

/* -------------------------------------------------------------------------- */
/*  Icon map                                                                  */
/* -------------------------------------------------------------------------- */

const iconMap: Record<string, React.ElementType> = {
  ArrowsClockwise,
  Bug,
  Lightning,
  Brain,
  Warning,
  Crosshair,
  TreeStructure,
  FileMagnifyingGlass,
};

/* -------------------------------------------------------------------------- */
/*  FeatureCard                                                               */
/* -------------------------------------------------------------------------- */

interface FeatureCardProps {
  feature: Feature;
}

function FeatureCard({ feature }: FeatureCardProps) {
  const { i18n } = useTranslation();
  const lang = i18n.language?.slice(0, 2) || 'en';

  const title = feature.title[lang] || feature.title.en;
  const description =
    feature.shortDescription[lang] || feature.shortDescription.en;

  const Icon = iconMap[feature.icon];

  return (
    <Link
      to={`/features/${feature.slug}`}
      className="group relative flex flex-col rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-6 overflow-hidden transition-all duration-300 hover:shadow-lg hover:shadow-indigo-500/5 hover:border-indigo-500/20 dark:hover:border-indigo-500/20"
    >
      {/* Hover glow */}
      <div className="absolute -inset-1 rounded-2xl bg-gradient-to-br from-indigo-500/20 via-purple-500/20 to-pink-500/20 opacity-0 group-hover:opacity-100 blur-xl transition-opacity duration-500 pointer-events-none" />

      <div className="relative flex flex-col flex-1">
        {/* Icon + Category */}
        <div className="flex items-start justify-between mb-4">
          {Icon && (
            <div className="w-10 h-10 rounded-lg bg-indigo-50 dark:bg-indigo-500/10 flex items-center justify-center text-indigo-600 dark:text-indigo-400 group-hover:scale-110 transition-transform">
              <Icon size={22} />
            </div>
          )}
          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-indigo-500/10 text-indigo-600 dark:text-indigo-400">
            {feature.category}
          </span>
        </div>

        {/* Title */}
        <h3 className="font-display font-semibold text-lg mb-2 text-zinc-900 dark:text-white group-hover:text-indigo-600 dark:group-hover:text-indigo-400 transition-colors">
          {title}
        </h3>

        {/* Description */}
        <p className="text-sm text-zinc-600 dark:text-zinc-400 leading-relaxed flex-1 line-clamp-3">
          {description}
        </p>

        {/* Capabilities preview */}
        <div className="mt-4 flex flex-wrap gap-1.5">
          {feature.capabilities.slice(0, 3).map((cap) => (
            <span
              key={cap}
              className="inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium bg-zinc-100 dark:bg-white/5 text-zinc-500 dark:text-zinc-500"
            >
              {cap}
            </span>
          ))}
          {feature.capabilities.length > 3 && (
            <span className="inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium bg-zinc-100 dark:bg-white/5 text-zinc-500 dark:text-zinc-500">
              +{feature.capabilities.length - 3}
            </span>
          )}
        </div>

        {/* Read more */}
        <div className="mt-4 pt-4 border-t border-zinc-200/50 dark:border-white/5 flex items-center gap-1.5 text-sm font-medium text-indigo-600 dark:text-indigo-400 opacity-0 group-hover:opacity-100 transition-opacity">
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
/*  FeaturesPage — Hub listing all analysis features                          */
/* -------------------------------------------------------------------------- */

export default function FeaturesPage() {
  const { i18n } = useTranslation();
  const [activeCategory, setActiveCategory] = useState<string>('All');

  /* Refs for anime.js animations */
  const heroRef = useRef<HTMLDivElement>(null);
  const gridRef = useRef<HTMLDivElement>(null);
  const hasAnimatedGrid = useRef(false);
  const observerRef = useRef<IntersectionObserver | null>(null);

  const filtered =
    activeCategory === 'All'
      ? features
      : features.filter((f) => f.category === activeCategory);

  const breadcrumbItems = [{ label: 'Features' }];

  const jsonLd = buildGraphJsonLd([
    collectionPageEntity({
      url: `${SITE_URL}/features`,
      name: 'AigisCode Analysis Features',
      description:
        'Explore AigisCode analysis features: circular dependency detection, dead code detection, hardwiring detection, AI-assisted review, god class detection, bottleneck analysis, layer violation enforcement, and orphan file detection.',
      items: features.map((f) => ({
        name: f.title.en,
        url: `${SITE_URL}/features/${f.slug}`,
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

    const cards = gridRef.current.querySelectorAll('[data-feature-card]');
    if (cards.length === 0) return;

    if (prefersReducedMotion()) {
      cards.forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      return;
    }

    const rows = Math.ceil(cards.length / 3);
    animate(cards, {
      opacity: [0, 1],
      scale: [0.85, 1],
      translateY: [30, 0],
      duration: 500,
      delay: stagger(60, { grid: [3, rows], from: 'first' }),
      ease: 'outQuart',
    });
  }, []);

  /* ---- IntersectionObserver for grid — initial setup ---- */
  useEffect(() => {
    if (!gridRef.current) return;

    if (prefersReducedMotion()) {
      gridRef.current
        .querySelectorAll('[data-feature-card]')
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

  /* ---- Re-trigger animation on category change ---- */
  useEffect(() => {
    if (!gridRef.current || prefersReducedMotion()) {
      if (gridRef.current) {
        gridRef.current
          .querySelectorAll('[data-feature-card]')
          .forEach((el) => {
            (el as HTMLElement).style.opacity = '1';
          });
      }
      return;
    }

    // Reset animation state
    hasAnimatedGrid.current = false;

    // Reset card opacity so the animation replays
    gridRef.current.querySelectorAll('[data-feature-card]').forEach((el) => {
      (el as HTMLElement).style.opacity = '0';
      (el as HTMLElement).style.transform = '';
    });

    // Disconnect existing observer
    observerRef.current?.disconnect();

    // Re-observe
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
  }, [activeCategory, animateGrid]);

  return (
    <>
      <SEO
        title="Analysis Features — AigisCode"
        description="Explore AigisCode analysis features: circular dependency detection, dead code detection, hardwiring detection, AI-assisted review, god class detection, bottleneck analysis, layer violations, and orphan detection."
        canonical={`${SITE_URL}/features`}
        breadcrumbs={[
          { name: 'Home', url: SITE_URL },
          { name: 'Features', url: `${SITE_URL}/features` },
        ]}
        jsonLd={jsonLd}
      />

      {/* Hero */}
      <section className="pt-12 pb-8 md:pt-20 md:pb-12 relative overflow-hidden">
        <AnimatedGradient className="absolute inset-0" />
        <FloatingIcons className="absolute inset-0" iconSet="analysis" count={12} />

        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative">
          <Breadcrumbs items={breadcrumbItems} />

          <div ref={heroRef} className="max-w-3xl opacity-0">
            <h1 className="text-3xl md:text-4xl lg:text-5xl font-display font-bold tracking-tight text-zinc-900 dark:text-white">
              Analysis{' '}
              <span className="bg-clip-text text-transparent bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500">
                Features
              </span>
            </h1>
            <p className="mt-4 text-lg text-zinc-600 dark:text-zinc-400 leading-relaxed">
              Eight powerful detectors working together to find architectural
              issues, dead code, hardwired values, and structural problems that
              single-file linters miss.
            </p>
          </div>
        </div>
      </section>

      {/* Category Filter */}
      <section className="pb-8">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center gap-2 flex-wrap">
            <Funnel
              size={16}
              className="text-zinc-400 dark:text-zinc-500 flex-shrink-0"
            />
            <button
              onClick={() => setActiveCategory('All')}
              className={`px-3.5 py-1.5 rounded-full text-sm font-medium transition-colors ${
                activeCategory === 'All'
                  ? 'bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 border border-indigo-500/20'
                  : 'bg-zinc-100 dark:bg-white/5 text-zinc-600 dark:text-zinc-400 border border-transparent hover:border-zinc-300 dark:hover:border-white/10'
              }`}
            >
              All
            </button>
            {categories.map((cat) => (
              <button
                key={cat}
                onClick={() => setActiveCategory(cat)}
                className={`px-3.5 py-1.5 rounded-full text-sm font-medium transition-colors ${
                  activeCategory === cat
                    ? 'bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 border border-indigo-500/20'
                    : 'bg-zinc-100 dark:bg-white/5 text-zinc-600 dark:text-zinc-400 border border-transparent hover:border-zinc-300 dark:hover:border-white/10'
                }`}
              >
                {cat}
              </button>
            ))}
          </div>
        </div>
      </section>

      {/* Feature Grid */}
      <section className="pb-20 md:pb-32">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div
            ref={gridRef}
            className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6"
          >
            {filtered.map((f) => (
              <div key={f.slug} data-feature-card className="opacity-0">
                <FeatureCard feature={f} />
              </div>
            ))}
          </div>

          {filtered.length === 0 && (
            <p className="text-center text-zinc-500 dark:text-zinc-400 py-20">
              No features found for this category.
            </p>
          )}
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
