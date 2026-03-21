import { useEffect, useRef } from 'react';
import { animate } from 'animejs';
import { createDrawable } from 'animejs/svg';
import { prefersReducedMotion } from '@/hooks/useAnime';

/* -------------------------------------------------------------------------- */
/*  Pipeline Connection SVG — animated line connecting stages                  */
/* -------------------------------------------------------------------------- */

interface PipelineSVGProps {
  /** Number of pipeline stages to connect */
  stageCount: number;
  /** Whether the section is in view */
  isInView: boolean;
}

export default function PipelineSVG({ stageCount, isInView }: PipelineSVGProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const hasAnimated = useRef(false);

  useEffect(() => {
    if (!svgRef.current || !isInView || hasAnimated.current) return;
    if (prefersReducedMotion()) return;

    hasAnimated.current = true;

    const path = svgRef.current.querySelector<SVGGeometryElement>('.pipeline-line');
    if (!path) return;

    const drawables = createDrawable(path);

    animate(drawables, {
      draw: '0 1',
      duration: 2000,
      delay: 300,
      ease: 'inOutQuart',
    });
  }, [isInView, stageCount]);

  // Build a smooth curved path across all stages
  // Each stage is spaced equally. We create a gentle sine wave.
  const points = stageCount;
  const segWidth = 100 / (points - 1);

  // Build SVG path: horizontal line with small bumps at each node
  let d = `M 0 50`;
  for (let i = 1; i < points; i++) {
    const x = i * segWidth;
    const cpx1 = (i - 0.5) * segWidth;
    d += ` C ${cpx1} 50, ${cpx1} 50, ${x} 50`;
  }

  return (
    <svg
      ref={svgRef}
      viewBox="0 0 100 100"
      preserveAspectRatio="none"
      className="absolute top-6 left-[8%] right-[8%] h-[2px] pointer-events-none hidden lg:block"
      aria-hidden="true"
    >
      <defs>
        <linearGradient id="pipeline-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stopColor="#6366f1" />
          <stop offset="100%" stopColor="#a855f7" />
        </linearGradient>
      </defs>
      <path
        className="pipeline-line"
        d={d}
        stroke="url(#pipeline-gradient)"
        strokeWidth="8"
        fill="none"
        strokeLinecap="round"
      />
    </svg>
  );
}
