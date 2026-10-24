import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
const root = process.argv[2] || 'dist/performance/a5-dwell-2026-10-14';
const bytes = await readFile(join(root, 'dwell-evidence.json'));
const e = JSON.parse(bytes);
assert.equal(e.appPid, 25612);
assert.equal(e.helperReady.boardPid, 26164);
const successful = e.trials.filter(t => t.pageReceipts.length > 0);
assert.equal(successful.length, 4);
assert(successful.every(t => t.actions.find(a => a.action === 'press-down').before.pointPid === 9440));
assert.equal(e.trials.reduce((n, t) => n + t.recipients.plateDelta, 0), 3);
assert.equal(e.trials.filter(t => !t.pageReceipts.length && !t.recipients.plateDelta).length, 1);
const result = {
  verified: true,
  evidenceSha256: createHash('sha256').update(bytes).digest('hex'),
  spawnPid: { value: e.appPid, source: 'appPid; Node spawn return value' },
  boardPid: { value: e.helperReady.boardPid, source: 'helperReady.boardPid' },
  successfulPagePointWindowOwnerPid: { value: 9440, source: 'WindowFromPoint owner PID in pre-down queries; not spawn PID' },
  imageAndParentRelationship: 'not recorded; process/module role unconfirmed',
  recipients: { page: 4, board: 3, unobserved: 1 },
  scope: 'offline provenance correction only; bit correlation and unconfirmed cause unchanged'
};
await writeFile(join(root, 'pid-role-audit.json'), JSON.stringify(result, null, 2));
console.log(JSON.stringify(result, null, 2));
