import { useEffect, useRef, useMemo } from 'react';
import { animate, stagger, createTimeline } from 'animejs';
import {
  ShieldCheck,
  ArrowsClockwise,
  Trash,
  LinkBreak,
  Robot,
  TreeStructure,
  Bug,
} from '@phosphor-icons/react';
import { prefersReducedMotion } from '@/hooks/useAnime';

/* -------------------------------------------------------------------------- */
/*  HeroIllustration — Orbiting Icon Constellation                            */
/* -------------------------------------------------------------------------- */

/**
 * A decorative animated illustration for the hero section.
 *
 * Layout: a central shield icon surrounded by six satellite icons arranged
 * in orbital rings. SVG connection lines radiate from center to each
 * satellite with animated dash-offset. Floating particles drift between
 * the elements. Everything animates via anime.js with full reduced-motion
 * respect.
 *
 * Orbit technique: each satellite lives inside a square wrapper div whose
 * width/height equals the orbit diameter. The wrapper is centered in the
 * container. The icon is placed at the top-center of the wrapper via
 * absolute positioning. Rotating the wrapper spins the icon around the
 * center. A counter-rotation on the icon itself keeps it upright.
 */

/* -- Satellite configuration ------------------------------------------------ */

interface SatelliteConfig {
  Icon: React.ElementType;
  /** Orbital radius as fraction of container half-size (0-1) */
  orbit: number;
  /** Starting angle in degrees (0 = top, clockwise) */
  startAngle: number;
  /** Full revolution duration in ms */
  duration: number;
  /** Tailwind text color class */
  color: string;
  /** Tailwind gradient classes for glow behind icon */
  glow: string;
}

const SATELLITES: SatelliteConfig[] = [
  {
    Icon: ArrowsClockwise,
    orbit: 0.64,
    startAngle: 30,
    duration: 12000,
    color: 'text-indigo-400',
    glow: 'from-indigo-500/25 to-indigo-400/5',
  },
  {
    Icon: Trash,
    orbit: 0.82,
    startAngle: 90,
    duration: 15000,
    color: 'text-purple-400',
    glow: 'from-purple-500/25 to-purple-400/5',
  },
  {
    Icon: LinkBreak,
    orbit: 0.68,
    startAngle: 150,
    duration: 10000,
    color: 'text-pink-400',
    glow: 'from-pink-500/25 to-pink-400/5',
  },
  {
    Icon: Robot,
    orbit: 0.84,
    startAngle: 210,
    duration: 13000,
    color: 'text-violet-400',
    glow: 'from-violet-500/25 to-violet-400/5',
  },
  {
    Icon: TreeStructure,
    orbit: 0.62,
    startAngle: 270,
    duration: 9000,
    color: 'text-fuchsia-400',
    glow: 'from-fuchsia-500/25 to-fuchsia-400/5',
  },
  {
    Icon: Bug,
    orbit: 0.78,
    startAngle: 330,
    duration: 11000,
    color: 'text-rose-400',
    glow: 'from-rose-500/25 to-rose-400/5',
  },
];

/** Unique orbital radii for drawing ring tracks */
const RING_ORBITS = [0.62, 0.68, 0.78, 0.84];

/* -- Floating particle seeds ------------------------------------------------ */

interface ParticleSeed {
  /** Percentage x position (0-100) */
  cx: number;
  /** Percentage y position (0-100) */
  cy: number;
  /** Radius in px */
  r: number;
  /** Base opacity */
  opacity: number;
}

function generateParticleSeeds(count: number): ParticleSeed[] {
  const seeds: ParticleSeed[] = [];
  for (let i = 0; i < count; i++) {
    seeds.push({
      cx: 8 + Math.random() * 84,
      cy: 8 + Math.random() * 84,
      r: 1 + Math.random() * 2,
      opacity: 0.12 + Math.random() * 0.3,
    });
  }
  return seeds;
}

/* -- SVG coordinate helpers ------------------------------------------------- */

const SVG_SIZE = 400;
const SVG_CENTER = SVG_SIZE / 2;
const SVG_MAX_R = SVG_SIZE / 2; // 200

/** Convert orbit fraction + angle to SVG coordinates */
function toSvgPos(orbit: number, angleDeg: number) {
  const r = orbit * SVG_MAX_R;
  // Offset by -90 so 0deg = top
  const rad = ((angleDeg - 90) * Math.PI) / 180;
  return {
    x: SVG_CENTER + r * Math.cos(rad),
    y: SVG_CENTER + r * Math.sin(rad),
  };
}

/* -- Component -------------------------------------------------------------- */

export default function HeroIllustration() {
  const containerRef = useRef<HTMLDivElement>(null);
  const centerRef = useRef<HTMLDivElement>(null);
  const centerGlowRef = useRef<HTMLDivElement>(null);
  const svgRef = useRef<SVGSVGElement>(null);
  const orbitWrapperRefs = useRef<(HTMLDivElement | null)[]>([]);
  const iconInnerRefs = useRef<(HTMLDivElement | null)[]>([]);
  const particleRefs = useRef<(HTMLDivElement | null)[]>([]);
  const ringRefs = useRef<(HTMLDivElement | null)[]>([]);

  const particleSeeds = useMemo(() => generateParticleSeeds(20), []);

  const isReduced = prefersReducedMotion();

  /* -- Animations ---------------------------------------------------------- */

  useEffect(() => {
    if (!containerRef.current || prefersReducedMotion()) return;

    const anims: ReturnType<typeof animate>[] = [];
    const timeline = createTimeline({ autoplay: true });

    /* ---- Phase 1: Entry stagger ---- */

    // Center icon scales in
    if (centerRef.current) {
      timeline.add(centerRef.current, {
        opacity: [0, 1],
        scale: [0.3, 1],
        duration: 700,
        ease: 'outBack',
      }, 0);
    }

    // Center glow fades in
    if (centerGlowRef.current) {
      timeline.add(centerGlowRef.current, {
        opacity: [0, 0.7],
        scale: [0.5, 1],
        duration: 900,
        ease: 'outQuart',
      }, 150);
    }

    // Orbital ring tracks scale in
    const rings = ringRefs.current.filter(Boolean) as HTMLDivElement[];
    if (rings.length) {
      timeline.add(rings, {
        opacity: [0, 1],
        scale: [0.4, 1],
        duration: 700,
        delay: stagger(60),
        ease: 'outQuart',
      }, 250);
    }

    // SVG connection lines fade in
    if (svgRef.current) {
      const lines = svgRef.current.querySelectorAll<SVGLineElement>('.connection-line');
      if (lines.length) {
        timeline.add(lines, {
          opacity: [0, 0.6],
          duration: 600,
          delay: stagger(50),
          ease: 'outQuart',
        }, 400);
      }
    }

    // Satellite orbit wrappers pop in
    const wrappers = orbitWrapperRefs.current.filter(Boolean) as HTMLDivElement[];
    if (wrappers.length) {
      timeline.add(wrappers, {
        opacity: [0, 1],
        scale: [0.2, 1],
        duration: 550,
        delay: stagger(70),
        ease: 'outBack',
      }, 500);
    }

    // Particles fade in
    const particles = particleRefs.current.filter(Boolean) as HTMLDivElement[];
    if (particles.length) {
      timeline.add(particles, {
        opacity: [0, (_: unknown, i: number) => particleSeeds[i]?.opacity ?? 0.2],
        duration: 800,
        delay: stagger(25),
        ease: 'outQuart',
      }, 600);
    }

    /* ---- Phase 2: Continuous loops (after entry finishes) ---- */

    timeline.then(() => {

      // Central icon: gentle breathing pulse
      if (centerRef.current) {
        anims.push(
          animate(centerRef.current, {
            scale: [1, 1.06],
            duration: 3000,
            ease: 'inOutSine',
            loop: true,
            alternate: true,
          }),
        );
      }

      // Center glow: breathing
      if (centerGlowRef.current) {
        anims.push(
          animate(centerGlowRef.current, {
            opacity: [0.7, 0.3],
            scale: [1, 1.2],
            duration: 3500,
            ease: 'inOutSine',
            loop: true,
            alternate: true,
          }),
        );
      }

      // Orbital ring tracks: very slow rotation
      rings.forEach((ring, i) => {
        anims.push(
          animate(ring, {
            rotate: i % 2 === 0 ? [0, 360] : [0, -360],
            duration: 45000 + i * 10000,
            ease: 'linear',
            loop: true,
          }),
        );
      });

      // SVG connection line dash animation
      if (svgRef.current) {
        const lines = svgRef.current.querySelectorAll<SVGLineElement>('.connection-line');
        lines.forEach((line) => {
          anims.push(
            animate(line, {
              strokeDashoffset: [0, -20],
              duration: 1800,
              ease: 'linear',
              loop: true,
            }),
          );
        });
      }

      // Satellite orbiting: rotate the wrapper, counter-rotate the icon
      wrappers.forEach((wrapper, i) => {
        const config = SATELLITES[i];
        if (!config) return;

        anims.push(
          animate(wrapper, {
            rotate: [config.startAngle, config.startAngle + 360],
            duration: config.duration,
            ease: 'linear',
            loop: true,
          }),
        );

        const inner = iconInnerRefs.current[i];
        if (inner) {
          anims.push(
            animate(inner, {
              rotate: [-config.startAngle, -(config.startAngle + 360)],
              duration: config.duration,
              ease: 'linear',
              loop: true,
            }),
          );
        }
      });

      // Floating particles: gentle random drift
      particles.forEach((p, i) => {
        const dx = 6 + Math.random() * 14;
        const dy = 6 + Math.random() * 14;
        const baseOpacity = particleSeeds[i]?.opacity ?? 0.2;
        anims.push(
          animate(p, {
            translateX: [-dx, dx],
            translateY: [-dy, dy],
            opacity: [baseOpacity, baseOpacity * 0.3],
            duration: 4000 + Math.random() * 5000,
            ease: 'inOutSine',
            loop: true,
            alternate: true,
          }),
        );
      });
    });

    return () => {
      timeline.pause();
      anims.forEach((a) => a.pause());
    };
  }, [particleSeeds]);

  /* -- SVG satellite positions for connection lines ------------------------- */

  const svgPositions = SATELLITES.map((s) => toSvgPos(s.orbit, s.startAngle));

  /* -- Render --------------------------------------------------------------- */

  return (
    <div
      ref={containerRef}
      className="relative mx-auto pointer-events-none select-none w-[180px] h-[180px] sm:w-[220px] sm:h-[220px] md:w-[260px] md:h-[260px]"
      aria-hidden="true"
    >
    <div className="relative w-full h-full"
    >

      {/* --- Orbital ring tracks (decorative circles) --- */}
      {RING_ORBITS.map((orbit, i) => {
        const diameterPct = orbit * 100; // orbit as % of container
        return (
          <div
            key={`ring-${i}`}
            ref={(el) => { ringRefs.current[i] = el; }}
            className="absolute rounded-full border border-indigo-500/[0.07] dark:border-indigo-400/[0.05]"
            style={{
              width: `${diameterPct}%`,
              height: `${diameterPct}%`,
              top: `${(100 - diameterPct) / 2}%`,
              left: `${(100 - diameterPct) / 2}%`,
              opacity: isReduced ? 0.6 : 0,
              willChange: isReduced ? 'auto' : 'transform',
            }}
          />
        );
      })}

      {/* --- SVG layer: connection lines + gradient defs --- */}
      <svg
        ref={svgRef}
        viewBox={`0 0 ${SVG_SIZE} ${SVG_SIZE}`}
        className="absolute inset-0 w-full h-full"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <defs>
          <linearGradient id="hero-conn-grad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#818cf8" stopOpacity="0.6" />
            <stop offset="50%" stopColor="#a78bfa" stopOpacity="0.4" />
            <stop offset="100%" stopColor="#f472b6" stopOpacity="0.25" />
          </linearGradient>
          <radialGradient id="hero-center-dot" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="#818cf8" stopOpacity="0.8" />
            <stop offset="100%" stopColor="#818cf8" stopOpacity="0" />
          </radialGradient>
        </defs>

        {/* Soft glow at center (SVG-level) */}
        <circle cx={SVG_CENTER} cy={SVG_CENTER} r="6" fill="url(#hero-center-dot)" />

        {/* Connection lines from center to each satellite */}
        {svgPositions.map((pos, i) => (
          <line
            key={`conn-${i}`}
            className="connection-line"
            x1={SVG_CENTER}
            y1={SVG_CENTER}
            x2={pos.x}
            y2={pos.y}
            stroke="url(#hero-conn-grad)"
            strokeWidth="0.7"
            strokeDasharray="5 4"
            strokeLinecap="round"
            opacity={isReduced ? 0.4 : 0}
          />
        ))}

        {/* Small accent dots at satellite positions */}
        {svgPositions.map((pos, i) => (
          <circle
            key={`dot-${i}`}
            cx={pos.x}
            cy={pos.y}
            r="2"
            fill="url(#hero-center-dot)"
            className="connection-line"
            opacity={isReduced ? 0.5 : 0}
          />
        ))}
      </svg>

      {/* --- Floating particles --- */}
      {particleSeeds.map((seed, i) => (
        <div
          key={`p-${i}`}
          ref={(el) => { particleRefs.current[i] = el; }}
          className="absolute rounded-full bg-indigo-400/50 dark:bg-indigo-300/40"
          style={{
            width: seed.r * 2,
            height: seed.r * 2,
            left: `${seed.cx}%`,
            top: `${seed.cy}%`,
            opacity: isReduced ? seed.opacity * 0.5 : 0,
            willChange: isReduced ? 'auto' : 'transform',
          }}
        />
      ))}

      {/* --- Center glow (blurred radial gradient) --- */}
      <div
        ref={centerGlowRef}
        className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[38%] h-[38%] rounded-full pointer-events-none"
        style={{
          background: 'radial-gradient(circle, rgba(99,102,241,0.35) 0%, rgba(168,85,247,0.18) 35%, rgba(236,72,153,0.08) 60%, transparent 80%)',
          filter: 'blur(30px)',
          opacity: isReduced ? 0.4 : 0,
          willChange: isReduced ? 'auto' : 'transform, opacity',
        }}
      />

      {/* --- Central shield icon --- */}
      <div
        ref={centerRef}
        className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 flex items-center justify-center z-10"
        style={{
          opacity: isReduced ? 1 : 0,
          willChange: isReduced ? 'auto' : 'transform',
        }}
      >
        {/* Diffuse glow ring */}
        <div className="absolute w-12 h-12 sm:w-14 sm:h-14 md:w-16 md:h-16 rounded-full bg-gradient-to-br from-indigo-500/20 via-purple-500/15 to-pink-500/10 blur-lg" />

        {/* Glass card */}
        <div className="relative w-10 h-10 sm:w-12 sm:h-12 md:w-14 md:h-14 rounded-xl sm:rounded-2xl bg-white/10 dark:bg-white/[0.06] backdrop-blur-xl border border-white/25 dark:border-white/10 flex items-center justify-center shadow-2xl shadow-indigo-500/15">
          <ShieldCheck
            size={28}
            weight="duotone"
            className="text-indigo-400 dark:text-indigo-300 w-5 h-5 sm:w-6 sm:h-6 md:w-7 md:h-7"
          />
        </div>
      </div>

      {/* --- Orbiting satellite icons --- */}
      {/*
        Each satellite is inside a square wrapper div whose side = orbit diameter
        (as % of container). The wrapper is centered. The icon sits at the
        top-center of the wrapper. Rotating the wrapper orbits the icon.
        A counter-rotation on the inner keeps the icon upright.
      */}
      {SATELLITES.map((sat, i) => {
        const diameterPct = sat.orbit * 100;
        // Icon card size: ~44px (11 * 4 tailwind), offset by half to center at edge
        return (
          <div
            key={`orbit-${i}`}
            ref={(el) => { orbitWrapperRefs.current[i] = el; }}
            className="absolute"
            style={{
              width: `${diameterPct}%`,
              height: `${diameterPct}%`,
              top: `${(100 - diameterPct) / 2}%`,
              left: `${(100 - diameterPct) / 2}%`,
              transform: `rotate(${sat.startAngle}deg)`,
              opacity: isReduced ? 1 : 0,
              willChange: isReduced ? 'auto' : 'transform',
            }}
          >
            {/* Icon positioned at top-center of orbit wrapper */}
            <div
              ref={(el) => { iconInnerRefs.current[i] = el; }}
              className="absolute left-1/2 top-0 flex items-center justify-center"
              style={{
                transform: `translate(-50%, -50%) rotate(-${sat.startAngle}deg)`,
                willChange: isReduced ? 'auto' : 'transform',
              }}
            >
              <div className="relative">
                {/* Glow behind icon */}
                <div
                  className={`absolute -inset-1 rounded-full bg-gradient-to-br ${sat.glow} blur-sm`}
                />

                {/* Icon glass card */}
                <div className="relative w-6 h-6 sm:w-7 sm:h-7 md:w-8 md:h-8 rounded-lg bg-white/10 dark:bg-white/[0.06] backdrop-blur-lg border border-white/20 dark:border-white/10 flex items-center justify-center shadow-lg shadow-black/5 dark:shadow-black/20">
                  <sat.Icon
                    size={16}
                    weight="duotone"
                    className={`${sat.color} w-3.5 h-3.5 sm:w-4 sm:h-4`}
                  />
                </div>
              </div>
            </div>
          </div>
        );
      })}
    </div>
    </div>
  );
}
