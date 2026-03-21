import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import Graph from 'graphology';
import forceAtlas2 from 'graphology-layout-forceatlas2';
import noverlap from 'graphology-layout-noverlap';
import Sigma from 'sigma';
import EdgeCurveProgram from '@sigma/edge-curve';
import {
  ArrowsClockwise,
  CornersOut,
  CornersIn,
  CursorClick,
  HandGrabbing,
  MagnifyingGlassMinus,
  MagnifyingGlassPlus,
  Pulse,
} from '@phosphor-icons/react';
import type { AtlasEdge, AtlasNode, ArchitectureSurface } from '../../types/architecture';

interface RepositoryAtlasProps {
  surface: ArchitectureSurface;
  highlightedPaths: string[];
  focusedPath: string | null;
  onSelectPath: (filePath: string) => void;
}

interface AtlasNodeAttributes {
  x: number;
  y: number;
  size: number;
  color: string;
  label: string;
  baseLabel: string;
  filePath: string;
}

interface AtlasEdgeAttributes {
  size: number;
  color: string;
  label: string;
  type: 'curved';
}

const LANGUAGE_COLORS: Record<string, string> = {
  TypeScript: '#f59e0b',
  JavaScript: '#facc15',
  Python: '#14b8a6',
  PHP: '#fb7185',
  Ruby: '#f97316',
  Rust: '#93c5fd',
};

const TIER_COLORS: Record<string, string> = {
  ImportScoped: 'rgba(125, 211, 252, 0.56)',
  SameFile: 'rgba(250, 204, 21, 0.58)',
  Global: 'rgba(148, 163, 184, 0.26)',
};

function groupKey(filePath: string): string {
  return filePath.split('/')[0] ?? 'root';
}

function languageColor(language: string): string {
  return LANGUAGE_COLORS[language] ?? '#94a3b8';
}

function buildSeedLayout(nodes: AtlasNode[]): Map<string, { x: number; y: number }> {
  const groups = new Map<string, AtlasNode[]>();
  for (const node of nodes) {
    const key = groupKey(node.filePath);
    const bucket = groups.get(key);
    if (bucket) {
      bucket.push(node);
    } else {
      groups.set(key, [node]);
    }
  }

  const entries = Array.from(groups.entries()).sort(([left], [right]) =>
    left.localeCompare(right),
  );
  const positions = new Map<string, { x: number; y: number }>();
  const spread = Math.max(Math.sqrt(nodes.length) * 2.6, 8);

  entries.forEach(([_, bucket], groupIndex) => {
    const angle = (Math.PI * 2 * groupIndex) / Math.max(entries.length, 1) - Math.PI / 2;
    const centerX = Math.cos(angle) * spread;
    const centerY = Math.sin(angle) * spread;
    const ordered = [...bucket].sort((left, right) =>
      right.findingCount - left.findingCount ||
      right.bottleneckCentralityMillis - left.bottleneckCentralityMillis ||
      left.filePath.localeCompare(right.filePath),
    );

    ordered.forEach((node, nodeIndex) => {
      const ring = Math.floor(nodeIndex / 6);
      const slot = nodeIndex % 6;
      const localAngle = (Math.PI * 2 * slot) / 6;
      const distance = 1.2 + ring * 0.9;
      positions.set(node.filePath, {
        x: centerX + Math.cos(localAngle) * distance,
        y: centerY + Math.sin(localAngle) * distance,
      });
    });
  });

  return positions;
}

function nodeSize(node: AtlasNode): number {
  return (
    7 +
    node.findingCount * 1.2 +
    Math.min(node.bottleneckCentralityMillis / 180, 7) +
    Math.min((node.inboundEdges + node.outboundEdges) / 10, 5)
  );
}

function edgeColor(
  edge: AtlasEdge,
  highlightedSet: Set<string>,
  focusedPath: string | null,
): string {
  const touchesFocused =
    focusedPath !== null &&
    (edge.sourceFilePath === focusedPath || edge.targetFilePath === focusedPath);
  if (touchesFocused) {
    return '#f8fafc';
  }
  if (highlightedSet.has(edge.sourceFilePath) || highlightedSet.has(edge.targetFilePath)) {
    return '#f59e0b';
  }
  return TIER_COLORS[edge.strongestResolutionTier] ?? 'rgba(148, 163, 184, 0.22)';
}

function makeGraph(
  surface: ArchitectureSurface,
  highlightedSet: Set<string>,
  focusedPath: string | null,
): Graph<AtlasNodeAttributes, AtlasEdgeAttributes> {
  const graph = new Graph<AtlasNodeAttributes, AtlasEdgeAttributes>();
  const positions = buildSeedLayout(surface.atlas.nodes);

  for (const node of surface.atlas.nodes) {
    const position = positions.get(node.filePath) ?? { x: 0, y: 0 };
    const isFocused = focusedPath === node.filePath;
    const isHighlighted = highlightedSet.has(node.filePath);
    const label =
      isFocused || isHighlighted || node.findingCount > 0 || node.bottleneckCentralityMillis > 350
        ? node.filePath.split('/').at(-1) ?? node.filePath
        : '';

    graph.addNode(node.filePath, {
      x: position.x,
      y: position.y,
      size: isFocused ? nodeSize(node) + 3 : nodeSize(node),
      color: isFocused
        ? '#f8fafc'
        : isHighlighted
          ? '#f59e0b'
          : languageColor(node.language),
      label,
      baseLabel: node.filePath.split('/').at(-1) ?? node.filePath,
      filePath: node.filePath,
    });
  }

  for (const [index, edge] of surface.atlas.edges.entries()) {
    if (!graph.hasNode(edge.sourceFilePath) || !graph.hasNode(edge.targetFilePath)) {
      continue;
    }

    graph.addEdgeWithKey(
      `${edge.sourceFilePath}->${edge.targetFilePath}:${index}`,
      edge.sourceFilePath,
      edge.targetFilePath,
      {
        size: Math.max(1, Math.min(edge.edgeCount, 5)),
        color: edgeColor(edge, highlightedSet, focusedPath),
        label: `${edge.kinds.join(', ')} · ${edge.strongestResolutionTier}`,
        type: 'curved',
      },
    );
  }

  const inferred = forceAtlas2.inferSettings(graph);
  forceAtlas2.assign(graph, {
    iterations: graph.order > 220 ? 180 : 140,
    settings: {
      ...inferred,
      barnesHutOptimize: graph.order > 120,
      strongGravityMode: false,
      gravity: graph.order > 220 ? 0.18 : 0.24,
      scalingRatio: graph.order > 220 ? 20 : 14,
      slowDown: graph.order > 220 ? 5 : 3,
      adjustSizes: true,
    },
  });
  noverlap.assign(graph, {
    maxIterations: 80,
    settings: {
      margin: 6,
      ratio: 1.12,
    },
  });

  return graph;
}

export default function RepositoryAtlas({
  surface,
  highlightedPaths,
  focusedPath,
  onSelectPath,
}: RepositoryAtlasProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const sigmaRef = useRef<Sigma | null>(null);
  const graphRef = useRef<Graph<AtlasNodeAttributes, AtlasEdgeAttributes> | null>(null);
  const highlightedSet = useMemo(() => new Set(highlightedPaths), [highlightedPaths]);
  const [hoveredNode, setHoveredNode] = useState<string | null>(null);
  const [renderMode, setRenderMode] = useState<'atlas' | 'focus'>('atlas');

  const refreshCamera = useCallback((mode: 'atlas' | 'focus') => {
    const sigma = sigmaRef.current;
    if (!sigma) {
      return;
    }
    if (mode === 'focus' && focusedPath && graphRef.current?.hasNode(focusedPath)) {
      const attributes = graphRef.current.getNodeAttributes(focusedPath);
      sigma.getCamera().animate(
        { x: attributes.x, y: attributes.y, ratio: 0.45 },
        { duration: 450 },
      );
      return;
    }
    sigma.getCamera().animatedReset({ duration: 450 });
  }, [focusedPath]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) {
      return undefined;
    }

    const graph = makeGraph(surface, highlightedSet, focusedPath);
    graphRef.current = graph;

    const sigma = new Sigma(graph, container, {
      allowInvalidContainer: false,
      renderLabels: true,
      labelRenderedSizeThreshold: 9,
      labelDensity: 0.08,
      labelGridCellSize: 80,
      labelFont: 'Space Grotesk, sans-serif',
      defaultEdgeType: 'curved',
      edgeProgramClasses: {
        curved: EdgeCurveProgram,
      },
      minCameraRatio: 0.12,
      maxCameraRatio: 3.2,
      zIndex: true,
    });

    sigma.on('clickNode', ({ node }) => {
      onSelectPath(String(node));
      setRenderMode('focus');
    });
    sigma.on('enterNode', ({ node }) => {
      setHoveredNode(String(node));
    });
    sigma.on('leaveNode', () => {
      setHoveredNode(null);
    });

    sigmaRef.current = sigma;
    refreshCamera(renderMode);

    return () => {
      sigma.kill();
      sigmaRef.current = null;
      graph.clear();
      graphRef.current = null;
    };
  }, [focusedPath, highlightedSet, onSelectPath, refreshCamera, renderMode, surface]);

  const hoveredLabel = hoveredNode
    ? graphRef.current?.getNodeAttribute(hoveredNode, 'filePath') ?? hoveredNode
    : null;

  return (
    <div className="relative h-[480px] w-full overflow-hidden rounded-[2rem] border border-white/10 bg-[#071018] shadow-[0_24px_80px_rgba(0,0,0,0.35)]">
      <div
        className="absolute inset-0 opacity-60"
        style={{
          backgroundImage:
            'radial-gradient(circle at center, rgba(20,184,166,0.08), transparent 28%), radial-gradient(circle at 22% 18%, rgba(245,158,11,0.08), transparent 24%), linear-gradient(rgba(148,163,184,0.06) 1px, transparent 1px), linear-gradient(90deg, rgba(148,163,184,0.06) 1px, transparent 1px)',
          backgroundSize: '100% 100%, 100% 100%, 28px 28px, 28px 28px',
        }}
      />
      <div ref={containerRef} className="absolute inset-0" />

      <div className="absolute left-5 top-5 max-w-sm rounded-2xl border border-white/10 bg-[#08141e]/86 px-4 py-3 backdrop-blur-md">
        <p className="text-[0.68rem] uppercase tracking-[0.28em] text-[#94a3b8]">
          Repository Atlas
        </p>
        <p className="mt-1 text-sm leading-6 text-[#dbe4ef]">
          Nexus-style Sigma atlas on top of the Rust architecture surface: clustered seed layout,
          ForceAtlas2 settling, noverlap cleanup, and finding-driven highlighting.
        </p>
      </div>

      <div className="absolute right-5 top-5 flex flex-col gap-2">
        <AtlasControl
          icon={<MagnifyingGlassPlus size={16} />}
          label="Zoom in"
          onClick={() => sigmaRef.current?.getCamera().animatedZoom({ duration: 180 })}
        />
        <AtlasControl
          icon={<MagnifyingGlassMinus size={16} />}
          label="Zoom out"
          onClick={() => sigmaRef.current?.getCamera().animatedUnzoom({ duration: 180 })}
        />
        <AtlasControl
          icon={renderMode === 'focus' ? <CornersOut size={16} /> : <CornersIn size={16} />}
          label={renderMode === 'focus' ? 'Show full atlas' : 'Focus selection'}
          onClick={() => {
            const next = renderMode === 'focus' ? 'atlas' : 'focus';
            setRenderMode(next);
            requestAnimationFrame(() => refreshCamera(next));
          }}
        />
        <AtlasControl
          icon={<ArrowsClockwise size={16} />}
          label="Relayout"
          onClick={() => {
            const sigma = sigmaRef.current;
            const graph = graphRef.current;
            if (!sigma || !graph) {
              return;
            }
            const nextGraph = makeGraph(surface, highlightedSet, focusedPath);
            graphRef.current = nextGraph;
            sigma.setGraph(nextGraph);
            refreshCamera(renderMode);
          }}
        />
      </div>

      {hoveredLabel ? (
        <div className="absolute left-1/2 top-5 max-w-[70%] -translate-x-1/2 rounded-full border border-white/10 bg-[#07141e]/92 px-4 py-2 text-xs text-[#e2e8f0] backdrop-blur-md">
          {hoveredLabel}
        </div>
      ) : null}

      <div className="absolute bottom-4 left-4 right-4 grid gap-3 rounded-2xl border border-white/10 bg-[#08141e]/82 p-3 backdrop-blur-md lg:grid-cols-[1.2fr_0.8fr]">
        <div className="flex flex-wrap gap-x-4 gap-y-2 text-xs text-[#cbd5e1]">
          {Object.entries(LANGUAGE_COLORS).map(([language, color]) => (
            <div key={language} className="flex items-center gap-2">
              <span
                className="inline-block h-2.5 w-2.5 rounded-full"
                style={{ backgroundColor: color }}
              />
              {language}
            </div>
          ))}
        </div>
        <div className="flex flex-wrap justify-start gap-3 text-xs text-[#cbd5e1] lg:justify-end">
          <AtlasLegend icon={<CursorClick size={14} />} label="Click a node to focus a file" />
          <AtlasLegend icon={<HandGrabbing size={14} />} label="Drag to inspect neighborhoods" />
          <AtlasLegend icon={<Pulse size={14} />} label="Bright edges follow the current finding" />
        </div>
      </div>
    </div>
  );
}

function AtlasControl({
  icon,
  label,
  onClick,
}: {
  icon: ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="inline-flex items-center justify-center rounded-xl border border-white/10 bg-[#08141e]/84 p-2 text-[#dbe4ef] backdrop-blur-md transition hover:border-white/20 hover:bg-[#0c1a26]"
      aria-label={label}
      title={label}
    >
      {icon}
    </button>
  );
}

function AtlasLegend({
  icon,
  label,
}: {
  icon: ReactNode;
  label: string;
}) {
  return (
    <div className="flex items-center gap-2">
      <span className="text-[#94a3b8]">{icon}</span>
      <span>{label}</span>
    </div>
  );
}
