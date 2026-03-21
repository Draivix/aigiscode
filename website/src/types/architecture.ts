export interface SurfaceOverview {
  scannedFiles: number;
  analyzedFiles: number;
  symbols: number;
  references: number;
  resolvedEdges: number;
  unresolvedReferenceSites: number;
  strongCycleCount: number;
  totalCycleCount: number;
  bottleneckCount: number;
  orphanCount: number;
  runtimeEntryCount: number;
  deadCodeCount: number;
  hardwiringCount: number;
  overrideEdgeCount: number;
}

export interface LanguageCoverage {
  language: string;
  fileCount: number;
}

export interface HotspotFile {
  filePath: string;
  language: string;
  inboundEdges: number;
  outboundEdges: number;
  findingCount: number;
  bottleneckCentralityMillis: number;
  isOrphan: boolean;
  isRuntimeEntry: boolean;
}

export type FindingFamily = 'Graph' | 'DeadCode' | 'Hardwiring';
export type FindingSeverity = 'High' | 'Medium' | 'Low';

export interface SurfaceFinding {
  id: string;
  family: FindingFamily;
  severity: FindingSeverity;
  title: string;
  summary: string;
  filePaths: string[];
  line: number | null;
}

export interface AtlasNode {
  filePath: string;
  language: string;
  inboundEdges: number;
  outboundEdges: number;
  findingCount: number;
  bottleneckCentralityMillis: number;
  isOrphan: boolean;
  isRuntimeEntry: boolean;
}

export interface AtlasEdge {
  sourceFilePath: string;
  targetFilePath: string;
  edgeCount: number;
  kinds: string[];
  strongestResolutionTier: string;
  averageConfidenceMillis: number;
}

export interface RepositoryAtlas {
  nodes: AtlasNode[];
  edges: AtlasEdge[];
}

export interface PhaseTiming {
  phase: string;
  elapsedMs: number;
}

export interface ArchitectureSurface {
  root: string;
  overview: SurfaceOverview;
  languages: LanguageCoverage[];
  hotspots: HotspotFile[];
  highlights: SurfaceFinding[];
  atlas: RepositoryAtlas;
  timings: PhaseTiming[];
}

