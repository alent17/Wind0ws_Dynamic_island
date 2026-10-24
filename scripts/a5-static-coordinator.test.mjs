import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dryPlan, freezePoints, receiptVerdict, runStaticFour } from './a5-static-coordinator.mjs';

function fixture() {
  const width=40,height=30,alpha=new Uint8Array(width*height);
  for(let y=8;y<=22;y++)for(let x=12;x<=28;x++)alpha[y*width+x]=255;
  const frame={left:-1000,top:20,right:-960,bottom:50};
  return {frame,requestedFrame:{...frame},canvas:{width,height,alpha,dpr:1,viewportWidth:40,viewportHeight:30,fillVerified:true},scale:1,
    workArea:{left:-1920,top:0,right:0,bottom:1080},controlRect:{left:32,top:32,right:1032,bottom:782},
    sourceHidden:true,nativeReady:true,noActivateConfirmed:true,foregroundOwned:true,inputIdle:true,sourceRevision:7,labHwnd:101,boardHwnd:102,controlHwnd:103,sourceHwnd:104,nativeDpi:96};
}
function fake(options={}) {
  const data=fixture(),calls=[],board=[],lab=[];let index=0,clock=1000,statusCalls=0;
  const primary=(trusted,native)=>['down','up','click'].map(family=>({family,trusted,pointerType:family==='click'?null:'mouse',extraInfo:0xA10000+index,foregroundOwned:true,physicalPoint:{...io.frozen[index%2===1?'inside':'outside'],...(options.coordinateBad?{x:5}:{})},...(native?{origin:'native raw',trusted:null}:{type:family==='click'?'click':`pointer${family}`})}));
  const io={calls,data,board,lab,
    async prepare(){calls.push('prepare');options.change?.(data);return data;},
    async confirmPresentation(){calls.push('presentation');return{desktopTilesMatch:!options.noPixels,screenshotSaved:true,actualFrameMatches:true};},
    async freeze(plan){calls.push('freeze');if(options.mutateFreeze)plan.inside.x=2;io.frozen=plan;},
    async checkpoint(){calls.push('checkpoint');return{frame:{...data.frame},sourceHidden:true,foregroundOwned:!options.foreignBeforeStatus,inputIdle:true};},
    async status(reason){calls.push(`status:${reason}`);statusCalls++;return{after:{nativeReady:!(options.invalidAfterFirst&&reason==='after-1'),phase:options.invalidAfterFirst&&reason==='after-1'?'Invalid':'Applied'},queryMayInvalidate:true};},
    async native(action,args){calls.push(action);clock+=2;
      if(action==='begin-trial')index=args.index;
      if(action==='press-up'&&options.releaseFail)return{ok:false,error:'release failed',releaseOnError:{ok:false,held:true}};
      if(action==='press-down'&&options.downFail)return{ok:false,error:'unknown target before down',releaseOnError:{ok:true,held:false}};
      return{ok:true,inputStartedQpc:clock,inputEndedQpc:clock+1,apiStartedQpc:clock,apiEndedQpc:clock+1,qpcFrequency:1000,sent:1};},
    async receipts(){calls.push('receipts');return{board:board.slice(),lab:lab.slice()};},
    async drain(){calls.push('drain-after-up');const inside=index%2===1;
      if(options.split){board.push(primary(null,true)[0]);lab.push(...primary(true,false).slice(1));}
      else if(options.dual){board.push(...primary(null,true));lab.push(...primary(true,false));}
      else if(options.missing){(inside?lab:board).push(primary(inside,!inside)[0]);}
      else{const side=options.wrong?(inside?board:lab):(inside?lab:board);const e=primary(side===lab,side===board);if(options.untrusted)e.forEach(v=>v.trusted=false);if(options.tagBad)e.forEach(v=>v.extraInfo=0);if(options.activation)e.forEach(v=>v.foregroundOwned=false);side.push(...e);if(options.double)side.push({family:'doubleclick-diagnostic'});}
      return{board:board.slice(),lab:lab.slice()};},
    async ensureReleased(){calls.push('release-owned');if(options.releaseThrow)throw Error('transport unknown');return{ok:!options.releaseFail,held:Boolean(options.releaseFail)};},
    async retainUnsafe(){calls.push('retain-no-mutation');},
    async restore(){calls.push('restore-settings-foreground-cursor-readback');return{ok:!options.restoreFail,pending:Boolean(options.restoreFail),immediateReadback:!options.restoreFail};},
  };return io;
}
test('fixed four order, immutable points, zero deliberate waits and actual interval',async()=>{
  const io=fake(),r=await runStaticFour(io);assert(r.complete);assert.deepEqual(r.trials.map(t=>t.role),['inside','outside','inside','outside']);
  assert.equal(io.calls.filter(c=>c==='press-down').length,4);assert.equal(io.calls.filter(c=>c==='press-up').length,4);assert.equal(io.calls.filter(c=>c==='move').length,8);
  assert(io.calls.indexOf('freeze')<io.calls.indexOf('move'));assert(r.trials.every(t=>t.intentionalDwellMs===0&&t.actualMoveEndToDownMs===1));
  assert(Object.isFrozen(r.frozen.inside));assert.equal(io.calls.filter(c=>c.startsWith('status:')).length,8);
});
test('status never runs between move/down/up or before unsafe input guard',async()=>{
  const io=fake();await runStaticFour(io);
  for(let i=0;i<io.calls.length;i++)if(io.calls[i]==='press-down')assert.deepEqual(io.calls.slice(i-2,i+2),['move','move','press-down','press-up']);
  const unsafe=fake({foreignBeforeStatus:true}),r=await runStaticFour(unsafe);assert(!r.complete);assert(!unsafe.calls.some(c=>c.startsWith('status:')));assert(!unsafe.calls.includes('press-down'));
});
for(const [name,change] of [['frame',d=>d.requestedFrame.right++],['Canvas',d=>d.canvas.fillVerified=false],['DPR',d=>d.canvas.dpr=2],['native DPI',d=>d.nativeDpi=144],['no-activate',d=>d.noActivateConfirmed=false],['hidden source',d=>d.sourceHidden=false],['fixed start unavailable',d=>d.workArea.right=-950]]) {
  test(`${name} fails before down, no alternate point`,async()=>{const io=fake({change}),r=await runStaticFour(io);assert(r.failure);assert(!io.calls.includes('press-down'));});
}
test('captured desktop pixels required, eval alone not accepted',async()=>{const io=fake({noPixels:true}),r=await runStaticFour(io);assert(r.failure);assert(!io.calls.includes('freeze'));assert(!io.calls.includes('press-down'));});
test('frozen coordinate mutation rejected before input',async()=>{const io=fake({mutateFreeze:true}),r=await runStaticFour(io);assert(r.failure);assert(!io.calls.includes('press-down'));});
for(const option of ['dual','split','missing','wrong','untrusted','tagBad','activation','double','coordinateBad'])test(`${option} receipts retained; stop without retry`,async()=>{
  const io=fake({[option]:true}),r=await runStaticFour(io);const count=['tagBad','activation'].includes(option)?2:1;assert(!r.complete);assert.equal(r.trials.length,count);assert.equal(io.calls.filter(c=>c==='press-down').length,count);assert(r.trials.at(-1).board.length+r.trials.at(-1).lab.length>0);
  if(option==='dual'||option==='split')assert(r.trials[0].verdict.conflict);
});
test('query-associated invalidation kept apart from passed receipt result',async()=>{const io=fake({invalidAfterFirst:true}),r=await runStaticFour(io);assert.match(r.failure,/status-query-associated/);assert(r.trials[0].verdict.accepted);assert.equal(r.trials.length,1);});
test('failed up/unknown ownership skips all restoration mutations',async()=>{for(const options of [{releaseFail:true},{releaseThrow:true}]){const io=fake(options),r=await runStaticFour(io);assert(!r.complete);assert(r.restoration.pending);assert(io.calls.includes('retain-no-mutation'));assert(!io.calls.includes('restore-settings-foreground-cursor-readback'));}});
test('down failure is not retried and restoration failure is explicit',async()=>{const io=fake({downFail:true}),r=await runStaticFour(io);assert.equal(r.trials.length,1);assert(r.trials[0].downAttempted);assert(!r.trials[0].downSent);const failed=await runStaticFour(fake({restoreFail:true}));assert(failed.receiptsPassed);assert(!failed.complete);});
test('missing/duplicate/out-of-order packets cannot be filled by late native query',()=>{
  assert(!receiptVerdict([{family:'down'}],[],'board', {latePointPid:55}).accepted);
  assert(!receiptVerdict([{family:'down'},{family:'click'},{family:'up'}],[],'board').complete);
  assert(!receiptVerdict([{family:'down'},{family:'down'},{family:'up'},{family:'click'}],[],'board').complete);
  assert.equal(receiptVerdict([],[],'lab').lateQueryOverridesRecipient,false);
});
test('default CLI/helper and refused live branch cannot initialize GUI',()=>{
  assert.equal(dryPlan().approved,false);
  const plan=JSON.parse(execFileSync(process.execPath,['scripts/run-a5-static-lab.mjs'],{encoding:'utf8'}));assert.equal(plan.guiCreated,false);
  const native=JSON.parse(execFileSync('pwsh.exe',['-NoProfile','-File','scripts/a5-static-input.ps1'],{encoding:'utf8'}));assert.equal(native.nativeCalls,false);
  const refused=spawnSync(process.execPath,['scripts/run-a5-static-lab.mjs','--live'],{encoding:'utf8'});assert.notEqual(refused.status,0);assert.match(refused.stderr,/Refused before live imports/);
  const pending=spawnSync(process.execPath,['scripts/run-a5-static-lab.mjs','--live','--approval','docs/performance/a5-static-tools-2026-10-15/PENDING_APPROVAL.json','--output','dist/performance/a10-unapproved-never-created'],{encoding:'utf8'});assert.notEqual(pending.status,0);assert.match(pending.stderr,/Refused before live imports/);
  const helper=readFileSync('scripts/a5-static-input.ps1','utf8'),cs=readFileSync('scripts/a5-static-native.cs','utf8');assert(helper.indexOf('return')<helper.indexOf('Add-Type'));
  assert(helper.includes('Invoke-A5PressSession'));assert(helper.includes('ManualResetEvent'));
  assert(cs.includes('ShowWithoutActivation')&&cs.includes('0x08000080')&&cs.includes('new IntPtr(3)'));assert(!cs.includes('keybd_event'));
});
