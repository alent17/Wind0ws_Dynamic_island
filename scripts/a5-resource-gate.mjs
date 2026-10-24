export function resourceGate(report, matrix, status) {
  const errors = [];
  const ready = report.readiness;
  const start = Date.parse(matrix.sequenceWindow?.start.at);
  const end = Date.parse(matrix.sequenceWindow?.end.at);
  if (!(end > start)) errors.push('Missing/invalid explicit sequence window');
  if (!ready || !report.runId || ready.runId !== report.runId || ready.runId !== status.resourceRunId || ready.rootPid !== status.rootProcessId || report.rootPid !== status.rootProcessId || !(Date.parse(ready.readyUtc) >= Date.parse(report.probeStartedUtc)) || !(Date.parse(ready.readyUtc) <= start) || ready.validIncrementSampleIndex < 1 || !status.resourceReadiness || ready.validIncrementSampleIndex !== status.resourceReadiness.validIncrementSampleIndex || Date.parse(ready.readyUtc) !== Date.parse(status.resourceReadiness.readyUtc) || JSON.stringify(ready.processKeys) !== JSON.stringify(status.resourceReadiness.processKeys)) errors.push('Stale/missing/mismatched readiness');
  const samples = report.samples.map((s, index) => {
    const rows = s.processes.map(p => {
      const lo = Date.parse(p.previousCpuReadStartedUtc), hi = Date.parse(p.cpuReadEndedUtc);
      const valid = p.cpuSampleAvailable && Number.isFinite(p.cpuDeltaSeconds) && p.cpuDeltaSeconds >= 0 && hi > lo && Math.abs(p.cumulativeCpuSeconds - p.previousCumulativeCpuSeconds - p.cpuDeltaSeconds) < 1e-7;
      const category = !valid ? 'baseline/unavailable' : hi <= start ? 'before' : lo >= end ? 'after' : lo >= start && hi <= end ? 'inside' : 'boundary';
      return { pid:p.pid, processKey:p.processKey, valid, category, intervalStartUtc:p.previousCpuReadStartedUtc, intervalEndUtc:p.cpuReadEndedUtc, cpuDeltaSeconds:p.cpuDeltaSeconds };
    });
    return { index, valid: rows.length > 0 && rows.every(r=>r.valid), category: rows.length && rows.every(r=>r.category === rows[0].category) ? rows[0].category : 'mixed/boundary', rows };
  });
  const readySample = samples[ready?.validIncrementSampleIndex];
  if (!readySample?.valid || !readySample.rows.some(p=>p.pid===status.rootProcessId) || readySample.rows.some(p=>Date.parse(p.intervalEndUtc)>Date.parse(ready?.readyUtc))) errors.push('Readiness lacks valid completed initial increment');
  const inside = samples.filter(s=>s.valid && s.category==='inside');
  if (inside.length < 3) errors.push('Fewer than 3 complete tree CPU increment samples inside sequence');
  if (!samples.some(s=>s.valid && s.category==='before')) errors.push('Missing pre-sequence increment');
  if (!samples.some(s=>s.valid && s.category==='after')) errors.push('Tail truncated: missing complete post-sequence increment');
  const totals = Object.fromEntries(['before','inside','boundary','after'].map(c=>[c,samples.flatMap(s=>s.rows).filter(r=>r.valid && r.category===c).reduce((n,r)=>n+r.cpuDeltaSeconds,0)]));
  const range = values => { const v=values.filter(Number.isFinite); return v.length ? [Math.min(...v),Math.max(...v)] : null; };
  return { accepted:!errors.length, errors, sequenceWindow:matrix.sequenceWindow, sequenceDurationSeconds:(end-start)/1000, completeInsideSamples:inside.length, cpuSecondsByInterval:totals,
    memoryPrivateBytesSampledRange:range(report.samples.map(s=>s.totalPrivateBytes)), rendererPrivateBytesSampledRange:range(report.samples.map(s=>s.processes.filter(p=>p.role==='renderer').reduce((n,p)=>n+p.privateBytes,0))), threadsSampledRange:range(report.samples.map(s=>s.totalThreads)), handlesSampledRange:range(report.samples.map(s=>s.totalHandles)), gpuCommittedBytesSampledRange:range(report.samples.filter(s=>s.processes.some(p=>p.gpuMemory?.length)).map(s=>s.processes.flatMap(p=>p.gpuMemory).reduce((n,g)=>n+g.committedBytes,0))),
    scope:'CPU uses conservative previous-read-start through current-read-end per PID/start-time. Boundary increments remain whole and separate; no attribution to pure animation. Sequence includes controls, confirmation and gaps. Ranges cover all sampled phases, not guaranteed peaks.', samples };
}
