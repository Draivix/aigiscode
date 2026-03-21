import { useRef, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { motion, useInView } from 'motion/react';
import { animate, stagger } from 'animejs';
import { prefersReducedMotion } from '@/hooks/useAnime';
import {
  Buildings,
  Briefcase,
  CalendarBlank,
  Copy,
  Check,
  Envelope,
  GithubLogo,
  Globe,
  Handshake,
  Heart,
  Lightning,
  LinkedinLogo,
  MapPin,
  Newspaper,
  Rocket,
  ShieldCheck,
  Sparkle,
  Star,
  TrendUp,
  Trophy,
  UsersThree,
  XLogo,
} from '@phosphor-icons/react';
import SEO from '@/components/SEO';
import Breadcrumbs from '@/components/Breadcrumbs';
import { useState } from 'react';
import {
  SITE_URL,
  buildGraphJsonLd,
  organizationEntity,
  founderEntity,
  webPageEntity,
} from '@/content/seo-schema';

/* -------------------------------------------------------------------------- */
/*  Constants                                                                 */
/* -------------------------------------------------------------------------- */

const STATS = [
  { value: 6, label: 'Languages Supported', suffix: '' },
  { value: 8, label: 'Detector Types', suffix: '' },
  { value: 3, label: 'AI Backends', suffix: '' },
  { value: 100, label: 'Open Source', suffix: '%' },
] as const;

const TECHNOLOGY_PARTNERS = [
  { name: 'tree-sitter', desc: 'Multi-language parser' },
  { name: 'petgraph', desc: 'Graph analysis' },
  { name: 'Serde', desc: 'Typed serialization' },
  { name: 'thiserror', desc: 'Error contracts' },
  { name: 'walkdir', desc: 'Repository traversal' },
  { name: 'Vite', desc: 'Website build pipeline' },
  { name: 'React', desc: 'Website UI' },
  { name: 'Sigma', desc: 'Graph atlas rendering' },
  { name: 'Cargo', desc: 'Native Rust packaging' },
] as const;

const WHY_JOIN_US = [
  {
    icon: Rocket,
    titleKey: 'about.careers.benefit1Title',
    titleFallback: 'Build Cutting-Edge Tools',
    descKey: 'about.careers.benefit1Desc',
    descFallback: 'Work on AI-powered static analysis that pushes the boundaries of what developer tools can do.',
  },
  {
    icon: Globe,
    titleKey: 'about.careers.benefit2Title',
    titleFallback: 'Fully Remote',
    descKey: 'about.careers.benefit2Desc',
    descFallback: 'Work from anywhere in the world. We are a distributed team that values output over office hours.',
  },
  {
    icon: Heart,
    titleKey: 'about.careers.benefit3Title',
    titleFallback: 'Open Source First',
    descKey: 'about.careers.benefit3Desc',
    descFallback: 'Your work ships as open source. Contribute to the developer community while building your career.',
  },
  {
    icon: Lightning,
    titleKey: 'about.careers.benefit4Title',
    titleFallback: 'Move Fast',
    descKey: 'about.careers.benefit4Desc',
    descFallback: 'Small team, big impact. Ship features weekly, not quarterly. Your ideas go from concept to production fast.',
  },
] as const;

const UPCOMING_EVENTS = [
  {
    title: 'AigisCode v1.0 Launch Webinar',
    date: '2026 Q2',
    location: 'Online',
    desc: 'Live demo of the full analysis pipeline with Q&A session.',
  },
  {
    title: 'AI Developer Tools Summit',
    date: '2026 Q3',
    location: 'Berlin, Germany',
    desc: 'Presenting AI-assisted code analysis and the future of developer tooling.',
  },
  {
    title: 'Open Source Contributors Meetup',
    date: '2026 Q3',
    location: 'Online',
    desc: 'Community meetup for AigisCode contributors and plugin developers.',
  },
] as const;

/* -------------------------------------------------------------------------- */
/*  Section heading helper                                                     */
/* -------------------------------------------------------------------------- */

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
/*  Animated counter component                                                 */
/* -------------------------------------------------------------------------- */

function AnimatedStat({
  value,
  label,
  suffix,
  shouldStart,
}: {
  value: number;
  label: string;
  suffix: string;
  shouldStart: boolean;
}) {
  const numRef = useRef<HTMLSpanElement>(null);
  const hasAnimated = useRef(false);

  useEffect(() => {
    if (!shouldStart || hasAnimated.current || !numRef.current) return;
    hasAnimated.current = true;

    if (prefersReducedMotion()) {
      numRef.current.textContent = `${value}${suffix}`;
      return;
    }

    const obj = { val: 0 };
    animate(obj, {
      val: value,
      duration: 1500,
      ease: 'outExpo',
      onUpdate: () => {
        if (numRef.current) {
          numRef.current.textContent = `${Math.round(obj.val)}${suffix}`;
        }
      },
    });
  }, [shouldStart, value, suffix]);

  return (
    <div className="text-center p-6 rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl">
      <span
        ref={numRef}
        className="block text-4xl md:text-5xl font-display font-bold tracking-tight bg-clip-text text-transparent bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500"
      >
        0{suffix}
      </span>
      <span className="mt-2 block text-sm text-zinc-600 dark:text-zinc-400 font-medium">
        {label}
      </span>
    </div>
  );
}

/* -------------------------------------------------------------------------- */
/*  Hero Section                                                               */
/* -------------------------------------------------------------------------- */

function HeroSection() {
  const { t } = useTranslation();
  const titleRef = useRef<HTMLHeadingElement>(null);
  const subtitleRef = useRef<HTMLParagraphElement>(null);

  useEffect(() => {
    if (!titleRef.current) return;

    const h1 = titleRef.current;

    if (prefersReducedMotion()) {
      h1.style.opacity = '1';
      h1.querySelectorAll('[data-about-word]').forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      const gradientSpan = h1.querySelector('[data-about-gradient]') as HTMLElement | null;
      if (gradientSpan) gradientSpan.style.opacity = '1';
      return;
    }

    h1.style.opacity = '1';

    const words = h1.querySelectorAll('[data-about-word]');
    const gradientSpan = h1.querySelector('[data-about-gradient]');

    const wordAnim = animate(words, {
      opacity: [0, 1],
      translateY: [30, 0],
      scale: [0.9, 1],
      duration: 600,
      delay: stagger(80),
      ease: 'outQuart',
    });

    let gradientAnim: ReturnType<typeof animate> | null = null;
    if (gradientSpan) {
      gradientAnim = animate(gradientSpan, {
        opacity: [0, 1],
        translateY: [30, 0],
        scale: [0.85, 1.05, 1],
        duration: 700,
        delay: words.length * 80 + 100,
        ease: 'outQuart',
      });
    }

    return () => {
      wordAnim.pause();
      gradientAnim?.pause();
    };
  }, [t]);

  useEffect(() => {
    if (!subtitleRef.current) return;

    if (prefersReducedMotion()) {
      subtitleRef.current.style.opacity = '1';
      return;
    }

    const anim = animate(subtitleRef.current, {
      opacity: [0, 1],
      translateY: [25, 0],
      duration: 700,
      delay: 400,
      ease: 'outQuart',
    });

    return () => {
      anim.pause();
    };
  }, []);

  const title1 = t('about.hero.title1', 'About');
  const title1Words = title1.split(/\s+/);

  return (
    <section className="relative py-16 md:py-24 overflow-hidden">
      {/* Background glow */}
      <div className="absolute inset-0 pointer-events-none" aria-hidden="true">
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[300px] bg-gradient-to-r from-indigo-500/15 via-purple-500/10 to-pink-500/15 blur-[120px] rounded-full" />
      </div>

      <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 text-center relative z-10">
        {/* Breadcrumbs */}
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4 }}
          className="mb-8"
        >
          <Breadcrumbs items={[{ label: t('about.breadcrumb', 'About') }]} />
        </motion.div>

        <h1
          ref={titleRef}
          className="text-4xl md:text-5xl lg:text-6xl font-display font-bold tracking-tighter leading-[1.05]"
          style={{ opacity: 0 }}
        >
          {title1Words.map((word, i) => (
            <span
              key={i}
              data-about-word
              className="inline-block mr-[0.3em]"
              style={{ opacity: 0 }}
            >
              {word}
            </span>
          ))}
          <span
            data-about-gradient
            className="inline-block bg-clip-text text-transparent bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500"
            style={{ opacity: 0 }}
          >
            {t('about.hero.title2', 'Draivix')}
          </span>
        </h1>

        <p
          ref={subtitleRef}
          className="mt-6 text-lg md:text-xl text-zinc-600 dark:text-zinc-400 max-w-2xl mx-auto leading-relaxed"
          style={{ opacity: 0 }}
        >
          {t(
            'about.hero.subtitle',
            'Building AI-powered developer tools that make every engineering team more productive. From code analysis to architectural insights, we guard what matters most.'
          )}
        </p>

        {/* Table of Contents */}
        <motion.nav
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.15 }}
          className="mt-10 p-5 rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl inline-block text-left"
          aria-label="Table of contents"
        >
          <h2 className="text-sm font-semibold text-zinc-900 dark:text-white mb-3 uppercase tracking-wider">
            {t('common.tableOfContents', 'On this page')}
          </h2>
          <ul className="space-y-2 text-sm columns-1 sm:columns-2 gap-x-8">
            {[
              { id: 'company', label: t('about.toc.company', 'Company') },
              { id: 'leadership', label: t('about.toc.leadership', 'Leadership') },
              { id: 'vision', label: t('about.toc.vision', 'Vision & Mission') },
              { id: 'partners', label: t('about.toc.partners', 'Partners') },
              { id: 'careers', label: t('about.toc.careers', 'Careers') },
              { id: 'events', label: t('about.toc.events', 'Events') },
              { id: 'stories', label: t('about.toc.stories', 'Customer Stories') },
              { id: 'press', label: t('about.toc.press', 'Press & Media') },
              { id: 'contact', label: t('about.toc.contact', 'Contact') },
            ].map((item) => (
              <li key={item.id} className="break-inside-avoid">
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
  );
}

/* -------------------------------------------------------------------------- */
/*  Company Section                                                            */
/* -------------------------------------------------------------------------- */

function CompanySection() {
  const { t } = useTranslation();

  return (
    <motion.section
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: '-80px' }}
      transition={{ duration: 0.5 }}
      aria-labelledby="company"
    >
      <SectionHeading id="company" icon={Buildings}>
        {t('about.company.title', 'The Company')}
      </SectionHeading>

      <div className="space-y-5">
        <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">
          {t(
            'about.company.p1',
            'Draivix Corporation is an AI-focused software company building developer tools that analyze, understand, and guard codebases at scale. Our flagship product, AigisCode, brings AI-powered static analysis to engineering teams of every size.'
          )}
        </p>
        <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">
          {t(
            'about.company.p2',
            'Founded by David Strejc, Draivix is driven by a simple belief: the structural health of a codebase should be visible, measurable, and continuously guarded. We combine battle-tested parsers like tree-sitter with modern AI backends to detect issues that single-file linters miss entirely.'
          )}
        </p>

        {/* Product cards */}
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 mt-6">
          <div className="rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-6 group hover:border-indigo-500/20 dark:hover:border-indigo-500/20 transition-colors">
            <div className="mb-3 inline-flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500/10 to-purple-500/10 dark:from-indigo-500/20 dark:to-purple-500/20 text-indigo-500">
              <ShieldCheck size={24} weight="duotone" />
            </div>
            <h3 className="font-display font-semibold text-lg text-zinc-900 dark:text-white mb-2">
              AigisCode
            </h3>
            <p className="text-sm text-zinc-600 dark:text-zinc-400 leading-relaxed">
              {t(
                'about.company.aigisDesc',
                'AI-powered codebase evaluator. Six-stage pipeline covering circular dependencies, dead code, hardwired values, god classes, bottleneck files, and orphan modules. Open source under MIT.'
              )}
            </p>
          </div>

          <div className="rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-6 group hover:border-indigo-500/20 dark:hover:border-indigo-500/20 transition-colors">
            <div className="mb-3 inline-flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500/10 to-purple-500/10 dark:from-indigo-500/20 dark:to-purple-500/20 text-indigo-500">
              <Sparkle size={24} weight="duotone" />
            </div>
            <h3 className="font-display font-semibold text-lg text-zinc-900 dark:text-white mb-2">
              {t('about.company.platformName', 'Draivix Platform')}
            </h3>
            <p className="text-sm text-zinc-600 dark:text-zinc-400 leading-relaxed">
              {t(
                'about.company.platformDesc',
                'The broader Draivix ecosystem of AI-powered developer tools, infrastructure automation, and intelligent workflows designed for modern engineering teams.'
              )}
            </p>
          </div>
        </div>
      </div>
    </motion.section>
  );
}

/* -------------------------------------------------------------------------- */
/*  Leadership Section                                                         */
/* -------------------------------------------------------------------------- */

function LeadershipSection() {
  const { t } = useTranslation();
  const cardRef = useRef<HTMLDivElement>(null);
  const hasAnimated = useRef(false);

  useEffect(() => {
    if (!cardRef.current) return;

    if (prefersReducedMotion()) {
      const cards = cardRef.current.querySelectorAll('[data-leader-card]');
      cards.forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !hasAnimated.current) {
          hasAnimated.current = true;
          const cards = cardRef.current!.querySelectorAll('[data-leader-card]');
          animate(cards, {
            opacity: [0, 1],
            translateY: [30, 0],
            scale: [0.95, 1],
            duration: 600,
            delay: stagger(120),
            ease: 'outQuart',
          });
          observer.disconnect();
        }
      },
      { threshold: 0.1, rootMargin: '-60px' },
    );

    observer.observe(cardRef.current);
    return () => observer.disconnect();
  }, []);

  return (
    <motion.section
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: '-80px' }}
      transition={{ duration: 0.5 }}
      aria-labelledby="leadership"
    >
      <SectionHeading id="leadership" icon={UsersThree}>
        {t('about.leadership.title', 'Leadership')}
      </SectionHeading>

      <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mb-8">
        {t(
          'about.leadership.intro',
          'Draivix is led by engineers and researchers who believe AI should augment developer capabilities, not replace them.'
        )}
      </p>

      <div ref={cardRef} className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Founder card */}
        <div
          data-leader-card
          className="rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-6 md:col-span-2"
          style={{ opacity: 0 }}
        >
          <div className="flex flex-col sm:flex-row items-start gap-6">
            {/* Avatar placeholder */}
            <div className="w-20 h-20 rounded-2xl bg-gradient-to-br from-indigo-500 via-purple-500 to-pink-500 flex items-center justify-center text-white font-display font-bold text-2xl flex-shrink-0 shadow-lg shadow-indigo-500/25">
              DS
            </div>
            <div>
              <h3 className="text-xl font-display font-bold text-zinc-900 dark:text-white">
                David Strejc
              </h3>
              <p className="text-sm text-indigo-500 font-medium mb-3">
                {t('about.leadership.founderRole', 'Founder & CEO')}
              </p>
              <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">
                {t(
                  'about.leadership.founderBio',
                  'AI researcher and full-stack developer with deep expertise in static analysis, graph theory, and developer tooling. David founded Draivix to bring AI-powered code analysis to every engineering team, combining tree-sitter parsing with modern AI backends to catch the structural issues that single-file linters miss.'
                )}
              </p>
              <div className="mt-4 flex items-center gap-3">
                <a
                  href="https://github.com/david-strejc"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1.5 text-sm text-zinc-500 hover:text-zinc-900 dark:hover:text-white transition-colors"
                  aria-label="David Strejc on GitHub"
                >
                  <GithubLogo size={16} weight="bold" />
                  <span>GitHub</span>
                </a>
                <a
                  href="https://linkedin.com/in/david-strejc"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1.5 text-sm text-zinc-500 hover:text-zinc-900 dark:hover:text-white transition-colors"
                  aria-label="David Strejc on LinkedIn"
                >
                  <LinkedinLogo size={16} weight="bold" />
                  <span>LinkedIn</span>
                </a>
              </div>
            </div>
          </div>
        </div>
      </div>
    </motion.section>
  );
}

/* -------------------------------------------------------------------------- */
/*  Vision / Mission Section with Animated Stats                               */
/* -------------------------------------------------------------------------- */

function VisionSection() {
  const { t } = useTranslation();
  const statsRef = useRef<HTMLDivElement>(null);
  const isInView = useInView(statsRef, { once: true, margin: '-80px' });

  return (
    <motion.section
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: '-80px' }}
      transition={{ duration: 0.5 }}
      aria-labelledby="vision"
    >
      <SectionHeading id="vision" icon={Star}>
        {t('about.vision.title', 'Vision & Mission')}
      </SectionHeading>

      <div className="space-y-5 mb-10">
        <div className="rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-6">
          <h3 className="text-lg font-display font-semibold text-zinc-900 dark:text-white mb-2">
            {t('about.vision.missionLabel', 'Our Mission')}
          </h3>
          <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">
            {t(
              'about.vision.mission',
              'Make AI-powered developer tools accessible to every engineering team. Code quality should not be a luxury reserved for companies that can afford dedicated architecture review boards.'
            )}
          </p>
        </div>

        <div className="rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-6">
          <h3 className="text-lg font-display font-semibold text-zinc-900 dark:text-white mb-2">
            {t('about.vision.visionLabel', 'Our Vision')}
          </h3>
          <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">
            {t(
              'about.vision.vision',
              'A world where every codebase has a guardian. Where circular dependencies, dead code, and architectural drift are caught before they become technical debt. Where AI and deterministic analysis work together to make software better.'
            )}
          </p>
        </div>
      </div>

      {/* Animated Stats Bento Grid */}
      <div ref={statsRef} className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        {STATS.map((stat) => (
          <AnimatedStat
            key={stat.label}
            value={stat.value}
            label={stat.label}
            suffix={stat.suffix}
            shouldStart={isInView}
          />
        ))}
      </div>
    </motion.section>
  );
}

/* -------------------------------------------------------------------------- */
/*  Partners Section                                                           */
/* -------------------------------------------------------------------------- */

function PartnersSection() {
  const { t } = useTranslation();
  const gridRef = useRef<HTMLDivElement>(null);
  const hasAnimated = useRef(false);

  useEffect(() => {
    if (!gridRef.current) return;

    const badges = gridRef.current.querySelectorAll('[data-partner-badge]');

    if (prefersReducedMotion()) {
      badges.forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !hasAnimated.current) {
          hasAnimated.current = true;
          animate(badges, {
            opacity: [0, 1],
            scale: [0.8, 1],
            translateY: [15, 0],
            duration: 400,
            delay: stagger(60, { from: 'center' }),
            ease: 'outBack',
          });
          observer.disconnect();
        }
      },
      { threshold: 0.1, rootMargin: '-50px' },
    );

    observer.observe(gridRef.current);
    return () => observer.disconnect();
  }, []);

  return (
    <motion.section
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: '-80px' }}
      transition={{ duration: 0.5 }}
      aria-labelledby="partners"
    >
      <SectionHeading id="partners" icon={Handshake}>
        {t('about.partners.title', 'Technology Partners')}
      </SectionHeading>

      <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mb-8">
        {t(
          'about.partners.intro',
          'AigisCode is built on top of world-class open source technologies. We integrate battle-tested tools rather than reinventing the wheel, so our analysis is reliable, fast, and accurate.'
        )}
      </p>

      <div ref={gridRef} className="grid grid-cols-2 sm:grid-cols-4 gap-4">
        {TECHNOLOGY_PARTNERS.map((partner) => (
          <div
            key={partner.name}
            data-partner-badge
            className="rounded-xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-4 text-center group hover:border-indigo-500/20 dark:hover:border-indigo-500/20 transition-colors"
            style={{ opacity: 0 }}
          >
            <span className="block font-mono text-sm font-semibold text-zinc-900 dark:text-white group-hover:text-indigo-500 transition-colors">
              {partner.name}
            </span>
            <span className="block text-xs text-zinc-500 dark:text-zinc-500 mt-1">
              {partner.desc}
            </span>
          </div>
        ))}
      </div>
    </motion.section>
  );
}

/* -------------------------------------------------------------------------- */
/*  Careers Section                                                            */
/* -------------------------------------------------------------------------- */

function CareersSection() {
  const { t } = useTranslation();
  const gridRef = useRef<HTMLDivElement>(null);
  const hasAnimated = useRef(false);

  useEffect(() => {
    if (!gridRef.current) return;

    const cards = gridRef.current.querySelectorAll('[data-benefit-card]');

    if (prefersReducedMotion()) {
      cards.forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !hasAnimated.current) {
          hasAnimated.current = true;
          animate(cards, {
            opacity: [0, 1],
            scale: [0.85, 1],
            translateY: [40, 0],
            duration: 600,
            delay: stagger(80, { grid: [2, 2], from: 'first' }),
            ease: 'outQuart',
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
    <motion.section
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: '-80px' }}
      transition={{ duration: 0.5 }}
      aria-labelledby="careers"
    >
      <SectionHeading id="careers" icon={Briefcase}>
        {t('about.careers.title', 'Join Us')}
      </SectionHeading>

      <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mb-8">
        {t(
          'about.careers.intro',
          'We are always looking for passionate developers, researchers, and open source contributors who want to push the boundaries of AI-powered developer tools. Whether you want to contribute to AigisCode or join the Draivix team full-time, there is a place for you.'
        )}
      </p>

      {/* Benefits grid */}
      <div ref={gridRef} className="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-8">
        {WHY_JOIN_US.map(({ icon: Icon, titleKey, titleFallback, descKey, descFallback }) => (
          <div
            key={titleKey}
            data-benefit-card
            className="group relative rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-6 overflow-hidden hover:border-indigo-500/20 dark:hover:border-indigo-500/20 transition-colors"
            style={{ opacity: 0 }}
          >
            <div className="absolute -inset-1 rounded-2xl bg-gradient-to-br from-indigo-500/20 via-purple-500/20 to-pink-500/20 opacity-0 group-hover:opacity-100 blur-xl transition-opacity duration-500 pointer-events-none" />
            <div className="relative">
              <div className="mb-3 inline-flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500/10 to-purple-500/10 dark:from-indigo-500/20 dark:to-purple-500/20 text-indigo-500">
                <Icon size={22} weight="duotone" />
              </div>
              <h3 className="font-display font-semibold text-lg text-zinc-900 dark:text-white mb-2">
                {t(titleKey, titleFallback)}
              </h3>
              <p className="text-sm text-zinc-600 dark:text-zinc-400 leading-relaxed">
                {t(descKey, descFallback)}
              </p>
            </div>
          </div>
        ))}
      </div>

      {/* Open Source CTA */}
      <div className="rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-6 text-center">
        <h3 className="text-lg font-display font-semibold text-zinc-900 dark:text-white mb-2">
          {t('about.careers.ossTitle', 'Contribute to AigisCode')}
        </h3>
        <p className="text-sm text-zinc-600 dark:text-zinc-400 leading-relaxed mb-4 max-w-lg mx-auto">
          {t(
            'about.careers.ossDesc',
            'AigisCode is open source under the MIT License. Contribute detectors, language support, plugins, or documentation. Every pull request makes the tool better for the entire community.'
          )}
        </p>
        <a
          href="https://github.com/Draivix/aigiscode"
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-2 px-6 py-3 rounded-full bg-zinc-900 dark:bg-white text-white dark:text-black text-sm font-semibold hover:scale-105 active:scale-[0.98] transition-transform"
        >
          <GithubLogo size={18} weight="bold" />
          {t('about.careers.ossButton', 'View on GitHub')}
        </a>
      </div>
    </motion.section>
  );
}

/* -------------------------------------------------------------------------- */
/*  Events Section                                                             */
/* -------------------------------------------------------------------------- */

function EventsSection() {
  const { t } = useTranslation();

  return (
    <motion.section
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: '-80px' }}
      transition={{ duration: 0.5 }}
      aria-labelledby="events"
    >
      <SectionHeading id="events" icon={CalendarBlank}>
        {t('about.events.title', 'Events')}
      </SectionHeading>

      <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mb-8">
        {t(
          'about.events.intro',
          'Join us at upcoming conferences, webinars, and community meetups where we demonstrate AigisCode and discuss the future of AI-powered developer tooling.'
        )}
      </p>

      <div className="space-y-4">
        {UPCOMING_EVENTS.map((event) => (
          <div
            key={event.title}
            className="rounded-xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-5 flex flex-col sm:flex-row sm:items-center gap-4 hover:border-indigo-500/20 dark:hover:border-indigo-500/20 transition-colors"
          >
            <div className="flex-shrink-0">
              <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 text-xs font-medium">
                <CalendarBlank size={12} weight="bold" />
                {event.date}
              </span>
            </div>
            <div className="flex-1 min-w-0">
              <h3 className="font-display font-semibold text-zinc-900 dark:text-white">
                {event.title}
              </h3>
              <p className="text-sm text-zinc-500 dark:text-zinc-500 mt-0.5">{event.desc}</p>
            </div>
            <div className="flex items-center gap-1.5 text-sm text-zinc-500 flex-shrink-0">
              <MapPin size={14} weight="bold" />
              <span>{event.location}</span>
            </div>
          </div>
        ))}
      </div>
    </motion.section>
  );
}

/* -------------------------------------------------------------------------- */
/*  Customer Stories Section                                                    */
/* -------------------------------------------------------------------------- */

function StoriesSection() {
  const { t } = useTranslation();
  const gridRef = useRef<HTMLDivElement>(null);
  const hasAnimated = useRef(false);

  useEffect(() => {
    if (!gridRef.current) return;

    const cards = gridRef.current.querySelectorAll('[data-story-card]');

    if (prefersReducedMotion()) {
      cards.forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !hasAnimated.current) {
          hasAnimated.current = true;
          animate(cards, {
            opacity: [0, 1],
            translateY: [25, 0],
            duration: 500,
            delay: stagger(100),
            ease: 'outQuart',
          });
          observer.disconnect();
        }
      },
      { threshold: 0.1, rootMargin: '-60px' },
    );

    observer.observe(gridRef.current);
    return () => observer.disconnect();
  }, []);

  const stories = [
    {
      quote: t(
        'about.stories.quote1',
        'AigisCode found 47 circular dependencies in our Laravel monolith that we had been ignoring for years. The AI review stage reclassified 8 findings as false positives, saving us hours of manual triage.'
      ),
      author: t('about.stories.author1', 'Senior Engineering Lead'),
      company: t('about.stories.company1', 'SaaS Platform'),
    },
    {
      quote: t(
        'about.stories.quote2',
        'We integrated AigisCode into our CI pipeline and immediately caught 23 dead imports in our TypeScript monorepo. The policy engine let us suppress framework-specific patterns without losing detection coverage.'
      ),
      author: t('about.stories.author2', 'Staff Engineer'),
      company: t('about.stories.company2', 'FinTech Startup'),
    },
    {
      quote: t(
        'about.stories.quote3',
        'The hardwiring detector caught environment variable access outside our config layer. We externalized 15 hardcoded values in a single afternoon, improving our deployment flexibility across environments.'
      ),
      author: t('about.stories.author3', 'DevOps Lead'),
      company: t('about.stories.company3', 'Enterprise Software'),
    },
  ];

  return (
    <motion.section
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: '-80px' }}
      transition={{ duration: 0.5 }}
      aria-labelledby="stories"
    >
      <SectionHeading id="stories" icon={Trophy}>
        {t('about.stories.title', 'Customer Stories')}
      </SectionHeading>

      <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mb-8">
        {t(
          'about.stories.intro',
          'Engineering teams around the world use AigisCode to find and fix structural issues that other tools miss. Here are some of their experiences.'
        )}
      </p>

      <div ref={gridRef} className="grid grid-cols-1 md:grid-cols-3 gap-5">
        {stories.map((story, i) => (
          <div
            key={i}
            data-story-card
            className="rounded-2xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-6 flex flex-col"
            style={{ opacity: 0 }}
          >
            {/* Quotation mark */}
            <span className="text-4xl font-display font-bold text-indigo-500/30 leading-none mb-2" aria-hidden="true">
              &ldquo;
            </span>
            <p className="text-sm text-zinc-600 dark:text-zinc-400 leading-relaxed flex-1">
              {story.quote}
            </p>
            <div className="mt-4 pt-4 border-t border-zinc-200/30 dark:border-white/5">
              <p className="text-sm font-medium text-zinc-900 dark:text-white">
                {story.author}
              </p>
              <p className="text-xs text-zinc-500">{story.company}</p>
            </div>
          </div>
        ))}
      </div>
    </motion.section>
  );
}

/* -------------------------------------------------------------------------- */
/*  Press Section                                                              */
/* -------------------------------------------------------------------------- */

function PressSection() {
  const { t } = useTranslation();

  return (
    <motion.section
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: '-80px' }}
      transition={{ duration: 0.5 }}
      aria-labelledby="press"
    >
      <SectionHeading id="press" icon={Newspaper}>
        {t('about.press.title', 'Press & Media')}
      </SectionHeading>

      <div className="space-y-5">
        <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">
          {t(
            'about.press.intro',
            'For press inquiries, media resources, and company information, please reach out to our communications team. We are happy to provide product demos, executive interviews, and technical deep-dives.'
          )}
        </p>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div className="rounded-xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-5">
            <h3 className="font-display font-semibold text-zinc-900 dark:text-white mb-1">
              {t('about.press.inquiriesTitle', 'Press Inquiries')}
            </h3>
            <p className="text-sm text-zinc-500 dark:text-zinc-500 mb-3">
              {t(
                'about.press.inquiriesDesc',
                'For interviews, product reviews, and media coverage.'
              )}
            </p>
            <a
              href="mailto:press@aigiscode.com"
              className="text-sm text-indigo-500 hover:text-indigo-600 dark:hover:text-indigo-400 font-medium transition-colors"
            >
              press@aigiscode.com
            </a>
          </div>

          <div className="rounded-xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-5">
            <h3 className="font-display font-semibold text-zinc-900 dark:text-white mb-1">
              {t('about.press.resourcesTitle', 'Brand Resources')}
            </h3>
            <p className="text-sm text-zinc-500 dark:text-zinc-500 mb-3">
              {t(
                'about.press.resourcesDesc',
                'Logos, screenshots, and company fact sheet for media use.'
              )}
            </p>
            <span className="text-sm text-zinc-400 dark:text-zinc-600 font-medium">
              {t('about.press.comingSoon', 'Coming soon')}
            </span>
          </div>
        </div>

        {/* Investors sub-section */}
        <div id="investors" className="pt-4">
          <h3 className="text-lg font-display font-semibold text-zinc-900 dark:text-white mb-2 flex items-center gap-2">
            <TrendUp size={20} weight="duotone" className="text-indigo-500" />
            {t('about.investors.title', 'Investor Relations')}
          </h3>
          <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed">
            {t(
              'about.investors.desc',
              'Draivix Corporation is a privately held company. For investor inquiries and partnership opportunities, please contact us at investors@aigiscode.com.'
            )}
          </p>
        </div>
      </div>
    </motion.section>
  );
}

/* -------------------------------------------------------------------------- */
/*  Contact Section                                                            */
/* -------------------------------------------------------------------------- */

function ContactSection() {
  const { t } = useTranslation();
  const [copiedEmail, setCopiedEmail] = useState(false);

  const handleCopyEmail = async () => {
    try {
      await navigator.clipboard.writeText('hello@aigiscode.com');
      setCopiedEmail(true);
      setTimeout(() => setCopiedEmail(false), 2000);
    } catch {
      /* clipboard not available */
    }
  };

  return (
    <motion.section
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: '-80px' }}
      transition={{ duration: 0.5 }}
      aria-labelledby="contact"
    >
      <SectionHeading id="contact" icon={Envelope}>
        {t('about.contact.title', 'Contact Us')}
      </SectionHeading>

      <p className="text-zinc-600 dark:text-zinc-400 leading-relaxed mb-8">
        {t(
          'about.contact.intro',
          'Have questions about AigisCode, want to discuss a partnership, or need enterprise support? We would love to hear from you.'
        )}
      </p>

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        {/* Email */}
        <div className="rounded-xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-5 text-center">
          <div className="mb-3 inline-flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500/10 to-purple-500/10 dark:from-indigo-500/20 dark:to-purple-500/20 text-indigo-500">
            <Envelope size={22} weight="duotone" />
          </div>
          <h3 className="font-display font-semibold text-zinc-900 dark:text-white text-sm mb-1">
            {t('about.contact.emailLabel', 'Email')}
          </h3>
          <button
            onClick={handleCopyEmail}
            className="inline-flex items-center gap-1.5 text-sm text-indigo-500 hover:text-indigo-600 dark:hover:text-indigo-400 font-medium transition-colors"
            aria-label="Copy email address"
          >
            hello@aigiscode.com
            {copiedEmail ? (
              <Check size={14} className="text-emerald-400" weight="bold" />
            ) : (
              <Copy size={14} />
            )}
          </button>
        </div>

        {/* GitHub */}
        <a
          href="https://github.com/Draivix/aigiscode"
          target="_blank"
          rel="noopener noreferrer"
          className="rounded-xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-5 text-center hover:border-indigo-500/20 dark:hover:border-indigo-500/20 transition-colors block"
        >
          <div className="mb-3 inline-flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500/10 to-purple-500/10 dark:from-indigo-500/20 dark:to-purple-500/20 text-indigo-500">
            <GithubLogo size={22} weight="duotone" />
          </div>
          <h3 className="font-display font-semibold text-zinc-900 dark:text-white text-sm mb-1">
            GitHub
          </h3>
          <p className="text-sm text-zinc-500">Draivix/aigiscode</p>
        </a>

        {/* X / Twitter */}
        <a
          href="https://x.com/aigiscode"
          target="_blank"
          rel="noopener noreferrer"
          className="rounded-xl border border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-white/[0.02] backdrop-blur-xl p-5 text-center hover:border-indigo-500/20 dark:hover:border-indigo-500/20 transition-colors block"
        >
          <div className="mb-3 inline-flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500/10 to-purple-500/10 dark:from-indigo-500/20 dark:to-purple-500/20 text-indigo-500">
            <XLogo size={22} weight="duotone" />
          </div>
          <h3 className="font-display font-semibold text-zinc-900 dark:text-white text-sm mb-1">
            X
          </h3>
          <p className="text-sm text-zinc-500">@aigiscode</p>
        </a>
      </div>
    </motion.section>
  );
}

/* -------------------------------------------------------------------------- */
/*  CTA Section                                                                */
/* -------------------------------------------------------------------------- */

function CtaSection() {
  const { t } = useTranslation();
  const glowRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (prefersReducedMotion() || !glowRef.current) return;

    const anim = animate(glowRef.current, {
      opacity: [0.3, 0.8],
      scale: [0.9, 1.1],
      loop: true,
      alternate: true,
      duration: 4000,
      ease: 'inOutSine',
    });

    return () => {
      anim.pause();
    };
  }, []);

  return (
    <motion.section
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: '-80px' }}
      transition={{ duration: 0.5 }}
      className="rounded-2xl bg-zinc-900 dark:bg-zinc-950 border border-zinc-800 dark:border-zinc-800/60 p-8 md:p-12 text-center relative overflow-hidden"
    >
      {/* Glow */}
      <div className="absolute inset-0 pointer-events-none" aria-hidden="true">
        <div
          ref={glowRef}
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[400px] h-[200px] bg-gradient-to-r from-indigo-500/20 via-purple-500/15 to-pink-500/20 blur-[100px] rounded-full"
        />
      </div>

      <div className="relative z-10">
        <h2 className="text-2xl md:text-3xl font-display font-bold tracking-tight text-white">
          {t('about.cta.title', 'Start Guarding Your Code')}
        </h2>
        <p className="mt-3 text-zinc-400 max-w-md mx-auto leading-relaxed">
          {t(
            'about.cta.desc',
            'Install AigisCode in seconds and run your first analysis. Open source, MIT licensed, forever free.'
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
            {t('about.cta.github', 'View on GitHub')}
          </a>
          <div className="inline-flex items-center gap-3 px-5 py-3 bg-zinc-800/80 rounded-xl font-mono text-sm text-zinc-300 border border-zinc-700/50">
            <span className="text-zinc-500 select-none">$</span>
            <span className="select-all">cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</span>
          </div>
        </div>
      </div>
    </motion.section>
  );
}

/* -------------------------------------------------------------------------- */
/*  About Page                                                                 */
/* -------------------------------------------------------------------------- */

export default function AboutPage() {
  const { t } = useTranslation();

  const breadcrumbSchema = [
    { name: t('nav.home', 'Home'), url: `${SITE_URL}/` },
    { name: t('about.breadcrumb', 'About'), url: `${SITE_URL}/about` },
  ];

  const graphJsonLd = buildGraphJsonLd([
    webPageEntity(
      `${SITE_URL}/about`,
      'About Draivix - AigisCode',
      'Learn about Draivix Corporation, the AI-focused software company behind AigisCode. Meet our leadership team, explore partnerships, and discover career opportunities.',
    ),
    {
      ...organizationEntity(),
      description:
        'AI-focused software company building developer tools that analyze, understand, and guard codebases at scale.',
      contactPoint: {
        '@type': 'ContactPoint',
        email: 'hello@aigiscode.com',
        contactType: 'customer support',
      },
    },
    founderEntity(),
  ]);

  return (
    <>
      <SEO
        title={t('about.meta.title', 'About Draivix - AigisCode')}
        description={t(
          'about.meta.description',
          'Learn about Draivix Corporation, the AI-focused software company behind AigisCode. Meet our leadership team, explore partnerships, and discover career opportunities.'
        )}
        canonical={`${SITE_URL}/about`}
        breadcrumbs={breadcrumbSchema}
        jsonLd={graphJsonLd}
      />

      <HeroSection />

      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 pb-20 md:pb-32 space-y-20 md:space-y-28">
        <CompanySection />
        <LeadershipSection />
        <VisionSection />
        <PartnersSection />
        <CareersSection />
        <EventsSection />
        <StoriesSection />
        <PressSection />
        <ContactSection />
        <CtaSection />
      </div>
    </>
  );
}
