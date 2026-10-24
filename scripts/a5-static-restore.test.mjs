import { test } from 'node:test';
import assert from 'node:assert/strict';
import { restoreStaticSession, RESTORE_ORDER } from './a5-static-restore.mjs';
function backend(fail) {
  const state={studio:true,app:true,monitor:1,disk:'test',foreground:'studio',cursor:'target',trace:[]};
  const b={state};
  const step=(name,fn)=>b[name]=async()=>{state.trace.push(name);if(fail===name)throw Error('injected failure');fn();return{ok:true,immediateReadback:true};};
  step('closeLabWithForeground',()=>assert.equal(state.foreground,'studio'));
  step('closeStudioAndConfirmPersistedExit',()=>{state.monitor=1;state.studio=false;}); // real Studio close persists draft
  step('restoreSettingsAndSourceVisibility',()=>{assert(!state.studio);state.monitor=0;});
  step('closeOwnedAppAndConfirmExit',()=>{assert.equal(state.monitor,0);state.app=false;});
  step('restorePersistentBytesAndStartup',()=>{assert(!state.app);state.disk='original';});
  step('stopPlateAndRestoreNativeReadback',()=>{state.foreground='original';state.cursor=fail==='wrong-native-readback'?'wrong':'original';assert.equal(state.cursor,'original');});
  return b;
}
test('real restore sequence prevents Studio late-save and live-writer overwrite',async()=>{
  const b=backend(),r=await restoreStaticSession(b);assert(r.ok);assert.deepEqual(b.state.trace,RESTORE_ORDER);
  assert.equal(b.state.monitor,0);assert.equal(b.state.disk,'original');assert.equal(b.state.cursor,'original');
});
test('restore failures preserve pending state and stop dependent mutations',async()=>{
  for(const name of RESTORE_ORDER){const b=backend(name),r=await restoreStaticSession(b);assert(!r.ok);assert(r.pending);assert.equal(r.failedStep,name);assert.equal(b.state.trace.at(-1),name);}
});
test('API request success cannot hide wrong immediate cursor readback',async()=>{
  const b=backend('wrong-native-readback'),r=await restoreStaticSession(b);assert(!r.ok);assert(r.pending);assert.equal(r.failedStep,'stopPlateAndRestoreNativeReadback');
});
