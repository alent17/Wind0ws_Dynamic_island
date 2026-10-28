import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';
import test from 'node:test';

const rust = readFileSync(new URL('../src-tauri/src/services/netease_cdp.rs', import.meta.url), 'utf8');
const capture = rust.match(/const CLOUD_MUSIC_CAPTURE_TOOL: &str = r#"([\s\S]*?)"#;/)[1];
function fixture(entry, initialized = true) {
  const tool = { inited: initialized, app: { _store: { getState() {} } }, getStore: () => ({}) };
  const progress = () => ({ current: 12 });
  const queue = [[[], { 8: function () { this.app._store.getState(); } }, [[entry]]]];
  const runtime = { m: {}, c: { 8: { exports: { a: tool } }, 148: { exports: { b: progress } }, [entry]: { exports: {} } }, s: entry };
  queue.push = chunk => {
    const id = chunk[2][0][0];
    runtime.m[id] = chunk[1][id];
    runtime.c[id] = { exports: {} };
    runtime.s = id;
    chunk[1][id]({}, {}, runtime);
    return Array.prototype.push.call(queue, chunk);
  };
  return { tool, progress, queue, runtime, execute: () => vm.runInNewContext(`(${capture})(true)`, { webpackJsonp: queue }) };
}
for (const [version, entry] of [['3.1.41',1424],['3.1.42',1423],['future entry',2048]]) {
  test(`captures initialized NetEase ${version} without a fixed entry number`, () => {
    const fx = fixture(entry);
    const result = fx.execute();
    assert.equal(result.tool, fx.tool);
    assert.equal(result.progress, fx.progress);
    assert.equal(fx.runtime.s, entry);
    assert.equal(fx.queue.length, 1);
    assert.deepEqual(Object.keys(fx.runtime.m), []);
    assert.equal(Object.keys(fx.runtime.c).some(key => key.startsWith('__isle_')), false);
  });
}
test('an uninitialized page remains unavailable and transient modules are removed', () => {
  const fx = fixture(1423, false);
  assert.equal(Object.keys(fx.execute()).length, 0);
  assert.equal(fx.runtime.s, 1423);
  assert.equal(fx.queue.length, 1);
  assert.deepEqual(Object.keys(fx.runtime.m), []);
});

test('connection readiness does not depend on a loaded track or playback mode', () => {
  const ready = rust.match(/const CLOUD_MUSIC_READY_EXPRESSION: &str = r#"([\s\S]*?)"#;/)[1].replaceAll('__CAPTURE_TOOL__', capture);
  for (const initialized of [true, false]) {
    const fx = fixture(1423, initialized);
    const result = vm.runInNewContext(ready, { webpackJsonp: fx.queue });
    assert.equal(result.ready, initialized);
    assert.equal(fx.runtime.s, 1423);
    assert.equal(fx.queue.length, 1);
  }
});

test('mode reads tolerate an uninitialized player or temporarily absent mode state', () => {
  const expression = rust.match(/const CLOUD_MUSIC_READ_EXPRESSION: &str = r#"([\s\S]*?)"#;/)[1].replaceAll('__CAPTURE_TOOL__', capture);
  for (const initialized of [true, false]) {
    const fx = fixture(1423, initialized);
    assert.equal(Object.keys(vm.runInNewContext(expression, { webpackJsonp: fx.queue })).length, 0);
  }
  const fx = fixture(1423);
  fx.tool.getStore = () => ({ playing: { playingMode: 'playOne' } });
  assert.equal(vm.runInNewContext(expression, { webpackJsonp: fx.queue }).mode, 'playOne');
});
