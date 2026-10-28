import { expect, test } from "@playwright/test";

for (const surface of ["island", "floating"]) test(`Apple Music ${surface} keeps basic SMTC playback without enhanced controls`, async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(surface => {
    const fixture = window as any;
    fixture.settingsOverrides = { language: "zh-CN" };
    if (surface === "floating") fixture.mountFloating({ language: "zh-CN", enableMvPlayback: false });
    else { fixture.mountApp(); fixture.expand(); }
  }, surface);
  await page.evaluate(() => (window as any).media({
    title: "Apple fixture", artist: "Artist", source: "apple", sourceDisplay: "AppleInc.AppleMusicWin_nzyj5cx40ttqa!App",
    isPlaying: false, positionMs: 12_000, durationMs: 180_000, progressKnown: true,
    nativeEpoch: "apple-fixture", nativeRevision: 1, nativeTrackGeneration: 1,
    capabilities: { previous: true, playPause: true, next: true, seek: false, shuffle: false, repeat: false },
    // Old adapter payloads must not restore removed controls or trigger probes.
    appleMusic: { seek: true, shuffle: false, repeat: "none", liked: true },
  }));
  await expect(page.getByText("Apple fixture", { exact: true }).first()).toBeVisible();
  await expect(page.locator(".apple-like, .apple-mode, .smtc-modes button")).toHaveCount(0);
  await expect(page.locator('.progress-block input[type="range"]')).toHaveCount(0);
  if (surface === "floating") await page.locator(".player").hover();
  for (const label of ["播放", "上一首", "下一首"]) {
    const button = page.getByRole("button", { name: label, exact: true });
    await expect(button).toBeEnabled();
    await button.click({ force: surface === "floating" });
  }
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((call: string) => call === "control_media").length)).toBe(3);
  expect(await page.evaluate(() => (window as any).calls.filter((call: string) => /apple_music|netease_like|seek_media/.test(call)))).toEqual([]);
});
