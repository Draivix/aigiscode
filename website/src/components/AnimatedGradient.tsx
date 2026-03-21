import { useEffect, useRef } from 'react';
import { animate } from 'animejs';
import { prefersReducedMotion } from '@/hooks/useAnime';

/* -------------------------------------------------------------------------- */
/*  AnimatedGradient — Floating gradient blobs with anime.js                   */
/* -------------------------------------------------------------------------- */

interface AnimatedGradientProps {
  /** Extra CSS class for positioning wrapper */
  className?: string;
}

/**
 * Renders three animated gradient blobs (indigo, purple, pink) that float
 * continuously using anime.js loop/alternate. Each blob has a different
 * duration for an organic, non-repeating feel.
 *
 * Usage: position with `absolute inset-0` or similar via className.
 * The component is pointer-events-none and aria-hidden.
 */
export default function AnimatedGradient({ className = '' }: AnimatedGradientProps) {
  const blob1Ref = useRef<HTMLDivElement>(null);
  const blob2Ref = useRef<HTMLDivElement>(null);
  const blob3Ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (prefersReducedMotion()) return;

    const anims: ReturnType<typeof animate>[] = [];

    // Blob 1 — indigo, top-left drift
    if (blob1Ref.current) {
      anims.push(
        animate(blob1Ref.current, {
          translateX: [-30, 40],
          translateY: [-20, 30],
          scale: [0.9, 1.15],
          duration: 5000,
          ease: 'inOutSine',
          loop: true,
          alternate: true,
        }),
      );
    }

    // Blob 2 — purple, center-right drift (different timing)
    if (blob2Ref.current) {
      anims.push(
        animate(blob2Ref.current, {
          translateX: [20, -35],
          translateY: [25, -15],
          scale: [1.05, 0.85],
          duration: 7000,
          ease: 'inOutSine',
          loop: true,
          alternate: true,
        }),
      );
    }

    // Blob 3 — pink, bottom drift
    if (blob3Ref.current) {
      anims.push(
        animate(blob3Ref.current, {
          translateX: [15, -25],
          translateY: [-10, 35],
          scale: [0.95, 1.1],
          duration: 6000,
          ease: 'inOutSine',
          loop: true,
          alternate: true,
        }),
      );
    }

    return () => {
      anims.forEach((a) => a.pause());
    };
  }, []);

  if (prefersReducedMotion()) return null;

  return (
    <div
      className={`pointer-events-none overflow-hidden ${className}`}
      aria-hidden="true"
    >
      {/* Indigo blob */}
      <div
        ref={blob1Ref}
        className="absolute top-[10%] left-[15%] w-[300px] h-[300px] md:w-[400px] md:h-[400px] rounded-full bg-indigo-500/15 dark:bg-indigo-500/10 blur-[100px]"
      />

      {/* Purple blob */}
      <div
        ref={blob2Ref}
        className="absolute top-[30%] right-[10%] w-[250px] h-[250px] md:w-[350px] md:h-[350px] rounded-full bg-purple-500/15 dark:bg-purple-500/10 blur-[100px]"
      />

      {/* Pink blob */}
      <div
        ref={blob3Ref}
        className="absolute bottom-[10%] left-[30%] w-[280px] h-[280px] md:w-[380px] md:h-[380px] rounded-full bg-pink-500/12 dark:bg-pink-500/8 blur-[100px]"
      />
    </div>
  );
}
