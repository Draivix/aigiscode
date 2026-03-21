import { architectureSurfaceDemo } from '../fixtures/architectureSurfaceDemo';
import type { ArchitectureSurface } from '../types/architecture';

type LiveSource = 'live' | 'demo';

interface RawSurfaceOverview {
  scanned_files: number;
  analyzed_files: number;
  symbols: number;
  references: number;
  resolved_edges: number;
  unresolved_reference_sites: number;
  strong_cycle_count: number;
  total_cycle_count: number;
  bottleneck_count: number;
  orphan_count: number;
  runtime_entry_count: number;
  dead_code_count: number;
  hardwiring_count: number;
  override_edge_count: number;
}

interface RawLanguageCoverage {
  language: string;
  file_count: number;
}

interface RawHotspotFile {
  file_path: string;
  language: string;
  inbound_edges: number;
  outbound_edges: number;
  finding_count: number;
  bottleneck_centrality_millis: number;
  is_orphan: boolean;
  is_runtime_entry: boolean;
}

interface RawSurfaceFinding {
  id: string;
  family: 'Graph' | 'DeadCode' | 'Hardwiring';
  severity: 'High' | 'Medium' | 'Low';
  title: string;
  summary: string;
  file_paths: string[];
  line: number | null;
}

interface RawAtlasNode {
  file_path: string;
  language: string;
  inbound_edges: number;
  outbound_edges: number;
  finding_count: number;
  bottleneck_centrality_millis: number;
  is_orphan: boolean;
  is_runtime_entry: boolean;
}

interface RawAtlasEdge {
  source_file_path: string;
  target_file_path: string;
  edge_count: number;
  kinds: string[];
  strongest_resolution_tier: string;
  average_confidence_millis: number;
}

interface RawArchitectureSurface {
  root: string;
  overview: RawSurfaceOverview;
  languages: RawLanguageCoverage[];
  hotspots: RawHotspotFile[];
  highlights: RawSurfaceFinding[];
  atlas: {
    nodes: RawAtlasNode[];
    edges: RawAtlasEdge[];
  };
  timings: Array<{ phase: string; elapsed_ms: number }>;
}

function normalizeSurface(raw: RawArchitectureSurface): ArchitectureSurface {
  return {
    root: raw.root,
    overview: {
      scannedFiles: raw.overview.scanned_files,
      analyzedFiles: raw.overview.analyzed_files,
      symbols: raw.overview.symbols,
      references: raw.overview.references,
      resolvedEdges: raw.overview.resolved_edges,
      unresolvedReferenceSites: raw.overview.unresolved_reference_sites,
      strongCycleCount: raw.overview.strong_cycle_count,
      totalCycleCount: raw.overview.total_cycle_count,
      bottleneckCount: raw.overview.bottleneck_count,
      orphanCount: raw.overview.orphan_count,
      runtimeEntryCount: raw.overview.runtime_entry_count,
      deadCodeCount: raw.overview.dead_code_count,
      hardwiringCount: raw.overview.hardwiring_count,
      overrideEdgeCount: raw.overview.override_edge_count,
    },
    languages: raw.languages.map((entry) => ({
      language: entry.language,
      fileCount: entry.file_count,
    })),
    hotspots: raw.hotspots.map((item) => ({
      filePath: item.file_path,
      language: item.language,
      inboundEdges: item.inbound_edges,
      outboundEdges: item.outbound_edges,
      findingCount: item.finding_count,
      bottleneckCentralityMillis: item.bottleneck_centrality_millis,
      isOrphan: item.is_orphan,
      isRuntimeEntry: item.is_runtime_entry,
    })),
    highlights: raw.highlights.map((item) => ({
      id: item.id,
      family: item.family,
      severity: item.severity,
      title: item.title,
      summary: item.summary,
      filePaths: item.file_paths,
      line: item.line,
    })),
    atlas: {
      nodes: raw.atlas.nodes.map((node) => ({
        filePath: node.file_path,
        language: node.language,
        inboundEdges: node.inbound_edges,
        outboundEdges: node.outbound_edges,
        findingCount: node.finding_count,
        bottleneckCentralityMillis: node.bottleneck_centrality_millis,
        isOrphan: node.is_orphan,
        isRuntimeEntry: node.is_runtime_entry,
      })),
      edges: raw.atlas.edges.map((edge) => ({
        sourceFilePath: edge.source_file_path,
        targetFilePath: edge.target_file_path,
        edgeCount: edge.edge_count,
        kinds: edge.kinds,
        strongestResolutionTier: edge.strongest_resolution_tier,
        averageConfidenceMillis: edge.average_confidence_millis,
      })),
    },
    timings: raw.timings.map((item) => ({
      phase: item.phase,
      elapsedMs: item.elapsed_ms,
    })),
  };
}

function isRawArchitectureSurface(value: unknown): value is RawArchitectureSurface {
  if (typeof value !== 'object' || value === null) {
    return false;
  }
  const candidate = value as Partial<RawArchitectureSurface>;
  return (
    typeof candidate.root === 'string' &&
    typeof candidate.overview === 'object' &&
    candidate.overview !== null &&
    Array.isArray(candidate.languages) &&
    Array.isArray(candidate.hotspots) &&
    Array.isArray(candidate.highlights) &&
    typeof candidate.atlas === 'object' &&
    candidate.atlas !== null &&
    Array.isArray(candidate.atlas.nodes) &&
    Array.isArray(candidate.atlas.edges) &&
    Array.isArray(candidate.timings)
  );
}

export async function loadArchitectureSurface(): Promise<{
  surface: ArchitectureSurface;
  source: LiveSource;
}> {
  try {
    const response = await fetch('/__aigiscode/surface', {
      headers: { Accept: 'application/json' },
    });
    if (!response.ok) {
      throw new Error(`Surface endpoint returned ${response.status}`);
    }
    const payload: unknown = await response.json();
    if (!isRawArchitectureSurface(payload)) {
      throw new Error('Surface payload shape is invalid');
    }
    return {
      surface: normalizeSurface(payload),
      source: 'live',
    };
  } catch {
    return {
      surface: architectureSurfaceDemo,
      source: 'demo',
    };
  }
}

