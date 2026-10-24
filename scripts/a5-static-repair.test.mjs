import {test} from 'node:test';
import assert from 'node:assert/strict';
import vm from 'node:vm';
import {webcrypto} from 'node:crypto';
import {mkdtemp,readFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawn} from 'node:child_process';
import {createConnection} from 'node:net';
import {createCleanupResumer,RESTORE_ORDER} from './a5-static-restore.mjs';
import {serveCleanupOnly,requestCleanupOnly,CLEANUP_BUDGET} from './a5-static-cleanup-control.mjs';
import {installStudioAudit,auditStudioSettings} from './a5-static-settings-audit.mjs';
import {Helper} from './a5-static-helper-transport.mjs';
import {superviseCoordinator} from './a5-static-session-supervisor.mjs';

const initial={monitorIndex:0,playerOrderIds:['old','selected'],selectedPlayerIds:['selected'],volume:33};
test('actual IPC observer supports immutable Tauri invoke; audited discovery and exact patches only',async()=>{
  const receipts=[];
  const window={__a5AuditReceipt:async e=>receipts.push(e),fetch:async(_url,options)=>{
    const command=new URL(_url).pathname.slice(1);
    const value=command==='get_settings'?initial:command==='list_media_sessions'?[{id:'new'}]:null;
    return new Response(JSON.stringify(value),{headers:{'Content-Type':'application/json','Tauri-Response':'ok'}});
  }};
  window.__TAURI_INTERNALS__={};Object.defineProperty(window.__TAURI_INTERNALS__,'invoke',{value:()=>{throw Error('must not overwrite');}});
  const context=vm.createContext({window,URL,Headers,structuredClone,crypto:webcrypto,location:{href:'http://tauri.localhost/'}});
  vm.runInContext(`(${installStudioAudit.toString()})()`,context);
  const call=(command,payload={},tool=false)=>window.fetch(`http://ipc.localhost/${command}`,{body:JSON.stringify(payload),headers:tool?{'X-A5-Tool':'true'}:{}});
  await call('get_settings');await call('list_media_sessions');
  await call('update_settings',{patch:{playerOrderIds:['selected','new']}});
  await call('update_settings',{patch:{monitorIndex:1}},true);
  const events=JSON.parse(JSON.stringify(window.__a5SettingsAudit));
  const current={...initial,monitorIndex:1,playerOrderIds:['selected','new']};
  const accepted=auditStudioSettings(initial,events,current);assert.equal(accepted.changes.length,2);assert.equal(receipts.length,8);
  for(const mutate of [e=>e[2].args.patch={playerOrderIds:['new','selected']},e=>e[2].args.patch={volume:50},e=>e.splice(0,1),e=>e[1].transportFailure=true]){
    const bad=structuredClone(events);mutate(bad);assert.throws(()=>auditStudioSettings(initial,bad,current));
  }
  assert.throws(()=>auditStudioSettings(initial,events,{...current,volume:99}));
});
test('actual phased restore resumes only unfinished steps; unsafe release never enters backend/gate',async()=>{
  const trace=[];let fail=true,gates=0,persists=0;
  const backend=Object.fromEntries(RESTORE_ORDER.map(name=>[name,async()=>{trace.push(name);if(name===RESTORE_ORDER[2]&&fail)throw Error('pending');return{ok:true};}]));
  const cleaner=createCleanupResumer({backend,gate:async()=>{gates++;},persist:async()=>{persists++;}});
  assert((await cleaner.resume({ok:true,held:false})).pending);assert.equal(trace.length,3);
  const before=trace.length;assert((await cleaner.resume({ok:false,held:'unknown'})).pending);assert.equal(trace.length,before);assert.equal(gates,1);
  fail=false;assert((await cleaner.resume({ok:true,held:false})).ok);
  assert.deepEqual(trace,[...RESTORE_ORDER.slice(0,3),...RESTORE_ORDER.slice(2)]);assert.equal(persists,3);
  let diskFault=true;const completed=[];
  const durable=createCleanupResumer({backend:Object.fromEntries(RESTORE_ORDER.map(name=>[name,async()=>{completed.push(name);return{ok:true};}])),gate:async()=>{},persist:async()=>{if(diskFault)throw Error('journal disk fault');}});
  const failedSave=await durable.resume({ok:true,held:false});assert(!failedSave.ok&&failedSave.pending);
  diskFault=false;assert((await durable.resume({ok:true,held:false})).ok);assert.deepEqual(completed,RESTORE_ORDER);
});
test('actual cleanup-only endpoint uses same closure/session and closes normally',async()=>{
  const dir=await mkdtemp(join(tmpdir(),'a5-cleanup-'));let calls=0;
  const endpoint=await serveCleanupOnly(join(dir,'control.json'),'one-generation',async()=>({ok:++calls===2,steps:[{name:'preserved'}]}));
  const record=JSON.parse(await readFile(join(dir,'control.json'),'utf8'));
  for(const q of [{action:'press-down',token:record.token,sessionId:record.sessionId},{action:'cleanup-only',token:record.token,sessionId:'stale-generation'},{action:'cleanup-only',token:record.token,sessionId:record.sessionId,command:'restart'}]){
    const refused=await new Promise((resolve,reject)=>{const socket=createConnection(record.pipe);let text='';socket.setEncoding('utf8');socket.on('error',reject);socket.on('connect',()=>socket.write(JSON.stringify(q)+'\n'));socket.on('data',x=>text+=x);socket.on('end',()=>resolve(JSON.parse(text)));});
    assert(!refused.ok);assert.equal(calls,0);
  }
  const first=await requestCleanupOnly(join(dir,'control.json'));assert(!first.ok);assert.equal(calls,1);
  assert((await requestCleanupOnly(join(dir,'control.json'))).ok);await endpoint.closed;assert.equal(calls,2);
  await assert.rejects(requestCleanupOnly(join(dir,'control.json')),/persistence-only/);
});
test('real shared helper transport records normal child/stdio exit without GUI',async()=>{
  const code="const r=require('readline').createInterface({input:process.stdin});console.log(JSON.stringify({ready:true}));r.on('line',s=>{const q=JSON.parse(s);console.log(JSON.stringify({id:q.id,ok:true,action:q.action}));if(q.action==='stop'){console.log(JSON.stringify({stopped:true}));r.close();process.stdin.pause();}});";
  const trace=[],helper=new Helper('',trace,process.execPath,['-e',code]);
  await helper.next(x=>x.ready);assert((await helper.request('stop')).ok);await helper.next(x=>x.stopped);helper.process.stdin.end();
  const exit=await helper.exitAcknowledged;assert.equal(exit.code,0);assert(exit.stdioClosed);assert(trace.some(x=>x.source==='helper-exit'));
});
test('production cleanup transport returns full result after 5300 ms, beyond old read timeout',async()=>{
  const dir=await mkdtemp(join(tmpdir(),'a5-long-cleanup-')),path=join(dir,'control.json');
  const endpoint=await serveCleanupOnly(path,'slow-generation',async()=>{await new Promise(r=>setTimeout(r,5300));return{ok:true,completed:'slow cleanup'};});
  assert.equal((await requestCleanupOnly(path)).completed,'slow cleanup');await endpoint.closed;
});
test('production cleanup transport reports real disconnect as unknown; duplicate cannot run cleanup',async()=>{
  const dir=await mkdtemp(join(tmpdir(),'a5-broken-cleanup-')),path=join(dir,'control.json');
  let started,finish,calls=0;const began=new Promise(r=>started=r),held=new Promise(r=>finish=r);
  const endpoint=await serveCleanupOnly(path,'broken-generation',async()=>{calls++;if(calls===1){started();return held;}return{ok:true};});
  endpoint.server.once('connection',socket=>socket.once('data',()=>socket.destroy()));
  const broken=requestCleanupOnly(path);await began;
  await assert.rejects(broken,error=>error.code==='A5_CLEANUP_OUTCOME_UNKNOWN'&&/outcome unknown/.test(error.message));
  const concurrent=await requestCleanupOnly(path);assert(concurrent.cleanupStillRunning);assert.equal(calls,1);
  finish({ok:false,pending:true});await new Promise(r=>setImmediate(r));
  assert((await requestCleanupOnly(path)).ok);assert.equal(calls,2);await endpoint.closed;
});
test('shared execution budget returns explicit pending and retains mutex until existing cleanup settles',async()=>{
  const dir=await mkdtemp(join(tmpdir(),'a5-budget-cleanup-')),path=join(dir,'control.json');
  let finish,calls=0;const held=new Promise(r=>finish=r);
  const endpoint=await serveCleanupOnly(path,'budget-generation',async()=>{calls++;return calls===1?held:{ok:true};},{...CLEANUP_BUDGET,executeMs:25});
  const pending=await requestCleanupOnly(path);assert(pending.pending&&pending.cleanupStillRunning);assert.equal(calls,1);
  assert((await requestCleanupOnly(path)).cleanupStillRunning);assert.equal(calls,1);
  finish({ok:false,pending:true});await new Promise(r=>setImmediate(r));
  assert((await requestCleanupOnly(path)).ok);await endpoint.closed;
});
test('default and refused live exit before live module/GUI import; cold cleanup refuses',async()=>{
  const run=args=>new Promise(resolve=>{const child=spawn(process.execPath,['scripts/run-a5-static-lab.mjs',...args],{windowsHide:true});let stdout='',stderr='';child.stdout.on('data',x=>stdout+=x);child.stderr.on('data',x=>stderr+=x);child.on('close',code=>resolve({code,stdout,stderr}));});
  const dry=await run([]);assert.equal(dry.code,0);assert.equal(JSON.parse(dry.stdout).guiCreated,false);
  const denied=await run(['--live']);assert.notEqual(denied.code,0);assert.match(denied.stderr,/Refused before live imports/);
  const source=await readFile('scripts/run-a5-static-lab.mjs','utf8');assert(source.indexOf('Refused before live imports')<source.indexOf("import('./a5-static-live.mjs')"));
});
test('actual coordinator supervisor persists child exit acknowledgement using dry CLI',async()=>{
  const dir=await mkdtemp(join(tmpdir(),'a5-supervisor-'));
  const report=await superviseCoordinator(['scripts/run-a5-static-lab.mjs'],dir,{stdio:'ignore'});
  assert(report.normal);assert.equal(report.code,0);assert.deepEqual(JSON.parse(await readFile(join(dir,'coordinator-exit.json'),'utf8')),report);
});
