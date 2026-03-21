import { useState, useRef, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { motion, useInView } from 'motion/react';
import { animate, stagger, createTimeline } from 'animejs';
import {
  ArrowsClockwise,
  Trash,
  LinkBreak,
  Robot,
  Sliders,
  Code,
  Copy,
  Check,
  ArrowRight,
  GithubLogo,
} from '@phosphor-icons/react';
import ShieldBackground from '@/components/ShieldBackground';
import PipelineSVG from '@/components/PipelineSVG';
import { prefersReducedMotion } from '@/hooks/useAnime';
import SEO from '@/components/SEO';
import {
  SITE_URL,
  buildGraphJsonLd,
  webPageEntity,
  organizationEntity,
  founderEntity,
} from '@/content/seo-schema';

/* -------------------------------------------------------------------------- */
/*  Constants                                                                 */
/* -------------------------------------------------------------------------- */

const featureKeys = [
  { key: 'circular', Icon: ArrowsClockwise },
  { key: 'deadCode', Icon: Trash },
  { key: 'hardwiring', Icon: LinkBreak },
  { key: 'aiReview', Icon: Robot },
  { key: 'policy', Icon: Sliders },
  { key: 'multiLang', Icon: Code },
] as const;

const pipelineStageKeys = ['index', 'graph', 'detect', 'rules', 'review', 'report'] as const;

const supportedLanguages = ['PHP', 'Python', 'TypeScript', 'JavaScript', 'Vue'];

/* Terminal lines with embedded numeric targets for count-up animation */
const terminalLines = [
  { text: '$ aigiscode analyze .', color: 'text-zinc-300', delay: 0, numbers: [] as { value: number; label: string }[] },
  { text: '', color: '', delay: 0.15, numbers: [] as { value: number; label: string }[] },
  { text: '  AigisCode v0.1.0', color: 'text-indigo-400', delay: 0.3, numbers: [] as { value: number; label: string }[] },
  { text: '', color: '', delay: 0.35, numbers: [] as { value: number; label: string }[] },
  { text: '  Indexing...     {127} files parsed', color: 'text-zinc-400', delay: 0.5, numbers: [{ value: 127, label: 'files' }] },
  { text: '  Graphing...     {843} dependencies mapped', color: 'text-zinc-400', delay: 0.65, numbers: [{ value: 843, label: 'deps' }] },
  { text: '  Detecting...    dead code, hardwiring', color: 'text-zinc-400', delay: 0.8, numbers: [] as { value: number; label: string }[] },
  { text: '  Reviewing...    AI classifying {23} findings', color: 'text-zinc-400', delay: 0.95, numbers: [{ value: 23, label: 'findings' }] },
  { text: '', color: '', delay: 1.05, numbers: [] as { value: number; label: string }[] },
  { text: '  Results:', color: 'text-emerald-400', delay: 1.15, numbers: [] as { value: number; label: string }[] },
  { text: '  \u251c\u2500\u2500 {3} circular dependencies (strong)', color: 'text-amber-400', delay: 1.3, numbers: [{ value: 3, label: 'circular' }] },
  { text: '  \u251c\u2500\u2500 {12} unused imports', color: 'text-amber-400', delay: 1.45, numbers: [{ value: 12, label: 'unused' }] },
  { text: '  \u251c\u2500\u2500 {5} magic strings', color: 'text-amber-400', delay: 1.6, numbers: [{ value: 5, label: 'magic' }] },
  { text: '  \u2514\u2500\u2500 {3} findings reclassified by AI', color: 'text-emerald-400', delay: 1.75, numbers: [{ value: 3, label: 'reclass' }] },
  { text: '', color: '', delay: 1.85, numbers: [] as { value: number; label: string }[] },
  { text: '  Report: .aigiscode/aigiscode-report.md', color: 'text-zinc-500', delay: 1.95, numbers: [] as { value: number; label: string }[] },
];

/* -------------------------------------------------------------------------- */
/*  Hero Section                                                              */
/* -------------------------------------------------------------------------- */

function HeroSection() {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);

  /* Refs for anime.js animations */
  const titleRef = useRef<HTMLHeadingElement>(null);
  const subtitleRef = useRef<HTMLParagraphElement>(null);
  const ctaContainerRef = useRef<HTMLDivElement>(null);
  const heroGlowRef = useRef<HTMLDivElement>(null);
  const ctaButtonLeftRef = useRef<HTMLAnchorElement>(null);
  const ctaButtonRightRef = useRef<HTMLAnchorElement>(null);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText('curl -fsSL https://raw.githubusercontent.com/Draivix/aigiscode/main/install.sh | bash');
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      /* clipboard not available */
    }
  };

  /* --- Title word-level reveal animation --- */
  useEffect(() => {
    if (!titleRef.current) return;

    const h1 = titleRef.current;

    if (prefersReducedMotion()) {
      h1.style.opacity = '1';
      h1.querySelectorAll('[data-hero-word]').forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      const gradientSpan = h1.querySelector('[data-hero-gradient]') as HTMLElement | null;
      if (gradientSpan) gradientSpan.style.opacity = '1';
      return;
    }

    h1.style.opacity = '1';

    const words = h1.querySelectorAll('[data-hero-word]');
    const gradientSpan = h1.querySelector('[data-hero-gradient]');

    // Animate regular words with dramatic scale + blur reveal
    const wordAnim = animate(words, {
      opacity: [0, 1],
      translateY: [30, 0],
      scale: [0.7, 1.05, 1],
      filter: ['blur(8px)', 'blur(0px)'],
      duration: 700,
      delay: stagger(80),
      ease: 'outQuart',
    });

    // Animate gradient text last with 3D rotateX tilt and scale bounce
    let gradientAnim: ReturnType<typeof animate> | null = null;
    if (gradientSpan) {
      // Apply perspective to parent for 3D effect
      (gradientSpan as HTMLElement).style.perspective = '600px';
      gradientAnim = animate(gradientSpan, {
        opacity: [0, 1],
        translateY: [40, 0],
        scale: [0.7, 1.08, 1],
        rotateX: [15, 0],
        filter: ['blur(8px)', 'blur(0px)'],
        duration: 900,
        delay: words.length * 80 + 100,
        ease: 'outBack',
      });
    }

    return () => {
      wordAnim.pause();
      gradientAnim?.pause();
    };
  }, [t]);

  /* --- Subtitle fade+slide animation --- */
  useEffect(() => {
    if (!subtitleRef.current) return;

    if (prefersReducedMotion()) {
      subtitleRef.current.style.opacity = '1';
      return;
    }

    const anim = animate(subtitleRef.current, {
      opacity: [0, 1],
      translateY: [25, 0],
      filter: ['blur(4px)', 'blur(0px)'],
      duration: 700,
      delay: 400,
      ease: 'outQuart',
    });

    return () => {
      anim.pause();
    };
  }, []);

  /* --- CTA buttons slide in from opposite sides + install bar --- */
  useEffect(() => {
    if (!ctaContainerRef.current) return;

    const children = ctaContainerRef.current.children;
    if (!children.length) return;

    if (prefersReducedMotion()) {
      Array.from(children).forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      if (ctaButtonLeftRef.current) ctaButtonLeftRef.current.style.opacity = '1';
      if (ctaButtonRightRef.current) ctaButtonRightRef.current.style.opacity = '1';
      return;
    }

    const anims: ReturnType<typeof animate>[] = [];

    // Animate the CTA row container (opacity only)
    anims.push(animate(children[0], {
      opacity: [0, 1],
      duration: 400,
      delay: 600,
      ease: 'outQuart',
    }));

    // First CTA button slides in from the left
    if (ctaButtonLeftRef.current) {
      anims.push(animate(ctaButtonLeftRef.current, {
        opacity: [0, 1],
        translateX: [-60, 0],
        scale: [0.9, 1],
        duration: 600,
        delay: 650,
        ease: 'outBack',
      }));
    }

    // Second CTA button slides in from the right
    if (ctaButtonRightRef.current) {
      anims.push(animate(ctaButtonRightRef.current, {
        opacity: [0, 1],
        translateX: [60, 0],
        scale: [0.9, 1],
        duration: 600,
        delay: 750,
        ease: 'outBack',
      }));
    }

    // Install command bar fades up
    if (children[1]) {
      anims.push(animate(children[1], {
        opacity: [0, 1],
        translateY: [20, 0],
        scale: [0.95, 1],
        duration: 500,
        delay: 900,
        ease: 'outQuart',
      }));
    }

    return () => {
      anims.forEach((a) => a.pause());
    };
  }, []);

  /* --- Pulsing gradient glow behind hero title --- */
  useEffect(() => {
    if (!heroGlowRef.current || prefersReducedMotion()) return;

    const anim = animate(heroGlowRef.current, {
      opacity: [0.4, 0.8],
      scale: [0.85, 1.2],
      loop: true,
      alternate: true,
      duration: 4000,
      ease: 'inOutSine',
    });

    return () => {
      anim.pause();
    };
  }, []);

  /* Split title1 into word spans */
  const title1 = t('hero.title1');
  const title1Words = title1.split(/\s+/);

  return (
    <section className="relative py-20 md:py-32 lg:py-40 overflow-hidden">
      {/* Animated shield — draws itself behind the hero text */}
      <ShieldBackground />

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center relative z-10">
        {/* Pulsing gradient glow behind title */}
        <div
          ref={heroGlowRef}
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-[60%] w-[500px] h-[200px] md:w-[700px] md:h-[280px] bg-gradient-to-r from-indigo-500/25 via-purple-500/20 to-pink-500/25 blur-[100px] rounded-full pointer-events-none"
          aria-hidden="true"
        />

        {/* Title with perspective wrapper for 3D effects */}
        <div style={{ perspective: '800px' }}>
          <h1
            ref={titleRef}
            className="text-[3rem] md:text-[4.5rem] lg:text-[5.5rem] font-display font-bold tracking-tighter leading-[1.05]"
            style={{ opacity: 0 }}
          >
            {title1Words.map((word, i) => (
              <span
                key={i}
                data-hero-word
                className="inline-block mr-[0.3em]"
                style={{ opacity: 0 }}
              >
                {word}
              </span>
            ))}
            <span
              data-hero-gradient
              className="inline-block bg-clip-text text-transparent bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500"
              style={{ opacity: 0, transformStyle: 'preserve-3d' }}
            >
              {t('hero.title2')}
            </span>
          </h1>
        </div>

        {/* Subtitle */}
        <p
          ref={subtitleRef}
          className="mt-6 text-lg md:text-xl text-zinc-600 dark:text-zinc-400 max-w-2xl mx-auto leading-relaxed"
          style={{ opacity: 0 }}
        >
          {t('hero.subtitle')}
        </p>

        {/* CTAs + Install Command */}
        <div ref={ctaContainerRef}>
          <div
            className="mt-10 flex flex-col sm:flex-row items-center justify-center gap-4"
            style={{ opacity: 0 }}
          >
            <a
              ref={ctaButtonLeftRef}
              href="#get-started"
              className="inline-flex items-center gap-2 px-7 py-3.5 rounded-full bg-zinc-900 dark:bg-white text-white dark:text-black text-sm font-semibold hover:scale-105 active:scale-[0.98] transition-transform shadow-lg shadow-zinc-900/20 dark:shadow-white/10"
              style={{ opacity: 0 }}
            >
              {t('hero.cta1')}
              <ArrowRight size={16} weight="bold" />
            </a>
            <a
              ref={ctaButtonRightRef}
              href="https://github.com/Draivix/aigiscode"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 px-7 py-3.5 rounded-full border border-zinc-300 dark:border-white/10 text-zinc-700 dark:text-zinc-300 text-sm font-semibold hover:bg-zinc-100 dark:hover:bg-white/5 transition-colors"
              style={{ opacity: 0 }}
            >
              <GithubLogo size={18} weight="bold" />
              {t('hero.cta2')}
            </a>
          </div>

          <div
            className="mt-8 mx-4 sm:mx-auto max-w-full sm:w-fit grid grid-cols-[auto_1fr_auto] items-center gap-2 sm:gap-3 bg-zinc-900 dark:bg-zinc-800/80 text-zinc-300 rounded-xl px-3 sm:px-5 py-3 font-mono text-[11px] sm:text-sm border border-zinc-800 dark:border-zinc-700/50"
            style={{ opacity: 0 }}
          >
            <span className="text-zinc-500 select-none">$</span>
            <div className="overflow-x-auto whitespace-nowrap scrollbar-none select-all">{t('hero.install')}</div>
            <button
              onClick={handleCopy}
              className="p-1 rounded hover:bg-white/10 transition-colors"
              aria-label="Copy install command"
            >
              {copied ? (
                <Check size={16} className="text-emerald-400" weight="bold" />
              ) : (
                <Copy size={16} className="text-zinc-500" />
              )}
            </button>
          </div>
        </div>
      </div>
    </section>
  );
}

/* -------------------------------------------------------------------------- */
/*  Trusted Languages Bar                                                     */
/* -------------------------------------------------------------------------- */

function TrustedBar() {
  const { t } = useTranslation();
  const sectionRef = useRef<HTMLDivElement>(null);
  const titleRef = useRef<HTMLParagraphElement>(null);
  const hasAnimated = useRef(false);

  useEffect(() => {
    if (!sectionRef.current) return;

    const badges = sectionRef.current.querySelectorAll('[data-lang-badge]');

    if (prefersReducedMotion()) {
      badges.forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      if (titleRef.current) titleRef.current.style.opacity = '1';
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !hasAnimated.current) {
          hasAnimated.current = true;

          // Animate the section title first — fade in from top
          if (titleRef.current) {
            animate(titleRef.current, {
              opacity: [0, 1],
              translateY: [-15, 0],
              duration: 500,
              ease: 'outQuart',
            });
          }

          // Wave-like stagger with rotateZ wobble and spring bounce
          animate(badges, {
            opacity: [0, 1],
            scale: [0.5, 1.1, 1],
            translateY: [20, -5, 0],
            rotateZ: [-4, 2, 0],
            duration: 700,
            delay: stagger(80, { from: 'center' }),
            ease: 'outElastic(1, .6)',
          });

          observer.disconnect();
        }
      },
      { threshold: 0.1, rootMargin: '-50px' },
    );

    observer.observe(sectionRef.current);
    return () => observer.disconnect();
  }, []);

  return (
    <section className="relative py-12 md:py-16" aria-label={t('trustedBy.title')}>
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div ref={sectionRef} className="text-center">
          <p
            ref={titleRef}
            className="text-xs uppercase tracking-widest text-zinc-400 dark:text-zinc-500 font-medium mb-6"
            style={{ opacity: 0 }}
          >
            {t('trustedBy.title')}
          </p>
          <div className="flex flex-wrap items-center justify-center gap-4 md:gap-6">
            {supportedLanguages.map((lang) => (
              <span
                key={lang}
                data-lang-badge
                className="px-5 py-2 rounded-full bg-white/60 dark:bg-white/5 border border-zinc-200/60 dark:border-white/5 font-mono text-sm text-zinc-600 dark:text-zinc-400 backdrop-blur-sm"
                style={{ opacity: 0 }}
              >
                {lang}
              </span>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

/* -------------------------------------------------------------------------- */
/*  Feature Bento Grid — anime.js stagger wave on scroll                      */
/* -------------------------------------------------------------------------- */

function FeatureCard({ featureKey, Icon, index }: { featureKey: string; Icon: React.ElementType; index: number }) {
  const { t } = useTranslation();

  return (
    <div
      data-feature-card
      className="group relative rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-6 overflow-hidden transition-shadow hover:shadow-lg hover:shadow-indigo-500/5 opacity-0"
    >
      {/* Hover glow */}
      <div className="absolute -inset-1 rounded-2xl bg-gradient-to-br from-indigo-500/20 via-purple-500/20 to-pink-500/20 opacity-0 group-hover:opacity-100 blur-xl transition-opacity duration-500 pointer-events-none" />

      {/* Border shimmer overlay */}
      <div
        data-feature-shimmer
        className="absolute inset-0 rounded-2xl pointer-events-none opacity-0"
        style={{
          background: 'linear-gradient(90deg, transparent 0%, rgba(99,102,241,0.3) 50%, transparent 100%)',
          backgroundSize: '200% 100%',
          mask: 'linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0)',
          maskComposite: 'exclude',
          WebkitMaskComposite: 'xor',
          padding: '1px',
          borderRadius: 'inherit',
        }}
      />

      <div className="relative">
        <div
          data-feature-icon
          className="mb-4 inline-flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500/10 to-purple-500/10 dark:from-indigo-500/20 dark:to-purple-500/20 text-indigo-500"
          style={{ transform: 'scale(0)' }}
        >
          <Icon size={24} weight="duotone" />
        </div>
        <h3 className="font-display font-semibold text-lg mb-2 text-zinc-900 dark:text-white">
          {t(`features.${featureKey}.title`)}
        </h3>
        <p className="text-sm text-zinc-600 dark:text-zinc-400 leading-relaxed">
          {t(`features.${featureKey}.description`)}
        </p>
      </div>
    </div>
  );
}

function FeaturesSection() {
  const { t } = useTranslation();
  const gridRef = useRef<HTMLDivElement>(null);
  const hasAnimated = useRef(false);

  useEffect(() => {
    if (!gridRef.current || prefersReducedMotion()) {
      // If reduced motion, just show all cards immediately
      if (gridRef.current) {
        gridRef.current.querySelectorAll('[data-feature-card]').forEach((el) => {
          (el as HTMLElement).style.opacity = '1';
        });
        gridRef.current.querySelectorAll('[data-feature-icon]').forEach((el) => {
          (el as HTMLElement).style.transform = 'scale(1)';
        });
      }
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !hasAnimated.current) {
          hasAnimated.current = true;

          const cards = gridRef.current!.querySelectorAll('[data-feature-card]');
          const icons = gridRef.current!.querySelectorAll('[data-feature-icon]');
          const shimmers = gridRef.current!.querySelectorAll('[data-feature-shimmer]');

          // Cards enter with more dramatic translateY
          animate(cards, {
            opacity: [0, 1],
            scale: [0.85, 1],
            translateY: [60, 0],
            duration: 700,
            delay: stagger(80, { grid: [3, 2], from: 'first' }),
            ease: 'outQuart',
          });

          // Icons bounce in after cards with spring effect
          animate(icons, {
            scale: [0, 1.3, 1],
            duration: 600,
            delay: stagger(80, { grid: [3, 2], from: 'first', start: 300 }),
            ease: 'outElastic(1, .6)',
          });

          // Border shimmer sweep after cards appear
          animate(shimmers, {
            opacity: [0, 0.8, 0],
            backgroundPosition: ['200% 0%', '-200% 0%'],
            duration: 1200,
            delay: stagger(80, { grid: [3, 2], from: 'first', start: 600 }),
            ease: 'inOutSine',
          });

          observer.disconnect();
        }
      },
      { threshold: 0.15, rootMargin: '-60px' },
    );

    observer.observe(gridRef.current);
    return () => observer.disconnect();
  }, []);

  return (
    <section id="features" className="py-20 md:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <motion.div
          initial={{ opacity: 0, y: 40 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-100px' }}
          transition={{ duration: 0.6 }}
          className="text-center mb-16"
        >
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-display font-bold tracking-tight text-zinc-900 dark:text-white">
            {t('features.title')}
          </h2>
          <p className="mt-4 text-lg text-zinc-600 dark:text-zinc-400 max-w-2xl mx-auto">
            {t('features.subtitle')}
          </p>
        </motion.div>

        <div ref={gridRef} className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {featureKeys.map(({ key, Icon }, index) => (
            <FeatureCard key={key} featureKey={key} Icon={Icon} index={index} />
          ))}
        </div>
      </div>
    </section>
  );
}

/* -------------------------------------------------------------------------- */
/*  Pipeline / How It Works — with animated SVG connection                    */
/* -------------------------------------------------------------------------- */

function PipelineStage({ stageKey, index }: { stageKey: string; index: number }) {
  const { t } = useTranslation();

  return (
    <div
      data-pipeline-stage
      className="flex flex-col items-center text-center flex-1 min-w-0"
      style={{ opacity: 0 }}
    >
      {/* Number circle with glow backdrop */}
      <div className="relative flex-shrink-0">
        {/* Glow pulse behind circle */}
        <div
          data-pipeline-glow
          className="absolute inset-0 w-12 h-12 rounded-full bg-indigo-500/40 blur-lg opacity-0"
        />
        <div
          data-pipeline-circle
          className="w-12 h-12 rounded-full bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-white font-display font-bold text-sm shadow-lg shadow-indigo-500/25 relative z-10"
          style={{ transform: 'scale(0)' }}
        >
          {index + 1}
        </div>
      </div>
      <h3 className="mt-4 font-display font-semibold text-zinc-900 dark:text-white text-sm md:text-base">
        {t(`pipeline.stages.${stageKey}.title`)}
      </h3>
      <p className="mt-1 text-xs md:text-sm text-zinc-500 dark:text-zinc-400 leading-relaxed max-w-[160px]">
        {t(`pipeline.stages.${stageKey}.description`)}
      </p>
    </div>
  );
}

function PipelineSection() {
  const { t } = useTranslation();
  const pipelineRef = useRef<HTMLDivElement>(null);
  const stagesContainerRef = useRef<HTMLDivElement>(null);
  const mobileStagesRef = useRef<HTMLDivElement>(null);
  const isInView = useInView(pipelineRef, { once: true, margin: '-100px' });
  const hasAnimatedStages = useRef(false);

  /* Animate pipeline stages with anime.js stagger on scroll */
  useEffect(() => {
    const desktopContainer = stagesContainerRef.current;
    const mobileContainer = mobileStagesRef.current;

    // Gather all stage elements from both containers
    const allStages: Element[] = [];
    if (desktopContainer) {
      allStages.push(...Array.from(desktopContainer.querySelectorAll('[data-pipeline-stage]')));
    }
    if (mobileContainer) {
      allStages.push(...Array.from(mobileContainer.querySelectorAll('[data-pipeline-stage]')));
    }

    if (!allStages.length) return;

    if (prefersReducedMotion()) {
      allStages.forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
        const circle = el.querySelector('[data-pipeline-circle]') as HTMLElement | null;
        if (circle) circle.style.transform = 'scale(1)';
      });
      return;
    }

    const observeTarget = desktopContainer || mobileContainer;
    if (!observeTarget) return;

    const animateContainer = (container: HTMLElement) => {
      const stages = container.querySelectorAll('[data-pipeline-stage]');
      const circles = container.querySelectorAll('[data-pipeline-circle]');
      const glows = container.querySelectorAll('[data-pipeline-glow]');

      if (!stages.length) return;

      // Stages fade in and slide up
      animate(stages, {
        opacity: [0, 1],
        translateY: [30, 0],
        scale: [0.9, 1],
        duration: 500,
        delay: stagger(120, { from: 'first' }),
        ease: 'outQuart',
      });

      // Number circles pop with spring after stage appears
      animate(circles, {
        scale: [0, 1.2, 1],
        duration: 600,
        delay: stagger(120, { from: 'first', start: 200 }),
        ease: 'outElastic(1, .6)',
      });

      // Sequential glow pulse behind each circle
      animate(glows, {
        opacity: [0, 0.8, 0],
        scale: [0.8, 1.8, 1],
        duration: 800,
        delay: stagger(120, { from: 'first', start: 300 }),
        ease: 'outQuart',
      });
    };

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !hasAnimatedStages.current) {
          hasAnimatedStages.current = true;

          if (desktopContainer) animateContainer(desktopContainer);
          if (mobileContainer) animateContainer(mobileContainer);

          observer.disconnect();
        }
      },
      { threshold: 0.1, rootMargin: '-60px' },
    );

    observer.observe(observeTarget);
    return () => observer.disconnect();
  }, []);

  return (
    <section id="how-it-works" className="py-20 md:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <motion.div
          initial={{ opacity: 0, y: 40 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-100px' }}
          transition={{ duration: 0.6 }}
          className="text-center mb-16"
        >
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-display font-bold tracking-tight text-zinc-900 dark:text-white">
            {t('pipeline.title')}
          </h2>
          <p className="mt-4 text-lg text-zinc-600 dark:text-zinc-400 max-w-2xl mx-auto">
            {t('pipeline.subtitle')}
          </p>
        </motion.div>

        {/* Desktop: horizontal flow with animated SVG line */}
        <div ref={pipelineRef} className="hidden lg:flex items-start justify-center relative">
          <PipelineSVG stageCount={pipelineStageKeys.length} isInView={isInView} />
          <div ref={stagesContainerRef} className="flex items-start justify-center w-full">
            {pipelineStageKeys.map((key, i) => (
              <PipelineStage key={key} stageKey={key} index={i} />
            ))}
          </div>
        </div>

        {/* Mobile / Tablet: vertical list */}
        <div ref={mobileStagesRef} className="lg:hidden grid grid-cols-2 sm:grid-cols-3 gap-8">
          {pipelineStageKeys.map((key, i) => (
            <PipelineStage key={key} stageKey={key} index={i} />
          ))}
        </div>
      </div>
    </section>
  );
}

/* -------------------------------------------------------------------------- */
/*  Terminal Demo — Typewriter + Counter Animations                           */
/* -------------------------------------------------------------------------- */

/**
 * Renders a single terminal line with character-by-character typewriter.
 * Numbers wrapped in {N} are animated with count-up.
 */
function TypewriterLine({
  line,
  startTime,
}: {
  line: (typeof terminalLines)[number];
  startTime: number;
}) {
  const lineRef = useRef<HTMLDivElement>(null);
  const hasAnimated = useRef(false);

  useEffect(() => {
    if (!lineRef.current || hasAnimated.current) return;
    if (!line.text) return;

    hasAnimated.current = true;

    if (prefersReducedMotion()) {
      // Just show the text immediately with numbers replaced
      const text = line.text.replace(/\{(\d+)\}/g, '$1');
      lineRef.current.textContent = text;
      return;
    }

    const el = lineRef.current;

    // Parse the text: split on {N} patterns for counter animation
    const parts = line.text.split(/(\{\d+\})/g);
    el.innerHTML = '';

    const spans: HTMLSpanElement[] = [];
    const counterSpans: { span: HTMLSpanElement; target: number }[] = [];

    for (const part of parts) {
      const numMatch = part.match(/^\{(\d+)\}$/);
      if (numMatch) {
        // This is a number to animate
        const span = document.createElement('span');
        span.textContent = '0';
        span.style.opacity = '0';
        span.style.display = 'inline';
        el.appendChild(span);
        spans.push(span);
        counterSpans.push({ span, target: parseInt(numMatch[1], 10) });
      } else {
        // Regular characters — wrap each in a span
        for (const char of part) {
          const span = document.createElement('span');
          span.textContent = char;
          span.style.opacity = '0';
          span.style.display = 'inline';
          el.appendChild(span);
          spans.push(span);
        }
      }
    }

    // Typewriter: reveal characters one by one
    const charDelay = Math.min(30, 600 / Math.max(spans.length, 1));

    const tl = createTimeline({
      autoplay: true,
      delay: startTime,
    });

    // Reveal all character spans
    tl.add(spans, {
      opacity: [0, 1],
      duration: 50,
      delay: stagger(charDelay),
      ease: 'out',
    });

    // After typewriter finishes, count up the numbers
    for (const { span, target } of counterSpans) {
      const obj = { val: 0 };
      // Start counter a bit after the character is revealed
      animate(obj, {
        val: target,
        duration: 800,
        delay: startTime + spans.indexOf(span) * charDelay + 100,
        ease: 'outExpo',
        onUpdate: () => {
          span.textContent = String(Math.round(obj.val));
        },
      });
    }

    return () => {
      tl.pause();
    };
  }, [line.text, startTime]);

  if (!line.text) {
    return <div className={`${line.color} whitespace-pre`}>{'\u00A0'}</div>;
  }

  return (
    <div ref={lineRef} className={`${line.color} whitespace-pre`} />
  );
}

function TerminalCursor({ visible }: { visible: boolean }) {
  const cursorRef = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    if (!visible || !cursorRef.current || prefersReducedMotion()) return;

    const anim = animate(cursorRef.current, {
      opacity: [1, 0.2, 1],
      duration: 800,
      loop: true,
      ease: 'steps(2)',
    });

    return () => {
      anim.pause();
    };
  }, [visible]);

  if (!visible) return null;

  return (
    <span
      ref={cursorRef}
      className="inline-block w-[8px] h-[14px] bg-zinc-300 ml-0.5 align-middle"
    />
  );
}

function TerminalDemo() {
  const termRef = useRef<HTMLDivElement>(null);
  const terminalContainerRef = useRef<HTMLDivElement>(null);
  const shimmerRef = useRef<HTMLDivElement>(null);
  const isInView = useInView(termRef, { once: true, margin: '-100px' });
  const [showCursor, setShowCursor] = useState(false);
  const [terminalReady, setTerminalReady] = useState(false);

  /* Terminal open animation — scaleY from 0 to 1 */
  useEffect(() => {
    if (!isInView || !terminalContainerRef.current) return;

    if (prefersReducedMotion()) {
      terminalContainerRef.current.style.transform = 'scaleY(1)';
      terminalContainerRef.current.style.opacity = '1';
      setTerminalReady(true);
      return;
    }

    const anim = animate(terminalContainerRef.current, {
      scaleY: [0, 1],
      opacity: [0, 1],
      duration: 500,
      ease: 'outBack',
      onComplete: () => {
        setTerminalReady(true);
      },
    });

    return () => {
      anim.pause();
    };
  }, [isInView]);

  /* Show cursor and trigger shimmer after typing finishes */
  useEffect(() => {
    if (!terminalReady) return;

    setShowCursor(true);
    const totalTime = terminalLines[terminalLines.length - 1].delay * 1000 + 2000;

    // Processing shimmer after all lines typed
    const shimmerTimer = setTimeout(() => {
      if (shimmerRef.current && !prefersReducedMotion()) {
        animate(shimmerRef.current, {
          opacity: [0, 0.3, 0],
          translateX: ['-100%', '200%'],
          duration: 1000,
          ease: 'inOutSine',
        });
      }
    }, totalTime - 500);

    const cursorTimer = setTimeout(() => setShowCursor(false), totalTime);

    return () => {
      clearTimeout(shimmerTimer);
      clearTimeout(cursorTimer);
    };
  }, [terminalReady]);

  return (
    <section id="demo" className="py-20 md:py-32" aria-label="Terminal demo">
      <div ref={termRef} className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
        <div
          ref={terminalContainerRef}
          className="rounded-2xl bg-zinc-900 dark:bg-zinc-950 border border-zinc-800 dark:border-zinc-800/60 shadow-2xl shadow-black/20 overflow-hidden origin-top"
          style={{ transform: 'scaleY(0)', opacity: 0 }}
          role="img"
          aria-label="Terminal showing AigisCode analyzing a codebase and finding 3 circular dependencies, 12 unused imports, 5 magic strings"
        >
          {/* Title bar */}
          <div className="flex items-center gap-2 px-5 py-3 border-b border-zinc-800 dark:border-zinc-800/60" aria-hidden="true">
            <div className="w-3 h-3 rounded-full bg-red-500/80" />
            <div className="w-3 h-3 rounded-full bg-yellow-500/80" />
            <div className="w-3 h-3 rounded-full bg-green-500/80" />
            <span className="ml-3 text-xs text-zinc-500 font-mono">terminal</span>
          </div>

          {/* Terminal content */}
          <div className="p-5 md:p-6 font-mono text-xs md:text-sm leading-relaxed overflow-x-auto relative">
            {/* Processing shimmer overlay */}
            <div
              ref={shimmerRef}
              className="absolute inset-0 pointer-events-none opacity-0"
              style={{
                background: 'linear-gradient(90deg, transparent 0%, rgba(99,102,241,0.15) 50%, transparent 100%)',
                width: '50%',
              }}
              aria-hidden="true"
            />
            {terminalReady &&
              terminalLines.map((line, i) => (
                <TypewriterLine
                  key={i}
                  line={line}
                  startTime={line.delay * 1000}
                />
              ))}
            {showCursor && (
              <div className="inline-flex items-center">
                <TerminalCursor visible={showCursor} />
              </div>
            )}
          </div>
        </div>
      </div>
    </section>
  );
}

/* -------------------------------------------------------------------------- */
/*  CTA Section                                                               */
/* -------------------------------------------------------------------------- */

function CtaSection() {
  const { t } = useTranslation();
  const glowRef = useRef<HTMLDivElement>(null);
  const buttonGlowRef = useRef<HTMLDivElement>(null);
  const ctaContentRef = useRef<HTMLDivElement>(null);
  const buttonShineRef = useRef<HTMLDivElement>(null);

  /* Animate gradient glow with anime.js */
  useEffect(() => {
    if (prefersReducedMotion()) return;

    let glowAnim: ReturnType<typeof animate> | null = null;
    let buttonGlowAnim: ReturnType<typeof animate> | null = null;

    if (glowRef.current) {
      glowAnim = animate(glowRef.current, {
        opacity: [0.3, 0.8],
        scale: [0.9, 1.1],
        loop: true,
        alternate: true,
        duration: 4000,
        ease: 'inOutSine',
      });
    }

    if (buttonGlowRef.current) {
      buttonGlowAnim = animate(buttonGlowRef.current, {
        opacity: [0.4, 0.8],
        scale: [0.95, 1.08],
        loop: true,
        alternate: true,
        duration: 3000,
        ease: 'inOutSine',
      });
    }

    return () => {
      glowAnim?.pause();
      buttonGlowAnim?.pause();
    };
  }, []);

  /* Button shimmer/shine sweep — periodic animation */
  useEffect(() => {
    if (!buttonShineRef.current || prefersReducedMotion()) return;

    const anim = animate(buttonShineRef.current, {
      translateX: ['-100%', '300%'],
      duration: 1500,
      loop: true,
      delay: 3000,
      ease: 'inOutSine',
    });

    return () => {
      anim.pause();
    };
  }, []);

  /* CTA content entrance with subtle perspective tilt */
  useEffect(() => {
    if (!ctaContentRef.current || prefersReducedMotion()) return;

    const el = ctaContentRef.current;

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          animate(el, {
            opacity: [0, 1],
            translateY: [50, 0],
            rotateY: [3, 0],
            duration: 800,
            ease: 'outQuart',
          });
          observer.disconnect();
        }
      },
      { threshold: 0.2 },
    );

    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  return (
    <section id="get-started" className="py-20 md:py-32 bg-zinc-900 dark:bg-zinc-950 text-white relative overflow-hidden">
      {/* Glow */}
      <div className="absolute inset-0 pointer-events-none" aria-hidden="true">
        <div
          ref={glowRef}
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[300px] bg-gradient-to-r from-indigo-500/30 via-purple-500/20 to-pink-500/30 blur-[120px] rounded-full"
        />
      </div>

      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 relative" style={{ perspective: '1000px' }}>
        <div
          ref={ctaContentRef}
          className="text-center"
          style={{ opacity: 0, transformStyle: 'preserve-3d' }}
        >
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-display font-bold tracking-tight">
            {t('cta.title')}
          </h2>
          <p className="mt-4 text-lg text-zinc-400 max-w-xl mx-auto">
            {t('cta.subtitle')}
          </p>
          <div className="mt-10 relative inline-block">
            {/* Button glow */}
            <div
              ref={buttonGlowRef}
              className="absolute -inset-1 bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500 rounded-full blur-lg opacity-60"
              aria-hidden="true"
            />
            <a
              href="https://github.com/Draivix/aigiscode"
              target="_blank"
              rel="noopener noreferrer"
              className="relative inline-flex items-center gap-2 px-8 py-4 rounded-full bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500 text-white font-semibold text-sm hover:scale-105 active:scale-[0.98] transition-transform shadow-xl overflow-hidden"
            >
              {/* Shimmer/shine sweep */}
              <div
                ref={buttonShineRef}
                className="absolute inset-0 pointer-events-none"
                aria-hidden="true"
                style={{
                  background: 'linear-gradient(90deg, transparent 0%, rgba(255,255,255,0.3) 50%, transparent 100%)',
                  width: '40%',
                }}
              />
              <span className="relative z-10 flex items-center gap-2">
                {t('cta.button')}
                <ArrowRight size={16} weight="bold" />
              </span>
            </a>
          </div>
        </div>
      </div>
    </section>
  );
}

/* -------------------------------------------------------------------------- */
/*  HomePage                                                                  */
/* -------------------------------------------------------------------------- */

export default function HomePage() {
  const homeJsonLd = buildGraphJsonLd([
    organizationEntity(),
    founderEntity(),
    webPageEntity(
      SITE_URL,
      'AigisCode - AI-Powered Code Guardian | Static Analysis for AI Agents',
      'AI-powered static analysis that watches your entire codebase for dead code, circular dependencies, hardwired values, and architectural issues. Open source.',
    ),
  ]);

  return (
    <>
      <SEO
        title="AigisCode - AI-Powered Code Guardian | Static Analysis for AI Agents"
        description="AI-powered static analysis that watches your entire codebase for dead code, circular dependencies, hardwired values, and architectural issues. Open source, MIT-licensed."
        canonical={SITE_URL}
        jsonLd={homeJsonLd}
      />
      <HeroSection />
      <TrustedBar />
      <FeaturesSection />
      <PipelineSection />
      <TerminalDemo />
      <CtaSection />
    </>
  );
}
