import assert from 'node:assert/strict';

export const SCOPE = 'a5-static-lab-four-v3-candidate';
export const BINARIES = {
  exe: '38102A0F66B93A4370155CCFD6F420BC8A3831646216B7076C67DB493F506E7A',
  dll: 'CBD7290142D7BA3140814F74387EF0D4F7AA356A1BB22D2764526EE1264B737D',
};
export const ORDER = Object.freeze(['inside', 'outside', 'inside', 'outside']);
export function dryPlan() {
  return { scope: SCOPE, mode: 'dry-run', approved: false, guiCreated: false, inputInjected: false,
    maximumDowns: 4, normalUps: 4, cleanup: 'only release a successful still-owned down; never replay a click',
    selectedBinaries: BINARIES, order: ORDER, targetDwellMs: 0, outsideDwellMs: 0,
    pointRule: 'one 5x5 opaque tile nearest host centre (y/x tie); one 5x5 transparent tile first in y/x scan, margins 3; freeze once',
    startRule: 'host.right+32, host.bottom+32; must be inside secondary work area; no alternate point',
    preconditions: ['no existing Isle process', 'new Studio at primary working-area origin+32, unchanged size fits',
      'actual app foreground and input idle', 'source main on secondary, then hidden',
      'exact accepted revision; immutable lab, actual requested/observed frame equality',
      'Canvas bitmap/DPR/style/nativeReady plus one composited-desktop capture matches both 5x5 tiles',
      'no-activate plate foreground unchanged', 'all coordinates frozen before any cursor move/down'],
    statusRule: 'only after read-only input/foreground checks, never between target move and up; log before/after invalidation separately',
    restore: ['release owned input', 'app foreground for lab close', 'close lab, then Studio and wait for its persist/destroy',
      'restore changed settings and original source visibility with immediate checks', 'WM_CLOSE all newly owned application windows; wait process exit, no kill',
      'restore settings.json/.bak and Isle/legacy Run values from prelaunch snapshot',
      'close plate and restore original foreground/cursor with immediate readback'],
    stop: 'any invalid prerequisite, drift, missing/dual/wrong recipient, query invalidation or release failure; no retry/reselect/additional matrix',
    excluded: ['product hover', 'DPI/drag/capture matrix', 'performance', 'player control', 'independent metadata launch'],
  };
}
function within(p, r) { return p.x >= r.left && p.x < r.right && p.y >= r.top && p.y < r.bottom; }
function tile(alpha, w, x, y, wanted) {
  for (let dy = -2; dy <= 2; dy++) for (let dx = -2; dx <= 2; dx++)
    if (alpha[(y + dy) * w + x + dx] !== wanted) return false;
  return true;
}
export function freezePoints(context) {
  const { frame: f, canvas: c, workArea, controlRect } = context;
  assert.deepEqual(context.requestedFrame, f, 'actual setter/frame mismatch');
  assert(context.sourceHidden && context.nativeReady && context.noActivateConfirmed && context.foregroundOwned && context.inputIdle, 'native precondition failed');
  assert(Number.isSafeInteger(context.labHwnd) && Number.isSafeInteger(context.boardHwnd) && context.labHwnd !== context.boardHwnd && Number.isSafeInteger(context.controlHwnd) && Number.isSafeInteger(context.sourceHwnd));
  const w = f.right - f.left, h = f.bottom - f.top;
  assert(Number.isInteger(w) && Number.isInteger(h) && w > 6 && h > 6 && w * h <= 1_000_000);
  assert(c.width === w && c.height === h && c.alpha.length === w * h && c.dpr === context.scale);
  assert.equal(context.nativeDpi,96*c.dpr,'actual HWND DPI mismatch');
  assert(Math.abs(c.viewportWidth * c.dpr - w) < .01 && Math.abs(c.viewportHeight * c.dpr - h) < .01 && c.fillVerified, 'Canvas/frame/colour mismatch');
  assert(f.right <= controlRect.left || f.left >= controlRect.right || f.bottom <= controlRect.top || f.top >= controlRect.bottom, 'control overlaps lab');
  let inside, outside, best = Infinity;
  for (let y = 3; y < h - 3; y++) for (let x = 3; x < w - 3; x++) {
    if (!outside && tile(c.alpha, w, x, y, 0)) outside = { x, y };
    const distance = (x - w / 2) ** 2 + (y - h / 2) ** 2;
    if (distance < best && tile(c.alpha, w, x, y, 255)) { best = distance; inside = { x, y }; }
  }
  assert(inside && outside, 'no fixed safe inside/outside tile; do not reselect geometry');
  const screen = p => Object.freeze({ x: f.left + p.x, y: f.top + p.y });
  const start = { x: f.right + 32, y: f.bottom + 32 };
  assert(within(start, workArea) && !within(start, controlRect), 'fixed start unavailable');
  return Object.freeze({ frame: Object.freeze({ ...f }), inside: screen(inside), outside: screen(outside), start: Object.freeze(start),
    sourceRevision: context.sourceRevision, labHwnd: context.labHwnd, boardHwnd: context.boardHwnd, controlHwnd: context.controlHwnd, sourceHwnd: context.sourceHwnd, nativeDpi:context.nativeDpi, order: ORDER, targetDwellMs: 0 });
}
const PRIMARY = new Set(['down', 'up', 'click']);
export function receiptVerdict(board, lab, expected) {
  const b = board.filter(e => PRIMARY.has(e.family)), l = lab.filter(e => PRIMARY.has(e.family));
  const complete = (a, trusted) => ['down','up','click'].every(k => a.filter(e => e.family === k).length === 1)
    && a.length === 3 && a.map(e=>e.family).join(',')==='down,up,click'
    && (!trusted || a.every(e => e.trusted === true && (e.family==='click' || e.pointerType==='mouse')));
  const conflict = b.length > 0 && l.length > 0;
  const side = conflict ? 'conflict' : b.length ? 'board' : l.length ? 'lab' : 'unobserved';
  return { boardCounts: ['down','up','click'].map(k => b.filter(e => e.family === k).length),
    labCounts: ['down','up','click'].map(k => l.filter(e => e.family === k).length), side, conflict,
    complete: !conflict && (side === 'board' ? complete(b, false) : side === 'lab' ? complete(l, true) : false),
    accepted: !conflict && side === expected && complete(side === 'board' ? b : l, side === 'lab'),
    lateQueryOverridesRecipient: false, nativeTrusted: 'unknown; raw mouse messages, not browser isTrusted' };
}
function validateCheckpoint(value, frozen) {
  assert(value.foregroundOwned && value.inputIdle && value.sourceHidden, 'unsafe foreground/input/source; do not query status');
  assert.deepEqual(value.frame, frozen.frame, 'frozen frame drift');
}
export async function runStaticFour(io) {
  const result = { scope: SCOPE, trials: [], statusObservations: [], inputBudget: 4, approvedExecutionClaim: false };
  let frozen;
  const call = async (action, args = {}) => {
    const r = await io.native(action, args);
    if (!r.ok) throw Object.assign(new Error(`${action}: ${r.error ?? 'failed'}`), { releaseFailed: r.releaseOnError?.ok === false });
    return r;
  };
  const safeStatus = async reason => {
    const guard = await io.checkpoint(); validateCheckpoint(guard, frozen);
    const observation = await io.status(reason);
    result.statusObservations.push({ reason, guard, ...observation });
    assert(observation.after.nativeReady && observation.after.phase === 'Applied', 'status-query-associated invalidation; not an OS route diagnosis');
  };
  try {
    const context = await io.prepare();
    frozen = freezePoints(context);
    result.frozen = frozen;
    const presentation = await io.confirmPresentation(frozen);
    assert(presentation.desktopTilesMatch && presentation.screenshotSaved && presentation.actualFrameMatches, 'actual Canvas/desktop/frame unconfirmed');
    result.presentation = presentation;
    await io.freeze(frozen); // durable evidence/helper limits before any input
    let lastPrimary;
    for (let index = 0; index < ORDER.length; index++) {
      const role = ORDER[index], point = frozen[role];
      const row = { index: index + 1, role, point, intentionalDwellMs: 0 };
      result.trials.push(row);
      await safeStatus(`before-${index + 1}`);
      const baseline = await io.receipts();
      const primaryCounts = data => [data.board, data.lab].map(a => a.filter(e => PRIMARY.has(e.family)).length);
      if (lastPrimary) assert.deepEqual(primaryCounts(baseline), lastPrimary, 'late receipts between trials; stop before next down');
      await call('begin-trial', { index: index + 1 });
      await call('move', frozen.start);
      const move = await call('move', point);
      row.actions = { move };
      row.downAttempted = true;
      const down = await call('press-down'); row.downSent = true;
      row.actions.down = down;
      row.upAttempted = true;
      const up = await call('press-up'); row.upSent = true;
      row.actions.up = up;
      assert([move.apiEndedQpc,down.apiStartedQpc,up.apiStartedQpc,down.qpcFrequency].every(Number.isSafeInteger)
        && down.qpcFrequency>0 && down.apiStartedQpc>=move.apiEndedQpc && up.apiStartedQpc>=down.apiStartedQpc, 'QPC interval not trustworthy');
      row.intervalMeaning='native PInvoke boundaries, includes inter-command protocol/queries; not OS delivery timestamps';
      row.actualMoveEndToDownMs = (down.apiStartedQpc - move.apiEndedQpc) * 1000 / down.qpcFrequency;
      row.actualDownToUpMs = (up.apiStartedQpc - down.apiStartedQpc) * 1000 / down.qpcFrequency;
      const observed = await io.drain(); // bounded observation only AFTER up
      row.board = observed.board.slice(baseline.board.length);
      row.lab = observed.lab.slice(baseline.lab.length);
      row.verdict = receiptVerdict(row.board, row.lab, role === 'inside' ? 'lab' : 'board');
      row.verdict.nativeTagMismatch = row.board.some(e => ['down','up'].includes(e.family) && e.extraInfo !== 0xA10000 + index + 1);
      row.verdict.noActivateUnconfirmed = row.board.some(e=>PRIMARY.has(e.family)&&e.foregroundOwned!==true);
      row.verdict.doubleClickObserved = [...row.board,...row.lab].some(e=>e.family==='doubleclick-diagnostic');
      row.verdict.coordinateMismatch = [...row.board,...row.lab].some(e=>PRIMARY.has(e.family)&&
        (e.physicalPoint?.x!==point.x || e.physicalPoint?.y!==point.y));
      if (row.verdict.nativeTagMismatch || row.verdict.noActivateUnconfirmed || row.verdict.doubleClickObserved || row.verdict.coordinateMismatch) row.verdict.accepted = false;
      lastPrimary = primaryCounts(observed);
      assert(row.verdict.accepted, `receipts ${row.verdict.side}/complete=${row.verdict.complete}; no late-PID substitution`);
      await safeStatus(`after-${index + 1}`);
    }
    result.receiptsPassed = true;
  } catch (error) { result.failure = String(error); result.releaseFailureObserved = Boolean(error.releaseFailed); }
  finally {
    try { result.release = result.releaseFailureObserved ? {ok:false,held:true,reason:'prior release failed; no further automatic attempts'} : await io.ensureReleased(); }
    catch(error) { result.release = {ok:false,held:'unknown',error:String(error)}; }
    if (!result.release.ok || result.release.held) {
      result.restoration = { ok: false, pending: true, skipped: 'owned release unconfirmed; no cursor/foreground/window mutations or hard kill' };
      await io.retainUnsafe(result);
    } else {
      try { result.restoration = await io.restore(); }
      catch (error) { result.restoration = { ok: false, pending: true, error: String(error) }; }
    }
  }
  result.complete = result.receiptsPassed === true && result.restoration?.ok === true;
  return result;
}
