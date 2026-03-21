import { useEffect, useRef, useMemo } from 'react';
import { animate } from 'animejs';
import { prefersReducedMotion } from '@/hooks/useAnime';
import type { IconProps } from '@phosphor-icons/react';
import {
  ShieldCheck,
  Code,
  Terminal,
  GitBranch,
  FileCode,
  Database,
  Gear,
  Lightning,
  Graph,
  TreeStructure,
  MagnifyingGlass,
  Lock,
  Plugs,
  Bug,
  Stack,
  CloudArrowUp,
  CirclesThree,
  Cube,
  Browsers,
  GitPullRequest,
} from '@phosphor-icons/react';

/* -------------------------------------------------------------------------- */
/*  FloatingIcons — Scattered background decoration with anime.js             */
/* -------------------------------------------------------------------------- */

type IconComponent = React.ComponentType<IconProps>;

/** Preset icon pools keyed by theme */
const ICON_SETS: Record<string, IconComponent[]> = {
  analysis: [
    ShieldCheck, Graph, TreeStructure, MagnifyingGlass,
    Bug, CirclesThree, Lock, Stack,
  ],
  code: [
    Code, Terminal, GitBranch, FileCode,
    Database, Gear, Plugs, GitPullRequest,
  ],
  all: [
    ShieldCheck, Code, Terminal, GitBranch,
    FileCode, Database, Gear, Lightning,
    Graph, TreeStructure, MagnifyingGlass, Lock,
    Plugs, Bug, Stack, CloudArrowUp,
    CirclesThree, Cube, Browsers, GitPullRequest,
  ],
};

interface FloatingIconsProps {
  /** Number of icons to render (default 8) */
  count?: number;
  /** Container positioning class */
  className?: string;
  /** Preset icon set (default 'all') */
  iconSet?: 'analysis' | 'code' | 'all';
}

/* ---- deterministic pseudo-random from seed ---- */
function seededRandom(seed: number): () => number {
  let s = seed;
  return () => {
    s = (s * 16807 + 0) % 2147483647;
    return (s - 1) / 2147483646;
  };
}

/* ---- pick N icons without consecutive duplicates ---- */
function pickIcons(pool: IconComponent[], count: number, rand: () => number): IconComponent[] {
  const result: IconComponent[] = [];
  let lastIdx = -1;
  for (let i = 0; i < count; i++) {
    let idx: number;
    do {
      idx = Math.floor(rand() * pool.length);
    } while (idx === lastIdx && pool.length > 1);
    lastIdx = idx;
    result.push(pool[idx]);
  }
  return result;
}

/* ---- lerp helper ---- */
function lerp(min: number, max: number, t: number): number {
  return min + (max - min) * t;
}

interface IconConfig {
  Icon: IconComponent;
  top: number;         // percent
  left: number;        // percent
  size: number;        // px
  driftX: number;      // px range
  driftY: number;      // px range
  rotate: number;      // degrees (0 = no rotation)
  duration: number;    // ms
  delay: number;       // ms
  opacityLight: number; // 0.05-0.12
  opacityDark: number;  // 0.08-0.15
}

/**
 * Renders N icons scattered randomly across the container, each floating
 * with a gentle independent drift animation via anime.js.
 *
 * Icons are very subtle (low opacity, thin weight) and purely decorative.
 * Respects prefers-reduced-motion. Fully pointer-events-none and aria-hidden.
 *
 * Usage:
 * ```tsx
 * <div className="relative">
 *   <FloatingIcons className="absolute inset-0" iconSet="analysis" count={10} />
 *   {children}
 * </div>
 * ```
 */
export default function FloatingIcons({
  count = 8,
  className = '',
  iconSet = 'all',
}: FloatingIconsProps) {
  const iconRefs = useRef<(HTMLSpanElement | null)[]>([]);

  /* Build stable icon configs — deterministic from count + iconSet */
  const configs = useMemo<IconConfig[]>(() => {
    const pool = ICON_SETS[iconSet] ?? ICON_SETS.all;
    const seed = count * 7 + (iconSet === 'analysis' ? 1 : iconSet === 'code' ? 2 : 3);
    const rand = seededRandom(seed);
    const icons = pickIcons(pool, count, rand);

    return icons.map((Icon, i) => {
      /* Distribute positions in a grid with jitter to avoid clustering */
      const cols = Math.ceil(Math.sqrt(count));
      const rows = Math.ceil(count / cols);
      const col = i % cols;
      const row = Math.floor(i / cols);
      const cellW = 100 / cols;
      const cellH = 100 / rows;

      /* Place within 10%-85% of cell bounds */
      const top = cellH * row + lerp(cellH * 0.1, cellH * 0.85, rand());
      const left = cellW * col + lerp(cellW * 0.1, cellW * 0.85, rand());

      const opacityLight = lerp(0.05, 0.12, rand());

      return {
        Icon,
        top: Math.min(top, 92),
        left: Math.min(left, 92),
        size: Math.round(lerp(16, 28, rand())),
        driftX: lerp(8, 24, rand()) * (rand() > 0.5 ? 1 : -1),
        driftY: lerp(6, 20, rand()) * (rand() > 0.5 ? 1 : -1),
        rotate: rand() > 0.4 ? lerp(8, 25, rand()) * (rand() > 0.5 ? 1 : -1) : 0,
        duration: Math.round(lerp(3000, 8000, rand())),
        delay: Math.round(lerp(0, 2000, rand())),
        opacityLight,
        opacityDark: Math.min(opacityLight + 0.04, 0.15),
      };
    });
  }, [count, iconSet]);

  /* Keep refs array sized correctly */
  iconRefs.current = iconRefs.current.slice(0, configs.length);

  /* Start anime.js drift animations */
  useEffect(() => {
    if (prefersReducedMotion()) return;

    const anims: ReturnType<typeof animate>[] = [];

    configs.forEach((cfg, i) => {
      const el = iconRefs.current[i];
      if (!el) return;

      anims.push(
        animate(el, {
          translateX: [-cfg.driftX, cfg.driftX],
          translateY: [-cfg.driftY, cfg.driftY],
          ...(cfg.rotate !== 0 ? { rotate: [-cfg.rotate, cfg.rotate] } : {}),
          duration: cfg.duration,
          delay: cfg.delay,
          ease: 'inOutSine',
          loop: true,
          alternate: true,
        }),
      );
    });

    return () => {
      anims.forEach((a) => a.pause());
    };
  }, [configs]);

  if (prefersReducedMotion()) return null;

  return (
    <div
      className={`pointer-events-none select-none overflow-hidden ${className}`}
      aria-hidden="true"
    >
      {configs.map((cfg, i) => {
        const { Icon } = cfg;

        return (
          <span
            key={`fi-${i}`}
            ref={(el) => { iconRefs.current[i] = el; }}
            className="absolute will-change-transform"
            style={{
              top: `${cfg.top}%`,
              left: `${cfg.left}%`,
            }}
          >
            {/* Light mode icon */}
            <Icon
              weight="thin"
              className="block dark:hidden text-indigo-500"
              style={{
                width: cfg.size,
                height: cfg.size,
                opacity: cfg.opacityLight,
              }}
            />
            {/* Dark mode icon — slightly higher opacity */}
            <Icon
              weight="thin"
              className="hidden dark:block text-indigo-400"
              style={{
                width: cfg.size,
                height: cfg.size,
                opacity: cfg.opacityDark,
              }}
            />
          </span>
        );
      })}
    </div>
  );
}
