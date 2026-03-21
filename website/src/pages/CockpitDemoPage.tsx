import {
  startTransition,
  useDeferredValue,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react';
import { Helmet } from 'react-helmet-async';
import {
  ArrowsClockwise,
  ArrowSquareOut,
  CirclesThreePlus,
  CompassTool,
  Crosshair,
  Graph,
  Pulse,
  Scan,
  ShieldCheck,
  Sparkle,
  WarningDiamond,
} from '@phosphor-icons/react';
import RepositoryAtlas from '../components/cockpit/RepositoryAtlas';
import { architectureSurfaceDemo } from '../fixtures/architectureSurfaceDemo';
import { loadArchitectureSurface } from '../lib/architectureSurface';
import type {
  ArchitectureSurface,
  FindingFamily,
  FindingSeverity,
  HotspotFile,
  SurfaceFinding,
} from '../types/architecture';

const FAMILY_OPTIONS: Array<'All' | FindingFamily> = [
  'All',
  'Graph',
  'DeadCode',
  'Hardwiring',
];

const severityTone: Record<FindingSeverity, string> = {
  High: 'text-[#fca5a5] border-[#7f1d1d] bg-[#3f1518]',
  Medium: 'text-[#fdba74] border-[#7c2d12] bg-[#402218]',
  Low: 'text-[#fde68a] border-[#78350f] bg-[#3b2a18]',
};

function compactNumber(value: number): string {
  return new Intl.NumberFormat('en', { notation: 'compact' }).format(value);
}

function timingLabel(surfaceData: ArchitectureSurface): string {
  const total = surfaceData.timings.reduce((sum, item) => sum + item.elapsedMs, 0);
  return `${total} ms total`;
}

function evidenceEdges(surfaceData: ArchitectureSurface, filePaths: string[]) {
  const pathSet = new Set(filePaths);
  return surfaceData.atlas.edges.filter(
    (edge) =>
      pathSet.has(edge.sourceFilePath) ||
      pathSet.has(edge.targetFilePath),
  );
}

function hotspotForPath(surfaceData: ArchitectureSurface, filePath: string | null): HotspotFile | null {
  if (filePath === null) {
    return null;
  }
  return surfaceData.hotspots.find((item) => item.filePath === filePath) ?? null;
}

function familyCount(surfaceData: ArchitectureSurface, family: FindingFamily) {
  return surfaceData.highlights.filter((item) => item.family === family).length;
}

export default function CockpitDemoPage() {
  const [surface, setSurface] = useState<ArchitectureSurface | null>(null);
  const [surfaceSource, setSurfaceSource] = useState<'live' | 'demo'>('demo');
  const [isSurfaceLoading, setIsSurfaceLoading] = useState(true);
  const [familyFilter, setFamilyFilter] = useState<'All' | FindingFamily>('All');
  const [selectedFindingId, setSelectedFindingId] = useState<string>('');
  const [focusedPath, setFocusedPath] = useState<string | null>(null);
  const deferredFamilyFilter = useDeferredValue(familyFilter);
  const effectiveSurface = surface ?? architectureSurfaceDemo;

  useEffect(() => {
    let active = true;
    void loadArchitectureSurface().then(({ surface: nextSurface, source }) => {
      if (!active) {
        return;
      }
      setSurface(nextSurface);
      setSurfaceSource(source);
      setSelectedFindingId(nextSurface.highlights[0]?.id ?? '');
      setFocusedPath(
        nextSurface.highlights[0]?.filePaths[0] ??
          nextSurface.hotspots[0]?.filePath ??
          null,
      );
      setIsSurfaceLoading(false);
    });
    return () => {
      active = false;
    };
  }, []);

  const filteredFindings = useMemo(() => {
    if (deferredFamilyFilter === 'All') {
      return effectiveSurface.highlights;
    }
    return effectiveSurface.highlights.filter((item) => item.family === deferredFamilyFilter);
  }, [deferredFamilyFilter, effectiveSurface.highlights]);

  useEffect(() => {
    if (!filteredFindings.some((item) => item.id === selectedFindingId)) {
      const nextFinding = filteredFindings[0];
      if (nextFinding) {
        setSelectedFindingId(nextFinding.id);
      }
    }
  }, [filteredFindings, selectedFindingId]);

  const selectedFinding = useMemo<SurfaceFinding | null>(
    () =>
      filteredFindings.find((item) => item.id === selectedFindingId) ??
      filteredFindings[0] ??
      null,
    [filteredFindings, selectedFindingId],
  );

  useEffect(() => {
    if (selectedFinding?.filePaths.length) {
      setFocusedPath((current) =>
        current && selectedFinding.filePaths.includes(current)
          ? current
          : selectedFinding.filePaths[0] ?? null,
      );
    }
  }, [selectedFinding]);

  const focusedHotspot = hotspotForPath(effectiveSurface, focusedPath);
  const selectedEdges = useMemo(
    () => evidenceEdges(effectiveSurface, selectedFinding?.filePaths ?? []),
    [effectiveSurface, selectedFinding],
  );

  return (
    <>
      <Helmet>
        <title>Architecture Cockpit Demo | AigisCode</title>
        <meta
          name="description"
          content="AigisCode cockpit demo showing a finding-first architecture console for whole-codebase analysis."
        />
      </Helmet>

      <div className="relative overflow-hidden bg-[#061018] text-[#f8fafc]">
        <div
          className="absolute inset-0 opacity-90"
          style={{
            background:
              'radial-gradient(circle at 10% 15%, rgba(20,184,166,0.14), transparent 28%), radial-gradient(circle at 84% 12%, rgba(249,115,22,0.14), transparent 24%), radial-gradient(circle at 55% 80%, rgba(245,158,11,0.14), transparent 26%), linear-gradient(180deg, #061018 0%, #07141d 48%, #040b11 100%)',
          }}
        />

        <section className="relative z-10 px-4 pb-24 pt-8 sm:px-6 lg:px-10">
          <div className="mx-auto max-w-[1600px] space-y-6">
            <div className="overflow-hidden rounded-[2rem] border border-white/10 bg-[rgba(6,19,27,0.84)] shadow-[0_30px_120px_rgba(0,0,0,0.45)] backdrop-blur-xl">
              <div className="grid gap-6 px-6 py-8 lg:grid-cols-[1.2fr_0.8fr] lg:px-8">
                <div>
                  <div className="flex flex-wrap gap-2 text-[0.68rem] uppercase tracking-[0.28em] text-[#94a3b8]">
                    <span className="rounded-full border border-white/10 px-3 py-1">
                      {surfaceSource === 'live' ? 'Live Surface' : 'Demo Surface'}
                    </span>
                    <span
                      className={`rounded-full border px-3 py-1 ${
                        surfaceSource === 'live'
                          ? 'border-[#0f766e] bg-[#082c2c] text-[#99f6e4]'
                          : 'border-[#713f12] bg-[#2b1c12] text-[#fde68a]'
                      }`}
                    >
                      {surfaceSource === 'live'
                        ? 'Loaded from .aigiscode/architecture-surface.json'
                        : 'Fallback fixture using the same contract'}
                    </span>
                    <span className="rounded-full border border-white/10 px-3 py-1">
                      {isSurfaceLoading && surface === null ? 'Loading…' : 'Ready'}
                    </span>
                  </div>

                  <h1 className="mt-4 max-w-4xl font-display text-4xl font-semibold leading-tight text-white sm:text-5xl">
                    The architecture cockpit should start with risk, not with edge soup.
                  </h1>

                  <p className="mt-4 max-w-3xl text-base leading-7 text-[#c7d2de] sm:text-lg">
                    This prototype consumes the shared architecture surface and turns it into an
                    operator console: risk queue, coverage truth, repository atlas, and evidence
                    panels for the exact files under investigation.
                  </p>

                  <div className="mt-6 flex flex-wrap gap-3">
                    <div className="rounded-2xl border border-white/10 bg-[#0a1822] px-4 py-3">
                      <div className="flex items-center gap-2 text-xs uppercase tracking-[0.24em] text-[#94a3b8]">
                        <Crosshair size={16} />
                        Scan Status
                      </div>
                      <p className="mt-2 text-lg font-medium text-white">
                        {effectiveSurface.overview.scannedFiles} files scanned
                      </p>
                      <p className="text-sm text-[#9fb0c1]">
                        {effectiveSurface.overview.analyzedFiles} analyzed, {effectiveSurface.overview.unresolvedReferenceSites} unresolved sites
                      </p>
                    </div>

                    <div className="rounded-2xl border border-white/10 bg-[#0a1822] px-4 py-3">
                      <div className="flex items-center gap-2 text-xs uppercase tracking-[0.24em] text-[#94a3b8]">
                        <Pulse size={16} />
                        Risk Posture
                      </div>
                      <p className="mt-2 text-lg font-medium text-white">
                        {effectiveSurface.overview.strongCycleCount} strong cycles, {effectiveSurface.overview.bottleneckCount} bottlenecks
                      </p>
                      <p className="text-sm text-[#9fb0c1]">
                        {effectiveSurface.overview.deadCodeCount} dead-code findings, {effectiveSurface.overview.hardwiringCount} hardwiring findings
                      </p>
                    </div>

                    <div className="rounded-2xl border border-white/10 bg-[#0a1822] px-4 py-3">
                      <div className="flex items-center gap-2 text-xs uppercase tracking-[0.24em] text-[#94a3b8]">
                        <ArrowsClockwise size={16} />
                        Deterministic Pipeline
                      </div>
                      <p className="mt-2 text-lg font-medium text-white">{timingLabel(effectiveSurface)}</p>
                      <p className="text-sm text-[#9fb0c1]">
                        Scan → Structure → Parse → Resolve → Analyze
                      </p>
                    </div>
                  </div>
                </div>

                <div className="grid gap-4 rounded-[1.75rem] border border-white/10 bg-[#08141d] p-5">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="text-[0.68rem] uppercase tracking-[0.28em] text-[#94a3b8]">
                        Trust Surface
                      </p>
                      <h2 className="mt-2 font-display text-2xl font-semibold text-white">
                        Explainability before action
                      </h2>
                    </div>
                    <ShieldCheck size={28} className="text-[#99f6e4]" />
                  </div>

                  <div className="grid gap-3 sm:grid-cols-2">
                    {effectiveSurface.languages.map((entry) => {
                      const percentage = Math.max(
                        10,
                        Math.round((entry.fileCount / effectiveSurface.overview.scannedFiles) * 100),
                      );
                      return (
                        <div
                          key={entry.language}
                          className="rounded-2xl border border-white/8 bg-[#0c1b25] p-4"
                        >
                          <div className="flex items-center justify-between text-sm text-[#dbe4ef]">
                            <span>{entry.language}</span>
                            <span>{entry.fileCount}</span>
                          </div>
                          <div className="mt-3 h-2 rounded-full bg-white/6">
                            <div
                              className="h-2 rounded-full bg-gradient-to-r from-[#14b8a6] via-[#f59e0b] to-[#fb7185]"
                              style={{ width: `${percentage}%` }}
                            />
                          </div>
                        </div>
                      );
                    })}
                  </div>

                  <div className="rounded-2xl border border-dashed border-[#1e293b] bg-[#071119] p-4 text-sm leading-6 text-[#b8c5d3]">
                    The cockpit should never hide coverage gaps. Agents and humans need to see
                    unresolved references, unsupported files, and confidence tiers before they trust
                    a fix plan.
                  </div>
                </div>
              </div>
            </div>

            <div className="grid gap-6 xl:grid-cols-[1.4fr_0.6fr]">
              <div className="overflow-hidden rounded-[2rem] border border-white/10 bg-[rgba(7,20,29,0.82)] p-4 shadow-[0_24px_80px_rgba(0,0,0,0.35)] backdrop-blur-xl sm:p-5">
                <RepositoryAtlas
                  surface={effectiveSurface}
                  highlightedPaths={selectedFinding?.filePaths ?? []}
                  focusedPath={focusedPath}
                  onSelectPath={setFocusedPath}
                />
              </div>

              <div className="grid gap-4">
                <div className="rounded-[1.75rem] border border-white/10 bg-[rgba(7,20,29,0.84)] p-5 backdrop-blur-xl">
                  <div className="flex items-start justify-between gap-4">
                    <div>
                      <p className="text-[0.68rem] uppercase tracking-[0.28em] text-[#94a3b8]">
                        Command Deck
                      </p>
                      <h2 className="mt-2 font-display text-2xl font-semibold text-white">
                        What matters now
                      </h2>
                    </div>
                    <Sparkle size={24} className="text-[#f59e0b]" />
                  </div>

                  <div className="mt-5 grid gap-3 sm:grid-cols-2 xl:grid-cols-1">
                    <MetricTile
                      icon={<WarningDiamond size={18} />}
                      label="Strong cycles"
                      value={effectiveSurface.overview.strongCycleCount}
                      detail="Architectural loops that deserve first attention"
                    />
                    <MetricTile
                      icon={<CirclesThreePlus size={18} />}
                      label="Resolved edges"
                      value={effectiveSurface.overview.resolvedEdges}
                      detail="Cross-file evidence currently backing the atlas"
                    />
                    <MetricTile
                      icon={<CompassTool size={18} />}
                      label="Hotspots"
                      value={effectiveSurface.hotspots.length}
                      detail="Files pulling too much orchestration weight"
                    />
                    <MetricTile
                      icon={<Scan size={18} />}
                      label="Override edges"
                      value={effectiveSurface.overview.overrideEdgeCount}
                      detail="Inheritance and runtime dispatch evidence"
                    />
                  </div>
                </div>

                <div className="rounded-[1.75rem] border border-white/10 bg-[rgba(7,20,29,0.84)] p-5 backdrop-blur-xl">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="text-[0.68rem] uppercase tracking-[0.28em] text-[#94a3b8]">
                        Timings
                      </p>
                      <h2 className="mt-2 font-display text-xl font-semibold text-white">
                        Deterministic cost profile
                      </h2>
                    </div>
                    <Graph size={22} className="text-[#93c5fd]" />
                  </div>

                  <div className="mt-5 space-y-3">
                    {effectiveSurface.timings.map((entry) => {
                      const maxTiming = Math.max(...effectiveSurface.timings.map((item) => item.elapsedMs), 1);
                      const width = Math.max(12, Math.round((entry.elapsedMs / maxTiming) * 100));
                      return (
                        <div key={entry.phase}>
                          <div className="mb-2 flex items-center justify-between text-sm text-[#dbe4ef]">
                            <span>{entry.phase}</span>
                            <span>{entry.elapsedMs} ms</span>
                          </div>
                          <div className="h-2 rounded-full bg-white/6">
                            <div
                              className="h-2 rounded-full bg-gradient-to-r from-[#14b8a6] to-[#f59e0b]"
                              style={{ width: `${width}%` }}
                            />
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              </div>
            </div>

            <div className="grid gap-6 xl:grid-cols-[0.8fr_1.2fr]">
              <div className="rounded-[2rem] border border-white/10 bg-[rgba(7,20,29,0.84)] p-5 backdrop-blur-xl">
                <div className="flex flex-wrap items-center justify-between gap-3">
                  <div>
                    <p className="text-[0.68rem] uppercase tracking-[0.28em] text-[#94a3b8]">
                      Findings Queue
                    </p>
                    <h2 className="mt-2 font-display text-2xl font-semibold text-white">
                      Investigation workbench
                    </h2>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {FAMILY_OPTIONS.map((option) => {
                      const active = familyFilter === option;
                      const count =
                        option === 'All'
                          ? effectiveSurface.highlights.length
                          : familyCount(effectiveSurface, option);
                      return (
                        <button
                          key={option}
                          type="button"
                          onClick={() => {
                            startTransition(() => {
                              setFamilyFilter(option);
                            });
                          }}
                          className={`rounded-full border px-3 py-1.5 text-xs font-medium transition ${
                            active
                              ? 'border-[#f59e0b] bg-[#3d2a16] text-[#fde68a]'
                              : 'border-white/10 bg-white/5 text-[#c7d2de] hover:border-white/20 hover:bg-white/8'
                          }`}
                        >
                          {option} <span className="ml-1 text-[#94a3b8]">{count}</span>
                        </button>
                      );
                    })}
                  </div>
                </div>

                <div className="mt-5 space-y-3">
                  {filteredFindings.map((finding) => {
                    const active = selectedFinding?.id === finding.id;
                    return (
                      <button
                        key={finding.id}
                        type="button"
                        onClick={() => {
                          startTransition(() => {
                            setSelectedFindingId(finding.id);
                          });
                        }}
                        className={`w-full rounded-[1.4rem] border p-4 text-left transition ${
                          active
                            ? 'border-[#f59e0b] bg-[#18130d] shadow-[0_10px_40px_rgba(245,158,11,0.16)]'
                            : 'border-white/8 bg-[#0b1821] hover:border-white/15 hover:bg-[#0d1d28]'
                        }`}
                      >
                        <div className="flex items-start justify-between gap-3">
                          <div>
                            <div className="flex flex-wrap items-center gap-2">
                              <span className={`rounded-full border px-2 py-1 text-[0.68rem] uppercase tracking-[0.2em] ${severityTone[finding.severity]}`}>
                                {finding.severity}
                              </span>
                              <span className="text-[0.72rem] uppercase tracking-[0.22em] text-[#94a3b8]">
                                {finding.family}
                              </span>
                            </div>
                            <h3 className="mt-3 text-base font-medium text-white">
                              {finding.title}
                            </h3>
                          </div>
                          <ArrowSquareOut size={18} className="shrink-0 text-[#64748b]" />
                        </div>

                        <p className="mt-3 text-sm leading-6 text-[#bfd0de]">
                          {finding.summary}
                        </p>

                        <div className="mt-4 flex flex-wrap gap-2">
                          {finding.filePaths.map((filePath) => (
                            <span
                              key={filePath}
                              className="rounded-full border border-white/10 bg-white/5 px-2.5 py-1 text-xs text-[#dbe4ef]"
                            >
                              {filePath}
                            </span>
                          ))}
                        </div>
                      </button>
                    );
                  })}
                </div>
              </div>

              <div className="grid gap-6">
                <div className="rounded-[2rem] border border-white/10 bg-[rgba(7,20,29,0.84)] p-5 backdrop-blur-xl">
                  <div className="flex items-start justify-between gap-4">
                    <div>
                      <p className="text-[0.68rem] uppercase tracking-[0.28em] text-[#94a3b8]">
                        Evidence Panel
                      </p>
                      <h2 className="mt-2 font-display text-2xl font-semibold text-white">
                        {selectedFinding?.title ?? 'Select a finding'}
                      </h2>
                    </div>
                    {selectedFinding ? (
                      <span className={`rounded-full border px-2.5 py-1 text-[0.68rem] uppercase tracking-[0.2em] ${severityTone[selectedFinding.severity]}`}>
                        {selectedFinding.severity}
                      </span>
                    ) : null}
                  </div>

                  <p className="mt-4 max-w-3xl text-sm leading-7 text-[#c3d2df]">
                    {selectedFinding?.summary ??
                      'Choose a finding to inspect related files, atlas edges, and hotspot context.'}
                  </p>

                  <div className="mt-6 grid gap-4 lg:grid-cols-[0.9fr_1.1fr]">
                    <div className="rounded-[1.5rem] border border-white/8 bg-[#0b1821] p-4">
                      <p className="text-[0.68rem] uppercase tracking-[0.24em] text-[#94a3b8]">
                        Focused File
                      </p>
                      <h3 className="mt-2 break-all text-lg font-medium text-white">
                        {focusedPath ?? 'No file selected'}
                      </h3>

                      {focusedHotspot ? (
                        <div className="mt-4 grid gap-3 sm:grid-cols-2">
                          <EvidenceStat label="Language" value={focusedHotspot.language} />
                          <EvidenceStat
                            label="Finding Load"
                            value={compactNumber(focusedHotspot.findingCount)}
                          />
                          <EvidenceStat
                            label="Edges In"
                            value={compactNumber(focusedHotspot.inboundEdges)}
                          />
                          <EvidenceStat
                            label="Edges Out"
                            value={compactNumber(focusedHotspot.outboundEdges)}
                          />
                          <EvidenceStat
                            label="Centrality"
                            value={focusedHotspot.bottleneckCentralityMillis}
                          />
                          <EvidenceStat
                            label="Flags"
                            value={[
                              focusedHotspot.isRuntimeEntry ? 'Entry' : null,
                              focusedHotspot.isOrphan ? 'Orphan' : null,
                            ]
                              .filter(Boolean)
                              .join(' · ') || 'None'}
                          />
                        </div>
                      ) : (
                        <p className="mt-4 text-sm text-[#9fb0c1]">
                          Click an atlas node or select a finding to inspect file pressure.
                        </p>
                      )}
                    </div>

                    <div className="rounded-[1.5rem] border border-white/8 bg-[#0b1821] p-4">
                      <p className="text-[0.68rem] uppercase tracking-[0.24em] text-[#94a3b8]">
                        Related Evidence Edges
                      </p>
                      <div className="mt-4 space-y-3">
                        {selectedEdges.length ? (
                          selectedEdges.slice(0, 6).map((edge) => (
                            <button
                              key={`${edge.sourceFilePath}-${edge.targetFilePath}`}
                              type="button"
                              onClick={() => {
                                setFocusedPath(edge.sourceFilePath);
                              }}
                              className="w-full rounded-2xl border border-white/8 bg-[#0d1d28] px-4 py-3 text-left hover:border-white/15"
                            >
                              <div className="flex items-center justify-between gap-3 text-sm text-white">
                                <span className="truncate">{edge.sourceFilePath}</span>
                                <span className="text-[#64748b]">→</span>
                                <span className="truncate">{edge.targetFilePath}</span>
                              </div>
                              <p className="mt-2 text-xs uppercase tracking-[0.2em] text-[#94a3b8]">
                                {edge.kinds.join(' · ')} · {edge.strongestResolutionTier} · confidence {edge.averageConfidenceMillis}
                              </p>
                            </button>
                          ))
                        ) : (
                          <p className="text-sm text-[#9fb0c1]">
                            No atlas edges attached to the current finding.
                          </p>
                        )}
                      </div>
                    </div>
                  </div>
                </div>

                <div className="grid gap-4 lg:grid-cols-[0.95fr_1.05fr]">
                  <div className="rounded-[1.75rem] border border-white/10 bg-[rgba(7,20,29,0.84)] p-5 backdrop-blur-xl">
                    <p className="text-[0.68rem] uppercase tracking-[0.28em] text-[#94a3b8]">
                      Hotspot Board
                    </p>
                    <h2 className="mt-2 font-display text-xl font-semibold text-white">
                      Highest-pressure files
                    </h2>
                    <div className="mt-4 space-y-3">
                      {effectiveSurface.hotspots.slice(0, 5).map((hotspot) => (
                        <button
                          key={hotspot.filePath}
                          type="button"
                          onClick={() => setFocusedPath(hotspot.filePath)}
                          className={`w-full rounded-2xl border px-4 py-3 text-left transition ${
                            focusedPath === hotspot.filePath
                              ? 'border-[#14b8a6] bg-[#082322]'
                              : 'border-white/8 bg-[#0b1821] hover:border-white/15'
                          }`}
                        >
                          <div className="flex items-start justify-between gap-3">
                            <div>
                              <p className="text-sm font-medium text-white">{hotspot.filePath}</p>
                              <p className="mt-1 text-xs uppercase tracking-[0.2em] text-[#94a3b8]">
                                {hotspot.language}
                              </p>
                            </div>
                            <span className="rounded-full border border-white/10 px-2 py-1 text-xs text-[#e2e8f0]">
                              {hotspot.findingCount} findings
                            </span>
                          </div>
                        </button>
                      ))}
                    </div>
                  </div>

                  <div className="rounded-[1.75rem] border border-white/10 bg-[rgba(7,20,29,0.84)] p-5 backdrop-blur-xl">
                    <p className="text-[0.68rem] uppercase tracking-[0.28em] text-[#94a3b8]">
                      Why this beats a graph browser
                    </p>
                    <h2 className="mt-2 font-display text-xl font-semibold text-white">
                      Same evidence, better operating model
                    </h2>
                    <div className="mt-4 grid gap-3">
                      <ProductNote
                        title="Finding-first entry"
                        body="The queue decides what deserves attention before the atlas is even consulted."
                      />
                      <ProductNote
                        title="Coverage truth stays visible"
                        body="Unresolved references, analyzed file counts, and pipeline timings remain on-screen while triaging."
                      />
                      <ProductNote
                        title="Atlas is evidence, not decoration"
                        body="Node highlights and edge emphasis follow the active finding so the visual layer stays legible."
                      />
                      <ProductNote
                        title="One contract for FE and MCP"
                        body="This page and the native Rust MCP server both sit on the same architecture surface rather than reverse-engineering meaning from raw graph storage."
                      />
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>
      </div>
    </>
  );
}

function MetricTile({
  icon,
  label,
  value,
  detail,
}: {
  icon: ReactNode;
  label: string;
  value: number;
  detail: string;
}) {
  return (
    <div className="rounded-2xl border border-white/8 bg-[#0b1821] p-4">
      <div className="flex items-center gap-2 text-[0.72rem] uppercase tracking-[0.24em] text-[#94a3b8]">
        <span className="text-[#f59e0b]">{icon}</span>
        {label}
      </div>
      <p className="mt-3 font-display text-3xl font-semibold text-white">
        {compactNumber(value)}
      </p>
      <p className="mt-2 text-sm leading-6 text-[#9fb0c1]">{detail}</p>
    </div>
  );
}

function EvidenceStat({
  label,
  value,
}: {
  label: string;
  value: string | number;
}) {
  return (
    <div className="rounded-2xl border border-white/8 bg-[#0d1d28] p-3">
      <p className="text-[0.66rem] uppercase tracking-[0.22em] text-[#94a3b8]">{label}</p>
      <p className="mt-2 text-sm font-medium text-white">{value}</p>
    </div>
  );
}

function ProductNote({
  title,
  body,
}: {
  title: string;
  body: string;
}) {
  return (
    <div className="rounded-2xl border border-white/8 bg-[#0b1821] p-4">
      <p className="text-sm font-medium text-white">{title}</p>
      <p className="mt-2 text-sm leading-6 text-[#9fb0c1]">{body}</p>
    </div>
  );
}
