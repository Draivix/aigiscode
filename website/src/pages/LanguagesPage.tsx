import { useEffect, useRef, useCallback } from 'react';
import { LocalizedLink as Link } from '@/lib/i18n-routing';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { animate, stagger } from 'animejs';
import { prefersReducedMotion } from '@/hooks/useAnime';
import {
  ArrowRight,
  CheckCircle,
  XCircle,
  Code,
} from '@phosphor-icons/react';
import SEO from '@/components/SEO';
import Breadcrumbs from '@/components/Breadcrumbs';
import AnimatedGradient from '@/components/AnimatedGradient';
import FloatingIcons from '@/components/FloatingIcons';
import { languages } from '@/content/languages';
import type { Language } from '@/content/languages';
import {
  SITE_URL,
  buildGraphJsonLd,
  collectionPageEntity,
} from '@/content/seo-schema';

/* -------------------------------------------------------------------------- */
/*  LanguageCard                                                              */
/* -------------------------------------------------------------------------- */

function LanguageCard({ language }: { language: Language }) {
  const { i18n } = useTranslation();
  const lang = i18n.language?.slice(0, 2) || 'en';

  const title = language.title[lang] || language.title.en;
  const description =
    language.shortDescription[lang] || language.shortDescription.en;

  return (
    <Link
      to={`/languages/${language.slug}`}
      className="group relative flex flex-col rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-6 overflow-hidden transition-all duration-300 hover:shadow-lg hover:shadow-indigo-500/5 hover:border-indigo-500/20 dark:hover:border-indigo-500/20"
    >
      {/* Hover glow */}
      <div className="absolute -inset-1 rounded-2xl bg-gradient-to-br from-indigo-500/20 via-purple-500/20 to-pink-500/20 opacity-0 group-hover:opacity-100 blur-xl transition-opacity duration-500 pointer-events-none" />

      <div className="relative flex flex-col flex-1">
        {/* Icon + Parser badge row */}
        <div className="flex items-center justify-between mb-4">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500/10 to-purple-500/10 border border-indigo-500/10 flex items-center justify-center">
            <Code size={22} weight="duotone" className="text-indigo-500 dark:text-indigo-400" />
          </div>
          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-[11px] font-medium bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/10">
            {language.parser}
          </span>
        </div>

        {/* Title */}
        <h3 className="font-display font-semibold text-lg mb-1 text-zinc-900 dark:text-white group-hover:text-indigo-600 dark:group-hover:text-indigo-400 transition-colors">
          {title}
        </h3>

        {/* Extensions */}
        <p className="text-xs text-zinc-400 dark:text-zinc-500 font-mono mb-3">
          {language.extensions}
        </p>

        {/* Description */}
        <p className="text-sm text-zinc-600 dark:text-zinc-400 leading-relaxed flex-1 line-clamp-3">
          {description}
        </p>

        {/* Feature highlights */}
        <div className="mt-4 flex flex-wrap gap-1.5">
          {language.features.slice(0, 3).map((feature) => (
            <span
              key={feature}
              className="inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium bg-zinc-100 dark:bg-white/5 text-zinc-500 dark:text-zinc-500"
            >
              {feature}
            </span>
          ))}
          {language.features.length > 3 && (
            <span className="inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium bg-zinc-100 dark:bg-white/5 text-zinc-500 dark:text-zinc-500">
              +{language.features.length - 3}
            </span>
          )}
        </div>

        {/* Detector support indicators */}
        <div className="mt-4 pt-4 border-t border-zinc-200/50 dark:border-white/5">
          <div className="flex items-center gap-4 text-xs">
            <DetectorBadge label="Dead Code" supported={language.detectorSupport.deadCode} />
            <DetectorBadge label="Hardwiring" supported={language.detectorSupport.hardwiring} />
            <DetectorBadge label="Graph" supported={language.detectorSupport.graphAnalysis} />
          </div>
        </div>

        {/* Framework plugins */}
        {language.frameworkPlugins.length > 0 && (
          <div className="mt-3 flex items-center gap-2">
            <span className="text-[11px] text-zinc-400 dark:text-zinc-500">Plugins:</span>
            {language.frameworkPlugins.map((fw) => (
              <span
                key={fw}
                className="inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium bg-purple-500/10 text-purple-600 dark:text-purple-400"
              >
                {fw}
              </span>
            ))}
          </div>
        )}

        {/* Read more */}
        <div className="mt-4 flex items-center gap-1.5 text-sm font-medium text-indigo-600 dark:text-indigo-400 opacity-0 group-hover:opacity-100 transition-opacity">
          Learn more
          <ArrowRight size={14} weight="bold" className="transition-transform group-hover:translate-x-0.5" />
        </div>
      </div>
    </Link>
  );
}

/* -------------------------------------------------------------------------- */
/*  DetectorBadge                                                             */
/* -------------------------------------------------------------------------- */

function DetectorBadge({ label, supported }: { label: string; supported: boolean }) {
  return (
    <span className="flex items-center gap-1">
      {supported ? (
        <CheckCircle size={14} weight="fill" className="text-emerald-500" />
      ) : (
        <XCircle size={14} weight="fill" className="text-zinc-300 dark:text-zinc-600" />
      )}
      <span className={supported ? 'text-zinc-600 dark:text-zinc-400' : 'text-zinc-400 dark:text-zinc-600'}>
        {label}
      </span>
    </span>
  );
}

/* -------------------------------------------------------------------------- */
/*  LanguagesPage — Hub listing all supported languages                      */
/* -------------------------------------------------------------------------- */

export default function LanguagesPage() {
  const { i18n } = useTranslation();

  /* Refs for anime.js animations */
  const heroRef = useRef<HTMLDivElement>(null);
  const gridRef = useRef<HTMLDivElement>(null);
  const hasAnimatedGrid = useRef(false);
  const observerRef = useRef<IntersectionObserver | null>(null);

  const breadcrumbItems = [{ label: 'Languages' }];

  const jsonLd = buildGraphJsonLd([
    collectionPageEntity({
      url: `${SITE_URL}/languages`,
      name: 'Supported Languages - AigisCode',
      description:
        'AigisCode supports PHP, Python, TypeScript, JavaScript, Vue, and Ruby with deep structural analysis, circular dependency detection, dead code detection, and hardwired value detection.',
      items: languages.map((lang) => ({
        name: lang.title.en,
        url: `${SITE_URL}/languages/${lang.slug}`,
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

    const cards = gridRef.current.querySelectorAll('[data-language-card]');
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
      delay: stagger(80, { grid: [3, rows], from: 'first' }),
      ease: 'outQuart',
    });
  }, []);

  /* ---- IntersectionObserver for grid ---- */
  useEffect(() => {
    if (!gridRef.current) return;

    if (prefersReducedMotion()) {
      gridRef.current.querySelectorAll('[data-language-card]').forEach((el) => {
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

  return (
    <>
      <SEO
        title="Supported Languages — AigisCode"
        description="AigisCode analyzes PHP, Python, TypeScript, JavaScript, Vue, and Ruby codebases for circular dependencies, dead code, hardwired values, and architectural issues."
        canonical={`${SITE_URL}/languages`}
        breadcrumbs={[
          { name: 'Home', url: SITE_URL },
          { name: 'Languages', url: `${SITE_URL}/languages` },
        ]}
        jsonLd={jsonLd}
      />

      {/* Hero */}
      <section className="pt-12 pb-8 md:pt-20 md:pb-12 relative overflow-hidden">
        <AnimatedGradient className="absolute inset-0" />
        <FloatingIcons className="absolute inset-0" iconSet="code" count={10} />

        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative">
          <Breadcrumbs items={breadcrumbItems} />

          <div
            ref={heroRef}
            className="max-w-3xl opacity-0"
          >
            <h1 className="text-3xl md:text-4xl lg:text-5xl font-display font-bold tracking-tight text-zinc-900 dark:text-white">
              Supported Languages
            </h1>
            <p className="mt-4 text-lg text-zinc-600 dark:text-zinc-400 leading-relaxed">
              AigisCode provides deep structural analysis for six languages. Each parser extracts
              classes, functions, imports, and language-specific constructs to build an accurate
              dependency graph for cycle detection, dead code analysis, and hardwired value detection.
            </p>
          </div>
        </div>
      </section>

      {/* Language Grid */}
      <section className="pb-20 md:pb-32">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div ref={gridRef} className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {languages.map((language) => (
              <div key={language.slug} data-language-card className="opacity-0">
                <LanguageCard language={language} />
              </div>
            ))}
          </div>
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
              Analyze Any Supported Codebase
            </h2>
            <p className="mt-4 text-lg text-zinc-400 max-w-xl mx-auto">
              One command, six languages. Deep structural analysis that single-file linters miss.
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
