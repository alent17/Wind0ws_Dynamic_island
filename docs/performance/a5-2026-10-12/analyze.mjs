import { readFile, writeFile } from 'node:fs/promises';
const dir = 'dist/performance/c1-2026-10-12';
const out = 'dist/performance/a5-2026-10-12/analysis.json';
function parseCsv(text) {
  const lines = text.trim().split(/\r?\n/);
  const split = line => { const out=[]; let v='', q=false; for(let i=0;i<line.length;i++){ const c=line[i]; if(c==='"'){if(q&&line[i+1]==='"'){v+='"';i++;}else q=!q;} else if(c===','&&!q){out.push(v);v='';} else v+=c;} out.push(v); return out; };
  const head=split(lines[0]); return lines.slice(1).map(line=>Object.fromEntries(split(line).map((v,i)=>[head[i],v])));
}
const quant=(values,p)=>{const s=[...values].sort((a,b)=>a-b);return s.length?s[Math.min(s.length-1,Math.floor(s.length*p))]:null;};
const parseLocal=value=>{const m=/^(\d{4})-(\d{2})-(\d{2}) (\d{1,2}):(\d{2}):(\d{2})(?:\.(\d+))?$/.exec(value);if(!m)return NaN;return Date.UTC(Number(m[1]),Number(m[2])-1,Number(m[3]),Number(m[4]),Number(m[5]),Number(m[6]),Number((m[7]||'').padEnd(3,'0').slice(0,3)||0));};
const summarize=rows=>{const times=rows.map(r=>r.intervalMs);return {rows:rows.length,p50:quant(times,.5),p95:quant(times,.95),p99:quant(times,.99),max:times.length?Math.max(...times):null,over16_7:times.filter(x=>x>16.7).length,over50:times.filter(x=>x>50).length,cpuBusyP95:quant(rows.map(r=>r.cpuBusy).filter(Number.isFinite),.95),gpuTimeP95:quant(rows.map(r=>r.gpuTime).filter(Number.isFinite),.95)};};
const result={generatedAt:new Date().toISOString(),builds:{}};
for(const label of ['baseline','candidate']){
 result.builds[label]={rounds:[]};
 for(let n=1;n<=3;n++){
  const suffix=`a5-fixed-${label}-r${n}`;
  const status=JSON.parse(await readFile(`${dir}/matrix-presentmon-c1-${suffix}.csv.status.json`,'utf8'));
  const rows=parseCsv(await readFile(`${dir}/matrix-presentmon-c1-${suffix}.csv`,'utf8'));
  const first=rows.find(r=>r.CPUStartDateTime&&r.CPUStartDateTime!=='NA');
  const offsetHours=Math.round((parseLocal(first.CPUStartDateTime)-Date.parse(status.startedUtc))/3600000);
  const frames=rows.map(r=>{const displayedAt=parseLocal(r.TimeInDateTime)-offsetHours*3600000;const intervalMs=Number(r.MsBetweenDisplayChange);return {...r,displayedAt,intervalMs,intervalStart:displayedAt-intervalMs,cpuBusy:Number(r.MsCPUBusy),gpuTime:Number(r.MsGPUTime)};}).filter(r=>Number.isFinite(r.displayedAt)&&Number.isFinite(r.intervalMs));
  const bounds=parseCsv(await readFile(`${dir}/matrix-bounds-c1-${suffix}.csv`,'utf8'));
  const modes={};
  for(const mode of ['animation-only','combined']){
    const data=JSON.parse(await readFile(`${dir}/matrix-c1-${mode}-${suffix}.json`,'utf8'));
    const transitions=data.toggleRecords.map(t=>({index:t.index,start:Date.parse(t.animationStart.at),end:Date.parse(t.animationEnd.at),duration:t.observedDurationMs}));
    const start=Math.min(...transitions.map(t=>t.start)), end=Math.max(...transitions.map(t=>t.end));
    const windowRows=frames.filter(r=>r.displayedAt>=start&&r.displayedAt<=end);
    const outliers=windowRows.filter(r=>r.intervalMs>50).map(r=>{const overlap=transitions.filter(t=>r.intervalStart<t.end&&r.displayedAt>t.start);const inside=transitions.filter(t=>r.intervalStart>=t.start&&r.displayedAt<=t.end);return {ms:r.intervalMs,cpuBusy:r.cpuBusy,gpuTime:r.gpuTime,displayedAt:new Date(r.displayedAt).toISOString(),intervalStart:new Date(r.intervalStart).toISOString(),classification:inside.length?'inside-animation':overlap.length?'crosses-boundary':'outside-animation',transitions:inside.length?inside.map(t=>t.index):overlap.map(t=>t.index),pid:r.ProcessID,swapChain:r.SwapChainAddress,source:r.VidPnSourceId};});
    const stableRows=transitions.flatMap(t=>frames.filter(r=>r.displayedAt>=t.start&&r.displayedAt<=t.end));
    const insideRows=windowRows.filter(r=>transitions.some(t=>r.intervalStart>=t.start&&r.displayedAt<=t.end));
    modes[mode]={transitionCount:transitions.length,transitionDurationMs:{p50:quant(transitions.map(t=>t.duration),.5),p95:quant(transitions.map(t=>t.duration),.95),min:Math.min(...transitions.map(t=>t.duration)),max:Math.max(...transitions.map(t=>t.duration))},presentationWindow:summarize(windowRows),intervalsDuringAnimation:summarize(stableRows),fullyInsideIntervalSummary:summarize(insideRows),longIntervalsOver50ms:outliers};
  }
  const controls=JSON.parse(await readFile(`${dir}/matrix-c1-controls-only-${suffix}.json`,'utf8'));
  const animation=JSON.parse(await readFile(`${dir}/matrix-c1-animation-only-${suffix}.json`,'utf8'));
  const combined=JSON.parse(await readFile(`${dir}/matrix-c1-combined-${suffix}.json`,'utf8'));
  const uniqueBounds=[...new Set(bounds.map(r=>`${r.Left},${r.Top},${r.Right},${r.Bottom}`))];
  result.builds[label].rounds.push({round:n,capture:{complete:status.captureComplete,elevated:status.elevated,startedUtc:status.startedUtc,finishedUtc:status.finishedUtc,bytes:status.outputBytes,offsetHours,processIds:[...new Set(frames.map(r=>r.ProcessID))],swapChains:[...new Set(frames.map(r=>r.SwapChainAddress))],vidPnSources:[...new Set(frames.map(r=>r.VidPnSourceId))]},conditions:{monitor:animation.setup.monitors[1]?.name,scaleFactor:animation.setup.monitors[1]?.scaleFactor,sessions:animation.setup.sessions.map(s=>({id:s.id,source:s.source,isPlaying:s.isPlaying})),volume:animation.setup.initialVolume,mediaPaused:!animation.setup.initialMedia.isPlaying,mediaCommandSuccess:controls.mediaCommands.every(x=>x.ok),volumeCommandSuccess:controls.volumeCommands.every(x=>x.ok),finalVolume:controls.controlResult?.finalAudio?.volumePercent,finalPaused:controls.controlResult?.finalMedia? !controls.controlResult.finalMedia.isPlaying:null,windowDisplayCounts:Object.fromEntries([...new Set(bounds.map(r=>r.MonitorDevice))].map(d=>[d,bounds.filter(r=>r.MonitorDevice===d).length])),uniqueWindowRects:uniqueBounds.length},modes});
 }
}
for(const label of ['baseline','candidate']) for(const mode of ['animation-only','combined']){
 const rounds=result.builds[label].rounds.map(r=>r.modes[mode]);
 const durations=rounds.flatMap(r=>Array.from({length:r.transitionCount},()=>r.transitionDurationMs.p50));
 result.builds[label][mode]={roundMedianTransitionP50:quant(rounds.map(r=>r.transitionDurationMs.p50),.5),rounds:rounds.map(r=>({transitionDurationMs:r.transitionDurationMs,presentationWindow:r.presentationWindow,intervalsDuringAnimation:r.intervalsDuringAnimation,fullyInsideIntervalSummary:r.fullyInsideIntervalSummary,longIntervalsOver50ms:r.longIntervalsOver50ms}))};
}
await writeFile(out,JSON.stringify(result,null,2));
console.log(JSON.stringify({out,builds:Object.fromEntries(Object.entries(result.builds).map(([name,b])=>[name,Object.fromEntries(['animation-only','combined'].map(mode=>[mode,{roundMedianTransitionP50:b[mode].roundMedianTransitionP50,rounds:b[mode].rounds.map(r=>({duration:r.transitionDurationMs.p50,p95:r.presentationWindow.p95,max:r.presentationWindow.max,over50:r.presentationWindow.over50,inside:r.fullyInsideIntervalSummary.over50}))}]))])),conditions:result.builds.candidate.rounds.map(r=>r.conditions)}));
