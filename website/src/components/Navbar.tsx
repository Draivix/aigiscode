import { useState, useRef, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import { supportedLanguages, languageNames, changeLanguageSafe, resolveLanguage } from '@/i18n';
import type { SupportedLanguage } from '@/i18n';
import { useSwitchLanguage, useLocalizedPath } from '@/lib/i18n-routing';
import { motion, AnimatePresence } from 'motion/react';
import { animate, stagger, createTimeline } from 'animejs';
import { createDrawable } from 'animejs/svg';
import { prefersReducedMotion } from '@/hooks/useAnime';
import {
  ArrowsClockwise,
  Brain,
  Bug,
  BookOpen,
  BookBookmark,
  Briefcase,
  CalendarBlank,
  CaretDown,
  ChartBarHorizontal,
  Code,
  Circuitry,
  ClipboardText,
  Crosshair,
  FileCode,
  FileMagnifyingGlass,
  FlowArrow,
  GithubLogo,
  Globe,
  GraduationCap,
  Handshake,
  IdentificationBadge,
  Info,
  Kanban,
  Lifebuoy,
  Lightbulb,
  Lightning,
  List,
  MagicWand,
  MapPin,
  Moon,
  Newspaper,
  Package,
  PlayCircle,
  Plugs,
  Question,
  Robot,
  Scan,
  ShieldCheck,
  ShieldStar,
  SquaresFour,
  Star,
  Sun,
  Terminal,
  TreeStructure,
  TrendUp,
  UsersThree,
  Warning,
  X,
} from '@phosphor-icons/react';

interface MegaMenuItem {
  icon: React.ComponentType<{ className?: string }>;
  title: string;
  desc: string;
  href?: string;
}

interface MegaMenuCategory {
  name: string;
  items: MegaMenuItem[];
}

const megaMenuData: MegaMenuCategory[] = [
  {
    name: 'menu.analysis.name',
    items: [
      { icon: ArrowsClockwise, title: 'menu.analysis.circular.title', desc: 'menu.analysis.circular.desc', href: '/features/circular-dependencies' },
      { icon: Bug, title: 'menu.analysis.deadCode.title', desc: 'menu.analysis.deadCode.desc', href: '/features/dead-code' },
      { icon: Lightning, title: 'menu.analysis.hardwiring.title', desc: 'menu.analysis.hardwiring.desc', href: '/features/hardwiring' },
      { icon: Brain, title: 'menu.analysis.aiReview.title', desc: 'menu.analysis.aiReview.desc', href: '/features/ai-review' },
      { icon: Warning, title: 'menu.analysis.godClasses.title', desc: 'menu.analysis.godClasses.desc', href: '/features/god-classes' },
      { icon: Crosshair, title: 'menu.analysis.bottlenecks.title', desc: 'menu.analysis.bottlenecks.desc', href: '/features/bottlenecks' },
      { icon: TreeStructure, title: 'menu.analysis.layers.title', desc: 'menu.analysis.layers.desc', href: '/features/layer-violations' },
      { icon: FileMagnifyingGlass, title: 'menu.analysis.orphans.title', desc: 'menu.analysis.orphans.desc', href: '/features/orphan-detection' },
    ],
  },
  {
    name: 'menu.useCases.name',
    items: [
      { icon: ArrowsClockwise, title: 'menu.useCases.circularDeps.title', desc: 'menu.useCases.circularDeps.desc', href: '/use-cases/circular-dependency-detection' },
      { icon: Bug, title: 'menu.useCases.deadCode.title', desc: 'menu.useCases.deadCode.desc', href: '/use-cases/dead-code-detection' },
      { icon: Lightning, title: 'menu.useCases.hardwiredValues.title', desc: 'menu.useCases.hardwiredValues.desc', href: '/use-cases/hardwired-values-detection' },
      { icon: Brain, title: 'menu.useCases.codeReview.title', desc: 'menu.useCases.codeReview.desc', href: '/use-cases/ai-assisted-code-review' },
      { icon: FileCode, title: 'menu.useCases.laravel.title', desc: 'menu.useCases.laravel.desc', href: '/use-cases/laravel-project-analysis' },
      { icon: Code, title: 'menu.useCases.django.title', desc: 'menu.useCases.django.desc', href: '/use-cases/django-project-analysis' },
      { icon: Globe, title: 'menu.useCases.wordpress.title', desc: 'menu.useCases.wordpress.desc', href: '/use-cases/wordpress-analysis' },
      { icon: SquaresFour, title: 'menu.useCases.typescript.title', desc: 'menu.useCases.typescript.desc', href: '/use-cases/typescript-monorepo-analysis' },
      { icon: FlowArrow, title: 'menu.useCases.cicd.title', desc: 'menu.useCases.cicd.desc', href: '/use-cases/ci-cd-integration' },
      { icon: Terminal, title: 'menu.useCases.python.title', desc: 'menu.useCases.python.desc', href: '/use-cases/python-codebase-analysis' },
    ],
  },
  {
    name: 'menu.languages.name',
    items: [
      { icon: FileCode, title: 'menu.languages.php.title', desc: 'menu.languages.php.desc', href: '/languages/php' },
      { icon: Terminal, title: 'menu.languages.python.title', desc: 'menu.languages.python.desc', href: '/languages/python' },
      { icon: Code, title: 'menu.languages.typescript.title', desc: 'menu.languages.typescript.desc', href: '/languages/typescript' },
      { icon: Lightning, title: 'menu.languages.javascript.title', desc: 'menu.languages.javascript.desc', href: '/languages/javascript' },
      { icon: SquaresFour, title: 'menu.languages.vue.title', desc: 'menu.languages.vue.desc', href: '/languages/vue' },
      { icon: Star, title: 'menu.languages.ruby.title', desc: 'menu.languages.ruby.desc', href: '/languages/ruby' },
    ],
  },
  {
    name: 'menu.integrations.name',
    items: [
      { icon: FlowArrow, title: 'menu.integrations.cicd.title', desc: 'menu.integrations.cicd.desc', href: '/integrations/ci-cd-pipelines' },
      { icon: GithubLogo, title: 'menu.integrations.githubActions.title', desc: 'menu.integrations.githubActions.desc', href: '/integrations/github-actions' },
      { icon: Circuitry, title: 'menu.integrations.gitlabCi.title', desc: 'menu.integrations.gitlabCi.desc', href: '/integrations/gitlab-ci' },
      { icon: Robot, title: 'menu.integrations.aiAgents.title', desc: 'menu.integrations.aiAgents.desc', href: '/integrations/ai-coding-agents' },
      { icon: Kanban, title: 'menu.integrations.idePlugins.title', desc: 'menu.integrations.idePlugins.desc', href: '/integrations/ide-plugins' },
      { icon: ShieldStar, title: 'menu.integrations.preCommit.title', desc: 'menu.integrations.preCommit.desc', href: '/integrations/pre-commit-hooks' },
      { icon: Plugs, title: 'menu.integrations.restApi.title', desc: 'menu.integrations.restApi.desc', href: '/integrations/rest-api' },
      { icon: MagicWand, title: 'menu.integrations.webhooks.title', desc: 'menu.integrations.webhooks.desc', href: '/integrations/webhooks' },
    ],
  },
  {
    name: 'menu.platform.name',
    items: [
      { icon: GithubLogo, title: 'menu.platform.openSource.title', desc: 'menu.platform.openSource.desc', href: '/platform/open-source' },
      { icon: Package, title: 'menu.platform.plugins.title', desc: 'menu.platform.plugins.desc', href: '/platform/plugin-system' },
      { icon: ClipboardText, title: 'menu.platform.policy.title', desc: 'menu.platform.policy.desc', href: '/platform/policy-engine' },
      { icon: ChartBarHorizontal, title: 'menu.platform.reporting.title', desc: 'menu.platform.reporting.desc', href: '/platform/report-generator' },
      { icon: TreeStructure, title: 'menu.platform.architecture.title', desc: 'menu.platform.architecture.desc', href: '/platform/architecture-analysis' },
      { icon: Terminal, title: 'menu.platform.cli.title', desc: 'menu.platform.cli.desc', href: '/platform/cli-tools' },
      { icon: Scan, title: 'menu.platform.detectors.title', desc: 'menu.platform.detectors.desc', href: '/platform/custom-detectors' },
      { icon: Plugs, title: 'menu.platform.extensionApi.title', desc: 'menu.platform.extensionApi.desc', href: '/platform/extension-api' },
    ],
  },
  {
    name: 'menu.about.name',
    items: [
      { icon: Info, title: 'menu.about.company.title', desc: 'menu.about.company.desc', href: '/about#company' },
      { icon: IdentificationBadge, title: 'menu.about.leadership.title', desc: 'menu.about.leadership.desc', href: '/about#leadership' },
      { icon: Briefcase, title: 'menu.about.careers.title', desc: 'menu.about.careers.desc', href: '/about#careers' },
      { icon: Newspaper, title: 'menu.about.press.title', desc: 'menu.about.press.desc', href: '/about#press' },
      { icon: TrendUp, title: 'menu.about.investors.title', desc: 'menu.about.investors.desc', href: '/about#investors' },
      { icon: Handshake, title: 'menu.about.partners.title', desc: 'menu.about.partners.desc', href: '/about#partners' },
      { icon: MapPin, title: 'menu.about.contact.title', desc: 'menu.about.contact.desc', href: '/about#contact' },
      { icon: Lightbulb, title: 'menu.about.blog.title', desc: 'menu.about.blog.desc', href: '/blog' },
      { icon: CalendarBlank, title: 'menu.about.events.title', desc: 'menu.about.events.desc', href: '/about#events' },
      { icon: UsersThree, title: 'menu.about.stories.title', desc: 'menu.about.stories.desc', href: '/about#stories' },
    ],
  },
  {
    name: 'menu.docs.name',
    items: [
      { icon: BookOpen, title: 'menu.docs.gettingStarted.title', desc: 'menu.docs.gettingStarted.desc', href: '/docs' },
      { icon: Code, title: 'menu.docs.apiReference.title', desc: 'menu.docs.apiReference.desc', href: '/docs#report-structure' },
      { icon: Terminal, title: 'menu.docs.cliReference.title', desc: 'menu.docs.cliReference.desc', href: '/docs#cli-commands' },
      { icon: Package, title: 'menu.docs.pluginDev.title', desc: 'menu.docs.pluginDev.desc', href: '/docs#configuration' },
      { icon: GraduationCap, title: 'menu.docs.tutorials.title', desc: 'menu.docs.tutorials.desc', href: '/docs#quick-start' },
      { icon: BookBookmark, title: 'menu.docs.knowledgeBase.title', desc: 'menu.docs.knowledgeBase.desc', href: '/docs#configuration' },
      { icon: PlayCircle, title: 'menu.docs.videoTutorials.title', desc: 'menu.docs.videoTutorials.desc', href: '/docs' },
      { icon: Lifebuoy, title: 'menu.docs.support.title', desc: 'menu.docs.support.desc', href: '/about#contact' },
      { icon: Question, title: 'menu.docs.faq.title', desc: 'menu.docs.faq.desc', href: '/docs' },
    ],
  },
];

/* -------------------------------------------------------------------------- */
/*  NavShieldLogo — small animated SVG shield for the navbar logo             */
/* -------------------------------------------------------------------------- */

function NavShieldLogo() {
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!svgRef.current || prefersReducedMotion()) return;

    const paths = svgRef.current.querySelectorAll<SVGGeometryElement>('.nav-shield-path');
    if (!paths.length) return;

    const drawables = createDrawable(paths);

    const tl = createTimeline({ autoplay: true });

    // Draw the outline
    tl.add(drawables, {
      draw: '0 1',
      duration: 1500,
      ease: 'inOutQuart',
    });

    // Fade in the fill glow
    tl.add('.nav-shield-fill', {
      opacity: [0, 0.15],
      duration: 800,
      ease: 'inOutSine',
    }, '-=300');

    // Breathing pulse after draw completes
    const breathe = animate('.nav-shield-fill', {
      opacity: [0.15, 0.06],
      duration: 3000,
      ease: 'inOutSine',
      loop: true,
      alternate: true,
      autoplay: false,
    });

    tl.then(() => { breathe.play(); });

    return () => {
      tl.pause();
      breathe.pause();
    };
  }, []);

  return (
    <svg
      ref={svgRef}
      viewBox="0 0 400 480"
      className="w-7 h-7"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <defs>
        <radialGradient id="nav-shield-glow" cx="50%" cy="40%" r="60%">
          <stop offset="0%" stopColor="#818cf8" stopOpacity="0.6" />
          <stop offset="40%" stopColor="#a78bfa" stopOpacity="0.3" />
          <stop offset="100%" stopColor="#6366f1" stopOpacity="0" />
        </radialGradient>
        <linearGradient id="nav-shield-stroke" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#6366f1" stopOpacity="0.8" />
          <stop offset="50%" stopColor="#a855f7" stopOpacity="0.7" />
          <stop offset="100%" stopColor="#ec4899" stopOpacity="0.6" />
        </linearGradient>
      </defs>
      <path
        className="nav-shield-fill"
        d="M200 20 L360 100 L360 260 C360 340 280 420 200 460 C120 420 40 340 40 260 L40 100 Z"
        fill="url(#nav-shield-glow)"
        opacity="0"
      />
      <path
        className="nav-shield-path"
        d="M200 20 L360 100 L360 260 C360 340 280 420 200 460 C120 420 40 340 40 260 L40 100 Z"
        stroke="url(#nav-shield-stroke)"
        strokeWidth="12"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
      />
      <path
        className="nav-shield-path"
        d="M200 80 L320 140 L320 255 C320 310 270 375 200 410 C130 375 80 310 80 255 L80 140 Z"
        stroke="url(#nav-shield-stroke)"
        strokeWidth="6"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
        opacity="0.5"
      />
      <path
        className="nav-shield-path"
        d="M200 150 L200 340"
        stroke="url(#nav-shield-stroke)"
        strokeWidth="5"
        opacity="0.4"
      />
      <path
        className="nav-shield-path"
        d="M120 230 L280 230"
        stroke="url(#nav-shield-stroke)"
        strokeWidth="5"
        opacity="0.4"
      />
    </svg>
  );
}

/* -------------------------------------------------------------------------- */

interface NavbarProps {
  isDark: boolean;
  toggleTheme: () => void;
}

export default function Navbar({ isDark, toggleTheme }: NavbarProps) {
  const { t, i18n } = useTranslation();
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const [expandedMobileMenu, setExpandedMobileMenu] = useState<string | null>(null);
  const [isLangOpen, setIsLangOpen] = useState(false);
  const [isChangingLang, setIsChangingLang] = useState(false);
  const langRef = useRef<HTMLDivElement>(null);

  const currentLang = resolveLanguage(i18n.language) ?? 'en';
  const switchLanguage = useSwitchLanguage();
  const lp = useLocalizedPath();

  const handleLanguageChange = useCallback(async (lang: SupportedLanguage) => {
    if (lang === currentLang) {
      setIsLangOpen(false);
      return;
    }
    setIsChangingLang(true);
    try {
      await changeLanguageSafe(lang);
      switchLanguage(lang);
    } finally {
      setIsChangingLang(false);
      setIsLangOpen(false);
    }
  }, [currentLang, switchLanguage]);
  const megaMenuRef = useRef<HTMLDivElement>(null);
  const mobileMenuRef = useRef<HTMLDivElement>(null);

  // Lock body scroll when mobile menu is open
  useEffect(() => {
    if (isMenuOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => {
      document.body.style.overflow = '';
    };
  }, [isMenuOpen]);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (langRef.current && !langRef.current.contains(e.target as Node)) {
        setIsLangOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        setActiveMenu(null);
        setIsLangOpen(false);
        setIsMenuOpen(false);
      }
    }
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, []);

  // Desktop mega-menu item stagger animation
  useEffect(() => {
    if (!activeMenu) return;

    const container = megaMenuRef.current;
    if (!container) return;

    // If user prefers reduced motion, show items immediately without animation
    if (prefersReducedMotion()) {
      const items = container.querySelectorAll('[data-menu-item]');
      items.forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      return;
    }

    // Reset all items to invisible immediately
    const items = container.querySelectorAll('[data-menu-item]');
    items.forEach((el) => {
      (el as HTMLElement).style.opacity = '0';
    });

    // Brief delay to let the framer-motion height transition start
    const timer = setTimeout(() => {
      const container = megaMenuRef.current;
      if (!container) return;
      const items = container.querySelectorAll('[data-menu-item]');
      if (items.length === 0) return;

      animate(items, {
        opacity: [0, 1],
        translateY: [12, 0],
        scale: [0.95, 1],
        duration: 300,
        delay: stagger(30),
        ease: 'outQuart',
      });
    }, 100);

    return () => clearTimeout(timer);
  }, [activeMenu]);

  // Mobile menu item stagger animation
  useEffect(() => {
    if (!expandedMobileMenu) return;

    const container = mobileMenuRef.current;
    if (!container) return;

    // If user prefers reduced motion, show items immediately without animation
    if (prefersReducedMotion()) {
      const items = container.querySelectorAll('[data-mobile-item]');
      items.forEach((el) => {
        (el as HTMLElement).style.opacity = '1';
      });
      return;
    }

    // Reset all items to invisible immediately
    const items = container.querySelectorAll('[data-mobile-item]');
    items.forEach((el) => {
      (el as HTMLElement).style.opacity = '0';
    });

    const timer = setTimeout(() => {
      const container = mobileMenuRef.current;
      if (!container) return;
      const items = container.querySelectorAll('[data-mobile-item]');
      if (items.length === 0) return;

      animate(items, {
        opacity: [0, 1],
        translateX: [-10, 0],
        duration: 250,
        delay: stagger(25),
        ease: 'outQuart',
      });
    }, 80);

    return () => clearTimeout(timer);
  }, [expandedMobileMenu]);

  const activeCategory = megaMenuData.find(c => c.name === activeMenu);

  return (
    <>
    <nav
      aria-label="Main navigation"
      className="fixed top-0 left-0 right-0 z-50 border-b border-zinc-200/50 dark:border-white/5 bg-white/50 dark:bg-[#030303]/50 backdrop-blur-xl"
      onMouseLeave={() => setActiveMenu(null)}
    >
      <div className="w-full px-4 sm:px-6 lg:px-10">
        <div className="relative flex items-center justify-between h-16">
          <div className="flex items-center gap-2 shrink-0">
            <Link to={lp('/')} className="flex items-center gap-2 font-display font-bold text-xl tracking-tighter">
              <NavShieldLogo />
              <span>
                <span className="bg-clip-text text-transparent bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500">Aigis</span>
                <span className="text-zinc-900 dark:text-white">Code</span>
              </span>
            </Link>
          </div>

          {/* Desktop Megamenu Items */}
          <div className="hidden lg:flex items-center gap-1 absolute left-1/2 -translate-x-1/2">
            {megaMenuData.map((category) => (
              <button
                key={category.name}
                onMouseEnter={() => setActiveMenu(category.name)}
                onClick={() => setActiveMenu(activeMenu === category.name ? null : category.name)}
                aria-expanded={activeMenu === category.name}
                aria-haspopup="true"
                className={`px-3 py-2 text-[13px] font-medium transition-colors flex items-center gap-1 rounded-lg whitespace-nowrap ${
                  activeMenu === category.name
                    ? 'text-zinc-900 dark:text-white bg-zinc-100 dark:bg-white/10'
                    : 'text-zinc-600 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white'
                }`}
              >
                {t(category.name)}
                <CaretDown aria-hidden="true" className={`w-3 h-3 transition-transform duration-200 ${activeMenu === category.name ? 'rotate-180' : ''}`} />
              </button>
            ))}
            <Link
              to={lp('/blog')}
              onMouseEnter={() => setActiveMenu(null)}
              className="px-3 py-2 text-[13px] font-medium transition-colors rounded-lg whitespace-nowrap text-zinc-600 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white"
            >
              {t('menu.blog')}
            </Link>
          </div>

          <div className="flex items-center gap-3">
            {/* Language Switcher Dropdown */}
            <div ref={langRef} className="hidden lg:block relative">
              <button
                onClick={() => setIsLangOpen(!isLangOpen)}
                aria-label="Select language"
                aria-expanded={isLangOpen}
                aria-haspopup="true"
                disabled={isChangingLang}
                className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-[11px] font-medium bg-zinc-100/50 dark:bg-white/5 border border-zinc-200/50 dark:border-white/5 text-zinc-600 dark:text-zinc-300 hover:text-zinc-900 dark:hover:text-white transition-colors disabled:opacity-50"
              >
                {isChangingLang ? (
                  <span className="w-3 h-3 border-2 border-zinc-400 border-t-transparent rounded-full animate-spin" />
                ) : (
                  currentLang.toUpperCase()
                )}
                <CaretDown aria-hidden="true" className={`w-3 h-3 transition-transform duration-200 ${isLangOpen ? 'rotate-180' : ''}`} />
              </button>
              <AnimatePresence>
                {isLangOpen && (
                  <motion.div
                    initial={{ opacity: 0, y: -4 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -4 }}
                    transition={{ duration: 0.15, ease: 'easeOut' }}
                    className="absolute right-0 top-full mt-1.5 min-w-[140px] rounded-lg bg-white/90 dark:bg-[#111]/90 backdrop-blur-xl border border-zinc-200/50 dark:border-white/10 shadow-lg overflow-hidden"
                  >
                    {supportedLanguages.map((lang) => (
                      <button
                        key={lang}
                        onClick={() => handleLanguageChange(lang)}
                        disabled={isChangingLang}
                        className={`w-full px-3 py-2 text-[12px] font-medium text-left transition-colors flex items-center gap-2 ${
                          currentLang === lang
                            ? 'bg-zinc-100 dark:bg-white/10 text-zinc-900 dark:text-white'
                            : 'text-zinc-500 dark:text-zinc-400 hover:bg-zinc-50 dark:hover:bg-white/5 hover:text-zinc-900 dark:hover:text-white'
                        } disabled:opacity-50`}
                      >
                        <span className="w-5 text-[10px] text-zinc-400 dark:text-zinc-500 uppercase">{lang}</span>
                        <span>{languageNames[lang]}</span>
                      </button>
                    ))}
                  </motion.div>
                )}
              </AnimatePresence>
            </div>

            <button
              onClick={toggleTheme}
              className="p-2 rounded-full hover:bg-zinc-100 dark:hover:bg-white/10 transition-colors"
              aria-label="Toggle theme"
            >
              {isDark ? <Sun aria-hidden="true" className="w-4 h-4" /> : <Moon aria-hidden="true" className="w-4 h-4" />}
            </button>
            <a
              href="#get-started"
              className="hidden lg:inline-flex items-center justify-center px-5 py-2 text-sm font-medium text-white bg-zinc-900 dark:bg-white dark:text-black rounded-full hover:scale-105 transition-transform duration-300 shadow-[0_0_20px_rgba(0,0,0,0.1)] dark:shadow-[0_0_20px_rgba(255,255,255,0.1)]"
            >
              {t('nav.getStarted')}
            </a>
            <button
              className="lg:hidden p-2"
              onClick={() => setIsMenuOpen(!isMenuOpen)}
              aria-label={isMenuOpen ? 'Close menu' : 'Open menu'}
              aria-expanded={isMenuOpen}
            >
              {isMenuOpen ? <X className="w-6 h-6" /> : <List className="w-6 h-6" />}
            </button>
          </div>
        </div>
      </div>

      {/* Desktop Megamenu Dropdown */}
      <AnimatePresence>
        {activeMenu && activeCategory && (
          <motion.div
            key="megamenu-dropdown"
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            transition={{ duration: 0.2, ease: 'easeOut' }}
            className="hidden lg:block overflow-hidden border-t border-zinc-200/50 dark:border-white/5 bg-white/80 dark:bg-[#030303]/80 backdrop-blur-xl shadow-xl"
            ref={megaMenuRef}
          >
            <div className="w-full px-4 sm:px-6 lg:px-10 py-8">
              <div className={`grid grid-cols-2 gap-1 ${activeCategory.items.length > 8 ? 'lg:grid-cols-4' : 'lg:grid-cols-3'}`}>
                {activeCategory.items.map((item) => {
                  const content = (
                    <>
                      <div className="w-10 h-10 rounded-lg bg-indigo-50 dark:bg-indigo-500/10 flex items-center justify-center text-indigo-600 dark:text-indigo-400 shrink-0 group-hover/item:scale-110 transition-transform">
                        <item.icon className="w-5 h-5" />
                      </div>
                      <div>
                        <div className="text-sm font-medium text-zinc-900 dark:text-white mb-1">{t(item.title)}</div>
                        <div className="text-xs text-zinc-500 dark:text-zinc-400 leading-relaxed">{t(item.desc)}</div>
                      </div>
                    </>
                  );

                  return item.href ? (
                    <Link
                      key={item.title}
                      to={lp(item.href!)}
                      onClick={() => setActiveMenu(null)}
                      data-menu-item
                      style={{ opacity: 0 }}
                      className="flex items-start gap-4 p-4 rounded-xl hover:bg-zinc-100 dark:hover:bg-white/5 transition-colors group/item"
                    >
                      {content}
                    </Link>
                  ) : (
                    <div
                      key={item.title}
                      data-menu-item
                      style={{ opacity: 0 }}
                      className="flex items-start gap-4 p-4 rounded-xl hover:bg-zinc-100 dark:hover:bg-white/5 transition-colors group/item"
                    >
                      {content}
                    </div>
                  );
                })}
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </nav>

    {/* Mobile Menu — rendered OUTSIDE nav to avoid backdrop-filter containing block */}
    <AnimatePresence>
      {isMenuOpen && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.2 }}
          className="fixed inset-x-0 top-16 bottom-0 z-[60] bg-white/95 dark:bg-[#030303]/95 backdrop-blur-xl lg:hidden overflow-y-auto overscroll-contain touch-pan-y"
          ref={mobileMenuRef}
        >
          <div className="px-4 py-2">
            {/* Mobile Language Switcher */}
            <div className="flex flex-wrap items-center gap-1.5 py-3 mb-2">
              {supportedLanguages.map((lang) => (
                <button
                  key={lang}
                  onClick={() => handleLanguageChange(lang)}
                  disabled={isChangingLang}
                  className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-all ${
                    currentLang === lang
                      ? 'bg-zinc-200 dark:bg-white/15 text-zinc-900 dark:text-white'
                      : 'text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 bg-zinc-100 dark:bg-white/5'
                  } disabled:opacity-50`}
                >
                  {languageNames[lang]}
                </button>
              ))}
            </div>

            <div className="border-b border-zinc-200/50 dark:border-white/5">
              <Link
                to={lp('/blog')}
                onClick={() => setIsMenuOpen(false)}
                className="block py-4 text-sm font-medium text-zinc-900 dark:text-white"
              >
                {t('menu.blog')}
              </Link>
            </div>

            {megaMenuData.map((category) => (
              <div key={category.name} className="border-b border-zinc-200/50 dark:border-white/5">
                <button
                  onClick={() => setExpandedMobileMenu(expandedMobileMenu === category.name ? null : category.name)}
                  aria-expanded={expandedMobileMenu === category.name}
                  className="w-full flex items-center justify-between py-4 text-sm font-medium text-zinc-900 dark:text-white"
                >
                  {t(category.name)}
                  <CaretDown aria-hidden="true" className={`w-4 h-4 text-zinc-400 transition-transform duration-200 ${expandedMobileMenu === category.name ? 'rotate-180' : ''}`} />
                </button>
                <AnimatePresence>
                  {expandedMobileMenu === category.name && (
                    <motion.div
                      initial={{ height: 0, opacity: 0 }}
                      animate={{ height: 'auto', opacity: 1 }}
                      exit={{ height: 0, opacity: 0 }}
                      transition={{ duration: 0.3, ease: 'easeOut' }}
                      className="overflow-hidden"
                    >
                      <div className="pb-4 grid gap-1">
                        {category.items.map((item) => {
                          const content = (
                            <>
                              <item.icon className="w-4 h-4 text-indigo-500 shrink-0" />
                              <div>
                                <span className="text-sm text-zinc-900 dark:text-white">{t(item.title)}</span>
                                <p className="text-xs text-zinc-500 dark:text-zinc-400">{t(item.desc)}</p>
                              </div>
                            </>
                          );

                          return item.href ? (
                            <Link
                              key={item.title}
                              to={lp(item.href!)}
                              onClick={() => setIsMenuOpen(false)}
                              data-mobile-item
                              style={{ opacity: 0 }}
                              className="flex items-center gap-3 p-3 rounded-lg hover:bg-zinc-100 dark:hover:bg-white/5 transition-colors"
                            >
                              {content}
                            </Link>
                          ) : (
                            <div
                              key={item.title}
                              data-mobile-item
                              style={{ opacity: 0 }}
                              className="flex items-center gap-3 p-3 rounded-lg hover:bg-zinc-100 dark:hover:bg-white/5 transition-colors"
                            >
                              {content}
                            </div>
                          );
                        })}
                      </div>
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            ))}
            <div className="py-4">
              <a
                href="#get-started"
                onClick={() => setIsMenuOpen(false)}
                className="block w-full px-5 py-3 text-sm font-medium text-white bg-zinc-900 dark:bg-white dark:text-black rounded-full hover:scale-105 transition-transform duration-300 text-center"
              >
                {t('nav.getStarted')}
              </a>
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
    </>
  );
}
