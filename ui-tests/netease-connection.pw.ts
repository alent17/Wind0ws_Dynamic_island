import { expect, test } from "@playwright/test";

for (const surface of ["island", "floating"]) test(`NetEase ${surface} keeps the confirmed mode throughout a track change with unavailable progress`, async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  const state = { source: "netease", sourceDisplay: "cloudmusic.exe", title: "First", artist: "Artist", isPlaying: true,
    nativeEpoch: "mode-transition", nativeRevision: 1, nativeTrackGeneration: 1, positionMs: 1000, durationMs: 120000, progressKnown: true,
    shuffleActive: true, capabilities: { seek: true, shuffle: true, repeat: false },
    neteaseFeatures: { mode: { source: "cdp", readable: true, controllable: true }, like: { source: "cdp", readable: true, controllable: true }, progress: { source: "cdp", readable: true, controllable: true } } };
  await page.evaluate(({ surface, state }) => {
    const fixture = window as any;
    if (surface === "floating") { fixture.mediaInfoSnapshot = state; fixture.mountFloating({ language: "zh-CN", enableMvPlayback: false }); }
    else { fixture.mountApp(); fixture.expand(); }
  }, { surface, state });
  if (surface === "floating") await page.waitForFunction(() => (window as any).floatingReadyMessages.length > 0);
  else await page.evaluate(state => (window as any).media(state), state);
  const mode = page.locator(".mode-control");
  await expect(mode).toHaveAttribute("aria-label", /列表循环|Repeat list/);
  const confirmedMode = await mode.getAttribute("aria-label");
  await page.evaluate(() => {
    const fixture = window as any;
    fixture.modeFrames = [];
    fixture.modeObserver = new MutationObserver(() => fixture.modeFrames.push({
      mode: document.querySelector(".mode-control")?.getAttribute("aria-label"),
      systemControls: document.querySelectorAll(".smtc-modes").length,
    }));
    fixture.modeObserver.observe(document.body, { attributes: true, childList: true, subtree: true });
  });
  await page.evaluate(state => (window as any).media({ ...state, title: "Second", nativeRevision: 2, nativeTrackGeneration: 2,
    progressKnown: false, capabilities: { ...state.capabilities, seek: false },
    neteaseFeatures: { ...state.neteaseFeatures, progress: { source: "unavailable", readable: false, controllable: false } } }), state);
  await expect(page.getByText("Second", { exact: true }).first()).toBeAttached();
  await expect(mode).toHaveAttribute("aria-label", confirmedMode!);
  await page.evaluate(state => (window as any).media({ ...state, title: "Second", nativeRevision: 3, nativeTrackGeneration: 2 }), state);
  await expect(mode).toHaveAttribute("aria-label", confirmedMode!);
  const recorded = await page.evaluate(() => {
    const fixture = window as any;
    fixture.modeObserver.disconnect();
    return { frames: fixture.modeFrames, calls: fixture.calls };
  });
  expect(recorded.frames.length).toBeGreaterThan(0);
  expect(recorded.frames.every((frame: { mode?: string; systemControls: number }) => frame.mode === confirmedMode && frame.systemControls === 0)).toBe(true);
  expect(recorded.calls.filter((call: string) => ["cycle_netease_playback_mode", "toggle_shuffle", "cycle_repeat"].includes(call))).toEqual([]);
  expect(recorded.calls.filter((call: string) => call === "get_netease_playback_mode")).toHaveLength(1);
});

for (const surface of ["island", "floating"]) test(`NetEase ${surface} mode recovers from a transient read and a changed session`, async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(surface => {
    const fixture = window as any;
    fixture.modeReadFailures = 1;
    if (surface === "floating") fixture.mountFloating({ language: "zh-CN", enableMvPlayback: false });
    else { fixture.mountApp(); fixture.expand(); }
  }, surface);
  if(surface === "floating")await page.waitForFunction(() => (window as any).floatingReadyMessages.length > 0);
  const state = { source: "netease", sourceDisplay: "cloudmusic.exe", title: "Mode song", artist: "Artist", isPlaying: true, nativeEpoch: "mode-recovery", nativeRevision: 1, nativeTrackGeneration: 1,
    neteaseFeatures: { mode: { source: "cdp", readable: true, controllable: true }, like: { source: "cdp", readable: true, controllable: true }, progress: { source: "cdp", readable: true, controllable: true } } };
  await page.evaluate(state => (window as any).media(state), state);
  await expect(page.locator(".mode-control")).toHaveCount(1);
  expect(await page.evaluate(() => (window as any).calls.filter((c: string) => c === "get_netease_playback_mode").length)).toBe(2);
  await page.evaluate(state => (window as any).media({ ...state, nativeEpoch: "mode-replacement", nativeRevision: 2 }), state);
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((c: string) => c === "get_netease_playback_mode").length)).toBe(3);
  await expect(page.locator(".mode-control")).toHaveCount(1);
});

test("NetEase unknown mode stops retrying and can be recovered by clicking retry", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture = window as any;
    fixture.modeUnknown = true;
    fixture.mediaInfoSnapshot = { source: "netease", sourceDisplay: "cloudmusic.exe", title: "Mode song", artist: "Artist", nativeEpoch: "mode-manual", nativeRevision: 1, nativeTrackGeneration: 1,
      neteaseFeatures: { mode: { source: "cdp", readable: true, controllable: true }, like: { source: "cdp", readable: true, controllable: true }, progress: { source: "cdp", readable: true, controllable: true } } };
    fixture.mountFloating({ language: "zh-CN", enableMvPlayback: false });
  });
  const retry = page.locator("button.unknown-mode");
  await expect(retry).toBeEnabled({ timeout: 12_000 });
  await expect(retry).toHaveAttribute("aria-label", "播放模式暂不可用，点击重试");
  await expect(page.locator(".mode-control")).toHaveCount(0);
  expect(await page.evaluate(() => (window as any).calls.filter((c: string) => c === "get_netease_playback_mode").length)).toBe(4);
  await page.evaluate(() => { (window as any).modeUnknown = false; });
  await page.locator(".player").hover();
  await retry.click();
  await expect(page.locator(".mode-control")).toHaveCount(1);
});

test("NetEase enabling automatic maintenance immediately repairs Start menu entries", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture = window as any;
    fixture.settingsOverrides = { language: "zh-CN", neteaseAutoConnect: false };
    fixture.mountStudio();
  });
  await expect(page.getByText(/开启时立即修复桌面和开始菜单快捷方式/)).toBeVisible();
  await page.getByRole("checkbox", { name: "自动维护网易云连接" }).check();
  await expect.poll(() => page.evaluate(() => (window as any).neteaseSetupCalls)).toEqual([
    { restart: false, bind: true, configureOnly: true, repairStartMenu: true },
  ]);
  await expect(page.getByRole("status").filter({ hasText: "已配置 16 个普通启动入口" })).toBeVisible();
  await expect(page.getByRole("checkbox", { name: "自动维护网易云连接" })).toBeChecked();
});

test("NetEase disabling automatic maintenance does not repair or request elevation", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture = window as any;
    fixture.settingsOverrides = { language: "zh-CN", neteaseAutoConnect: true };
    fixture.mountStudio();
  });
  await page.getByRole("checkbox", { name: "自动维护网易云连接" }).uncheck();
  expect(await page.evaluate(() => (window as any).neteaseSetupCalls)).toEqual([]);
});

test("NetEase reports common Start menu entries still needing administrator approval", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture = window as any;
    fixture.settingsOverrides = { language: "zh-CN", neteaseAutoConnect: false };
    fixture.neteaseSetupResult = { connected: false, configured: true, executablePath: "C:\\NetEase\\cloudmusic.exe", shortcutPath: "", boundSessionId: null, bindingFailed: false, launchEntries: 2, pendingStartMenuEntries: 2 };
    fixture.mountStudio();
  });
  await page.getByRole("checkbox", { name: "自动维护网易云连接" }).check();
  await expect(page.getByRole("status").filter({ hasText: "仍有 2 个公共开始菜单入口需要管理员权限" })).toBeVisible();
  expect(await page.evaluate(() => (window as any).neteaseSetupCalls.length)).toBe(1);
});

test("NetEase reports real connection stages and ignores late progress after success", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture = window as any;
    fixture.settingsOverrides = { language: "zh-CN" };
    fixture.neteaseDeferSetup = true;
    fixture.mountStudio();
  });
  await page.getByRole("button", { name: "连接播放器", exact: true }).click();
  await page.waitForFunction(() => !!(window as any).neteaseSendStage);
  for (const [stage, message] of [
    ["waiting", "正在等待已有连接任务完成…"],
    ["detecting", "正在检测网易云播放器…"],
    ["configuring", "正在配置网易云启动入口…"],
    ["waitingPlayer", "正在等待网易云加载，连接就绪后将自动继续…"],
    ["binding", "正在匹配网易云播放会话，识别后将自动绑定…"],
    ["saving", "正在保存连接设置…"],
  ]) {
    await page.evaluate(stage => (window as any).neteaseSendStage(stage), stage);
    await expect(page.getByRole("status").filter({ hasText: message })).toBeVisible();
    await expect(page.getByRole("button", { name: "检测连接", exact: true })).toBeDisabled();
  }
  expect(await page.evaluate(() => (window as any).neteaseSetupCalls.length)).toBe(1);
  await page.evaluate(() => (window as any).resolveNeteaseSetup({ connected: true, configured: true, executablePath: "C:\\NetEase\\cloudmusic.exe", shortcutPath: "", boundSessionId: "cloudmusic.exe", bindingFailed: false, launchEntries: 1 }));
  await expect(page.getByRole("button", { name: "连接播放器", exact: true })).toBeEnabled();
  await page.evaluate(() => (window as any).neteaseProgress.onmessage("waitingPlayer"));
  await expect(page.getByText("已连接并绑定网易云，真实进度与拖拽控制已启用。", { exact: true })).toBeVisible();
});

for (const [error, message] of [
  ["NETEASE_SETUP_BUSY", "连接任务正在处理中，请稍后重试。"],
  ["NETEASE_SETUP_TIMEOUT", "启动入口检查超时，请稍后点击「检测连接」重试。"],
  ["NETEASE_RESTART_FAILED", "网易云未能完全退出，请手动退出网易云后再点击「连接播放器」。"],
]) test(`NetEase setup explains ${error} and allows retry`, async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(error => {
    const fixture = window as any;
    fixture.settingsOverrides = { language: "zh-CN" };
    fixture.neteaseSetupError = error;
    fixture.mountStudio();
  }, error);
  const connect = page.getByRole("button", { name: "连接播放器", exact: true });
  await connect.click();
  await expect(page.getByText(message, { exact: true })).toBeVisible();
  await expect(connect).toBeEnabled();
});

test("NetEase successful binding survives a failed media or settings refresh", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture = window as any;
    fixture.settingsOverrides = { language: "zh-CN" };
    fixture.neteaseSetupResult = { connected: true, configured: true, executablePath: "C:\\NetEase\\cloudmusic.exe", shortcutPath: "", boundSessionId: "cloudmusic.exe", bindingFailed: false, launchEntries: 1 };
    fixture.neteaseRefreshFails = true;
    fixture.mountStudio();
  });
  const connect = page.getByRole("button", { name: "连接播放器", exact: true });
  await connect.click();
  await expect(page.getByText("已连接并绑定网易云，真实进度与拖拽控制已启用。", { exact: true })).toBeVisible();
  await expect(connect).toBeEnabled();
});

test("NetEase defaults to CDP and exposes automatic maintenance without configuring on mount", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => { (window as any).settingsOverrides = { language: "zh-CN" }; (window as any).mountStudio(); });
  await expect(page.getByRole("button", { name: /网易云直连（CDP）/ })).toHaveAttribute("aria-pressed", "true");
  await expect(page.getByRole("checkbox", { name: "自动维护网易云连接" })).toBeVisible();
  expect(await page.evaluate(() => (window as any).neteaseSetupCalls)).toEqual([]);
});

test("NetEase standard settings expose no launch configuration actions", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => { (window as any).settingsOverrides = { language: "zh-CN", neteaseConnectionMode: "standard" }; (window as any).mountStudio(); });
  await expect(page.getByRole("button", { name: /标准连接/ })).toHaveAttribute("aria-pressed", "true");
  await expect(page.getByRole("button", { name: "配置普通启动入口", exact: true })).toHaveCount(0);
  expect(await page.evaluate(() => (window as any).neteaseSetupCalls)).toEqual([]);
});

for (const surface of ["island", "floating"]) test(`NetEase ${surface} falls back to unknown state without CDP requests`, async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(surface => {
    const fixture = window as any;
    if (surface === "floating") fixture.mountFloating({ language: "zh-CN", enableMvPlayback: false });
    else { fixture.mountApp(); fixture.expand(); }
  }, surface);
  const standard = {
    title: "Like Song", artist: "Artist", source: "netease", sourceDisplay: "cloudmusic.exe", isPlaying: true,
    positionMs: 0, durationMs: 0, progressKnown: false, nativeEpoch: "connection", nativeRevision: 1, nativeTrackGeneration: 1,
    capabilities: { previous: true, playPause: true, next: true, seek: false, shuffle: false, repeat: false },
    neteaseFeatures: {
      progress: { source: "unavailable", readable: false, controllable: false },
      like: { source: "unavailable", readable: false, controllable: false },
      mode: { source: "smtc", readable: false, controllable: false },
    },
  };
  await page.evaluate(payload => (window as any).media(payload), standard);
  const heart = page.locator(".like-control");
  await expect(heart).toBeDisabled();
  await expect(heart).not.toHaveAttribute("aria-pressed");
  await expect(page.locator(".time.elapsed")).toHaveText("--:--");
  await expect(page.locator('input[aria-label="调整播放进度"]')).toHaveCount(0);
  expect(await page.evaluate(() => (window as any).calls.filter((call: string) => ["netease_song_like", "get_netease_playback_mode"].includes(call)))).toEqual([]);

  const enhanced = {
    ...standard, nativeRevision: 2, progressKnown: true, positionMs: 10_000, durationMs: 180_000,
    capabilities: { ...standard.capabilities, seek: true },
    neteaseFeatures: {
      progress: { source: "cdp", readable: true, controllable: true },
      like: { source: "cdp", readable: true, controllable: true },
      mode: { source: "cdp", readable: true, controllable: true },
    },
  };
  await page.evaluate(payload => (window as any).media(payload), enhanced);
  await expect(heart).toBeEnabled();
  await expect(page.locator(".mode-control")).toHaveCount(1);
  await heart.click({ force: surface === "floating" });
  await expect(heart).toHaveAttribute("aria-pressed", "true");
  await page.evaluate(payload => (window as any).media(payload), { ...standard, nativeRevision: 3 });
  await expect(heart).toBeDisabled();
  await expect(heart).not.toHaveAttribute("aria-pressed");
  await expect(heart.locator("svg")).toHaveAttribute("fill", "none");
  await expect(page.locator(".mode-control")).toHaveCount(0);
  await expect(page.locator(".time.elapsed")).toHaveText("--:--");

  // Independently supported system modes must update even on the same track.
  await page.evaluate(payload => (window as any).media(payload), {
    ...standard, nativeRevision: 4, shuffleActive: false, repeatMode: "none",
    capabilities: { ...standard.capabilities, shuffle: true, repeat: true },
    neteaseFeatures: { ...standard.neteaseFeatures, mode: { source: "smtc", readable: true, controllable: true } },
  });
  await expect(page.locator(".smtc-modes button")).toHaveCount(2);
  for (const button of await page.locator(".smtc-modes button").all()) await expect(button).toBeEnabled();
  await expect(heart).toBeDisabled();
});
