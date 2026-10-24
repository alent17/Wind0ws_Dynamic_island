import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import assert from 'node:assert/strict';

const root = resolve(process.argv[2] ?? 'docs/performance/first-hit-minimal-2026-10-16');
const rows = name => readFileSync(resolve(root, name), 'utf8').trim().split(/\r?\n/).map(line => line.split('\t'));
const native = (name, role, x, y) => {
  const data = rows(name);
  assert(data.some(r => r[4] === 'ready'));
  assert(data.some(r => r[4] === 'closed'));
  assert(data.every(r => r[3] === role));
  const input = data.filter(r => ['down', 'up', 'click'].includes(r[4]));
  assert.equal(input.length, 3);
  for (const kind of ['down', 'up', 'click']) {
    assert.equal(input.filter(r => r[4] === kind).length, 1);
  }
  assert(input.every(r => Number(r[5]) === x && Number(r[6]) === y));
  assert.equal(new Set(data.map(r => r[1])).size, 1);
  return Number(data[0][1]);
};
const islandPid = native('island.tsv', 'island', 320, 160);
const boardPid = native('board.tsv', 'board', 165, 105);
assert.notEqual(islandPid, boardPid);
const webBoardPid = native('board-webview-final.tsv', 'board', 165, 105);
const web = rows('webview-run-04/webview.tsv');
assert(web.some(r => r[2] === 'Isle minimal webview ready'));
const prefix = 'Isle minimal webview ';
const samples = web.filter(r => r[2].startsWith(prefix + '[')).map(r => JSON.parse(r[2].slice(prefix.length)));
assert(samples.length > 0);
const final = samples.at(-1);
assert.deepEqual(final.map(e => e.kind), ['pointerdown', 'pointerup', 'click']);
assert(final.every(e => e.trusted === true && e.x === 320 && e.y === 160));
assert(samples.every(s => s.length <= 3 && JSON.stringify(s) === JSON.stringify(final.slice(0, s.length))));
assert.equal(new Set(web.map(r => r[1])).size, 1);
assert.notEqual(Number(web[0][1]), webBoardPid);
console.log(JSON.stringify({
  native: 'two distinct processes, one complete center and one complete corner click',
  webview: 'trusted center sequence and separate process corner sequence',
  testClicks: 4,
  limitation: 'target activation; no controlled move/down interval; not product acceptance',
}, null, 2));
