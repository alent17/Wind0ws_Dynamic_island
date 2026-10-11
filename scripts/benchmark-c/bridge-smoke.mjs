import assert from 'node:assert/strict';
import { chromium } from '@playwright/test';
import { spawn } from 'node:child_process';
import { copyFile, mkdir, readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';

const variant = process.argv[2];
assert(['before', 'after'].includes(variant));
const root = resolve(process.env.ISLE_BENCH_ROOT || 'dist/performance/m4-2026-10-10');
const output = join(root, 'bridge-smoke-retry3', variant);
await mkdir(output, { recursive: true });
await copyFile(join(root, 'binaries', variant, 'isle.exe'), join(output, 'isle.exe'));
await copyFile(join(root, 'frozen-covers.json'), join(output, 'frozen-covers.json'));
const env = {
  ...process.env,
  ISLE_BENCH_DIR: output,
  WEBVIEW2_USER_DATA_FOLDER: join(output, 'webview'),
  WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-port=9236 --remote-debugging-address=127.0.0.1',
  NO_PROXY: '127.0.0.1,127.0.0.2,localhost',
  no_proxy: '127.0.0.1,127.0.0.2,localhost',
};
const app = spawn(join(output, 'isle.exe'), [], { env, windowsHide: true, stdio: 'ignore' });
const pause = milliseconds => new Promise(done => setTimeout(done, milliseconds));
async function rows() {
  return (await readFile(join(output, 'events.jsonl'), 'utf8')).trim().split('\n').filter(Boolean).map(JSON.parse);
}
let browser;
try {
  for (let attempt = 0; attempt < 100; attempt++) {
    if ((await rows().catch(() => [])).some(row => row.event.kind === 'started')) break;
    if (attempt === 99) throw new Error('Native benchmark adapter did not start');
    await pause(200);
  }
  let targets;
  for (let attempt = 0; attempt < 50; attempt++) {
    try { targets = await (await fetch('http://127.0.0.1:9236/json')).json(); break; }
    catch { if (attempt === 49) throw new Error('WebView2 diagnostic port did not open'); await pause(200); }
  }
  browser = await chromium.connectOverCDP('http://127.0.0.1:9236');
  const context = browser.contexts()[0];
  const main = context.pages().find(page => !page.url().includes('window=floating') && /index|tauri|localhost/.test(page.url()));
  assert(main, JSON.stringify(targets.map(target => target.url)));
  const mainEnabled = await main.evaluate(() => window.__TAURI_INTERNALS__.invoke('benchmark_c_enabled'));
  await main.evaluate(() => window.__TAURI_INTERNALS__.invoke('benchmark_c_report', { value: { kind: 'bridge-probe', window: 'main' } }));
  await writeFile(join(output, 'command.json'), JSON.stringify({ seq: 1, action: 'open' }));
  let floating;
  for (let attempt = 0; attempt < 60; attempt++) {
    floating = context.pages().find(page => page.url().includes('window=floating'));
    if (floating) break;
    await pause(200);
  }
  assert(floating, 'Floating webview did not open');
  const artworkState = await floating.evaluate(() => ({
    title: document.querySelector('.track-title')?.textContent?.trim(),
    coverPresent: !!document.querySelector('img.compact-cover-image')?.src,
    coverLength: document.querySelector('img.compact-cover-image')?.src?.length || 0,
    canvases: [...document.querySelectorAll('canvas')].map(canvas => ({ className: canvas.className, connected: canvas.isConnected, width: canvas.width, height: canvas.height })),
    errors: (window).__ISLE_BENCHMARKC_ERROR__ || null,
  }));
  const floatingEnabled = await floating.evaluate(() => window.__TAURI_INTERNALS__.invoke('benchmark_c_enabled'));
  await floating.evaluate(() => window.__TAURI_INTERNALS__.invoke('benchmark_c_report', { value: { kind: 'bridge-probe', window: 'floating' } }));
  await pause(300);
  const events = await rows();
  const probes = events.filter(row => row.event.kind === 'bridge-probe').map(row => row.event.window);
  assert(mainEnabled && floatingEnabled && probes.includes('main') && probes.includes('floating'), JSON.stringify({ mainEnabled, floatingEnabled, probes }));
  const result = { variant, mainEnabled, floatingEnabled, probes, artworkState, targetUrls: targets.map(target => target.url) };
  await writeFile(join(output, 'bridge.json'), JSON.stringify(result, null, 2));
  console.log(JSON.stringify(result));
} finally {
  if (browser) await browser.close();
  if (app.exitCode === null) app.kill();
}
