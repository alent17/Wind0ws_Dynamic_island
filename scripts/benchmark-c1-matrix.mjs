import assert from 'node:assert/strict';
import { appendFile, mkdir, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { summarizeCommandCoverage } from './a5-command-coverage.mjs';

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.A5_PLAYWRIGHT_MODULE_PATH || 'playwright');

const port = Number(process.argv[2] || 9227);
const mode = process.argv[3] || 'combined';
const outputPath = process.argv[4] || `dist/performance/c1-2026-10-12/${mode}.json`;
const instrument = process.argv.includes('--instrument') || process.env.A5_INSTRUMENT === '1';
assert(['animation-only', 'controls-only', 'combined'].includes(mode), `Unknown mode: ${mode}`);
const sequenceGapMs = Number(process.argv.find(a=>a.startsWith('--sequence-gap-ms='))?.split('=')[1] || 0);
assert(Number.isFinite(sequenceGapMs) && sequenceGapMs >= 0 && sequenceGapMs <= 1000);
const progressPath = `${outputPath}.progress.jsonl`;
await mkdir(outputPath.replace(/[\\/][^\\/]+$/, ''), { recursive: true });
const mark = async (stage, extra = {}) => appendFile(progressPath, `${JSON.stringify({ at: new Date().toISOString(), stage, mode, ...extra })}\n`);
const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
await mark('start');
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
let main;
let setup;
let result;
let sequenceWindow;
const mediaCommands = [];
const volumeCommands = [];
const transitionControls = [];
const toggleRecords = [];

try {
  const context = browser.contexts()[0];
  const pages = context.pages();
  main = pages.find(page => !page.url().includes('window=') && /index|tauri\.localhost\/$/.test(page.url()));
  assert(main, `Main island window not found: ${pages.map(page => page.url()).join(', ')}`);
  await main.evaluate(source => { window.__a5ExpectedSource = source; }, process.env.A5_EXPECT_MEDIA_SOURCE || null);
  setup = await main.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) throw new Error('Tauri invoke bridge unavailable');
    const originalSettings = await invoke('get_settings');
    const monitors = await invoke('get_monitors');
    if (monitors.length > 1) await invoke('move_to_monitor', { monitorIndex: 1 });
    const call = async name => ({ name, startedAt: new Date().toISOString(), value: await invoke(name) });
    const state = await invoke('get_system_audio_state');
    const sessions = await invoke('list_media_sessions');
    const userMedia = await invoke('get_media_info_cmd');
    let selectionChanged = false;
    const expectedSource = window.__a5ExpectedSource;
    if (expectedSource && userMedia.source !== expectedSource) {
      const selected = sessions.find(s => s.source === expectedSource);
      if (!selected) throw new Error('Expected real player session absent');
      await invoke('update_settings', { patch: { selectedPlayerIds: [selected.id], playerOrderIds: [selected.id] } });
      selectionChanged = true;
    }
    const media = await invoke('get_media_info_cmd');
    window.__a5OriginalState = { audio: state, media, userMedia, originalSettings, selectionChanged };
    const settings = await invoke('get_settings');
    // All matrix cells begin in the same paused/33% state. Leave that state in
    // place after each cell so the next build starts from the same condition.
    if (state.volumePercent !== 33) await invoke('set_system_volume', { volumePercent: 33 });
    let mediaNow = media;
    for (let retry = 0; retry < 2 && mediaNow.isPlaying; retry++) {
      await invoke('control_media', { action: 'play_pause' });
      await new Promise(resolve => setTimeout(resolve, 220));
      mediaNow = await invoke('get_media_info_cmd');
    }
    if (mediaNow.isPlaying) throw new Error('Could not establish paused initial media state');

    const surface = document.querySelector('.island-surface');
    if (!surface) throw new Error('Island surface not found');
    window.__c1AnimationLog = [];
    const snapshot = () => ({
      at: new Date().toISOString(),
      performanceMs: performance.now(),
      expanded: surface.classList.contains('expanded-surface'),
      width: surface.style.width,
      height: surface.style.height,
      radius: surface.style.borderRadius,
      clipPath: surface.style.clipPath,
    });
    window.__c1StyleObserver = { lastChange: performance.now(), changes: 0, snapshot, transitionEvents: [] };
    const observer = new MutationObserver(records => {
      if (!records.some(record => record.attributeName === 'style')) return;
      const state = window.__c1StyleObserver;
      state.lastChange = performance.now();
      state.changes++;
      state.lastSnapshot = snapshot();
      if (state.waitingForStart && !state.firstStyleChange) {
        state.firstStyleChange = state.lastSnapshot;
        state.onFirstMutation?.();
        state.onFirstMutation = null;
      }
      if (state.waitingForStart) state.lastStyleChange = state.lastSnapshot;
    });
    observer.observe(surface, { attributes: true, attributeFilter: ['style'] });
    const handlers = [];
    for (const type of ['transitionrun', 'transitionstart', 'transitionend', 'transitioncancel']) {
      const handler = event => {
        const state = window.__c1StyleObserver;
        if (!state.waitingForStart) return;
        state.transitionEvents.push({ type, propertyName: event.propertyName, elapsedTime: event.elapsedTime,
          at: new Date().toISOString(), performanceMs: performance.now() });
      };
      surface.addEventListener(type, handler, true);
      handlers.push([type, handler]);
    }
    window.__c1StyleObserver.disconnect = () => { observer.disconnect(); for (const [type, handler] of handlers) surface.removeEventListener(type, handler, true); };
    window.__a5PendingControls = [];
    window.__a5ControlInFlight = 0;
    window.__a5ControlPeak = 0;
    return { monitors, sessions, originalSettings, selectionChanged, userMedia, originalVolume: state.volumePercent, originalMedia: media, settings,
      devicePixelRatio: devicePixelRatio, viewport: { width: innerWidth, height: innerHeight },
      initialVolume: 33, initialMedia: mediaNow, surfaceExpanded: surface.classList.contains('expanded-surface') };
  });
  await mark('setup-complete', { monitorCount: setup.monitors.length, sessions: setup.sessions.length, initialMedia: setup.initialMedia, initialVolume: setup.initialVolume });

  const invokeTimed = async (name, args, command = name) => {
    const startedAt = new Date().toISOString();
    const before = process.hrtime.bigint();
    await mark('invoke-start', { name, startedAt });
    try {
      const dispatched = await main.evaluate(async ({ command, args }) => {
        const startedAt = new Date().toISOString();
        const startedPerformanceMs = performance.now();
        try {
          const value = await window.__TAURI_INTERNALS__.invoke(command, args);
          return { ok: true, value, pageStartedAt: startedAt, pageEndedAt: new Date().toISOString(),
            pageStartedPerformanceMs: startedPerformanceMs, pageEndedPerformanceMs: performance.now() };
        } catch (error) {
          return { ok: false, error: String(error), pageStartedAt: startedAt, pageEndedAt: new Date().toISOString(),
            pageStartedPerformanceMs: startedPerformanceMs, pageEndedPerformanceMs: performance.now() };
        }
      }, { command, args });
      const elapsedMs = Number(process.hrtime.bigint() - before) / 1e6;
      await mark('invoke-end', { name, ok: dispatched.ok, elapsedMs });
      return { name, command, startedAt, endedAt: new Date().toISOString(), elapsedMs, ...dispatched };
    } catch (error) {
      const elapsedMs = Number(process.hrtime.bigint() - before) / 1e6;
      await mark('invoke-end', { name, ok: false, elapsedMs, error: String(error) });
      return { name, startedAt, endedAt: new Date().toISOString(), elapsedMs, ok: false, error: String(error) };
    }
  };

  let instrumentation = null;
  if (instrument) await main.evaluate(() => {
    const callbacks = window.__TAURI_INTERNALS__.callbacks;
    const originalFetch = window.fetch;
    const originalSet = callbacks.set;
    const oldSetDescriptor = Object.getOwnPropertyDescriptor(callbacks, 'set');
    const webview = window.chrome?.webview;
    const originalPost = webview?.postMessage;
    const records = [];
    const pending = new Map();
    const wrappers = new Map();
    let inFlight = 0;
    let peakInFlight = 0;
    let dropped = 0;
    let unsupportedPayloads = 0;
    const observe = (command, callback, error, argumentsJson, transport) => {
      if (pending.has(String(callback))) return; // A transport retry is not a new invoke.
      const record = { command, startedPerformanceMs: performance.now(), startedAt: new Date().toISOString(),
        callback: String(callback), errorCallback: String(error), transport,
        argumentsJson, argumentBytes: new TextEncoder().encode(argumentsJson).length };
      inFlight++;
      peakInFlight = Math.max(peakInFlight, inFlight);
      if (records.length < 20000) records.push(record); else dropped++;
      pending.set(String(callback), { record, ok: true });
      pending.set(String(error), { record, ok: false });
    };
    callbacks.set = function(id, callback) {
      const wrapped = data => {
        const entry = pending.get(String(id));
        if (entry) {
          const record = entry.record;
          record.ok = entry.ok;
          record.endedPerformanceMs = performance.now(); record.endedAt = new Date().toISOString();
          pending.delete(record.callback); pending.delete(record.errorCallback); inFlight--;
        }
        return callback(data);
      };
      wrappers.set(id, { wrapped, callback });
      return originalSet.call(callbacks, id, wrapped);
    };
    window.fetch = function(input, init, ...rest) {
      const url = String(input);
      if (url.includes('ipc.localhost') || url.startsWith('ipc:')) {
        const headers = new Headers(init?.headers);
        const callback = headers.get('Tauri-Callback'), error = headers.get('Tauri-Error');
        if (callback !== null && error !== null) {
          if (typeof init?.body === 'string') observe(decodeURIComponent(new URL(url).pathname.replace(/^\//, '')), callback, error, init.body, 'custom-protocol fetch');
          else unsupportedPayloads++;
        }
      }
      return originalFetch.call(this, input, init, ...rest);
    };
    if (webview && originalPost) webview.postMessage = function(message) {
      try {
        const data = typeof message === 'string' ? JSON.parse(message) : message;
        if (data?.cmd && data.callback != null) observe(data.cmd, data.callback, data.error, JSON.stringify(data.payload ?? null), 'WebView2 postMessage');
      } catch {}
      return originalPost.call(this, message);
    };
    if (window.fetch === originalFetch || callbacks.set === originalSet) throw new Error('Diagnostic transport/callback proxy could not be installed');
    let stopped = false;
    window.__a5Instrumentation = { records, async stop() {
      if (!stopped) {
        stopped = true;
        window.fetch = originalFetch;
        if (webview && originalPost) webview.postMessage = originalPost;
        if (oldSetDescriptor) Object.defineProperty(callbacks, 'set', oldSetDescriptor); else delete callbacks.set;
        const deadline = performance.now() + 300;
        while (inFlight && performance.now() < deadline) await new Promise(resolve => setTimeout(resolve, 10));
        for (const [id, { wrapped, callback }] of wrappers) if (callbacks.get(id) === wrapped) originalSet.call(callbacks, id, callback);
      }
      return { records, peakInFlight, inFlightAtStop: inFlight, dropped, unsupportedPayloads,
        scope: 'Main-page invoke requests observed at custom-protocol fetch/WebView2 transport and completed at registered callbacks; retries deduplicated by callback ID. UTF-8 payload bytes exclude headers/response/events. Transport/callback proxy adds overhead.' };
    } };
  });
  const doControls = async () => {
    if (setup.sessions.length > 0) {
      mediaCommands.push(await invokeTimed('control_media_play', { action: 'play_pause' }, 'control_media'));
      await wait(250);
      mediaCommands.push(await invokeTimed('seek_media', { positionMs: 10000 }));
      mediaCommands.push(await invokeTimed('control_media_pause', { action: 'play_pause' }, 'control_media'));
      mediaCommands.push(await invokeTimed('seek_media_restore', { positionMs: setup.initialMedia.positionMs }, 'seek_media'));
    }
    for (const volumePercent of [32, 34, 32, 34, 33]) {
      volumeCommands.push(await invokeTimed('set_system_volume', { volumePercent }));
      await wait(120);
    }
    volumeCommands.push(await invokeTimed('set_system_volume_restore', { volumePercent: 33 }, 'set_system_volume'));
    const finalAudio = await invokeTimed('get_system_audio_final', {}, 'get_system_audio_state');
    if (setup.sessions.length > 0) mediaCommands.push(await invokeTimed('get_media_info_final', {}, 'get_media_info_cmd'));
    return { finalAudio: finalAudio.value, finalMedia: mediaCommands.at(-1)?.value };
  };

  const surface = main.locator('.island-surface').first();
  await surface.waitFor({ state: 'visible', timeout: 10000 });
  const doAnimation = async (withControls = false) => {
    sequenceWindow = { start: await main.evaluate(() => ({ at: new Date().toISOString(), performanceMs: performance.now() })) };
    await mark("sequence-start", sequenceWindow.start);
    for (let index = 0; index < 16; index++) {
      const clickAt = new Date().toISOString();
      const before = await main.evaluate(() => {
        const element = document.querySelector('.island-surface');
        const tracker = window.__c1StyleObserver;
        tracker.firstStyleChange = null;
        tracker.lastStyleChange = null;
        tracker.transitionEvents = [];
        tracker.changes = 0;
        tracker.waitingForStart = true;
        return tracker.snapshot();
      });
      await mark('transition-click', { index: index + 1, clickAt, fromExpanded: before.expanded, before });
      const clickDispatched = await surface.evaluate(element => {
        const stamp = { at: new Date().toISOString(), performanceMs: performance.now() };
        element.click();
        return stamp;
      });
      // Start an identical command pair for every transition. Serial admission
      // between transitions prevents an ever-growing worker/CDP queue.
      const controlsPromise = withControls ? main.evaluate(async transitionIndex => {
        const tracker = window.__c1StyleObserver;
        if (!tracker.firstStyleChange) await new Promise((resolve, reject) => {
          const timer = setTimeout(() => { tracker.onFirstMutation = null; reject(new Error('First mutation absent; no controls dispatched')); }, 2000);
          tracker.onFirstMutation = () => { clearTimeout(timer); resolve(); };
        });
        const invoke = async (command, args) => {
          const record = { name: command, command, args, transitionIndex, pageStartedAt: new Date().toISOString(), pageStartedPerformanceMs: performance.now() };
          window.__a5ControlInFlight++;
          window.__a5ControlPeak = Math.max(window.__a5ControlPeak, window.__a5ControlInFlight);
          try { record.value = await window.__TAURI_INTERNALS__.invoke(command, args); record.ok = true; }
          catch (error) { record.ok = false; record.error = error; }
          finally { record.pageEndedAt = new Date().toISOString(); record.pageEndedPerformanceMs = performance.now(); window.__a5ControlInFlight--; }
          return record;
        };
        const pending = Promise.all([invoke('control_media', { action: 'play_pause' }), invoke('set_system_volume', { volumePercent: transitionIndex % 2 ? 32 : 33 })]);
        window.__a5PendingControls.push(pending);
        return await pending;
      }, index + 1).then(records => {
        mediaCommands.push(records[0]); volumeCommands.push(records[1]); return records;
      }) : null;
      controlsPromise?.catch(() => {}); // Retain the failure for serial admission below.
      let start = null;
      let end = null;
      const deadline = Date.now() + 3000;
      while (Date.now() < deadline) {
        const state = await main.evaluate(() => {
          const tracker = window.__c1StyleObserver;
          const current = tracker.snapshot();
          return { current, firstStyleMutation: tracker.firstStyleChange, lastStyleMutation: tracker.lastStyleChange,
            lastChange: tracker.lastChange, now: performance.now(), changes: tracker.changes, transitionEvents: tracker.transitionEvents };
        });
        if (!start && state.firstStyleMutation) start = state.firstStyleMutation;
        if (start && state.current.expanded !== before.expanded && state.now - state.lastChange >= 100) {
          end = { targetSnapshot: state.current, lastStyleMutation: state.lastStyleMutation,
            confirmation: { at: state.current.at, performanceMs: state.current.performanceMs },
            mutationCount: state.changes, transitionEvents: state.transitionEvents };
          break;
        }
        await wait(10);
      }
      await main.evaluate(() => { window.__c1StyleObserver.waitingForStart = false; });
      assert(end, `Transition ${index + 1} did not settle within 3 s`);
      const record = { index: index + 1, clickAt, clickDispatched,
        clickToFirstMutationMs: start ? start.performanceMs - clickDispatched.performanceMs : null,
        fromExpanded: before.expanded, toExpanded: end.targetSnapshot.expanded,
        firstStyleMutation: start, lastStyleMutation: end.lastStyleMutation, settleConfirmation: end.confirmation,
        mutationCount: end.mutationCount, transitionEvents: end.transitionEvents,
        firstToLastStyleMutationMs: start && end.lastStyleMutation ? end.lastStyleMutation.performanceMs - start.performanceMs : null,
        firstMutationToConfirmationMs: start ? end.confirmation.performanceMs - start.performanceMs : null };
      toggleRecords.push(record);
      await mark('transition-settled', record);
      if (controlsPromise) {
        let timer;
        const commands = await Promise.race([controlsPromise, new Promise((_, reject) => { timer = setTimeout(() => reject(new Error('Control pair timed out; stop admission instead of queuing more')), 10000); })]).finally(() => clearTimeout(timer));
        transitionControls.push({ index: index + 1, commands });
        assert(commands.every(command => command.ok), `Transition ${index + 1} has a failed control request`);
      }
      if (index < 15 && sequenceGapMs) await wait(sequenceGapMs);
    }
    sequenceWindow.end = await main.evaluate(() => ({ at: new Date().toISOString(), performanceMs: performance.now() }));
    await mark("sequence-end", sequenceWindow.end);
  };

  let controlResult = null;
  if (mode === 'animation-only') await doAnimation();
  if (mode === 'controls-only') controlResult = await doControls();
  if (mode === 'combined') {
    assert(setup.sessions.length > 0, 'Full concurrent capture requires a real media session');
    if (process.env.A5_EXPECT_MEDIA_SOURCE) assert.equal(setup.initialMedia.source, process.env.A5_EXPECT_MEDIA_SOURCE);
    await doAnimation(true);
    const finalAudio = await invokeTimed('get_system_audio_final', {}, 'get_system_audio_state');
    const finalMedia = await invokeTimed('get_media_info_final', {}, 'get_media_info_cmd');
    controlResult = { finalAudio: finalAudio.value, finalMedia: finalMedia.value };
    assert(finalAudio.ok && finalAudio.value.volumePercent === 33, 'Volume did not return to 33%');
    assert(finalMedia.ok && finalMedia.value.isPlaying === false, 'Media did not return to paused');
  }
  if (instrument) instrumentation = await main.evaluate(() => window.__a5Instrumentation.stop());
  const coverage = summarizeCommandCoverage(toggleRecords, [...mediaCommands, ...volumeCommands]);
  const tail = await main.evaluate(() => {
    window.__c1StyleObserver.disconnect();
    return { state: window.__c1StyleObserver.snapshot(), sessions: window.__TAURI_INTERNALS__ ? 'bridge-present' : 'bridge-missing' };
  });
  if (instrument) assert(instrumentation?.records.length > 0 && instrumentation.unsupportedPayloads === 0, 'IPC diagnostic wrapper must produce complete JSON payload records');
  result = { generatedAt: new Date().toISOString(), mode, port, invocation: process.argv.slice(2), instrumentRequested: instrument, sequenceGapMs, sequenceWindow, setup, toggleRecords, mediaCommands, volumeCommands, transitionControls, controlResult, coverage, instrumentation, tail,
    controlSchedule: mode === 'combined' ? 'One concurrent media toggle and volume write dispatched for EACH transition; await completion before the next transition.' : mode,
    timingDefinition: 'Browser performance.now records dispatch, first/last inline-style mutation and separate >=100ms confirmation. Page-side invoke envelopes are bridge request/response times, not worker execution traces. PresentMon QPC plus MsUntilDisplayed defines display time. MutationObserver performs no layout reads.' };
  await writeFile(outputPath, JSON.stringify(result, null, 2));
  await mark('complete', { transitions: toggleRecords.length, mediaCalls: mediaCommands.length, volumeCalls: volumeCommands.length });
  console.log(JSON.stringify({ outputPath, mode, transitions: toggleRecords.length, mediaCalls: mediaCommands.length, volumeCalls: volumeCommands.length }));
  if (mode === 'combined') assert(coverage.accepted, `Concurrent coverage incomplete: ${JSON.stringify(coverage)}`);
} catch (error) {
  result ??= { generatedAt: new Date().toISOString(), mode, port, setup, toggleRecords, mediaCommands, volumeCommands, transitionControls };
  result.failure = String(error);
  process.exitCode = 1;
} finally {
  // Restore state even when a transition or command fails mid-run. Preserve
  // the paused/33% convention of the matched captures, and report failures.
  try {
    if (main) result ??= { generatedAt: new Date().toISOString(), mode, port, setup, toggleRecords, mediaCommands, volumeCommands, transitionControls };
    if (main) result.restoration = await main.evaluate(async () => {
        let timer;
        await Promise.race([Promise.allSettled(window.__a5PendingControls ?? []), new Promise(resolve => { timer = setTimeout(resolve, 3000); })]);
        clearTimeout(timer);
        const inFlight = window.__a5ControlInFlight ?? 0;
        if (inFlight) return { accepted: false, inFlight, error: 'Outstanding control requests; restoration not issued concurrently' };
        await window.__a5Instrumentation?.stop();
        window.__c1StyleObserver?.disconnect();
        const invoke = window.__TAURI_INTERNALS__?.invoke;
        if (!invoke) return { accepted: false, error: 'Missing bridge' };
        const original = window.__a5OriginalState;
        await invoke('set_system_volume', { volumePercent: original?.audio.volumePercent ?? 33 });
        const media = await invoke('get_media_info_cmd');
        if (media.isPlaying !== (original?.media.isPlaying ?? false)) await invoke('control_media', { action: 'play_pause' });
        const restoredAudio = await invoke('get_system_audio_state');
        const restoredMedia = await invoke('get_media_info_cmd');
        delete restoredMedia.albumArt;
        if (original?.selectionChanged) await invoke('update_settings', { patch: { selectedPlayerIds: original.originalSettings.selectedPlayerIds, playerOrderIds: original.originalSettings.playerOrderIds } });
        if (original?.originalSettings.monitorIndex !== undefined) await invoke('move_to_monitor', { monitorIndex: original.originalSettings.monitorIndex });
        const restoredSettings = await invoke('get_settings');
        return { restoredSettings, selectionRestored: JSON.stringify(restoredSettings.selectedPlayerIds) === JSON.stringify(original?.originalSettings.selectedPlayerIds), accepted: restoredAudio.volumePercent === original?.audio.volumePercent && restoredMedia.isPlaying === original?.media.isPlaying && JSON.stringify(restoredSettings.selectedPlayerIds) === JSON.stringify(original?.originalSettings.selectedPlayerIds) && JSON.stringify(restoredSettings.playerOrderIds) === JSON.stringify(original?.originalSettings.playerOrderIds) && restoredSettings.monitorIndex === original?.originalSettings.monitorIndex,
          peakControlInFlight: window.__a5ControlPeak, inFlight, restoredAudio, restoredMedia };
      });
    if (result?.restoration?.accepted === false) process.exitCode = 1;
  } catch (error) { if (result) result.restorationError = String(error); console.error(`State restoration failed: ${error}`); process.exitCode = 1; }
  if (result) await writeFile(outputPath, JSON.stringify(result, null, 2));
  await browser.close();
}
