import assert from 'node:assert/strict';
import { readFile,writeFile } from 'node:fs/promises';
import { join } from 'node:path';
const root=process.argv[2]||'dist/performance/a5-native-2026-10-14';
const load=async p=>JSON.parse(await readFile(join(root,p),'utf8'));
const first=await load('native-matrix-first.json'),extra=await load('supplement/native-matrix.json');
for(const e of [first,extra]){
 assert.deepEqual(e.original.settings,e.restoration.settings);
 assert.deepEqual(e.original.audio,e.restoration.audio);
 assert.equal(e.original.media.isPlaying,e.restoration.media.isPlaying);
 assert.equal(e.foregroundRestore.foreground,e.foregroundRestore.originalForeground);
 assert.equal(e.binaries[0].sha256,'EA80838E71C64A9253364885A60D9AFD7EFF3C8DC5445A4AE7351D6DADA92D2C');
 assert.equal(e.binaries[1].sha256,'33B43C71DB2A218DD0DAFEE6EB4DD34FE59D5CC3FE78B8112B8DA0BD7CD0C292');
 assert(e.pointerEvents.filter(p=>p.type==='click').every(p=>p.trusted));
}
const failed=[];
for(const edge of ['top','right','bottom','left']){
 const s=first.scenarios.find(s=>s.id==='edge-'+edge);assert.equal(s.status,'fail');
 assert(first.helperStopped.clicks.some(p=>p.utc>=s.startedUtc&&p.utc<=s.endedUtc));
 assert(!first.pointerEvents.some(p=>p.type==='click'&&p.utc>=s.startedUtc&&p.utc<=s.endedUtc));
 failed.push(s.id);
}
const focus=first.snapshots.find(s=>s.label==='native-tab-focus');
assert.equal(focus.dom.active.tag,'DIV');assert(first.pointerEvents.some(p=>p.key==='Tab'&&p.trusted));
const hide=first.scenarios.find(s=>s.id==='manual-hide-restore').result;
assert(hide.hidden.native.rect.Top<0);assert.equal(hide.restored.native.rect.Top,0);assert(hide.hitAfterRestore.dom.expanded);
const reduced=first.scenarios.find(s=>s.id==='native-page-reduced-motion').result;assert(reduced.before.dom.reduced);assert(reduced.after.dom.expanded&&reduced.after.dom.reduced);
assert.equal(extra.scenarios.find(s=>s.id==='loaded-cover-continuity').status,'pass');
assert(extra.coverTrajectory.some(f=>f.covers.some(c=>c.class.includes('in-motion')&&c.images.some(i=>i.complete&&i.width>0))));
const through=extra.scenarios.find(s=>s.id==='isolated-transparent-native-click').result;
assert.equal(through.after.clicks.length,through.before.clicks.length+1);
assert.equal(through.domClicksBefore,through.domClicksAfter);assert.equal(through.after.pointPid,extra.helperReady.boardPid);
const result={evidenceVerified:true,productMatrixAccepted:false,failedNativeEdgeClickObservations:failed,focusVisibility:'Unknown: first classifier excluded focused DIV slider',realCoverLoadedTrajectoryFrames:extra.coverTrajectory.length,transparentNativeDeliveryVerified:true,restorationVerified:true,missing:['Loaded runtime/version during tested runs','Completed native expansion/reversal for failed edges','Complete focus-visible/keyboard activation acceptance','Real cross-DPI','Subjective feel','Matched performance/IPC/native-region costs'],scope:'Offline consistency of saved evidence, not fresh native test or overall acceptance'};
await writeFile(join(root,'offline-evidence-audit.json'),JSON.stringify(result,null,2));console.log(JSON.stringify(result));
