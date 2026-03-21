import { useState, useEffect, useRef, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { animate, stagger } from 'animejs';
import { prefersReducedMotion } from '@/hooks/useAnime';
import { ArrowRight, Funnel } from '@phosphor-icons/react';
import SEO from '@/components/SEO';
import Breadcrumbs from '@/components/Breadcrumbs';
import UseCaseCard from '@/components/UseCaseCard';
import { useCases, categories } from '@/content/use-cases';
import {
  SITE_URL,
  buildGraphJsonLd,
  collectionPageEntity,
} from '@/content/seo-schema';

/* -------------------------------------------------------------------------- */
/*  UseCasesPage — Hub listing all use cases                                 */
/* -------------------------------------------------------------------------- */

export default function UseCasesPage() {
  const { t, i18n } = useTranslation();
  const [activeCategory, setActiveCategory] = useState<string>('All');
  const lang = i18n.language?.slice(0, 2) || 'en';

  /* Refs for anime.js animations */
  const heroRef = useRef<HTMLDivElement>(null);
  const gridRef = useRef<HTMLDivElement>(null);
  const hasAnimatedGrid = useRef(false);
  const observerRef = useRef<IntersectionObserver | null>(null);

  const filtered =
    activeCategory === 'All'
      ? useCases
      : useCases.filter((uc) => uc.category === activeCategory);

  const breadcrumbItems = [{ label: 'Use Cases' }];

  const jsonLd = buildGraphJsonLd([
    collectionPageEntity({
      url: `${SITE_URL}/use-cases`,
      name: 'AigisCode Use Cases',
      description:
        'Explore how AigisCode detects circular dependencies, dead code, hardwired values, and more across PHP, Python, TypeScript, and JavaScript codebases.',
      items: useCases.map((uc) => ({
        name: uc.title.en,
        url: `${SITE_URL}/use-cases/${uc.slug}`,
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

    return () => { anim.pause(); };
  }, []);

  /* ---- Grid stagger animation handler ---- */
  const animateGrid = useCallback(() => {
    if (!gridRef.current) return;

    const cards = gridRef.current.querySelectorAll('[data-use-case-card]');
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
      gridRef.current.querySelectorAll('[data-use-case-card]').forEach((el) => {
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

    return () => { observerRef.current?.disconnect(); };
  }, [animateGrid]);

  /* ---- Re-trigger animation on category change ---- */
  useEffect(() => {
    if (!gridRef.current || prefersReducedMotion()) {
      // Still ensure cards are visible on reduced motion
      if (gridRef.current) {
        gridRef.current.querySelectorAll('[data-use-case-card]').forEach((el) => {
          (el as HTMLElement).style.opacity = '1';
        });
      }
      return;
    }

    // Reset animation state
    hasAnimatedGrid.current = false;

    // Reset card opacity so the animation replays
    gridRef.current.querySelectorAll('[data-use-case-card]').forEach((el) => {
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

    return () => { observerRef.current?.disconnect(); };
  }, [activeCategory, animateGrid]);

  return (
    <>
      <SEO
        title="Use Cases — AigisCode"
        description="Explore how AigisCode detects circular dependencies, dead code, hardwired values, and architectural issues across PHP, Python, TypeScript, and JavaScript codebases."
        canonical={`${SITE_URL}/use-cases`}
        breadcrumbs={[
          { name: 'Home', url: SITE_URL },
          { name: 'Use Cases', url: `${SITE_URL}/use-cases` },
        ]}
        jsonLd={jsonLd}
      />

      {/* Hero */}
      <section className="pt-12 pb-8 md:pt-20 md:pb-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <Breadcrumbs items={breadcrumbItems} />

          <div
            ref={heroRef}
            className="max-w-3xl opacity-0"
          >
            <h1 className="text-3xl md:text-4xl lg:text-5xl font-display font-bold tracking-tight text-zinc-900 dark:text-white">
              Use Cases
            </h1>
            <p className="mt-4 text-lg text-zinc-600 dark:text-zinc-400 leading-relaxed">
              Discover how AigisCode helps teams detect architectural issues, eliminate dead code,
              and maintain clean codebases across frameworks and languages.
            </p>
          </div>
        </div>
      </section>

      {/* Category Filter */}
      <section className="pb-8">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center gap-2 flex-wrap">
            <Funnel size={16} className="text-zinc-400 dark:text-zinc-500 flex-shrink-0" />
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

      {/* Use Case Grid */}
      <section className="pb-20 md:pb-32">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div ref={gridRef} className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {filtered.map((uc) => (
              <div key={uc.slug} data-use-case-card className="opacity-0">
                <UseCaseCard useCase={uc} />
              </div>
            ))}
          </div>

          {filtered.length === 0 && (
            <p className="text-center text-zinc-500 dark:text-zinc-400 py-20">
              No use cases found for this category.
            </p>
          )}
        </div>
      </section>

      {/* CTA */}
      <section className="py-20 md:py-32 bg-zinc-900 dark:bg-zinc-950 text-white relative overflow-hidden">
        <div className="absolute inset-0 pointer-events-none" aria-hidden="true">
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
              One command to analyze your entire project. Open source, forever free.
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
