import assert from 'node:assert/strict';
import { resourceGate } from './a5-resource-gate.mjs';
import { createHash } from 'node:crypto';
import { access, readFile, writeFile } from 'node:fs/promises';
import { basename, join } from 'node:path';
import { summarizeCommandCoverage } from './a5-command-coverage.mjs';

const root = process.argv[4] || 'dist/performance/a5-boundary-2026-10-12';
const builds = ['baseline', 'candidate'];
const thresholdMs = 50;
const mode = process.argv[2] || 'animation-only';
const captureSuffix = process.argv[3] || 'pair-r1';
assert(['animation-only', 'combined'].includes(mode));
const metadataOnlyBinaries = process.argv.includes('--metadata-only-binaries');
const artifactPath = async (dir, original) => {
  const portable = join(dir, basename(original.replaceAll('\\', '/')));
  try { await access(portable); return portable; } catch { return original; }
};

function parseCsv(text) {
  const lines = text.replace(/^\uFEFF/, '').trim().split(/\r?\n/);
  const split = line => {
    const values = []; let value = ''; let quoted = false;
    for (let i = 0; i < line.length; i++) {
      const char = line[i];
      if (char === '"') {
        if (quoted && line[i + 1] === '"') { value += '"'; i++; }
        else quoted = !quoted;
      } else if (char === ',' && !quoted) { values.push(value); value = ''; }
      else value += char;
    }
    values.push(value);
    return values;
  };
  const headers = split(lines[0]);
  return lines.slice(1).filter(Boolean).map(line => Object.fromEntries(split(line).map((v, i) => [headers[i], v])));
}

const quantile = (values, p) => {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted.length ? sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * p))] : null;
};
const iso = ms => Number.isFinite(ms) ? new Date(ms).toISOString() : null;
const maximum = values => values.length ? Math.max(...values) : null;
const overlaps = (a0, a1, b0, b1) => a0 < b1 && a1 > b0;
const overlapMs = (a0, a1, b0, b1) => Math.max(0, Math.min(a1, b1) - Math.max(a0, b0));
assert.equal(overlapMs(0, 100, 90, 120), 10);
assert.equal(overlapMs(0, 100, 100, 120), 0);

const result = {
  generatedAt: new Date().toISOString(),
  measurementVersion: 3,
  mode,
  presentMonTiming: {
    source: 'PresentMon TimeInQPC is the Present start QPC; bounds probe QPC/UTC pairs map it onto the benchmark UTC clock. MsUntilDisplayed is added to derive displayedAtUtc.',
    displayInterval: 'For each displayed frame, intervalStartUtc = displayedAtUtc - MsBetweenDisplayChange; interval selection uses interval/window intersection, not only displayedAtUtc.',
    sourceUrl: 'https://github.com/GameTechDev/PresentMon/blob/v2.6.0/IntelPresentMon/CommonUtilities/mc/MetricsCalculatorAnimation.cpp',
  },
  thresholdMs,
  builds: {},
};

for (const build of builds) {
  const suffix = `${build}-${captureSuffix}`;
  const dir = `${root}/${build}`;
  const status = JSON.parse((await readFile(`${dir}/capture-${suffix}.status.json`, 'utf8')).replace(/^\uFEFF/, ''));
  for (const key of ['csvPath', 'matrixJsonPath', 'boundsCsvPath']) status[key] = await artifactPath(dir, status[key]);
  const rawText = await readFile(status.csvPath, 'utf8');
  const rawRows = parseCsv(rawText);
  const matrix = JSON.parse(await readFile(status.matrixJsonPath, 'utf8'));
  const bounds = parseCsv(await readFile(status.boundsCsvPath, 'utf8'));
  const clockAnchors = bounds.map(row => ({
    qpc: Number(row.Qpc), frequency: Number(row.QpcFrequency), utc: Date.parse(row.Utc),
  })).filter(anchor => Number.isFinite(anchor.qpc) && Number.isFinite(anchor.frequency) && Number.isFinite(anchor.utc))
    .sort((a, b) => a.qpc - b.qpc);
  assert(clockAnchors.length > 1, `${build} needs at least two QPC/UTC clock anchors`);
  assert(clockAnchors.every(anchor => anchor.frequency > 0));
  const anchorSpacingMs = clockAnchors.slice(1).map((a, i) => (a.qpc - clockAnchors[i].qpc) * 1000 / a.frequency);
  const qpcToUtc = qpc => {
    let low = 0; let high = clockAnchors.length;
    while (low < high) { const mid = (low + high) >> 1; if (clockAnchors[mid].qpc < qpc) low = mid + 1; else high = mid; }
    const index = Math.min(clockAnchors.length - 1, low);
    const anchor = clockAnchors[index];
    return anchor.utc + (qpc - anchor.qpc) * 1000 / anchor.frequency;
  };
  assert.equal(matrix.mode, mode);
  assert.equal(status.captureComplete, true);
  assert.equal(new Set(clockAnchors.map(a => a.frequency)).size, 1);
  assert(matrix.toggleRecords.length > 0, `${build} has no transition records`);

  const transitions = matrix.toggleRecords.map(record => {
    const start = Date.parse(record.firstStyleMutation.at);
    const lastMutation = Date.parse(record.lastStyleMutation.at);
    const confirmed = Date.parse(record.settleConfirmation.at);
    return {
      index: record.index,
      direction: record.fromExpanded ? 'collapse' : 'expand',
      clickAtUtc: record.clickAt,
      clickDispatchedUtc: record.clickDispatched?.at ?? null,
      clickToFirstMutationMs: record.clickToFirstMutationMs ?? null,
      preDispatchToFirstMutationMs: start - Date.parse(record.clickAt),
      firstStyleMutationUtc: record.firstStyleMutation.at,
      lastStyleMutationUtc: record.lastStyleMutation.at,
      confirmationUtc: record.settleConfirmation.at,
      firstToLastStyleMutationMs: record.firstToLastStyleMutationMs,
      firstMutationToConfirmationMs: record.firstMutationToConfirmationMs,
      lastMutationToConfirmationMs: confirmed - lastMutation,
      mutationCount: record.mutationCount,
      transitionEvents: record.transitionEvents,
      _start: start,
      _last: lastMutation,
      _confirmed: confirmed,
    };
  });
  const firstWindowStart = Math.min(...transitions.map(t => t._start));
  const lastWindowEnd = Math.max(...transitions.map(t => t._confirmed));
  const commandRecords = [...matrix.mediaCommands, ...matrix.volumeCommands];
  const commandCoverage = matrix.transitionControls ? summarizeCommandCoverage(matrix.toggleRecords, commandRecords) : null;
  if (commandCoverage) assert.equal(commandCoverage.accepted, true, `${build} page-clock media AND volume coverage must be 16/16`);
  for (const transition of transitions) {
    assert(transition._start <= transition._last && transition._last <= transition._confirmed, `${build} invalid transition boundaries`);
    const pageTransition = matrix.toggleRecords.find(r => r.index === transition.index);
    transition.overlappingCommands = commandRecords.filter(command => Number.isFinite(command.pageStartedPerformanceMs)
      ? overlaps(command.pageStartedPerformanceMs, command.pageEndedPerformanceMs, pageTransition.firstStyleMutation.performanceMs, pageTransition.lastStyleMutation.performanceMs)
      : overlaps(Date.parse(command.startedAt), Date.parse(command.endedAt), transition._start, transition._last))
      .map(command => ({ name: command.name, ok: command.ok, transitionIndex: command.transitionIndex,
        clock: Number.isFinite(command.pageStartedPerformanceMs) ? 'same-page performance.now' : 'legacy Node/CDP envelope',
        overlapMs: Number.isFinite(command.pageStartedPerformanceMs)
          ? overlapMs(command.pageStartedPerformanceMs, command.pageEndedPerformanceMs, pageTransition.firstStyleMutation.performanceMs, pageTransition.lastStyleMutation.performanceMs)
          : overlapMs(Date.parse(command.startedAt), Date.parse(command.endedAt), transition._start, transition._last) }));
  }

  let missingPresentStartRows = 0;
  let missingMsUntilDisplayedRows = 0;
  let missingDisplayIntervalRows = 0;
  const frames = [];
  for (const row of rawRows) {
    const presentStartQpc = row.TimeInQPC?.trim() ? Number(row.TimeInQPC) : NaN;
    const presentStart = qpcToUtc(presentStartQpc);
    const msUntilDisplayed = Number(row.MsUntilDisplayed);
    const intervalMs = Number(row.MsBetweenDisplayChange);
    if (!Number.isFinite(presentStart)) { missingPresentStartRows++; continue; }
    if (!Number.isFinite(msUntilDisplayed)) { missingMsUntilDisplayedRows++; continue; }
    const displayedAt = presentStart + msUntilDisplayed;
    if (!Number.isFinite(intervalMs) || intervalMs <= 0) { missingDisplayIntervalRows++; continue; }
    frames.push({
      ...row,
      presentStartQpc,
      presentStartUtc: presentStart,
      displayedAtUtc: displayedAt,
      intervalStartUtc: displayedAt - intervalMs,
      intervalMs,
    });
  }

  const intersectingFrames = frames.filter(frame => frame.intervalStartUtc < lastWindowEnd && frame.displayedAtUtc > firstWindowStart);
  const swapChainMap = new Map();
  for (const frame of frames) {
    const key = frame.SwapChainAddress;
    const entry = swapChainMap.get(key) ?? { swapChainAddress: key, rowCount: 0, displayedDuringTransitionCount: 0, transitionIndices: new Set(), vidPnSources: new Set(), layerIndices: new Set() };
    entry.rowCount++;
    entry.vidPnSources.add(frame.VidPnSourceId);
    entry.layerIndices.add(frame.LayerIndex);
    for (const transition of transitions) {
      if (frame.displayedAtUtc >= transition._start && frame.displayedAtUtc <= transition._last) {
        entry.displayedDuringTransitionCount++;
        entry.transitionIndices.add(transition.index);
      }
    }
    swapChainMap.set(key, entry);
  }
  const swapChains = [...swapChainMap.values()].map(entry => ({
    swapChainAddress: entry.swapChainAddress,
    rowCount: entry.rowCount,
    displayedDuringTransitionCount: entry.displayedDuringTransitionCount,
    transitionCountWithPresents: entry.transitionIndices.size,
    transitionIndicesWithPresents: [...entry.transitionIndices].sort((a, b) => a - b),
    vidPnSourceIds: [...entry.vidPnSources],
    layerIndices: [...entry.layerIndices],
  })).sort((a, b) => b.transitionCountWithPresents - a.transitionCountWithPresents || b.displayedDuringTransitionCount - a.displayedDuringTransitionCount);
  const mainIslandSwapchain = swapChains[0]?.transitionCountWithPresents > 0 && swapChains[0].transitionCountWithPresents > (swapChains[1]?.transitionCountWithPresents ?? 0)
    ? { ...swapChains[0], mapping: 'unique-present-correlation-during-main-HWND style-mutation intervals', confidence: 'high temporal match; PresentMon does not emit HWND' }
    : { swapChainAddress: null, mapping: 'ambiguous: no unique swapchain correlated to the recorded transitions', confidence: 'unresolved' };
  assert(mainIslandSwapchain.swapChainAddress, `${build} main-island swapchain is ambiguous; do not publish a selected-chain summary`);
  if (commandCoverage) assert.equal(mainIslandSwapchain.transitionCountWithPresents, 16, `${build} primary swapchain must contain presents in all 16 mutation spans`);

  const targetRows = mainIslandSwapchain.swapChainAddress
    ? intersectingFrames.filter(frame => frame.SwapChainAddress === mainIslandSwapchain.swapChainAddress)
    : intersectingFrames;
  const mapInterval = frame => {
    const active = transitions.filter(t => overlaps(frame.intervalStartUtc, frame.displayedAtUtc, t._start, t._last));
    const confirmationTail = transitions.filter(t => overlaps(frame.intervalStartUtc, frame.displayedAtUtc, t._last, t._confirmed));
    const fullyInsideActive = transitions.filter(t => frame.intervalStartUtc >= t._start && frame.displayedAtUtc <= t._last);
    const fullyInsideTail = transitions.filter(t => frame.intervalStartUtc >= t._last && frame.displayedAtUtc <= t._confirmed);
    let classification = 'outside-transition-windows';
    if (fullyInsideActive.length) classification = 'animation-window';
    else if (fullyInsideTail.length) classification = 'confirmation-tail';
    else if (active.length || confirmationTail.length) classification = 'crosses-transition-boundary';
    return {
      processId: frame.ProcessID,
      swapChainAddress: frame.SwapChainAddress,
      vidPnSourceId: frame.VidPnSourceId,
      layerIndex: frame.LayerIndex,
      presentStartUtc: iso(frame.presentStartUtc),
      presentStartUtcMs: frame.presentStartUtc,
      presentStartQpc: frame.presentStartQpc,
      displayedAtUtc: iso(frame.displayedAtUtc),
      intervalStartUtc: iso(frame.intervalStartUtc),
      displayedAtUtcMs: frame.displayedAtUtc,
      intervalStartUtcMs: frame.intervalStartUtc,
      msUntilDisplayed: Number(frame.MsUntilDisplayed),
      msBetweenDisplayChange: frame.intervalMs,
      msCpuBusy: Number(frame.MsCPUBusy),
      msGpuTime: Number(frame.MsGPUTime),
      classification,
      animationTransitions: active.map(t => t.index),
      confirmationTransitions: confirmationTail.map(t => t.index),
      fullyInsideAnimationTransitions: fullyInsideActive.map(t => t.index),
      fullyInsideConfirmationTransitions: fullyInsideTail.map(t => t.index),
      activeOverlapMs: active.reduce((sum, t) => sum + overlapMs(frame.intervalStartUtc, frame.displayedAtUtc, t._start, t._last), 0),
      activeOverlapByTransition: active.map(t => ({ index: t.index, ms: overlapMs(frame.intervalStartUtc, frame.displayedAtUtc, t._start, t._last) })),
      confirmationOverlapMs: confirmationTail.reduce((sum, t) => sum + overlapMs(frame.intervalStartUtc, frame.displayedAtUtc, t._last, t._confirmed), 0),
    };
  };
  const intervalRows = targetRows.map(mapInterval);
  const allSwapchainIntervals = intersectingFrames.map(mapInterval);
  const summarize = rows => {
    const values = rows.map(row => row.msBetweenDisplayChange);
    return {
      rows: rows.length,
      p50Ms: quantile(values, .50), p95Ms: quantile(values, .95), p99Ms: quantile(values, .99),
      maxMs: values.length ? Math.max(...values) : null,
      over16_7ms: values.filter(v => v > 16.7).length,
      over50ms: values.filter(v => v > thresholdMs).length,
    };
  };
  const outliers = intervalRows.filter(row => row.msBetweenDisplayChange > thresholdMs);
  const activeSeconds = transitions.reduce((sum, t) => sum + (t._last - t._start) / 1000, 0);
  const byClass = Object.fromEntries(['animation-window', 'confirmation-tail', 'crosses-transition-boundary', 'outside-transition-windows'].map(classification => [classification, summarize(intervalRows.filter(row => row.classification === classification))]));
  const rawSha256 = createHash('sha256').update(await readFile(status.csvPath)).digest('hex').toUpperCase();
  if (commandCoverage) {
    assert(clockAnchors.at(-1).utc >= lastWindowEnd, `${build} bounds probe truncated the confirmation tail`);
    assert(Math.max(...frames.map(f => f.displayedAtUtc)) >= lastWindowEnd, `${build} displayed-frame records do not reach final confirmation`);
    assert(!matrix.failure && matrix.restoration?.accepted, `${build} failure or state restoration gap`);
  }
  assert.equal(rawSha256, status.csvSha256, `${build} raw CSV hash does not match capture status`);
  const binarySha256 = metadataOnlyBinaries ? status.binarySha256 : createHash('sha256').update(await readFile(status.binaryPath)).digest('hex').toUpperCase();
  const nativeDllSha256 = metadataOnlyBinaries ? status.nativeDllSha256 : createHash('sha256').update(await readFile(status.nativeDllPath)).digest('hex').toUpperCase();
  assert.equal(binarySha256, status.binarySha256);
  assert.equal(nativeDllSha256, status.nativeDllSha256);
  const ipc = matrix.instrumentation;
  if (status.instrumented) assert(ipc?.records.length > 0 && ipc.dropped === 0, `${build} diagnostic requested but proxy data is missing/truncated`);
  const ipcPerTransition = ipc ? matrix.toggleRecords.map(t => {
    const lo = t.firstStyleMutation.performanceMs, hi = t.lastStyleMutation.performanceMs;
    const dispatched = ipc.records.filter(r => r.startedPerformanceMs >= t.clickDispatched.performanceMs && r.startedPerformanceMs <= t.settleConfirmation.performanceMs);
    const region = dispatched.filter(r => r.command === 'set_island_interaction_region');
    const events = ipc.records.filter(r => r.startedPerformanceMs < hi && (r.endedPerformanceMs ?? Infinity) > lo)
      .flatMap(r => [[Math.max(lo, r.startedPerformanceMs), 1], [Math.min(hi, r.endedPerformanceMs ?? hi), -1]])
      .sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    let active = 0, peak = 0;
    for (const [, delta] of events) { active += delta; peak = Math.max(peak, active); }
    return { index: t.index, invokeRequests: dispatched.length, argumentUtf8Bytes: dispatched.reduce((n, r) => n + r.argumentBytes, 0),
      regionRequests: region.length, regionArgumentUtf8Bytes: region.reduce((n, r) => n + r.argumentBytes, 0), activePeakInFlight: peak };
  }) : null;
  let resources = { available: false, reason: 'Resource capture not requested' };
  if (status.resourceJsonPath) {
    try {
      const resourcePath = await artifactPath(dir, status.resourceJsonPath);
      const bytes = await readFile(resourcePath);
      const report = JSON.parse(bytes.toString('utf8').replace(/^\uFEFF/, ''));
      resources = { available: true, resourcePath, sha256: createHash('sha256').update(bytes).digest('hex'), ...resourceGate(report, matrix, status) };
    } catch (error) { resources = { available: false, accepted: false, reason: String(error) }; }
  }

  result.builds[build] = {
    capture: {
      rootProcessId: status.rootProcessId,
      mainWindowProcessId: status.mainWindowProcessId,
      mainHwnd: bounds[0]?.Hwnd ?? null,
      webViewGpuProcessId: status.webViewGpuProcessId,
      webViewGpuCommandLine: status.webViewGpuCommandLine,
      timezoneId: status.timezoneId,
      timezoneOffsetMinutes: status.timezoneOffsetMinutes,
      qpcClockAnchors: clockAnchors.length,
      qpcFrequency: clockAnchors[0].frequency,
      clockMapping: 'First QPC/UTC anchor at or after the Present start QPC; convert delta using that anchor frequency. Outside probe range extrapolates from an endpoint; UTC/mutation stamps have millisecond resolution.',
      firstClockAnchor: clockAnchors[0],
      lastClockAnchor: clockAnchors.at(-1),
      actualClockAnchorSpacingP50Ms: quantile(anchorSpacingMs, .5),
      actualClockAnchorSpacingMaxMs: maximum(anchorSpacingMs),
      binarySha256,
      nativeDllSha256,
      binaryVerification: metadataOnlyBinaries ? 'capture metadata only; binaries are not included in this replay' : 'EXE and DLL bytes hashed and matched to capture metadata',
      matrixJsonPath: status.matrixJsonPath,
      matrixJsonSha256: createHash('sha256').update(await readFile(status.matrixJsonPath)).digest('hex'),
      boundsCsvPath: status.boundsCsvPath,
      boundsCsvSha256: createHash('sha256').update(await readFile(status.boundsCsvPath)).digest('hex'),
      startedUtc: status.startedUtc,
      finishedUtc: status.finishedUtc,
      presentMonVersion: status.presentMonVersion,
      presentMonArguments: status.presentMonArguments,
      presentMonCaptureComplete: status.captureComplete,
      rawCsvPath: status.csvPath,
      rawCsvSha256: rawSha256,
      rawCsvBytes: status.csvBytes,
      rawPresentMonRows: rawRows.length,
      frameRowsWithDisplayTime: frames.length,
      missingPresentStartRows,
      missingMsUntilDisplayedRows,
      missingDisplayIntervalRows,
      boundsRows: bounds.length,
      monitorDeviceIds: [...new Set(bounds.map(row => row.MonitorDevice))],
      observedMainHwnd: bounds[0]?.Hwnd ?? null,
      sampledDistinctRects: [...new Set(bounds.map(r => [r.Left, r.Top, r.Right, r.Bottom].join(',')))],
      sampledRectChangeCount: bounds.slice(1).filter((r, i) => ['Left', 'Top', 'Right', 'Bottom'].some(k => r[k] !== bounds[i][k])).length,
    },
    swapChains,
    mainIslandSwapchain,
    transitions: transitions.map(({ _start, _last, _confirmed, ...transition }) => transition),
    commands: { media: matrix.mediaCommands, volume: matrix.volumeCommands, finalState: matrix.controlResult },
    commandCoverage,
    restoration: matrix.restoration ?? null,
    setup: matrix.setup,
    instrumented: status.instrumented ?? false,
    ipc: ipc ? { scope: ipc.scope, dropped: ipc.dropped, peakInFlight: ipc.peakInFlight, inFlightAtStop: ipc.inFlightAtStop, perTransition: ipcPerTransition } : null,
    resourceRequested: Boolean(status.resourceJsonPath),
    resources,
    diagnosticSummary: {
      activeSeconds,
      fullyInsideActiveOver50msPerActiveSecond: activeSeconds ? outliers.filter(row => row.classification === 'animation-window').length / activeSeconds : null,
      transitionsOverlappingCommands: transitions.filter(t => t.overlappingCommands.length).map(t => t.index),
      clickDelayDefinition: 'Browser performance.now from programmatic click dispatch to first style mutation; not physical input latency. Earlier captures lack click-dispatch stamps.',
      clickToFirstMutationP50Ms: quantile(transitions.map(t => t.clickToFirstMutationMs).filter(Number.isFinite), .5),
      clickToFirstMutationMaxMs: maximum(transitions.map(t => t.clickToFirstMutationMs).filter(Number.isFinite)),
      over50msActiveOverlapTotalMs: outliers.reduce((sum, row) => sum + row.activeOverlapMs, 0),
      over50msActiveOverlapMaxMs: Math.max(...outliers.map(row => row.activeOverlapMs), 0),
    },
    intersectingPresentIntervals: {
      windowStartUtc: iso(firstWindowStart),
      windowEndUtc: iso(lastWindowEnd),
      selectionRule: 'Keep rows where intervalStartUtc < windowEndUtc AND displayedAtUtc > windowStartUtc.',
      allSwapchains: summarize(allSwapchainIntervals),
      selectedMainSwapchain: summarize(intervalRows),
      bySwapchain: Object.fromEntries(swapChains.map(chain => [chain.swapChainAddress, summarize(allSwapchainIntervals.filter(row => row.swapChainAddress === chain.swapChainAddress))])),
      byClassification: byClass,
      over50msRows: intervalRows.filter(row => row.msBetweenDisplayChange > thresholdMs),
    },
  };
}

const analysisName = ['pair-r1', 'combined-r1'].includes(captureSuffix)
  ? (mode === 'animation-only' ? 'paired-analysis' : 'combined-paired-analysis')
  : `${mode}-${captureSuffix}-analysis`;
const outPath = `${root}/${analysisName}.json`;
await writeFile(outPath, JSON.stringify(result, null, 2));
console.log(JSON.stringify({
  outPath,
  builds: Object.fromEntries(Object.entries(result.builds).map(([build, data]) => [build, {
    mainIslandSwapchain: data.mainIslandSwapchain,
    rawPresentMonRows: data.capture.rawPresentMonRows,
    missingMsUntilDisplayedRows: data.capture.missingMsUntilDisplayedRows,
    capturedTransitions: data.transitions.length,
    classes: data.intersectingPresentIntervals.byClassification,
  }]))
}));

for (const [build,data] of Object.entries(result.builds)) if(data.resourceRequested) assert(data.resources.available && data.resources.accepted, `${build} resource gate failed`);
