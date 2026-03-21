import { useEffect, useRef, useCallback } from 'react';
import { animate, createTimeline } from 'animejs';
import type { AnimationParams, TimelineParams } from 'animejs';
import type { JSAnimation } from 'animejs';
import type { Timeline } from 'animejs';

/* -------------------------------------------------------------------------- */
/*  Reduced-motion preference                                                 */
/* -------------------------------------------------------------------------- */

export function prefersReducedMotion(): boolean {
  if (typeof window === 'undefined') return false;
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

/* -------------------------------------------------------------------------- */
/*  useAnimate — run an anime.js animation on a ref element                   */
/* -------------------------------------------------------------------------- */

type AnimateOptions = {
  /** Animation params (excluding targets) */
  params: AnimationParams;
  /** Skip animation entirely if prefers-reduced-motion */
  respectMotionPref?: boolean;
  /** Only run once (default true) */
  once?: boolean;
};

export function useAnimate<T extends HTMLElement | SVGElement>(
  options: AnimateOptions,
  deps: React.DependencyList = [],
) {
  const ref = useRef<T>(null);
  const hasRun = useRef(false);

  useEffect(() => {
    if (!ref.current) return;
    if (options.once !== false && hasRun.current) return;
    if (options.respectMotionPref !== false && prefersReducedMotion()) return;

    hasRun.current = true;
    const anim = animate(ref.current, options.params);

    return () => {
      anim.pause();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);

  return ref;
}

/* -------------------------------------------------------------------------- */
/*  useTimeline — create and return a timeline ref                            */
/* -------------------------------------------------------------------------- */

export function useTimeline(
  params?: TimelineParams,
  respectMotionPref = true,
): React.RefObject<Timeline | null> {
  const tlRef = useRef<Timeline | null>(null);

  useEffect(() => {
    if (respectMotionPref && prefersReducedMotion()) return;
    const tl = createTimeline(params);
    tlRef.current = tl;

    return () => {
      tl.pause();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return tlRef;
}

/* -------------------------------------------------------------------------- */
/*  useIntersectionAnimate — animate when element scrolls into view           */
/* -------------------------------------------------------------------------- */

type IntersectionAnimateOptions = {
  params: AnimationParams;
  /** IntersectionObserver threshold (default 0.1) */
  threshold?: number;
  /** Root margin (default '-80px') */
  rootMargin?: string;
  /** Respect reduced motion (default true) */
  respectMotionPref?: boolean;
};

export function useIntersectionAnimate<T extends HTMLElement | SVGElement>(
  options: IntersectionAnimateOptions,
) {
  const ref = useRef<T>(null);
  const hasAnimated = useRef(false);

  useEffect(() => {
    if (!ref.current) return;
    if (options.respectMotionPref !== false && prefersReducedMotion()) return;

    const el = ref.current;

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !hasAnimated.current) {
          hasAnimated.current = true;
          animate(el, options.params);
          observer.disconnect();
        }
      },
      {
        threshold: options.threshold ?? 0.1,
        rootMargin: options.rootMargin ?? '-80px',
      },
    );

    observer.observe(el);

    return () => {
      observer.disconnect();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return ref;
}

/* -------------------------------------------------------------------------- */
/*  useCountUp — animate a number from 0 to target                           */
/* -------------------------------------------------------------------------- */

export function useCountUp(
  target: number,
  duration = 1200,
  shouldStart = false,
): [React.RefObject<HTMLSpanElement | null>, number] {
  const ref = useRef<HTMLSpanElement | null>(null);
  const valueRef = useRef({ val: 0 });
  const animRef = useRef<JSAnimation | null>(null);
  const currentVal = useRef(0);

  const update = useCallback(() => {
    if (ref.current) {
      ref.current.textContent = String(Math.round(valueRef.current.val));
    }
  }, []);

  useEffect(() => {
    if (!shouldStart || prefersReducedMotion()) {
      if (ref.current) ref.current.textContent = String(target);
      return;
    }

    valueRef.current.val = 0;
    animRef.current = animate(valueRef.current, {
      val: target,
      duration,
      ease: 'outExpo',
      modifier: (v: number) => Math.round(v),
      onUpdate: update,
    });

    return () => {
      animRef.current?.pause();
    };
  }, [shouldStart, target, duration, update]);

  return [ref, currentVal.current];
}
