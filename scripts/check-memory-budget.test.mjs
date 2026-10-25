import { test } from 'node:test';
import assert from 'node:assert/strict';
import { checkMemoryBudget } from './check-memory-budget.mjs';
import { mkdtemp, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
const report = (processes) => ({rootPid:1,samples:[{processes},{processes}]});
const row = (pid, bytes, role = 'app') => ({pid,role,privateWorkingSetBytes:bytes,privateBytes:bytes*2});
const complete = (appBytes, rendererBytes = 0) => report([row(1,appBytes),row(2,0,'browser'),row(3,rendererBytes,'renderer')]);
test('includes child renderers rather than passing on the small Rust process alone', () => {
  const result=checkMemoryBudget(complete(10_000_000,95_000_000));
  assert.equal(result.passed,false);
  assert.equal(result.privateWorkingSet.peakBytes,105_000_000);
});
test('strictly enforces below 100 decimal MB and reports private commit independently', () => {
  assert.equal(checkMemoryBudget(complete(100_000_000)).passed,false);
  const result=checkMemoryBudget(complete(60_000_000));
  assert.equal(result.passed,true);
  assert.equal(result.commitPassed,false);
});
test('missing counters, missing root, and control-only reports cannot pass', () => {
  const missingCounters=complete(1);
  missingCounters.samples[0].processes[2].privateWorkingSetBytes=null;
  assert.throws(()=>checkMemoryBudget(missingCounters),/Memory counters unavailable/);
  assert.throws(()=>checkMemoryBudget(report([row(2,1,'browser'),row(3,1,'renderer')])),/Root process missing/);
  assert.throws(()=>checkMemoryBudget({...complete(1),control:true}));
});
test('a tiny Rust-only process or a missing renderer cannot qualify as a running WebView app', () => {
  assert.throws(()=>checkMemoryBudget(report([row(1,6_000_000)])), /WebView2 browser missing/);
  assert.throws(()=>checkMemoryBudget(report([row(1,6_000_000),row(2,1,'browser')])), /WebView2 renderer missing/);
});
test('CLI replaces a stale passing result when the new sample is invalid', async () => {
  const directory=await mkdtemp(join(tmpdir(),'isle-memory-gate-'));
  try {
    const input=join(directory,'input.json'),output=join(directory,'summary.json');
    await writeFile(input,JSON.stringify(report([row(1,6_000_000)])));
    await writeFile(output,JSON.stringify({passed:true}));
    const result=spawnSync(process.execPath,['scripts/check-memory-budget.mjs',input,output],{encoding:'utf8'});
    assert.equal(result.status,2);
    const summary=JSON.parse(await readFile(output,'utf8'));
    assert.equal(summary.valid,false);assert.equal(summary.passed,false);
    assert.match(summary.error,/WebView2 browser missing/);
  } finally { await rm(directory,{recursive:true,force:true}); }
});
