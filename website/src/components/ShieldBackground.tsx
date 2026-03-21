import { useEffect, useRef } from 'react';
import { animate, createTimeline } from 'animejs';
import { createDrawable } from 'animejs/svg';
import { prefersReducedMotion } from '@/hooks/useAnime';

/* -------------------------------------------------------------------------- */
/*  Animated Shield SVG — draws itself on page load                           */
/* -------------------------------------------------------------------------- */

/**
 * A large, subtle SVG shield that draws its outline with anime.js,
 * then pulses with a gentle glow. Placed absolutely behind the hero text.
 */
export default function ShieldBackground() {
  const svgRef = useRef<SVGSVGElement>(null);
  const glowRef = useRef<SVGRadialGradientElement>(null);

  useEffect(() => {
    if (!svgRef.current || prefersReducedMotion()) return;

    const pathEls = svgRef.current.querySelectorAll<SVGGeometryElement>('.shield-path');
    if (!pathEls.length) return;

    // Create drawable wrappers for SVG path drawing
    const drawables = createDrawable(pathEls);

    // Timeline: draw outline, then fade in fill glow
    const tl = createTimeline({ autoplay: true });

    // Phase 1: Draw the shield outline
    tl.add(drawables, {
      draw: '0 1',
      duration: 2000,
      ease: 'inOutQuart',
    });

    // Phase 2: Fade in the glow fill
    tl.add('.shield-fill', {
      opacity: [0, 0.12],
      duration: 1000,
      ease: 'inOutSine',
    }, '-=400');

    // Phase 3: Subtle breathing pulse on the glow (loops forever)
    const breathe = animate('.shield-fill', {
      opacity: [0.12, 0.06],
      duration: 3000,
      ease: 'inOutSine',
      loop: true,
      alternate: true,
      autoplay: false,
    });

    // Start breathing after timeline completes
    tl.then(() => {
      breathe.play();
    });

    return () => {
      tl.pause();
      breathe.pause();
    };
  }, []);

  return (
    <div className="absolute inset-0 flex items-center justify-center pointer-events-none select-none" aria-hidden="true">
      <svg
        ref={svgRef}
        viewBox="0 0 400 480"
        className="w-[280px] md:w-[380px] lg:w-[440px] h-auto opacity-100"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <defs>
          <radialGradient
            ref={glowRef}
            id="shield-glow"
            cx="50%"
            cy="40%"
            r="60%"
          >
            <stop offset="0%" stopColor="#818cf8" stopOpacity="0.6" />
            <stop offset="40%" stopColor="#a78bfa" stopOpacity="0.3" />
            <stop offset="100%" stopColor="#6366f1" stopOpacity="0" />
          </radialGradient>
          <linearGradient id="shield-stroke-gradient" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#6366f1" stopOpacity="0.5" />
            <stop offset="50%" stopColor="#a855f7" stopOpacity="0.4" />
            <stop offset="100%" stopColor="#ec4899" stopOpacity="0.3" />
          </linearGradient>
        </defs>

        {/* Fill (starts invisible, faded in by timeline) */}
        <path
          className="shield-fill"
          d="M200 20 L360 100 L360 260 C360 340 280 420 200 460 C120 420 40 340 40 260 L40 100 Z"
          fill="url(#shield-glow)"
          opacity="0"
        />

        {/* Outer shield outline (drawn by anime.js) */}
        <path
          className="shield-path"
          d="M200 20 L360 100 L360 260 C360 340 280 420 200 460 C120 420 40 340 40 260 L40 100 Z"
          stroke="url(#shield-stroke-gradient)"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
          fill="none"
        />

        {/* Inner shield detail line */}
        <path
          className="shield-path"
          d="M200 60 L330 125 L330 255 C330 320 265 390 200 425 C135 390 70 320 70 255 L70 125 Z"
          stroke="url(#shield-stroke-gradient)"
          strokeWidth="0.8"
          strokeLinecap="round"
          strokeLinejoin="round"
          fill="none"
          opacity="0.5"
        />

        {/* Central cross / emblem lines */}
        <path
          className="shield-path"
          d="M200 120 L200 360"
          stroke="url(#shield-stroke-gradient)"
          strokeWidth="0.6"
          opacity="0.3"
        />
        <path
          className="shield-path"
          d="M100 220 L300 220"
          stroke="url(#shield-stroke-gradient)"
          strokeWidth="0.6"
          opacity="0.3"
        />
      </svg>
    </div>
  );
}
