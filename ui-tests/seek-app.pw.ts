import { expect, test } from "@playwright/test";

const track = {
  title: "Seek App fixture",
  artist: "Fixture artist",
  albumArt: "",
  isPlaying: true,
  positionMs: 12_000,
  durationMs: 180_000,
  lastUpdatedTimestamp: Date.now(),
  source: "netease",
  sourceDisplay: "网易云音乐",
  capabilities: { previous: true, playPause: true, next: true, seek: true, shuffle: false, repeat: false },
  nativeEpoch: "seek-app-fixture",
  nativeRevision: 1,
  nativeTrackGeneration: 1,
};

test("the full App exposes a range and forwards a failed drag to the seek alert", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html", { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate((media) => {
    const fixture = window as any;
    fixture.rejectNextSeek = true;
    fixture.settingsOverrides = { language: "zh-CN" };
    fixture.mountApp();
  });
  await expect.poll(() => page.evaluate(() => (window as any).calls.includes("show_main_window"))).toBe(true);
  await expect.poll(() => page.evaluate(() => (window as any).mediaListenerCount())).toBe(1);
  await page.evaluate(async (media) => {
    const fixture = window as any;
    await fixture.media(media);
    fixture.expand();
    fixture.hoverIsland();
  }, track);
  await page.waitForTimeout(1_000);

  const range = page.locator('.expanded-layer input[type="range"]');
  await expect(range).toHaveCount(1);
  await expect(range).toHaveAttribute("max", "180000");
  await range.focus();
  await range.press("ArrowRight");
  await expect(page.getByRole("alert")).toContainText("fixture seek failure");
  expect(await page.evaluate(() => (window as any).calls.filter((call: string) => call === "seek_media").length)).toBe(1);
  await page.evaluate(() => (window as any).destroy());
});

test("the full App omits its range when native seek capability or duration is unavailable", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html", { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate((media) => {
    const fixture = window as any;
    fixture.settingsOverrides = { language: "zh-CN" };
    fixture.disableSongMetadataRecovery = true;
    fixture.mountApp();
  });
  await expect.poll(() => page.evaluate(() => (window as any).calls.includes("show_main_window"))).toBe(true);
  await expect.poll(() => page.evaluate(() => (window as any).mediaListenerCount())).toBe(1);
  await page.evaluate(async (media) => {
    const fixture = window as any;
    await fixture.media({
      ...media,
      source: "generic",
      sourceDisplay: "Fixture",
      nativeEpoch: "generic-gate",
      nativeRevision: 1,
      nativeTrackGeneration: 1,
      durationMs: 0,
      capabilities: { ...media.capabilities, seek: true },
    });
    fixture.expand();
    fixture.hoverIsland();
  }, track);
  await page.waitForTimeout(1_000);
  await expect(page.locator('.expanded-layer input[type="range"]')).toHaveCount(0);

  await page.evaluate((media) => (window as any).media({
    ...media,
    source: "generic",
    sourceDisplay: "Fixture",
    nativeEpoch: "generic-gate",
    nativeRevision: 2,
    nativeTrackGeneration: 1,
    capabilities: { ...media.capabilities, seek: false },
  }), track);
  await page.waitForTimeout(550);
  await expect(page.locator('.expanded-layer input[type="range"]')).toHaveCount(0);

  await page.evaluate((media) => (window as any).media({
    ...media,
    source: "generic",
    sourceDisplay: "Fixture",
    nativeEpoch: "generic-gate",
    nativeRevision: 3,
    nativeTrackGeneration: 1,
    capabilities: { ...media.capabilities, seek: true },
  }), track);
  await page.waitForTimeout(550);
  await expect(page.locator('.expanded-layer input[type="range"]')).toHaveCount(1);
  await page.evaluate(() => (window as any).destroy());
});

test("the full App cycles the verified NetEase mode and shows failures without optimistic mode changes", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html", { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture = window as any;
    fixture.settingsOverrides = { language: "zh-CN" };
    fixture.mockMode = "repeat_list";
    fixture.modeCycles = [];
    fixture.rejectModeCycle = false;
    const invoke = window.__TAURI_INTERNALS__!.invoke;
    window.__TAURI_INTERNALS__!.invoke = async (command: string, args: unknown, options: unknown) => {
      if (command === "get_netease_playback_mode") return fixture.mockMode;
      if (command === "cycle_netease_playback_mode") {
        fixture.modeCycles.push({ command, args });
        if (fixture.rejectModeCycle) throw new Error("fixture mode verification failed");
        fixture.mockMode = fixture.nextMockMode;
        return fixture.mockMode;
      }
      return invoke(command, args, options);
    };
    fixture.mountApp();
  });
  await expect.poll(() => page.evaluate(() => (window as any).calls.includes("show_main_window"))).toBe(true);
  await expect.poll(() => page.evaluate(() => (window as any).mediaListenerCount())).toBe(1);
  await page.evaluate(async (media) => {
    const fixture = window as any;
    await fixture.media(media);
    fixture.expand();
    fixture.hoverIsland();
  }, track);
  await page.waitForTimeout(1_000);
  const listMode = page.getByRole("button", { name: "切换网易云播放模式（当前：列表循环）" });
  await expect(listMode).toBeVisible();
  await page.evaluate(() => { (window as any).nextMockMode = "repeat_one"; });
  await listMode.click();
  const repeatOne = page.getByRole("button", { name: "切换网易云播放模式（当前：单曲循环）" });
  await expect(repeatOne).toBeVisible();
  expect(await page.evaluate(() => (window as any).modeCycles.length)).toBe(1);

  await page.evaluate(() => { (window as any).rejectModeCycle = true; });
  await repeatOne.click();
  await expect(page.getByRole("alert")).toContainText("fixture mode verification failed");
  await expect(page.getByRole("button", { name: "切换网易云播放模式（当前：单曲循环）" })).toBeVisible();
  expect(await page.evaluate(() => (window as any).modeCycles.length)).toBe(2);
  await page.evaluate(() => (window as any).destroy());
});

test("the full App disables repeat clicks while a mode read-back is pending and refreshes after track change", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html", { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture = window as any;
    fixture.settingsOverrides = { language: "zh-CN" };
    fixture.mockMode = "sequential";
    fixture.nextMockMode = "repeat_list";
    fixture.modeCycles = [];
    fixture.pendingModeResolve = undefined;
    const invoke = window.__TAURI_INTERNALS__!.invoke;
    window.__TAURI_INTERNALS__!.invoke = async (command: string, args: unknown, options: unknown) => {
      if (command === "get_netease_playback_mode") return fixture.mockMode;
      if (command === "cycle_netease_playback_mode") {
        fixture.modeCycles.push({ command, args });
        return new Promise<string>((resolve) => { fixture.pendingModeResolve = resolve; });
      }
      return invoke(command, args, options);
    };
    fixture.mountApp();
  });
  await expect.poll(() => page.evaluate(() => (window as any).calls.includes("show_main_window"))).toBe(true);
  await expect.poll(() => page.evaluate(() => (window as any).mediaListenerCount())).toBe(1);
  await page.evaluate(async (media) => {
    const fixture = window as any;
    fixture.settingsOverrides = { language: "zh-CN" };
    await fixture.media(media);
    fixture.expand();
    fixture.hoverIsland();
  }, track);
  await page.waitForTimeout(1_000);

  const sequential = page.getByRole("button", { name: "切换网易云播放模式（当前：顺序播放）" });
  await expect(sequential).toBeVisible();
  await sequential.click();
  await expect(sequential).toBeDisabled();
  await sequential.click({ force: true });
  expect(await page.evaluate(() => (window as any).modeCycles.length)).toBe(1);
  await page.evaluate(() => (window as any).pendingModeResolve("repeat_list"));
  await expect(page.getByRole("button", { name: "切换网易云播放模式（当前：列表循环）" })).toBeEnabled();

  await page.evaluate(async (media) => {
    const fixture = window as any;
    fixture.settingsOverrides = { language: "zh-CN" };
    fixture.mockMode = "shuffle";
    await fixture.media({ ...media, title: "Next fixture track", nativeRevision: 2, nativeTrackGeneration: 2 });
  }, track);
  await expect(page.getByRole("button", { name: "切换网易云播放模式（当前：随机播放）" })).toBeVisible();
  await page.evaluate(() => (window as any).destroy());
});

test("the full App removes 15-second buttons and keeps the slider when progress is available", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html", { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture = window as any;
    fixture.settingsOverrides = { language: "zh-CN", neteaseProgressMode: "estimate" };
    fixture.seekCalls = [];
    const invoke = window.__TAURI_INTERNALS__!.invoke;
    window.__TAURI_INTERNALS__!.invoke = async (command: string, args: any, options: unknown) => {
      if (command === "get_netease_playback_mode") return "repeat_list";
      return invoke(command, args, options);
    };
    fixture.mountApp();
  });
  await expect.poll(() => page.evaluate(() => (window as any).calls.includes("show_main_window"))).toBe(true);
  await expect.poll(() => page.evaluate(() => (window as any).mediaListenerCount())).toBe(1);
  await page.evaluate(async (media) => {
    const fixture = window as any;
    await fixture.media({ ...media, isPlaying: false });
    fixture.expand();
    fixture.hoverIsland();
  }, track);
  await page.waitForTimeout(1_000);

  await expect(page.getByRole("button", { name: /后退 15 秒|前进 15 秒|Back 15 seconds|Forward 15 seconds/ })).toHaveCount(0);
  const range = page.locator('.expanded-layer input[type="range"]');
  await expect(range).toHaveCount(1);
  await expect(page.getByRole("button", { name: "切换网易云播放模式（当前：列表循环）" })).toBeVisible();
  await page.evaluate(() => (window as any).destroy());
});

test("the full App shows unknown time and preserves its warning instead of displaying a valid zero timeline", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html", { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture = window as any;
    fixture.settingsOverrides = { language: "zh-CN", neteaseProgressMode: "estimate" };
    fixture.mountApp();
  });
  await expect.poll(() => page.evaluate(() => (window as any).calls.includes("show_main_window"))).toBe(true);
  await expect.poll(() => page.evaluate(() => (window as any).mediaListenerCount())).toBe(1);
  await page.evaluate(async (media) => {
    const fixture = window as any;
    await fixture.media({ ...media, isPlaying: false, durationMs: 0, positionMs: 0, progressKnown: false });
    fixture.expand();
    fixture.hoverIsland();
  }, track);
  await page.waitForTimeout(1_000);

  await expect(page.getByRole("button", { name: /后退 15 秒|前进 15 秒|Back 15 seconds|Forward 15 seconds/ })).toHaveCount(0);
  await expect(page.locator('.expanded-layer input[type="range"]')).toHaveCount(0);
  await expect(page.locator(".expanded-layer .progress-block.with-times .time.elapsed .rolling-number-sr")).toHaveText("--:--");
  await expect(page.locator(".expanded-layer [role=status]")).toContainText("网易云当前曲目的进度锚点尚未确认。");
  await page.evaluate(() => (window as any).destroy());
});

test("Studio persists both NetEase progress modes and reflects the settings broadcast", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html", { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture = window as any;
    fixture.settingsOverrides = { language: "zh-CN", neteaseProgressMode: "real" };
    fixture.settingsUpdates = [];
    const invoke = window.__TAURI_INTERNALS__!.invoke;
    window.__TAURI_INTERNALS__!.invoke = async (command: string, args: any, options: unknown) => {
      if (command === "update_settings") {
        fixture.settingsUpdates.push(args.patch);
        fixture.settingsOverrides = { ...fixture.settingsOverrides, ...args.patch };
        await fixture.settingsUpdated({ ...fixture.settingsOverrides });
        return;
      }
      return invoke(command, args, options);
    };
    fixture.mountStudio();
  });

  const real = page.getByRole("button", { name: /真实进度/ });
  const estimate = page.getByRole("button", { name: /计时估算/ });
  await expect(real).toBeVisible();
  await expect(real).toHaveAttribute("aria-pressed", "true");
  await estimate.click();
  await expect(estimate).toHaveAttribute("aria-pressed", "true");
  await expect.poll(() => page.evaluate(() => (window as any).settingsUpdates.length)).toBe(1);
  expect(await page.evaluate(() => (window as any).settingsUpdates[0].neteaseProgressMode)).toBe("estimate");

  await real.click();
  await expect(real).toHaveAttribute("aria-pressed", "true");
  await expect.poll(() => page.evaluate(() => (window as any).settingsUpdates.length)).toBe(2);
  expect(await page.evaluate(() => (window as any).settingsUpdates[1].neteaseProgressMode)).toBe("real");
  await page.evaluate(() => (window as any).destroy());
});
