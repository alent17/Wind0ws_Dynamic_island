import assert from 'node:assert/strict';
import test from 'node:test';
import { summarizeCommandCoverage } from './a5-command-coverage.mjs';
const transitions = Array.from({ length: 16 }, (_, i) => ({ index: i + 1,
  firstStyleMutation: { performanceMs: i * 1000 + 100 }, lastStyleMutation: { performanceMs: i * 1000 + 500 } }));
const calls = transitions.flatMap(t => ['control_media', 'set_system_volume'].map(command => ({ command,
  transitionIndex: t.index, ok: true, pageStartedPerformanceMs: t.firstStyleMutation.performanceMs + 5,
  pageEndedPerformanceMs: t.firstStyleMutation.performanceMs + 10 })));
test('all 16 transitions require both successful commands in the page clock', () => {
  assert.equal(summarizeCommandCoverage(transitions, calls).accepted, true);
});
test('an opening burst cannot satisfy later transitions', () => {
  const result = summarizeCommandCoverage(transitions, calls.filter(c => c.transitionIndex <= 4));
  assert.equal(result.accepted, false); assert.equal(result.missingTransitions.length, 12);
});
test('Node/CDP envelopes cannot fabricate page-clock overlap', () => {
  const legacy = calls.map(({ pageStartedPerformanceMs, pageEndedPerformanceMs, ...call }) =>
    ({ ...call, startedAt: '2026-10-13T00:00:00Z', endedAt: '2026-10-13T01:00:00Z' }));
  assert.equal(summarizeCommandCoverage(transitions, legacy).coveredTransitions.length, 0);
});
test('failure, edge-only contact, wrong transition and invalid time are rejected', () => {
  for (const replacement of [
    { ok: false }, { pageEndedPerformanceMs: 100 }, { pageStartedPerformanceMs: 500 },
    { transitionIndex: 2 }, { pageStartedPerformanceMs: NaN }, { pageEndedPerformanceMs: 50 },
    { pageStartedPerformanceMs: 110, pageEndedPerformanceMs: 110 },
  ]) {
    const changed = [{ ...calls[0], ...replacement }, ...calls.slice(1)];
    assert.equal(summarizeCommandCoverage(transitions, changed).accepted, false);
  }
});
test('exact conversion identities 1 through 16 are required', () => {
  const shifted = transitions.map(t => ({ ...t, index: t.index + 16 }));
  const shiftedCalls = calls.map(c => ({ ...c, transitionIndex: c.transitionIndex + 16 }));
  assert.equal(summarizeCommandCoverage(shifted, shiftedCalls).accepted, false);
});
