import assert from 'node:assert/strict';import {readFile,writeFile} from 'node:fs/promises';import {join} from 'node:path';
const root=process.argv[2]||'dist/performance/a5-dwell-2026-10-14';const e=JSON.parse(await readFile(join(root,'dwell-evidence.json'),'utf8'));
const expected=[['edge-top',15,0],['edge-top',15,50],['edge-top',15,250],['floating',26,0],['floating',26,50],['floating',26,250],['edge-top-original-border',2,0],['edge-top-original-border',2,250]];
assert.equal(e.trials.length,8);assert.equal(e.native.filter(n=>n.action==='press-down').length,8);assert.equal(e.native.filter(n=>n.action==='press-up').length,8);
const table=[];
for(let i=0;i<8;i++){
 const t=e.trials[i];assert.equal(t.style,expected[i][0]);assert.deepEqual(t.point,{x:-960,y:expected[i][1]});assert.equal(t.prescribedDwellMs,expected[i][2]);assert(t.frozenCandidate.valid&&t.beforeDownDom.valid);
 const down=t.actions.find(n=>n.action==='press-down'),up=t.actions.find(n=>n.action==='press-up'),move=t.actions.find(n=>n.action==='move'&&n.after.cursor.x===t.point.x),base=t.actions.find(n=>n.action==='pump'),late=t.actions.at(-1);
 assert.equal(down.sent,1);assert.equal(up.sent,1);assert(up.inputStartedQpc>=down.inputEndedQpc);
 const actual=(down.inputStartedQpc-move.inputEndedQpc)*1000/down.before.qpcFrequency;assert(Math.abs(actual-t.actualMoveEndToDownMs)<1e-7);
 const delta=late.bottomPlateReceipts.length-base.bottomPlateReceipts.length;assert.equal(delta,t.recipients.plateDelta);assert.equal(t.pageReceipts.length,t.recipients.pageClicks);
 assert.equal(t.recipients.conflict,delta>0&&t.pageReceipts.length>0);assert(t.pageReceipts.every(p=>p.trusted));
 table.push({index:i+1,style:t.style,point:t.point,prescribedMs:t.prescribedDwellMs,actualMs:actual,plateCallbacks:delta,trustedPageClicks:t.pageReceipts.length,recipient:t.recipients.deliveredTo,preDownBit20:down.before.transparentBit20,lateBit20:late.after.transparentBit20,preDownPointPid:down.before.pointPid,latePointPid:late.after.pointPid});
}
assert.deepEqual(e.original.settings,e.restoration.settings);assert.deepEqual(e.original.audio,e.restoration.audio);assert.equal(e.original.media.isPlaying,e.restoration.media.isPlaying);assert(e.releaseAudit.inputCleanupOk&&!e.releaseAudit.releaseCleanup.held);assert(e.helperStopped.inputCleanupOk&&!e.helperStopped.cursorForegroundRestoreSkipped);assert.equal(e.helperStopped.originalForeground,e.helperStopped.foreground);
const result={verified:true,table,restoration:{settings:true,audio:true,playState:true,foreground:true,ownedPressReleased:true,cursor:'Restore requested; immediate readback not recorded'},runtime:e.normalSessionRuntime.Browser,rootCause:'unconfirmed',scope:'8 single trials; receipt types are board MouseDown handler vs page click, not full symmetric down/up routing; no performance claim'};
await writeFile(join(root,'offline-audit.json'),JSON.stringify(result,null,2));console.log(JSON.stringify(result));
