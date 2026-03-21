import { LocalizedLink as Link } from '@/lib/i18n-routing';
import { useTranslation } from 'react-i18next';
import {
  FacebookLogo,
  GithubLogo,
  InstagramLogo,
  LinkedinLogo,
  ShieldCheck,
  XLogo,
  YoutubeLogo,
} from '@phosphor-icons/react';

export default function Footer() {
  const { t } = useTranslation();
  const year = new Date().getFullYear();

  return (
    <footer className="border-t border-zinc-200 dark:border-white/5 bg-white dark:bg-[#030303] relative z-10">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-16 pb-8">
        {/* Top: Logo + Newsletter */}
        <div className="flex flex-col md:flex-row items-start justify-between gap-8 mb-16">
          <div>
            <Link to="/" className="flex items-center gap-2 font-display font-bold text-2xl tracking-tighter">
              <ShieldCheck size={24} className="text-indigo-500" weight="duotone" />
              <span>
                <span className="bg-clip-text text-transparent bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500">Aigis</span>
                <span className="text-zinc-900 dark:text-white">Code</span>
              </span>
            </Link>
            <p className="mt-3 text-sm text-zinc-500 max-w-xs">{t('footer.tagline')}</p>
          </div>
          <div className="w-full md:w-auto">
            <p className="text-sm font-medium text-zinc-900 dark:text-white mb-2">{t('footer.newsletter')}</p>
            <form onSubmit={e => e.preventDefault()} className="flex gap-2">
              <label htmlFor="newsletter-email" className="sr-only">{t('footer.emailPlaceholder')}</label>
              <input
                id="newsletter-email"
                type="email"
                placeholder={t('footer.emailPlaceholder')}
                className="px-4 py-2 rounded-lg bg-zinc-100 dark:bg-white/5 border border-zinc-200 dark:border-white/10 text-sm placeholder:text-zinc-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 flex-1 min-w-0 md:w-64 md:flex-none"
              />
              <button className="px-5 py-2 rounded-lg bg-zinc-900 dark:bg-white text-white dark:text-black text-sm font-medium hover:scale-105 transition-transform">
                {t('footer.subscribe')}
              </button>
            </form>
          </div>
        </div>

        {/* 4-Column Grid */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-8 mb-16">
          {/* Column 1: Product */}
          <nav aria-label="Product links">
            <h4 className="text-sm font-semibold text-zinc-900 dark:text-white mb-4">{t('footer.product')}</h4>
            <ul className="space-y-3 text-sm text-zinc-500">
              <li><Link to="/features" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.productLinks.analysis')}</Link></li>
              <li><Link to="/use-cases" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.productLinks.useCases')}</Link></li>
              <li><Link to="/languages" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.productLinks.languages')}</Link></li>
              <li><Link to="/integrations" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.productLinks.integrations')}</Link></li>
              <li><Link to="/platform/plugin-system" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.productLinks.plugins')}</Link></li>
              <li><Link to="/platform/policy-engine" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.productLinks.policy')}</Link></li>
              <li><Link to="/platform/report-generator" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.productLinks.reporting')}</Link></li>
              <li><Link to="/docs#installation" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.productLinks.pricing')}</Link></li>
              <li><a href="https://github.com/Draivix/aigiscode/releases" target="_blank" rel="noopener noreferrer" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.productLinks.whatsNew')}</a></li>
            </ul>
          </nav>

          {/* Column 2: Company */}
          <nav aria-label="Company links">
            <h4 className="text-sm font-semibold text-zinc-900 dark:text-white mb-4">{t('footer.company')}</h4>
            <ul className="space-y-3 text-sm text-zinc-500">
              <li><Link to="/about#company" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.companyLinks.about')}</Link></li>
              <li><Link to="/about#leadership" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.companyLinks.leadership')}</Link></li>
              <li><Link to="/about#careers" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.companyLinks.careers')} <span className="ml-1 px-1.5 py-0.5 rounded text-[10px] font-semibold bg-indigo-500/10 text-indigo-600 dark:text-indigo-400">{t('footer.companyLinks.hiring')}</span></Link></li>
              <li><Link to="/about#press" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.companyLinks.press')}</Link></li>
              <li><Link to="/blog" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.companyLinks.blog')}</Link></li>
              <li><Link to="/about#stories" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.companyLinks.stories')}</Link></li>
              <li><Link to="/about#partners" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.companyLinks.partners')}</Link></li>
              <li><Link to="/about#investors" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.companyLinks.investors')}</Link></li>
              <li><Link to="/about#events" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.companyLinks.events')}</Link></li>
              <li><Link to="/about#contact" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.companyLinks.contact')}</Link></li>
            </ul>
          </nav>

          {/* Column 3: Resources */}
          <nav aria-label="Resource links">
            <h4 className="text-sm font-semibold text-zinc-900 dark:text-white mb-4">{t('footer.resources')}</h4>
            <ul className="space-y-3 text-sm text-zinc-500">
              <li><Link to="/docs" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.resourceLinks.docs')}</Link></li>
              <li><Link to="/docs#report-structure" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.resourceLinks.api')}</Link></li>
              <li><Link to="/about#contact" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.resourceLinks.help')}</Link></li>
              <li><a href="https://github.com/Draivix/aigiscode" target="_blank" rel="noopener noreferrer" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.resourceLinks.community')}</a></li>
              <li><Link to="/platform" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.resourceLinks.devHub')}</Link></li>
              <li><Link to="/integrations" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.resourceLinks.integrations')}</Link></li>
              <li><a href="https://github.com/Draivix/aigiscode" target="_blank" rel="noopener noreferrer" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.resourceLinks.status')}</a></li>
              <li><a href="https://github.com/Draivix/aigiscode/releases" target="_blank" rel="noopener noreferrer" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.resourceLinks.changelog')}</a></li>
              <li><Link to="/blog" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.resourceLinks.webinars')}</Link></li>
              <li><Link to="/docs" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.resourceLinks.templates')}</Link></li>
            </ul>
          </nav>

          {/* Column 4: Legal & Compliance */}
          <nav aria-label="Legal links">
            <h4 className="text-sm font-semibold text-zinc-900 dark:text-white mb-4">{t('footer.legal')}</h4>
            <ul className="space-y-3 text-sm text-zinc-500">
              {(['terms', 'privacy', 'cookies', 'gdpr', 'soc2', 'iso', 'dpa', 'security', 'ai', 'accessibility'] as const).map((key) => {
                const legalUrls: Record<string, string> = {
                  security: 'https://github.com/Draivix/aigiscode/blob/main/SECURITY.md',
                  terms: '/about#contact',
                  privacy: '/about#contact',
                  cookies: '/about#contact',
                  gdpr: '/about#contact',
                  soc2: '/about#contact',
                  iso: '/about#contact',
                  dpa: '/about#contact',
                  ai: '/about#contact',
                  accessibility: '/about#contact',
                };
                const url = legalUrls[key] || '/about#contact';
                const isExternal = url.startsWith('http');
                return (
                  <li key={key}>
                    {isExternal ? (
                      <a href={url} target="_blank" rel="noopener noreferrer" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t(`footer.legalLinks.${key}`)}</a>
                    ) : (
                      <Link to={url} className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t(`footer.legalLinks.${key}`)}</Link>
                    )}
                  </li>
                );
              })}
            </ul>
          </nav>
        </div>

        {/* Bottom Bar: Social + Copyright */}
        <div className="border-t border-zinc-200 dark:border-white/5 pt-8 flex flex-col items-center md:flex-row md:justify-between gap-4 md:gap-6">
          <div className="flex flex-col sm:flex-row items-center gap-2 sm:gap-4">
            <p className="text-sm text-zinc-500">{t('footer.copyright', { year })}</p>
            <span className="hidden sm:inline text-xs text-zinc-300 dark:text-zinc-600">|</span>
            <a
              href="https://draivix.com"
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 transition-colors"
            >
              A <span className="font-semibold">Draivix Corporation</span> product
            </a>
          </div>

          <div className="flex items-center gap-5">
            <a href="https://x.com/aigiscode" target="_blank" rel="noopener noreferrer" className="text-zinc-500 hover:text-zinc-900 dark:hover:text-white transition-colors" aria-label="X / Twitter">
              <XLogo aria-hidden="true" className="w-5 h-5" />
            </a>
            <a href="https://www.linkedin.com/company/aigiscode" target="_blank" rel="noopener noreferrer" className="text-zinc-500 hover:text-zinc-900 dark:hover:text-white transition-colors" aria-label="LinkedIn">
              <LinkedinLogo aria-hidden="true" className="w-5 h-5" />
            </a>
            <a href="https://github.com/Draivix/aigiscode" target="_blank" rel="noopener noreferrer" className="text-zinc-500 hover:text-zinc-900 dark:hover:text-white transition-colors" aria-label="GitHub">
              <GithubLogo aria-hidden="true" className="w-5 h-5" />
            </a>
            <a href="https://www.youtube.com/@aigiscode" target="_blank" rel="noopener noreferrer" className="text-zinc-500 hover:text-zinc-900 dark:hover:text-white transition-colors" aria-label="YouTube">
              <YoutubeLogo aria-hidden="true" className="w-5 h-5" />
            </a>
            <a href="https://www.facebook.com/aigiscode" target="_blank" rel="noopener noreferrer" className="text-zinc-500 hover:text-zinc-900 dark:hover:text-white transition-colors" aria-label="Facebook">
              <FacebookLogo aria-hidden="true" className="w-5 h-5" />
            </a>
            <a href="https://www.instagram.com/aigiscode" target="_blank" rel="noopener noreferrer" className="text-zinc-500 hover:text-zinc-900 dark:hover:text-white transition-colors" aria-label="Instagram">
              <InstagramLogo aria-hidden="true" className="w-5 h-5" />
            </a>
          </div>

          <div className="flex items-center gap-4 text-xs text-zinc-500">
            <a href="/sitemap.xml" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.sitemap')}</a>
            <span>·</span>
            <Link to="/about#contact" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.doNotSell')}</Link>
            <span>·</span>
            <Link to="/about#contact" className="hover:text-zinc-900 dark:hover:text-white transition-colors">{t('footer.cookiePrefs')}</Link>
          </div>
        </div>
      </div>
    </footer>
  );
}
