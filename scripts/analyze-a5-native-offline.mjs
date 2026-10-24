import { readFile,writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';import { join } from 'node:path';
const root=process.argv[2]||'dist/performance/a5-offline-2026-10-14';
const raw=await readFile(join(root,'inputs/first.json'));const e=JSON.parse(raw);
const ns=s=>{const [t,f='']=s.replace(/Z$/,'').split('.');return BigInt(Date.parse(t+'Z'))*1000000n+BigInt(f.padEnd(9,'0').slice(0,9));};
const diff=(a,b)=>Number(ns(a)-ns(b))/1e6;
const output=[];
for(const id of ['floating-hover-interrupt','floating-reverse','edge-top','edge-right','edge-bottom','edge-left','style-switch-native-hit']){
 const scene=e.scenarios.find(s=>s.id===id),lo=scene.startedUtc,hi=scene.endedUtc;
 const clicks=e.native.filter(n=>n.request.action==='click'&&n.utc>=lo&&n.utc<=hi);
 const plate=e.helperStopped.clicks.filter(c=>c.utc>=lo&&c.utc<=hi);
 const page=e.pointerEvents.filter(p=>p.utc>=lo&&p.utc<=hi);
 const changes=e.commands.filter(c=>c.command==='update_settings'&&c.utc<=hi);
 const lastSnapshot=e.snapshots.filter(s=>s.dom.utc<=hi).at(-1);
 const stages=clicks.map(c=>{
  const i=e.native.indexOf(c);const preceding=e.native.slice(0,i),move=preceding.filter(n=>n.request.action==='move').at(-1),state=preceding.filter(n=>n.request.action==='state').at(-1);
  const down=plate.find(p=>p.x===c.request.x&&p.y===c.request.y),pClick=page.filter(p=>p.type==='click'&&p.screenX===c.request.x&&p.screenY===c.request.y&&p.utc<=c.utc).at(-1);
  return {mergedRequest:c.request,postQuery:c,previousExplicitMove:move??null,previousNativeState:state??null,bottomPlateCallback:down??null,pageClick:pClick??null,
   postQueryMinusPlateCallbackMs:down?diff(c.utc,down.utc):null,postQueryMinusPageClickMs:pClick?diff(c.utc,pClick.utc):null,
   postQueryMinusPreviousMoveQueryMs:move?diff(c.utc,move.utc):null,
   moveAndClickSamePoint:move?move.request.x===c.request.x&&move.request.y===c.request.y:null,
   coordinateUnderPostRect:{x:c.request.x-c.rect.Left,y:c.request.y-c.rect.Top},
   missing:['move execution timestamp','pre-down cursor/WindowFromPoint/extended-style','separate press-down/up stamps','last backend accepted region/revision','DOM candidate stamp at native press'],
   interpretation:'Native utc is after implicit message pump >=40ms and queries, not injection time; board UTC is callback processing during DoEvents, not OS down timestamp. Previous move query precedes later click but does not prove exact dwell. Post-rect local coordinate is a later geometry observation.'};
 });
 output.push({id,sceneStatus:scene.status,sceneStart:lo,sceneEnd:hi,lastSettingResponse:changes.at(-1)??null,lastSavedSnapshot:lastSnapshot??null,stages,pageEvents:page,boardCallbacks:plate});
}
await writeFile(join(root,'offline-timeline.json'),JSON.stringify({inputSha256:createHash('sha256').update(raw).digest('hex'),timelines:output,rootCause:'unconfirmed',scope:'Only saved UTC chronology; missing pre-down observations cannot be reconstructed'},null,2));
console.log(JSON.stringify(output.map(s=>({id:s.id,plate:s.boardCallbacks.length,pageClicks:s.pageEvents.filter(p=>p.type==='click').length,postDeltas:s.stages.map(p=>p.postQueryMinusPlateCallbackMs??p.postQueryMinusPageClickMs),moveSamePoint:s.stages.map(p=>p.moveAndClickSamePoint)}))));
