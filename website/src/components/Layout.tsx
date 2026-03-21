import { useState, useEffect, memo } from 'react';
import { Outlet, useLocation } from 'react-router-dom';
import { IconContext } from '@phosphor-icons/react';
import Navbar from './Navbar';
import Footer from './Footer';
import ScrollProgress from './ScrollProgress';

/**
 * Scroll to top + analytics page view tracking on route change.
 * Fires GA4 and Matomo page views for SPA navigation.
 */
function ScrollToTop() {
  const { pathname } = useLocation();

  useEffect(() => {
    window.scrollTo(0, 0);

    // GA4 SPA page view
    if (typeof window.gtag === 'function') {
      window.gtag('event', 'page_view', {
        page_path: pathname,
        page_location: window.location.href,
        page_title: document.title,
      });
    }

    // Matomo SPA page view
    if (Array.isArray(window._paq)) {
      window._paq.push(['setCustomUrl', window.location.href]);
      window._paq.push(['setDocumentTitle', document.title]);
      window._paq.push(['trackPageView']);
    }
  }, [pathname]);

  return null;
}

/**
 * Ambient background blobs - memoized to prevent unnecessary re-renders
 * when parent state (isDark, toggleTheme) changes. Uses CSS containment
 * and compositor hints via the ambient-bg class defined in index.css.
 */
const AmbientBackground = memo(function AmbientBackground() {
  return (
    <div className="fixed inset-0 z-0 pointer-events-none overflow-hidden ambient-bg" aria-hidden="true">
      <div className="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] rounded-full bg-indigo-500/20 dark:bg-indigo-500/10 blur-[120px] mix-blend-screen" />
      <div className="absolute top-[20%] right-[-10%] w-[30%] h-[50%] rounded-full bg-violet-500/20 dark:bg-violet-500/10 blur-[120px] mix-blend-screen" />
      <div className="absolute bottom-[-20%] left-[20%] w-[50%] h-[50%] rounded-full bg-blue-500/20 dark:bg-blue-500/10 blur-[120px] mix-blend-screen" />
    </div>
  );
});

export default function Layout() {
  const [isDark, setIsDark] = useState(() => window.matchMedia('(prefers-color-scheme: dark)').matches);

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, [isDark]);

  const toggleTheme = () => setIsDark(!isDark);

  return (
    <IconContext.Provider value={{ weight: 'thin' }}>
      <ScrollToTop />
      <div className="min-h-screen font-sans bg-zinc-50 dark:bg-[#030303] text-zinc-900 dark:text-zinc-50 selection:bg-indigo-500/30 overflow-x-hidden transition-colors duration-500">
        <a href="#main-content" className="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:z-[100] focus:px-4 focus:py-2 focus:bg-indigo-600 focus:text-white focus:rounded-lg">Skip to main content</a>

        <AmbientBackground />

        <ScrollProgress />
        <Navbar isDark={isDark} toggleTheme={toggleTheme} />

        <main id="main-content" className="relative z-10 pt-16">
          <Outlet context={{ isDark, toggleTheme }} />
        </main>

        <Footer />
      </div>
    </IconContext.Provider>
  );
}
