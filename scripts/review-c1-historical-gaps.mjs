import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { join } from 'node:path';
const root=process.argv[2] || 'dist/performance/c1-review-2026-10-14';
const input=join(root,'inputs');
const json=async p=>JSON.parse((await readFile(p,'utf8')).replace(/^\uFEFF/,''));
const hashes=await json(join(root,'source-hashes.json'));
for(const h of hashes){const b=await readFile(join(input,h.name));assert.equal(b.length,h.bytes);assert.equal(createHash('sha256').update(b).digest('hex'),h.sha256);}
const csv=async name=>{
  const text=(await readFile(join(input,name),'utf8')).replace(/^\uFEFF/,'');
  const lines=text.trimEnd().split(/\r?\n/);
  const split=line=>{let value='',quoted=false;const fields=[];for(let i=0;i<line.length;i++){const c=line[i];if(c==='"'){if(quoted&&line[i+1]==='"'){value+='"';i++;}else quoted=!quoted;}else if(c===','&&!quoted){fields.push(value);value='';}else value+=c;}fields.push(value);return fields;};
  const header=split(lines[0]);
  return {header,rows:lines.slice(1).map((line,i)=>({lineNumber:i+2,rawLine:line,fields:Object.fromEntries(split(line).map((v,j)=>[header[j],v]))}))};
};
const ns=value=>{const s=value.replace(' ','T').replace(/Z$/,'');const [seconds,fraction='']=s.split('.');return BigInt(Date.parse(seconds+'Z'))*1000000n+BigInt(fraction.padEnd(9,'0').slice(0,9));};
const ms=value=>BigInt(Math.round(Number(value)*1e6));
const iso=n=>new Date(Number(n/1000000n)).toISOString().slice(0,19)+'.'+(n%1000000000n).toString().padStart(9,'0')+'Z';
const old=await json(join(input,'matrix-analysis-user-r4.json'));
const captures={};
for(const build of ['m6','c1']){
  const name=`matrix-presentmon-${build}-user-r4.csv`,data=await csv(name),status=await json(join(input,name+'.status.json')),bounds=await csv(`matrix-bounds-${build}-user-r4.csv`);
  assert(!data.header.includes('TimeInQPC'));
  const firstCpu=data.rows.find(r=>r.fields.CPUStartDateTime!=='NA').fields.CPUStartDateTime;
  const inferredHours=Math.round((Date.parse(firstCpu.replace(' ','T')+'Z')-Date.parse(status.startedUtc))/3600000);
  assert.equal(inferredHours,old.timestampNormalization[build].offsetHours);
  const valid=data.rows.filter(r=>r.fields.TimeInDateTime!=='NA'&&r.fields.MsUntilDisplayed!=='NA');
  const rawDisplays=valid.map(r=>ns(r.fields.TimeInDateTime)+ms(r.fields.MsUntilDisplayed));
  const rawMin=rawDisplays.reduce((a,b)=>a<b?a:b),rawMax=rawDisplays.reduce((a,b)=>a>b?a:b);
  captures[build]={csv:name,frameHeaders:data.header,frameQpcAvailable:false,windowProbeQpcAvailable:bounds.header.includes('Qpc'),status,firstCpuRaw:firstCpu,inferredHours,
    clockMappingStatus:'Unconfirmed: rounded first CPU start minus process-launch metadata is not a paired frame/UTC anchor; no per-frame QPC, timezone conversion provenance or residual-clock bound saved.',
    conditionalOffsetRangeSeconds:[Number(rawMax-ns(status.finishedUtc))/1e9,Number(rawMin-ns(status.startedUtc))/1e9],
    conditionalRangeAssumption:'Only a coarse plausibility range if every displayed row lies within coordinator launch/finish UTC; not a calibration or uncertainty guarantee.',
    boundsFirst:bounds.rows[0].fields,boundsLast:bounds.rows.at(-1).fields};
}
const data=await csv('matrix-presentmon-c1-user-r4.csv');
const findings=[];
for(const [mode,index] of [['animation-only',13],['combined',14]]){
  const matrix=await json(join(input,`matrix-c1-${mode}-user-r4.json`));
  const transition=matrix.toggleRecords.find(t=>t.index===index);
  const oldInterval=old.builds.c1.modes[mode].outlierClassificationForEachTransition.find(t=>t.transition===`${mode}-transition-${index}`).interval.outliersOver50ms.find(r=>r.classification==='inside-animation');
  assert(oldInterval);
  const offset=BigInt(captures.c1.inferredHours)*3600000000000n;
  const matches=data.rows.filter(r=>Number(r.fields.MsBetweenDisplayChange)===oldInterval.msBetweenDisplayChange && new Date(Date.parse(r.fields.TimeInDateTime.replace(' ','T')+'Z')-Number(offset/1000000n)).toISOString()===oldInterval.displayedAtUtc);
  assert.equal(matches.length,1);
  const row=matches[0],f=row.fields;
  assert.equal(f.ProcessID,oldInterval.pid);assert.equal(f.SwapChainAddress,oldInterval.swapChain);
  const legacyEnd=Date.parse(f.TimeInDateTime.replace(' ','T')+'Z')-Number(offset/1000000n),legacyStart=legacyEnd-Number(f.MsBetweenDisplayChange);
  assert.equal(new Date(legacyStart).toISOString(),oldInterval.intervalStartUtc);
  assert(legacyStart>=Date.parse(transition.animationStart.at) && legacyEnd<=Date.parse(transition.animationEnd.at));
  const displayed=ns(f.TimeInDateTime)-offset+ms(f.MsUntilDisplayed),start=displayed-ms(f.MsBetweenDisplayChange),confirmed=ns(transition.animationEnd.at),cutoff=confirmed-100000000n;
  const progressText=await readFile(join(input,`matrix-c1-${mode}-user-r4.json.progress.jsonl`),'utf8');
  const progress=progressText.trim().split(/\r?\n/).map((line,i)=>({lineNumber:i+1,value:JSON.parse(line)})).find(r=>r.value.stage==='transition-settled'&&r.value.index===index);
  assert.deepEqual(progress.value.animationEnd,transition.animationEnd);
  findings.push({mode,transitionIndex:index,sourceCsv:'matrix-presentmon-c1-user-r4.csv',lineNumber:row.lineNumber,rawLine:row.rawLine,fields:f,
    recordedTransition:transition,progressConfirmation:progress,
    missingPreciseBoundaries:['lastStyleMutation','lastChange','settleConfirmation distinct from last mutation','per-frame QPC'],
    legacy:{offsetHours:captures.c1.inferredHours,intervalStartUtc:oldInterval.intervalStartUtc,intervalEndUtc:oldInterval.displayedAtUtc,classification:oldInterval.classification,reason:'Used TimeInDateTime directly, omitted MsUntilDisplayed, selected rows by endpoint within animationStart→animationEnd; animationEnd is a settled snapshot, not a recorded final mutation.'},
    conditional:{assumptions:['Exact fixed subtraction of 16 hours is correct (not established).','TimeInDateTime is Present start and MsUntilDisplayed applies to this row.','Saved animationEnd really follows >=100ms style stability as described; historical exact harness hash/last-change value are not saved.'],
      displayedIntervalStartUtc:iso(start),displayedIntervalEndUtc:iso(displayed),confirmationUtc:iso(confirmed),confirmationMinus100msUtc:iso(cutoff),startMinusCutoffMs:Number(start-cutoff)/1e6,
      classification:start>=cutoff && displayed<=confirmed?'Inside approximate confirmation tail under stated assumptions':'Crossing or outside approximate tail',
      exactLastMutationUtc:null,clockShiftSensitivityMs:[-5,0,5].map(shift=>({additionalSubtractedMs:shift,startMinusCutoffMs:Number(start-cutoff)/1e6-shift}))},
    established:['CSV records this display interval value and PID/swapchain.','Legacy inside-animation classification is reproducible for its stored snapshot envelope.','No exact last mutation or paired frame-clock anchor is recorded.'],
    unresolved:['Unambiguous display UTC mapping','Actual style-active overlap or full containment','Native thread blocking/root cause'],
    conclusion:'Retract genuine/fully-inside actual animation claim. Interval exists; activity attribution and cause cannot be confirmed from these inputs.'});
}
const result={inputHashesVerified:hashes.length,captures,findings,scope:'Read-only historical replay; no new capture/build or product edits. Conditional results are not confirmed clock mapping or precise last-mutation reconstruction.'};
await writeFile(join(root,'historical-review.json'),JSON.stringify(result,null,2));
await writeFile(join(root,'source-rows.csv'),[data.header.join(','),...findings.map(f=>f.rawLine)].join('\n')+'\n');
console.log(JSON.stringify({inputHashesVerified:hashes.length,sourceRows:findings.map(f=>({line:f.lineNumber,mode:f.mode,gap:f.fields.MsBetweenDisplayChange,conditionalStartAfterCutoffMs:f.conditional.startMinusCutoffMs})),evidenceSupportsConfirmedAnimationContainment:false,attribution:'unconfirmed'}));
