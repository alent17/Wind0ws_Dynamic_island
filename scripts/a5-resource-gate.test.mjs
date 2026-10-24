import { test } from 'node:test';
import assert from 'node:assert/strict';
import { resourceGate } from './a5-resource-gate.mjs';
const utc=n=>new Date(n*1000).toISOString();
function fixture(){
  const ready={runId:'r',rootPid:1,readyUtc:utc(2),validIncrementSampleIndex:1};
  const samples=[0,1,3,4,5,7,9].map((t,i)=>({timestampUtc:utc(100),totalPrivateBytes:1,totalThreads:1,totalHandles:1,processes:[{pid:1,processKey:'1/birth',role:'app',cpuSampleAvailable:i>0,previousCpuReadStartedUtc:i?utc(t-1):null,cpuReadEndedUtc:utc(t),cumulativeCpuSeconds:t,previousCumulativeCpuSeconds:i?t-1:null,cpuDeltaSeconds:i?1:null,privateBytes:1,gpuMemory:[]}]}));
  return [{runId:'r',rootPid:1,probeStartedUtc:utc(0),readiness:ready,samples},{sequenceWindow:{start:{at:utc(2)},end:{at:utc(6)}}},{rootProcessId:1,resourceRunId:'r',resourceReadiness:ready}];
}
test('valid intervals use reads, not delayed JSON timestamp',()=>{const r=resourceGate(...fixture());assert.equal(r.accepted,true);assert.equal(r.completeInsideSamples,3);assert.equal(r.cpuSecondsByInterval.inside,3);});
test('zero samples rejected',()=>{const f=fixture();f[0].samples=[];assert.equal(resourceGate(...f).accepted,false);});
test('stale identity rejected',()=>{const f=fixture();f[2].resourceRunId='old';assert.equal(resourceGate(...f).accepted,false);});
test('tail truncation rejected',()=>{const f=fixture();f[0].samples=f[0].samples.slice(0,5);assert.equal(resourceGate(...f).accepted,false);});
test('cross boundary increment excluded from inside cost',()=>{const f=fixture();f[0].samples[2].processes[0].previousCpuReadStartedUtc=utc(1);const r=resourceGate(...f);assert.equal(r.completeInsideSamples,2);assert.equal(r.cpuSecondsByInterval.boundary,1);assert.equal(r.accepted,false);});
test('baseline point cannot satisfy readiness',()=>{const f=fixture();f[0].readiness.validIncrementSampleIndex=0;assert.equal(resourceGate(...f).accepted,false);});

test('same readiness instant with different decimal formatting accepted',()=>{const f=fixture();f[2].resourceReadiness={...f[0].readiness,readyUtc:'1970-01-01T00:00:02.0000000Z'};assert.equal(resourceGate(...f).accepted,true);});
test('future or pre-launch readiness rejected',()=>{for(const t of [-1,8]){const f=fixture();f[0].readiness.readyUtc=utc(t);assert.equal(resourceGate(...f).accepted,false);}});
test('PID reuse baseline is not valid increment',()=>{const f=fixture();f[0].samples[2].processes[0].cpuSampleAvailable=false;assert.equal(resourceGate(...f).accepted,false);});
