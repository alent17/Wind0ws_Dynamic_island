import assert from 'node:assert/strict';
import { appendFile, mkdir, writeFile } from 'node:fs/promises';
import { chromium } from 'playwright';

const port = Number(process.argv[2] || 9227);
const outputPath = process.argv[3] || 'dist/performance/c1-2026-10-12/interactive.json';
const startedAt = new Date().toISOString();
const progressPath = `${outputPath}.progress.jsonl`;
await mkdir(outputPath.replace(/[\\/][^\\/]+$/, ''), { recursive: true });
const mark = async (stage, extra = {}) => appendFile(progressPath, `${JSON.stringify({ at: new Date().toISOString(), stage, ...extra })}\n`);
await mark('start');
const wait = milliseconds => new Promise(resolve => setTimeout(resolve, milliseconds));
const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);

try {
  await mark('cdp-connected');
  const context = browser.contexts()[0];
  const pages = context.pages();
  const main = pages.find(page => !page.url().includes('window=floating') && /index|tauri\.localhost\/$/.test(page.url()));
  assert(main, `Main island window was not found: ${pages.map(page => page.url()).join(', ')}`);

  await mark('setup-start');
  const start = await main.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    if (!invoke) throw new Error('Tauri invoke bridge is unavailable');
    const call = async name => {
      const started = performance.now();
      try {
        const value = await invoke(name);
        return { value, ms: performance.now() - started, ok: true };
      } catch (error) {
        return { error: String(error), ms: performance.now() - started, ok: false };
      }
    };
    const monitors = await invoke('get_monitors');
    if (monitors.length > 1) await invoke('move_to_monitor', { monitorIndex: 1 });
    const [stateCall, sessionsCall, mediaStateCall, devicesCall] = await Promise.all([
      call('get_system_audio_state'),
      call('list_media_sessions'),
      call('get_media_info_cmd'),
      call('list_audio_output_devices'),
    ]);
    if (![stateCall, sessionsCall, mediaStateCall, devicesCall].every(result => result.ok)) {
      throw new Error(JSON.stringify([stateCall, sessionsCall, mediaStateCall, devicesCall]));
    }
    window.__c1FrameDeltas = [];
    let previous;
    window.__c1FrameActive = true;
    const frame = timestamp => {
      if (!window.__c1FrameActive) return;
      if (previous !== undefined) window.__c1FrameDeltas.push(timestamp - previous);
      previous = timestamp;
      requestAnimationFrame(frame);
    };
    requestAnimationFrame(frame);
    return {
      monitors,
      state: stateCall.value,
      sessions: sessionsCall.value,
      mediaState: mediaStateCall.value,
      devices: devicesCall.value,
      initialCommandLatency: [
        { name: 'get_system_audio_state', ms: stateCall.ms },
        { name: 'list_media_sessions', ms: sessionsCall.ms },
        { name: 'get_media_info_cmd', ms: mediaStateCall.ms },
        { name: 'list_audio_output_devices', ms: devicesCall.ms },
      ],
    };
  });
  await mark('setup-complete', { sessionCount: start.sessions.length });

  const invokeTiming = async (name, args, command = name) => {
    await mark('invoke-start', { name });
    const started = performance.now();
    try {
      const value = await main.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), { command, args });
      await mark('invoke-complete', { name, ok: true, elapsedMs: performance.now() - started });
      return { name, ms: performance.now() - started, ok: true, value };
    } catch (error) {
      await mark('invoke-complete', { name, ok: false, elapsedMs: performance.now() - started, error: String(error) });
      return { name, ms: performance.now() - started, ok: false, error: String(error) };
    }
  };

  const audio = [];
  const volumeValues = Array.from({ length: 30 }, (_, index) => index % 2 === 0 ? 34 : 32);
  volumeValues.push(start.state.volumePercent);
  const volumeRun = (async () => {
    for (const volumePercent of volumeValues) {
      audio.push(await invokeTiming('set_system_volume', { volumePercent }));
      await wait(140);
    }
  })();

  const media = [];
  const mediaRun = (async () => {
    if (start.sessions.length === 0) return;
    media.push(await invokeTiming('control_media', { action: 'play_pause' }));
    media.push(await invokeTiming('seek_media', { positionMs: 10000 }));
    media.push(await invokeTiming('seek_media_restore', { positionMs: start.mediaState.positionMs }, 'seek_media'));
    const afterToggle = await invokeTiming('get_media_info_cmd');
    if (afterToggle.ok && afterToggle.value.isPlaying !== start.mediaState.isPlaying) {
      media.push(await invokeTiming('control_media_restore', { action: 'play_pause' }, 'control_media'));
    }
  })();

  const surface = main.locator('.island-surface').first();
  await surface.waitFor({ state: 'visible', timeout: 10000 });
  const toggles = [];
  for (let index = 0; index < 16; index++) {
    const wasExpanded = await surface.evaluate(element => element.classList.contains('expanded-surface'));
    await mark('toggle-start', { index: index + 1, wasExpanded });
    const started = performance.now();
    await surface.evaluate(element => element.click());
    await wait(240);
    const isExpanded = await surface.evaluate(element => element.classList.contains('expanded-surface'));
    toggles.push({ ms: performance.now() - started, wasExpanded, isExpanded });
    await mark('toggle-complete', { index: index + 1, isExpanded, elapsedMs: performance.now() - started });
    assert.notEqual(isExpanded, wasExpanded, `Expansion toggle ${index + 1} did not change state`);
  }
  await Promise.all([volumeRun, mediaRun]);
  audio.push(await invokeTiming('set_system_volume_restore', { volumePercent: start.state.volumePercent }, 'set_system_volume'));
  const finalAudioState = await invokeTiming('get_system_audio_state');
  const finalMediaState = start.sessions.length > 0
    ? await invokeTiming('get_media_info_final', {}, 'get_media_info_cmd')
    : null;

  const frames = await main.evaluate(() => {
    window.__c1FrameActive = false;
    return window.__c1FrameDeltas || [];
  });
  const sortedFrames = [...frames].sort((a, b) => a - b);
  const percentile = p => sortedFrames[Math.min(sortedFrames.length - 1, Math.floor(sortedFrames.length * p))] ?? null;
  const report = {
    startedAt,
    generatedAt: new Date().toISOString(),
    monitorHint: 'Use the user-selected monitor; record actual HWND monitor separately.',
    initialVolumePercent: start.state.volumePercent,
    finalAudioState: finalAudioState.value,
    sessions: start.sessions,
    mediaState: start.mediaState,
    finalMediaState: finalMediaState?.value ?? null,
    monitors: start.monitors,
    initialCommandLatency: start.initialCommandLatency,
    devices: start.devices.map(({ id, name, isDefault }) => ({ id, name, isDefault })),
    mediaCommands: media,
    volumeCommands: audio,
    toggles,
    animationFrameDeltasMs: {
      samples: frames.length,
      p50: percentile(0.5),
      p95: percentile(0.95),
      p99: percentile(0.99),
      max: sortedFrames.at(-1) ?? null,
      over50ms: frames.filter(delta => delta > 50).length,
      over100ms: frames.filter(delta => delta > 100).length,
      limitation: 'WebView requestAnimationFrame cadence; this is not DWM/physical presentation timing.',
    },
  };
  await mkdir(outputPath.replace(/[\\/][^\\/]+$/, ''), { recursive: true });
  await writeFile(outputPath, JSON.stringify(report, null, 2));
  console.log(JSON.stringify({ outputPath, sessions: report.sessions.length, volumeCalls: audio.length, toggles: toggles.length, frames: report.animationFrameDeltasMs }));
} finally {
  await browser.close();
}
