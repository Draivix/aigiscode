import { useEffect, useRef, useState } from 'react';
import { useLocation } from 'react-router-dom';
import { animate } from 'animejs';
import { prefersReducedMotion } from '@/hooks/useAnime';

/* -------------------------------------------------------------------------- */
/*  ScrollProgress — Thin progress bar at page top                             */
/* -------------------------------------------------------------------------- */

/** Route patterns that should show the scroll progress indicator */
const DETAIL_ROUTE_PATTERNS = [
  /^\/features\/[^/]+$/,
  /^\/languages\/[^/]+$/,
  /^\/integrations\/[^/]+$/,
  /^\/platform\/[^/]+$/,
  /^\/blog\/[^/]+$/,
];

/**
 * A thin gradient progress bar fixed to the top of the viewport.
 * Smoothly animates width based on scroll position using anime.js.
 * Only visible on detail pages (FeaturePage, LanguagePage, IntegrationPage,
 * PlatformFeaturePage, BlogPostPage).
 */
export default function ScrollProgress() {
  const location = useLocation();
  const barRef = useRef<HTMLDivElement>(null);
  const progressRef = useRef({ value: 0 });
  const animRef = useRef<ReturnType<typeof animate> | null>(null);
  const rafRef = useRef<number>(0);

  // Determine if current route is a detail page
  const isDetailPage = DETAIL_ROUTE_PATTERNS.some((pattern) =>
    pattern.test(location.pathname),
  );

  // Reset progress when navigating away
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    setVisible(isDetailPage);

    // Reset bar width on route change
    if (barRef.current) {
      barRef.current.style.width = '0%';
      progressRef.current.value = 0;
    }
  }, [isDetailPage, location.pathname]);

  useEffect(() => {
    if (!visible || !barRef.current) return;
    if (prefersReducedMotion()) {
      // Still show progress, just without animation smoothing
      const updateDirect = () => {
        if (!barRef.current) return;
        const scrollTop = window.scrollY;
        const docHeight = document.documentElement.scrollHeight - window.innerHeight;
        const progress = docHeight > 0 ? (scrollTop / docHeight) * 100 : 0;
        barRef.current.style.width = `${progress}%`;
      };

      window.addEventListener('scroll', updateDirect, { passive: true });
      updateDirect();
      return () => window.removeEventListener('scroll', updateDirect);
    }

    const bar = barRef.current;

    const onScroll = () => {
      const scrollTop = window.scrollY;
      const docHeight = document.documentElement.scrollHeight - window.innerHeight;
      const targetProgress = docHeight > 0 ? (scrollTop / docHeight) * 100 : 0;

      // Cancel any running animation and create a new smooth one
      animRef.current?.pause();
      animRef.current = animate(progressRef.current, {
        value: targetProgress,
        duration: 150,
        ease: 'outQuart',
        onUpdate: () => {
          bar.style.width = `${progressRef.current.value}%`;
        },
      });
    };

    window.addEventListener('scroll', onScroll, { passive: true });
    // Initial calculation
    onScroll();

    return () => {
      window.removeEventListener('scroll', onScroll);
      animRef.current?.pause();
      cancelAnimationFrame(rafRef.current);
    };
  }, [visible]);

  if (!visible) return null;

  return (
    <div
      className="fixed top-0 left-0 right-0 h-[3px] z-50 pointer-events-none"
      aria-hidden="true"
    >
      <div
        ref={barRef}
        className="h-full bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500 shadow-sm shadow-indigo-500/20"
        style={{ width: '0%', willChange: 'width' }}
      />
    </div>
  );
}
