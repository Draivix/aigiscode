import { useEffect, useRef, useMemo } from 'react';
import { animate } from 'animejs';
import { prefersReducedMotion } from '@/hooks/useAnime';

/* -------------------------------------------------------------------------- */
/*  Floating Particle Constellation                                           */
/* -------------------------------------------------------------------------- */

interface Particle {
  x: number;
  y: number;
  el: HTMLDivElement | null;
}

interface ParticleFieldProps {
  /** Number of particles (default 40) */
  count?: number;
  /** Max line connection distance in px (default 120) */
  connectionDistance?: number;
  /** CSS class for the container */
  className?: string;
}

export default function ParticleField({
  count = 40,
  connectionDistance = 120,
  className = '',
}: ParticleFieldProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const particlesRef = useRef<Particle[]>([]);
  const rafRef = useRef<number>(0);
  const animsRef = useRef<ReturnType<typeof animate>[]>([]);

  // Generate stable random initial positions
  const initialPositions = useMemo(() => {
    return Array.from({ length: count }, () => ({
      x: Math.random() * 100,
      y: Math.random() * 100,
    }));
  }, [count]);

  useEffect(() => {
    if (!containerRef.current || !canvasRef.current) return;
    if (prefersReducedMotion()) return;

    const container = containerRef.current;
    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Size canvas to container
    const resizeCanvas = () => {
      const rect = container.getBoundingClientRect();
      canvas.width = rect.width * window.devicePixelRatio;
      canvas.height = rect.height * window.devicePixelRatio;
      canvas.style.width = `${rect.width}px`;
      canvas.style.height = `${rect.height}px`;
      ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
    };
    resizeCanvas();

    // Create particle data objects (anime.js will animate these plain objects)
    const particles: Particle[] = initialPositions.map((pos) => ({
      x: (pos.x / 100) * container.offsetWidth,
      y: (pos.y / 100) * container.offsetHeight,
      el: null,
    }));
    particlesRef.current = particles;

    // Animate each particle to drift around
    const anims = particles.map((p) => {
      const w = container.offsetWidth;
      const h = container.offsetHeight;
      const driftRange = 80;
      return animate(p, {
        x: [p.x, p.x + (Math.random() - 0.5) * driftRange * 2],
        y: [p.y, p.y + (Math.random() - 0.5) * driftRange * 2],
        duration: 4000 + Math.random() * 6000,
        ease: 'inOutSine',
        loop: true,
        alternate: true,
        // Keep within bounds via modifier
        modifier: (v: number) => {
          const maxDim = Math.max(w, h);
          return ((v % maxDim) + maxDim) % maxDim;
        },
      });
    });
    animsRef.current = anims;

    // Draw connections on a rAF loop
    const containerWidth = container.offsetWidth;
    const containerHeight = container.offsetHeight;
    const maxDistSq = connectionDistance * connectionDistance;

    // Get computed style for dark mode detection
    const isDark = () => document.documentElement.classList.contains('dark');

    function drawFrame() {
      if (!ctx) return;
      // Clear at physical resolution
      ctx.setTransform(1, 0, 0, 1, 0, 0);
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const dpr = window.devicePixelRatio;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

      const dark = isDark();

      // Draw connection lines
      for (let i = 0; i < particles.length; i++) {
        for (let j = i + 1; j < particles.length; j++) {
          const dx = particles[i].x - particles[j].x;
          const dy = particles[i].y - particles[j].y;
          const distSq = dx * dx + dy * dy;
          if (distSq < maxDistSq) {
            const dist = Math.sqrt(distSq);
            const opacity = (1 - dist / connectionDistance) * 0.2;
            ctx.beginPath();
            ctx.moveTo(particles[i].x, particles[i].y);
            ctx.lineTo(particles[j].x, particles[j].y);
            ctx.strokeStyle = dark
              ? `rgba(129, 140, 248, ${opacity})`
              : `rgba(99, 102, 241, ${opacity})`;
            ctx.lineWidth = 0.5;
            ctx.stroke();
          }
        }
      }

      // Draw particles as dots
      for (const p of particles) {
        ctx.beginPath();
        ctx.arc(p.x, p.y, 1.5, 0, Math.PI * 2);
        ctx.fillStyle = dark
          ? 'rgba(165, 180, 252, 0.4)'
          : 'rgba(99, 102, 241, 0.25)';
        ctx.fill();
      }

      rafRef.current = requestAnimationFrame(drawFrame);
    }

    rafRef.current = requestAnimationFrame(drawFrame);

    // Handle resize
    const handleResize = () => {
      resizeCanvas();
    };
    window.addEventListener('resize', handleResize);

    return () => {
      cancelAnimationFrame(rafRef.current);
      anims.forEach((a) => a.pause());
      window.removeEventListener('resize', handleResize);
    };
  }, [initialPositions, connectionDistance, count]);

  if (prefersReducedMotion()) return null;

  return (
    <div
      ref={containerRef}
      className={`absolute inset-0 overflow-hidden pointer-events-none ${className}`}
      aria-hidden="true"
    >
      <canvas
        ref={canvasRef}
        className="absolute inset-0 w-full h-full"
      />
    </div>
  );
}
