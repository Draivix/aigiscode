import { useEffect, useRef } from 'react';
import { animate, stagger, createTimeline } from 'animejs';
import { createDrawable, createMotionPath } from 'animejs/svg';
import { prefersReducedMotion } from '@/hooks/useAnime';

/* -------------------------------------------------------------------------- */
/*  CodeFlowSVG — Animated analysis pipeline visualization                    */
/* -------------------------------------------------------------------------- */

interface CodeFlowSVGProps {
  /** Extra CSS classes for the wrapper */
  className?: string;
  /** Trigger entrance animation when true */
  isInView?: boolean;
}

/* ---------- Geometry constants ------------------------------------------- */

// Input file positions (left side)
const INPUT_FILES = [
  { x: 40, y: 40, label: '.php' },
  { x: 40, y: 90, label: '.py' },
  { x: 40, y: 140, label: '.ts' },
];

// Center processing node
const CENTER = { x: 400, y: 100 };
const CENTER_RINGS = [32, 44, 56];

// Output result positions (right side)
const OUTPUTS = [
  { x: 720, y: 65, type: 'check' as const },
  { x: 720, y: 135, type: 'warning' as const },
];

// Particle count per flow path
const PARTICLES_PER_PATH = 3;

/* ---------- Path generators ---------------------------------------------- */

function inputPath(file: { x: number; y: number }): string {
  const dx = CENTER.x - file.x;
  const cp1x = file.x + dx * 0.35;
  const cp2x = file.x + dx * 0.65;
  return `M ${file.x + 50} ${file.y + 15} C ${cp1x} ${file.y + 15}, ${cp2x} ${CENTER.y}, ${CENTER.x - 58} ${CENTER.y}`;
}

function outputPath(out: { x: number; y: number }): string {
  const dx = out.x - CENTER.x;
  const cp1x = CENTER.x + dx * 0.35;
  const cp2x = CENTER.x + dx * 0.65;
  return `M ${CENTER.x + 58} ${CENTER.y} C ${cp1x} ${CENTER.y}, ${cp2x} ${out.y}, ${out.x - 30} ${out.y}`;
}

/* ---------- Component ---------------------------------------------------- */

export default function CodeFlowSVG({
  className = '',
  isInView = false,
}: CodeFlowSVGProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const hasAnimated = useRef(false);
  const animsRef = useRef<ReturnType<typeof animate>[]>([]);
  const tlRef = useRef<ReturnType<typeof createTimeline> | null>(null);

  useEffect(() => {
    if (!svgRef.current || !isInView || hasAnimated.current) return;
    if (prefersReducedMotion()) return;

    hasAnimated.current = true;
    const svg = svgRef.current;
    const anims: ReturnType<typeof animate>[] = [];

    /* ---- Phase 0: Gather elements ---- */
    const inputPathEls = svg.querySelectorAll<SVGGeometryElement>('.cf-input-path');
    const outputPathEls = svg.querySelectorAll<SVGGeometryElement>('.cf-output-path');
    const fileIcons = svg.querySelectorAll<SVGElement>('.cf-file-icon');
    const centerRings = svg.querySelectorAll<SVGElement>('.cf-center-ring');
    const centerCore = svg.querySelector<SVGElement>('.cf-center-core');
    const outputIcons = svg.querySelectorAll<SVGElement>('.cf-output-icon');
    const inputParticles = svg.querySelectorAll<SVGElement>('.cf-particle-in');
    const outputParticles = svg.querySelectorAll<SVGElement>('.cf-particle-out');
    const labelEls = svg.querySelectorAll<SVGElement>('.cf-label');
    const centerLabel = svg.querySelector<SVGElement>('.cf-center-label');

    /* ---- Phase 1: Build entrance timeline ---- */
    const tl = createTimeline({
      autoplay: true,
      defaults: { ease: 'outExpo' },
    });
    tlRef.current = tl;

    // 1a. File icons appear with stagger
    tl.add(fileIcons, {
      opacity: [0, 1],
      translateX: [-20, 0],
      scale: [0.6, 1],
      duration: 600,
      delay: stagger(120),
    }, 0);

    // 1b. File labels
    tl.add(labelEls, {
      opacity: [0, 0.7],
      duration: 400,
      delay: stagger(120),
    }, 100);

    // 1c. Draw input paths
    const inputDrawables = createDrawable(inputPathEls);
    tl.add(inputDrawables, {
      draw: '0 1',
      duration: 1000,
      delay: stagger(100),
      ease: 'inOutQuart',
    }, 200);

    // 1d. Center core appears
    tl.add(centerCore!, {
      opacity: [0, 1],
      scale: [0.3, 1],
      duration: 500,
      ease: 'outBack',
    }, 800);

    // 1e. Center label
    tl.add(centerLabel!, {
      opacity: [0, 0.9],
      duration: 400,
    }, 1000);

    // 1f. Center rings expand outward with stagger
    tl.add(centerRings, {
      opacity: [0, (_, i) => 0.6 - i * 0.15],
      scale: [0.5, 1],
      duration: 600,
      delay: stagger(80),
      ease: 'outQuart',
    }, 900);

    // 1g. Draw output paths
    const outputDrawables = createDrawable(outputPathEls);
    tl.add(outputDrawables, {
      draw: '0 1',
      duration: 800,
      delay: stagger(100),
      ease: 'inOutQuart',
    }, 1200);

    // 1h. Output icons appear
    tl.add(outputIcons, {
      opacity: [0, 1],
      translateX: [20, 0],
      scale: [0.5, 1],
      duration: 500,
      delay: stagger(120),
      ease: 'outBack',
    }, 1600);

    /* ---- Phase 2: Looping animations (start after entrance) ---- */
    tl.then(() => {
      // 2a. Center ring rotation / pulse
      centerRings.forEach((ring, i) => {
        anims.push(
          animate(ring, {
            rotate: i % 2 === 0 ? [0, 360] : [360, 0],
            duration: 8000 + i * 4000,
            ease: 'linear',
            loop: true,
          }),
        );
        anims.push(
          animate(ring, {
            scale: [1, 1.06],
            opacity: [0.6 - i * 0.15, 0.3 - i * 0.05],
            duration: 2000 + i * 500,
            ease: 'inOutSine',
            loop: true,
            alternate: true,
          }),
        );
      });

      // 2b. Center core gentle pulse
      if (centerCore) {
        anims.push(
          animate(centerCore, {
            scale: [1, 1.08],
            duration: 2000,
            ease: 'inOutSine',
            loop: true,
            alternate: true,
          }),
        );
      }

      // 2c. Input flow particles — travel along input paths
      const inputPathElArr = Array.from(inputPathEls);
      inputParticles.forEach((particle, idx) => {
        const pathIdx = Math.floor(idx / PARTICLES_PER_PATH);
        const particleOffset = (idx % PARTICLES_PER_PATH) / PARTICLES_PER_PATH;
        const pathEl = inputPathElArr[pathIdx];
        if (!pathEl) return;

        const motionPath = createMotionPath(pathEl, particleOffset * 100);
        if (!motionPath) return;

        anims.push(
          animate(particle, {
            translateX: motionPath.translateX,
            translateY: motionPath.translateY,
            opacity: [
              { to: 0, duration: 0 },
              { to: 0.9, duration: 300 },
              { to: 0.9, duration: 1200 },
              { to: 0, duration: 300 },
            ],
            duration: 1800,
            delay: particleOffset * 1800 + pathIdx * 200,
            ease: 'linear',
            loop: true,
            loopDelay: PARTICLES_PER_PATH * 600,
          }),
        );
      });

      // 2d. Output flow particles — travel along output paths
      const outputPathElArr = Array.from(outputPathEls);
      outputParticles.forEach((particle, idx) => {
        const pathIdx = Math.floor(idx / PARTICLES_PER_PATH);
        const particleOffset = (idx % PARTICLES_PER_PATH) / PARTICLES_PER_PATH;
        const pathEl = outputPathElArr[pathIdx];
        if (!pathEl) return;

        const motionPath = createMotionPath(pathEl, particleOffset * 100);
        if (!motionPath) return;

        anims.push(
          animate(particle, {
            translateX: motionPath.translateX,
            translateY: motionPath.translateY,
            opacity: [
              { to: 0, duration: 0 },
              { to: 0.9, duration: 200 },
              { to: 0.9, duration: 1000 },
              { to: 0, duration: 200 },
            ],
            duration: 1400,
            delay: particleOffset * 1400 + pathIdx * 250,
            ease: 'linear',
            loop: true,
            loopDelay: PARTICLES_PER_PATH * 500,
          }),
        );
      });
    });

    animsRef.current = anims;

    return () => {
      tl.pause();
      anims.forEach((a) => a.pause());
    };
  }, [isInView]);

  /* ---- Render ----------------------------------------------------------- */
  return (
    <div
      className={`pointer-events-none select-none ${className}`}
      aria-hidden="true"
    >
      <svg
        ref={svgRef}
        viewBox="0 0 800 200"
        className="w-full h-auto"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <defs>
          {/* Input path gradient: indigo */}
          <linearGradient id="cf-grad-input" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#818cf8" stopOpacity="0.6" />
            <stop offset="100%" stopColor="#a78bfa" stopOpacity="0.8" />
          </linearGradient>

          {/* Output path gradient: purple to emerald */}
          <linearGradient id="cf-grad-output" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#a78bfa" stopOpacity="0.8" />
            <stop offset="100%" stopColor="#34d399" stopOpacity="0.7" />
          </linearGradient>

          {/* Center node radial glow */}
          <radialGradient id="cf-center-glow" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="#a78bfa" stopOpacity="0.35" />
            <stop offset="50%" stopColor="#818cf8" stopOpacity="0.15" />
            <stop offset="100%" stopColor="#6366f1" stopOpacity="0" />
          </radialGradient>

          {/* Center core gradient */}
          <radialGradient id="cf-core-grad" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="#c4b5fd" />
            <stop offset="60%" stopColor="#a78bfa" />
            <stop offset="100%" stopColor="#7c3aed" />
          </radialGradient>

          {/* Ring stroke gradients */}
          <linearGradient id="cf-ring-grad-1" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#818cf8" />
            <stop offset="100%" stopColor="#a78bfa" />
          </linearGradient>
          <linearGradient id="cf-ring-grad-2" x1="100%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stopColor="#a78bfa" />
            <stop offset="100%" stopColor="#c084fc" />
          </linearGradient>
          <linearGradient id="cf-ring-grad-3" x1="0%" y1="100%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#6366f1" />
            <stop offset="100%" stopColor="#818cf8" />
          </linearGradient>

          {/* Input particle glow */}
          <radialGradient id="cf-particle-glow-in" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="#a5b4fc" stopOpacity="1" />
            <stop offset="100%" stopColor="#818cf8" stopOpacity="0" />
          </radialGradient>

          {/* Output particle glow */}
          <radialGradient id="cf-particle-glow-out" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="#6ee7b7" stopOpacity="1" />
            <stop offset="100%" stopColor="#34d399" stopOpacity="0" />
          </radialGradient>

          {/* Check icon gradient */}
          <linearGradient id="cf-check-grad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#34d399" />
            <stop offset="100%" stopColor="#10b981" />
          </linearGradient>

          {/* Warning icon gradient */}
          <linearGradient id="cf-warn-grad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#fbbf24" />
            <stop offset="100%" stopColor="#f59e0b" />
          </linearGradient>

          {/* File icon gradient */}
          <linearGradient id="cf-file-grad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#a5b4fc" stopOpacity="0.9" />
            <stop offset="100%" stopColor="#818cf8" stopOpacity="0.7" />
          </linearGradient>

          {/* Soft shadow filter for file icons */}
          <filter id="cf-glow-sm" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur in="SourceGraphic" stdDeviation="2" result="blur" />
            <feComposite in="SourceGraphic" in2="blur" operator="over" />
          </filter>

          {/* Particle trail filter */}
          <filter id="cf-particle-blur" x="-100%" y="-100%" width="300%" height="300%">
            <feGaussianBlur stdDeviation="1.5" />
          </filter>
        </defs>

        {/* ================================================================== */}
        {/*  Background ambient glow                                           */}
        {/* ================================================================== */}
        <circle
          cx={CENTER.x}
          cy={CENTER.y}
          r="90"
          fill="url(#cf-center-glow)"
          opacity="0.5"
        />

        {/* ================================================================== */}
        {/*  Input file icons                                                  */}
        {/* ================================================================== */}
        {INPUT_FILES.map((file, i) => (
          <g
            key={`file-${i}`}
            className="cf-file-icon"
            opacity="0"
            style={{ transformOrigin: `${file.x + 25}px ${file.y + 15}px` }}
          >
            {/* File body - rounded rectangle */}
            <rect
              x={file.x}
              y={file.y}
              width="50"
              height="30"
              rx="4"
              ry="4"
              fill="none"
              stroke="url(#cf-file-grad)"
              strokeWidth="1.2"
            />
            {/* File fold corner */}
            <path
              d={`M ${file.x + 38} ${file.y} L ${file.x + 50} ${file.y + 12}`}
              stroke="url(#cf-file-grad)"
              strokeWidth="0.8"
              fill="none"
              opacity="0.6"
            />
            {/* Code lines inside file */}
            <line
              x1={file.x + 6}
              y1={file.y + 10}
              x2={file.x + 28}
              y2={file.y + 10}
              stroke="#818cf8"
              strokeWidth="1"
              opacity="0.4"
              strokeLinecap="round"
            />
            <line
              x1={file.x + 6}
              y1={file.y + 15}
              x2={file.x + 34}
              y2={file.y + 15}
              stroke="#818cf8"
              strokeWidth="1"
              opacity="0.3"
              strokeLinecap="round"
            />
            <line
              x1={file.x + 6}
              y1={file.y + 20}
              x2={file.x + 22}
              y2={file.y + 20}
              stroke="#818cf8"
              strokeWidth="1"
              opacity="0.25"
              strokeLinecap="round"
            />
          </g>
        ))}

        {/* File extension labels */}
        {INPUT_FILES.map((file, i) => (
          <text
            key={`label-${i}`}
            className="cf-label"
            x={file.x + 25}
            y={file.y - 4}
            textAnchor="middle"
            fill="#a5b4fc"
            fontSize="7"
            fontFamily="monospace"
            opacity="0"
          >
            {file.label}
          </text>
        ))}

        {/* ================================================================== */}
        {/*  Input flow paths                                                  */}
        {/* ================================================================== */}
        {INPUT_FILES.map((file, i) => (
          <path
            key={`in-path-${i}`}
            className="cf-input-path"
            d={inputPath(file)}
            stroke="url(#cf-grad-input)"
            strokeWidth="1.2"
            fill="none"
            strokeLinecap="round"
            opacity="0.7"
          />
        ))}

        {/* ================================================================== */}
        {/*  Center processing node                                            */}
        {/* ================================================================== */}

        {/* Concentric rings (outer to inner for z-order) */}
        {CENTER_RINGS.slice()
          .reverse()
          .map((r, revIdx) => {
            const i = CENTER_RINGS.length - 1 - revIdx;
            // Dashed rings for a technical/scanning feel
            const dashArray =
              i === 0
                ? '4 6'
                : i === 1
                  ? '8 4'
                  : '3 8';
            return (
              <circle
                key={`ring-${i}`}
                className="cf-center-ring"
                cx={CENTER.x}
                cy={CENTER.y}
                r={r}
                stroke={`url(#cf-ring-grad-${i + 1})`}
                strokeWidth={1.2 - i * 0.2}
                strokeDasharray={dashArray}
                fill="none"
                opacity="0"
                style={{ transformOrigin: `${CENTER.x}px ${CENTER.y}px` }}
              />
            );
          })}

        {/* Core filled circle */}
        <circle
          className="cf-center-core"
          cx={CENTER.x}
          cy={CENTER.y}
          r="20"
          fill="url(#cf-core-grad)"
          opacity="0"
          style={{ transformOrigin: `${CENTER.x}px ${CENTER.y}px` }}
        />

        {/* Aegis icon inside core — stylized "A" / shield hint */}
        <g
          className="cf-center-core"
          opacity="0"
          style={{ transformOrigin: `${CENTER.x}px ${CENTER.y}px` }}
        >
          {/* Shield outline inside core */}
          <path
            d={`M ${CENTER.x} ${CENTER.y - 11}
                L ${CENTER.x + 9} ${CENTER.y - 5}
                L ${CENTER.x + 9} ${CENTER.y + 3}
                C ${CENTER.x + 9} ${CENTER.y + 8}, ${CENTER.x + 4} ${CENTER.y + 12}, ${CENTER.x} ${CENTER.y + 13}
                C ${CENTER.x - 4} ${CENTER.y + 12}, ${CENTER.x - 9} ${CENTER.y + 8}, ${CENTER.x - 9} ${CENTER.y + 3}
                L ${CENTER.x - 9} ${CENTER.y - 5} Z`}
            stroke="#fff"
            strokeWidth="1"
            fill="none"
            opacity="0.8"
          />
          {/* Checkmark inside shield */}
          <path
            d={`M ${CENTER.x - 4} ${CENTER.y} L ${CENTER.x - 1} ${CENTER.y + 3} L ${CENTER.x + 5} ${CENTER.y - 4}`}
            stroke="#fff"
            strokeWidth="1.2"
            fill="none"
            strokeLinecap="round"
            strokeLinejoin="round"
            opacity="0.9"
          />
        </g>

        {/* Center label */}
        <text
          className="cf-center-label"
          x={CENTER.x}
          y={CENTER.y + 74}
          textAnchor="middle"
          fill="#c4b5fd"
          fontSize="8"
          fontFamily="monospace"
          letterSpacing="2"
          opacity="0"
        >
          ANALYZE
        </text>

        {/* ================================================================== */}
        {/*  Output flow paths                                                 */}
        {/* ================================================================== */}
        {OUTPUTS.map((out, i) => (
          <path
            key={`out-path-${i}`}
            className="cf-output-path"
            d={outputPath(out)}
            stroke="url(#cf-grad-output)"
            strokeWidth="1.2"
            fill="none"
            strokeLinecap="round"
            opacity="0.7"
          />
        ))}

        {/* ================================================================== */}
        {/*  Output result icons                                               */}
        {/* ================================================================== */}

        {/* Checkmark circle */}
        <g
          className="cf-output-icon"
          opacity="0"
          style={{ transformOrigin: `${OUTPUTS[0].x}px ${OUTPUTS[0].y}px` }}
        >
          <circle
            cx={OUTPUTS[0].x}
            cy={OUTPUTS[0].y}
            r="16"
            fill="none"
            stroke="url(#cf-check-grad)"
            strokeWidth="1.2"
          />
          <path
            d={`M ${OUTPUTS[0].x - 6} ${OUTPUTS[0].y} L ${OUTPUTS[0].x - 2} ${OUTPUTS[0].y + 5} L ${OUTPUTS[0].x + 7} ${OUTPUTS[0].y - 5}`}
            stroke="url(#cf-check-grad)"
            strokeWidth="1.8"
            fill="none"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
          <text
            x={OUTPUTS[0].x}
            y={OUTPUTS[0].y + 30}
            textAnchor="middle"
            fill="#34d399"
            fontSize="7"
            fontFamily="monospace"
            opacity="0.7"
          >
            CLEAN
          </text>
        </g>

        {/* Warning triangle */}
        <g
          className="cf-output-icon"
          opacity="0"
          style={{ transformOrigin: `${OUTPUTS[1].x}px ${OUTPUTS[1].y}px` }}
        >
          <path
            d={`M ${OUTPUTS[1].x} ${OUTPUTS[1].y - 14}
                L ${OUTPUTS[1].x + 15} ${OUTPUTS[1].y + 10}
                L ${OUTPUTS[1].x - 15} ${OUTPUTS[1].y + 10} Z`}
            fill="none"
            stroke="url(#cf-warn-grad)"
            strokeWidth="1.2"
            strokeLinejoin="round"
          />
          {/* Exclamation mark */}
          <line
            x1={OUTPUTS[1].x}
            y1={OUTPUTS[1].y - 7}
            x2={OUTPUTS[1].x}
            y2={OUTPUTS[1].y + 2}
            stroke="url(#cf-warn-grad)"
            strokeWidth="1.5"
            strokeLinecap="round"
          />
          <circle
            cx={OUTPUTS[1].x}
            cy={OUTPUTS[1].y + 6}
            r="1"
            fill="#fbbf24"
          />
          <text
            x={OUTPUTS[1].x}
            y={OUTPUTS[1].y + 30}
            textAnchor="middle"
            fill="#fbbf24"
            fontSize="7"
            fontFamily="monospace"
            opacity="0.7"
          >
            FINDING
          </text>
        </g>

        {/* ================================================================== */}
        {/*  Flow particles — input side                                       */}
        {/* ================================================================== */}
        {INPUT_FILES.flatMap((_, pathIdx) =>
          Array.from({ length: PARTICLES_PER_PATH }, (__, pIdx) => (
            <circle
              key={`p-in-${pathIdx}-${pIdx}`}
              className="cf-particle-in"
              r="2.5"
              fill="url(#cf-particle-glow-in)"
              filter="url(#cf-particle-blur)"
              opacity="0"
            />
          )),
        )}

        {/* ================================================================== */}
        {/*  Flow particles — output side                                      */}
        {/* ================================================================== */}
        {OUTPUTS.flatMap((_, pathIdx) =>
          Array.from({ length: PARTICLES_PER_PATH }, (__, pIdx) => (
            <circle
              key={`p-out-${pathIdx}-${pIdx}`}
              className="cf-particle-out"
              r="2.5"
              fill="url(#cf-particle-glow-out)"
              filter="url(#cf-particle-blur)"
              opacity="0"
            />
          )),
        )}

        {/* ================================================================== */}
        {/*  Decorative scanning lines around center                           */}
        {/* ================================================================== */}
        <line
          className="cf-center-core"
          x1={CENTER.x - 70}
          y1={CENTER.y}
          x2={CENTER.x - 60}
          y2={CENTER.y}
          stroke="#818cf8"
          strokeWidth="0.6"
          opacity="0"
          strokeLinecap="round"
        />
        <line
          className="cf-center-core"
          x1={CENTER.x + 60}
          y1={CENTER.y}
          x2={CENTER.x + 70}
          y2={CENTER.y}
          stroke="#818cf8"
          strokeWidth="0.6"
          opacity="0"
          strokeLinecap="round"
        />
        <line
          className="cf-center-core"
          x1={CENTER.x}
          y1={CENTER.y - 70}
          x2={CENTER.x}
          y2={CENTER.y - 60}
          stroke="#818cf8"
          strokeWidth="0.6"
          opacity="0"
          strokeLinecap="round"
        />
        <line
          className="cf-center-core"
          x1={CENTER.x}
          y1={CENTER.y + 60}
          x2={CENTER.x}
          y2={CENTER.y + 70}
          stroke="#818cf8"
          strokeWidth="0.6"
          opacity="0"
          strokeLinecap="round"
        />
      </svg>
    </div>
  );
}
