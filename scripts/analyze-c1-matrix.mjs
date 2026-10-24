import { readFile, writeFile } from 'node:fs/promises';

const dir = 'dist/performance/c1-2026-10-12';
const parseCsv = text => {
  const lines = text.trim().split(/\r?\n/);
  const fields = line => {
    const out = []; let value = ''; let quoted = false;
    for (let i = 0; i < line.length; i++) {
      const c = line[i];
      if (c === '"') { if (quoted && line[i + 1] === '"') { value += '"'; i++; } else quoted = !quoted; }
      else if (c === ',' && !quoted) { out.push(value); value = ''; }
      else value += c;
    }
    out.push(value); return out;
  };
  const header = fields(lines[0]);
  return lines.slice(1).map(line => Object.fromEntries(fields(line).map((v, i) => [header[i], v])));
};
const percentile = (values, p) => {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted.length ? sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * p))] : null;
};
const parseUtc = value => Date.parse(value);
const loadCapture = async (build, suffix = 'user-r2') => {
  const captureName = `matrix-presentmon-${build}-${suffix}.csv`;
  const status = JSON.parse(await readFile(`${dir}/${captureName}.status.json`, 'utf8'));
  const rows = parseCsv(await readFile(`${dir}/${captureName}`, 'utf8'));
  const firstCpu = Date.parse(rows.find(row => row.CPUStartDateTime && row.CPUStartDateTime !== 'NA').CPUStartDateTime.replace(' ', 'T') + 'Z');
  const captureStart = parseUtc(status.startedUtc);
  const offsetHours = Math.round((firstCpu - captureStart) / 3_600_000);
  const offsetMs = offsetHours * 3_600_000;
  const frames = rows.map(row => {
    const rawDate = row.TimeInDateTime;
    const displayedAt = Date.parse(rawDate.replace(' ', 'T') + 'Z') - offsetMs;
    const intervalMs = Number(row.MsBetweenDisplayChange);
    return { ...row, displayedAt, intervalStart: displayedAt - intervalMs, intervalMs };
  }).filter(row => Number.isFinite(row.displayedAt) && Number.isFinite(row.intervalMs));
  return { build, captureName, status, offsetHours, rows: frames };
};
const loadMode = async (build, mode) => JSON.parse(await readFile(`${dir}/matrix-${build}-${mode}-user-r2.json`, 'utf8'));
const summarizeWindow = (capture, start, end) => {
  const rows = capture.rows.filter(row => row.displayedAt >= start && row.displayedAt <= end);
  const values = rows.map(row => row.intervalMs);
  return { rows: rows.length, p50Ms: percentile(values, .5), p95Ms: percentile(values, .95), p99Ms: percentile(values, .99), maxMs: values.length ? Math.max(...values) : null,
    over16_7ms: values.filter(v => v > 16.7).length, over50ms: values.filter(v => v > 50).length };
};
const summarizeIntervals = (capture, intervals) => {
  const rows = capture.rows.filter(row => intervals.some(interval => row.displayedAt >= interval.start && row.displayedAt <= interval.end));
  const values = rows.map(row => row.intervalMs);
  const outliers = rows.filter(row => row.intervalMs > 50).map(row => {
    const overlaps = intervals.filter(interval => row.intervalStart < interval.end && row.displayedAt > interval.start);
    const fullyInside = intervals.filter(interval => row.intervalStart >= interval.start && row.displayedAt <= interval.end);
    return { displayedAtUtc: new Date(row.displayedAt).toISOString(), intervalStartUtc: new Date(row.intervalStart).toISOString(), msBetweenDisplayChange: row.intervalMs,
      msCpuBusy: Number(row.MsCPUBusy), msGpuTime: Number(row.MsGPUTime), pid: row.ProcessID, swapChain: row.SwapChainAddress,
      vidPnSourceId: row.VidPnSourceId, layerIndex: row.LayerIndex, overlaps: overlaps.map(i => i.label), fullyInside: fullyInside.map(i => i.label),
      classification: fullyInside.length ? (fullyInside.every(i => /transition-\d+/.test(i.label)) ? 'inside-animation' : 'within-window') : overlaps.length ? 'crosses-window-boundary' : 'outside-window' };
  });
  return { rows: rows.length, p50Ms: percentile(values, .5), p95Ms: percentile(values, .95), p99Ms: percentile(values, .99), maxMs: values.length ? Math.max(...values) : null,
    over16_7ms: values.filter(v => v > 16.7).length, over50ms: values.filter(v => v > 50).length, outliersOver50ms: outliers };
};

const report = { generatedAt: new Date().toISOString(), timestampNormalization: {}, builds: {} };
for (const build of ['m6', 'c1']) {
  const capture = await loadCapture(build, 'user-r4');
  const combinedCapture = capture;
  report.timestampNormalization[build] = { method: 'Infer fixed offset from first CPUStartDateTime and capture-start UTC, rounded to nearest hour; subtract it from TimeInDateTime before joining browser ISO timestamps.', offsetHours: capture.offsetHours };
  const boundsText = await readFile(`${dir}/matrix-bounds-${build}-user-r4.csv`, 'utf8');
  const bounds = parseCsv(boundsText);
  const modes = {};
  const allAnims = [];
  for (const mode of ['animation-only', 'controls-only', 'combined']) {
    const data = JSON.parse(await readFile(`${dir}/matrix-${build}-${mode}-user-r4.json`, 'utf8'));
    const commandTimes = [...data.mediaCommands, ...data.volumeCommands].filter(x => x.startedAt && x.endedAt).map(x => ({ start: parseUtc(x.startedAt), end: parseUtc(x.endedAt), label: x.name }));
    const transitions = data.toggleRecords.map(x => ({ start: parseUtc(x.animationStart.at), end: parseUtc(x.animationEnd.at), label: `${mode}-transition-${x.index}` }));
    allAnims.push(...transitions);
    const interactionWindows = [...commandTimes, ...transitions];
    const broadStart = Math.min(...interactionWindows.map(x => x.start));
    const broadEnd = Math.max(...interactionWindows.map(x => x.end));
    modes[mode] = {
      scenarioStartUtc: new Date(broadStart).toISOString(), scenarioEndUtc: new Date(broadEnd).toISOString(),
      transitionCount: transitions.length, transitionTimes: transitions.map(x => ({ startUtc: new Date(x.start).toISOString(), endUtc: new Date(x.end).toISOString(), durationMs: x.end - x.start, label: x.label })), transitionDurationMs: transitions.map(x => x.end - x.start),
      commandCount: commandTimes.length,
      commandLatencyMs: [...data.mediaCommands, ...data.volumeCommands].filter(x => Number.isFinite(x.elapsedMs)).map(x => ({ name: x.name, elapsedMs: x.elapsedMs, ok: x.ok })),
      presentation: summarizeIntervals(mode === 'combined' ? combinedCapture : capture, [{ start: broadStart, end: broadEnd, label: `${mode}-scenario-envelope` }]),
      initialState: { paused: data.setup.initialMedia?.isPlaying === false, volumePercent: data.setup.initialVolume },
      finalState: { paused: data.controlResult?.finalMedia?.isPlaying === false || data.setup.initialMedia?.isPlaying === false,
        volumePercent: data.controlResult?.finalAudio?.volumePercent ?? data.setup.initialVolume },
      outlierClassificationForEachTransition: transitions.map(interval => ({ transition: interval.label, interval: summarizeIntervals(mode === 'combined' ? combinedCapture : capture, [interval]) }))
    };
  }
  const deviceCounts = Object.fromEntries([...new Set(bounds.map(x => x.MonitorDevice))].map(name => [name, bounds.filter(x => x.MonitorDevice === name).length]));
  const chainCounts = {};
  const sourceCounts = {};
  for (const row of capture.rows) {
    chainCounts[row.SwapChainAddress] = (chainCounts[row.SwapChainAddress] ?? 0) + 1;
    sourceCounts[row.VidPnSourceId] = (sourceCounts[row.VidPnSourceId] ?? 0) + 1;
  }
  report.builds[build] = {
    capture: { ...capture.status, timestampOffsetHours: capture.offsetHours, frameRows: capture.rows.length,
      processIds: [...new Set(capture.rows.map(row => row.ProcessID))], swapChainCounts: chainCounts, vidPnSourceCounts: sourceCounts,
      intervals: { firstUtc: new Date(capture.rows[0].displayedAt).toISOString(), lastUtc: new Date(capture.rows.at(-1).displayedAt).toISOString() } },
    nativeWindow: { processId: Number(bounds[0]?.ProcessId), hwnd: bounds[0]?.Hwnd, sampleCount: bounds.length, firstUtc: bounds[0]?.Utc, lastUtc: bounds.at(-1)?.Utc, displayCounts: deviceCounts,
      rectangles: [...new Map(bounds.map(x => [`${x.Left},${x.Top},${x.Right},${x.Bottom}`, { left: Number(x.Left), top: Number(x.Top), right: Number(x.Right), bottom: Number(x.Bottom) }])).values()],
      finalBounds: bounds.at(-1) && { left: Number(bounds.at(-1).Left), top: Number(bounds.at(-1).Top), right: Number(bounds.at(-1).Right), bottom: Number(bounds.at(-1).Bottom) } },
    combinedCapture: { ...combinedCapture.status, timestampOffsetHours: combinedCapture.offsetHours, frameRows: combinedCapture.rows.length,
      processIds: [...new Set(combinedCapture.rows.map(row => row.ProcessID))], swapChainCounts: Object.fromEntries([...new Set(combinedCapture.rows.map(row => row.SwapChainAddress))].map(key => [key, combinedCapture.rows.filter(row => row.SwapChainAddress === key).length])),
      vidPnSourceCounts: Object.fromEntries([...new Set(combinedCapture.rows.map(row => row.VidPnSourceId))].map(key => [key, combinedCapture.rows.filter(row => row.VidPnSourceId === key).length])),
      intervals: { firstUtc: new Date(combinedCapture.rows[0].displayedAt).toISOString(), lastUtc: new Date(combinedCapture.rows.at(-1).displayedAt).toISOString() } },
    modes,
    combinedPresentation: modes.combined.presentation
  };
}
const path = `${dir}/matrix-analysis-user-r4.json`;
await writeFile(path, JSON.stringify(report, null, 2));
console.log(JSON.stringify({ path, captures: Object.fromEntries(Object.entries(report.builds).map(([key, value]) => [key, { complete: value.capture.captureComplete && value.combinedCapture.captureComplete, baseRows: value.capture.frameRows, combinedRows: value.combinedCapture.frameRows, swapchains: value.combinedCapture.swapChainCounts, sources: value.combinedCapture.vidPnSourceCounts }])), modes: Object.fromEntries(Object.entries(report.builds).map(([key, value]) => [key, Object.fromEntries(Object.entries(value.modes).map(([mode, item]) => [mode, { rows: item.presentation.rows, p95: item.presentation.p95Ms, max: item.presentation.maxMs, over50: item.presentation.over50ms, transition15: item.outlierClassificationForEachTransition[14]?.interval.outliersOver50ms }]))])) }));
