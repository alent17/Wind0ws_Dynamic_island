import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";

async function dragExpandedIslandSlider(page: Page) {
  const slider = page.getByRole("slider");
  const box = await slider.boundingBox();
  expect(box).not.toBeNull();
  const y = box!.y + box!.height / 2;
  await page.mouse.move(box!.x + box!.width * 0.2, y);
  await page.mouse.down();
  await page.mouse.move(box!.x + box!.width * 0.68, y, { steps: 6 });
  const valueBeforeRelease = Number(await slider.inputValue());
  await page.mouse.up();
  return valueBeforeRelease;
}

test.beforeEach(async ({ page }) => {
  await page.goto("/ui-tests/seek-main-island.html");
  const slider = page.getByRole("slider");
  await expect(slider).toBeVisible();
  await expect(page.locator(".expanded-layer")).toHaveAttribute("aria-hidden", "false");
});

test("main-island has no 15-second controls and keeps playback controls within the row when scaled", async ({ page }) => {
  const row = page.locator(".transport-controls");
  const playPause = row.locator(".play");

  await expect(row.locator(".seek-step")).toHaveCount(0);
  await expect(playPause).toBeVisible();
  await expect(page.locator(".progress-block .seek-step")).toHaveCount(0);

  const pauseBox = await playPause.boundingBox();
  expect(pauseBox).not.toBeNull();

  await page.setViewportSize({ width: 560, height: 420 });
  await page.evaluate(() => document.querySelector(".expanded-shell")?.style.setProperty("--content-scale", "0.75"));
  const fit = await row.evaluate((element) => {
    const rowRect = element.getBoundingClientRect();
    return [...element.querySelectorAll("button")].every((button) => {
      const rect = button.getBoundingClientRect();
      return rect.left >= rowRect.left - 1 && rect.right <= rowRect.right + 1;
    });
  });
  expect(fit).toBe(true);
});

test("expanded main-island drag submits one seek_media call on release", async ({ page }) => {
  const valueBeforeRelease = await dragExpandedIslandSlider(page);

  await expect.poll(() => page.evaluate(() => window.__seekMediaInvokes)).toEqual([
    { command: "seek_media", positionMs: valueBeforeRelease },
  ]);
  expect(valueBeforeRelease).toBeGreaterThan(12_000);
});

test("expanded main-island keyboard range adjustment submits seek_media", async ({ page }) => {
  const slider = page.getByRole("slider");
  await slider.focus();
  await slider.press("ArrowRight");
  await expect.poll(() => page.evaluate(() => window.__seekMediaInvokes)).toHaveLength(1);
  const requestedPosition = await page.evaluate(() => window.__seekMediaInvokes[0]?.positionMs);
  expect(requestedPosition).toBe(13_000);
  await expect(page.evaluate(() => window.__seekMediaInvokes)).resolves.toEqual([
    { command: "seek_media", positionMs: requestedPosition },
  ]);
});

test("App seek handler reports a late 4s failure after four 1s native samples without rolling back newer UI", async ({ page }) => {
  await dragExpandedIslandSlider(page);
  await expect.poll(() => page.evaluate(() => window.__seekMediaInvokes)).toHaveLength(1);

  for (let second = 1; second <= 4; second++) {
    await page.waitForTimeout(1_000);
    await page.evaluate((sample) => window.__pushNativeSnapshot(sample), 20_000 + second * 1_000);
  }
  await page.evaluate(() => window.__setDisplayedPosition(40_000));
  await page.evaluate(() => window.__rejectSeek({ code: 4204, message: "CDP read-back timed out" }));

  await expect(page.getByRole("alert")).toHaveText("[4204] CDP read-back timed out");
  await expect.poll(() => page.evaluate(() => window.__seekState())).toMatchObject({
    positionMs: 40_000,
    nativeRevision: 4,
    error: "[4204] CDP read-back timed out",
  });
  await dragExpandedIslandSlider(page);
  await expect(page.getByRole("alert")).toHaveCount(0);
});

test("App seek handler applies CDP confirmation after four 1s native samples", async ({ page }) => {
  await dragExpandedIslandSlider(page);
  await expect.poll(() => page.evaluate(() => window.__seekMediaInvokes)).toHaveLength(1);

  for (let second = 1; second <= 4; second++) {
    await page.waitForTimeout(1_000);
    await page.evaluate((sample) => window.__pushNativeSnapshot(sample), 20_000 + second * 1_000);
  }
  await page.evaluate(() => window.__resolveSeek({ trackId: "fixture-track", positionMs: 76_000, durationMs: 180_000 }));

  await expect.poll(() => page.evaluate(() => window.__seekState().positionMs)).toBe(76_000);
  await expect(page.getByRole("alert")).toHaveCount(0);
});
