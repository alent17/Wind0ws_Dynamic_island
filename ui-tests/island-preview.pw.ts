import { expect, test } from "@playwright/test";

test("Studio dropdowns share a dark menu and keep mouse and keyboard selection", async ({ page }) => {
  await page.goto("/studio.html");
  const preferences = page.locator(".preferences-section");
  const zone = preferences.locator(".studio-select").nth(1);
  const trigger = zone.locator(".select-trigger");

  await trigger.click();
  const menu = zone.getByRole("listbox");
  await expect(menu).toBeVisible();
  await expect(menu).toHaveCSS("background-color", "rgb(23, 25, 29)");
  await menu.getByRole("option", { name: "Asia/Tokyo" }).click();
  await expect(trigger).toContainText("Asia/Tokyo");
  await expect(menu).toBeHidden();

  await trigger.press("ArrowDown");
  await expect(menu.getByRole("option", { name: "Asia/Tokyo" })).toBeFocused();
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Enter");
  await expect(trigger).toContainText("America/New_York");
  await trigger.click();
  await page.locator(".preferences-section h2").click();
  await expect(menu).toBeHidden();

  await expect(page.locator(".select-row .select-trigger")).toBeDisabled();
});

test("circular artwork disables MV loop without changing the regular mode toggle", async ({ page }) => {
  await page.goto("/studio.html");
  await expect(page.locator(".stage .island-surface")).toBeVisible();
  const features = page.locator(".media-features-section");
  const circular = features.locator(".setting-choice").nth(1);
  const mvLoop = features.locator(".setting-choice").nth(2);

  await expect(circular).toHaveAttribute("aria-pressed", "false");
  await expect(mvLoop).toHaveAttribute("aria-pressed", "true");

  await circular.click();
  await expect(circular).toHaveAttribute("aria-pressed", "true");
  await expect(mvLoop).toBeDisabled();
  await expect(mvLoop).toHaveAttribute("aria-pressed", "false");

  await circular.click();
  await expect(mvLoop).toBeEnabled();
  await expect(mvLoop).toHaveAttribute("aria-pressed", "false");
  await mvLoop.click();
  await expect(mvLoop).toHaveAttribute("aria-pressed", "true");
});

test("feature buttons reveal their background only while hovered", async ({ page }) => {
  await page.goto("/ui-tests/island-fixture.html");
  const timerButton = page.locator(".function-icon").first();
  const background = () => timerButton.evaluate((element) => getComputedStyle(element).backgroundColor);

  await expect(timerButton).toBeVisible();
  await expect.poll(background).toBe("rgba(0, 0, 0, 0)");
  await timerButton.hover();
  await expect.poll(background).toBe("rgba(255, 255, 255, 0.09)");

  await timerButton.click();
  await page.mouse.move(0, 0);
  await expect(timerButton).toHaveClass(/active/);
  await expect.poll(background).toBe("rgba(0, 0, 0, 0)");

  await timerButton.hover();
  await expect.poll(background).toBe("rgba(255, 255, 255, 0.09)");
});

test("music controls reveal the same background only while hovered", async ({ page }) => {
  await page.goto("/ui-tests/island-fixture.html");
  const controls = page.locator(".control-row:not(.idle-controls) .controls");
  const buttons = controls.locator("button:not(:disabled)");
  const buttonBackground = (button: import("@playwright/test").Locator) =>
    button.evaluate((element) => getComputedStyle(element).backgroundColor);

  await expect(buttons).toHaveCount(3);
  for (const button of await buttons.all()) {
    await expect.poll(() => buttonBackground(button)).toBe("rgba(0, 0, 0, 0)");
    await expect.poll(() => button.evaluate((element) => {
      const { width, height } = element.getBoundingClientRect();
      return Math.abs(width - 48) < 0.5 && Math.abs(height - 48) < 0.5
        && getComputedStyle(element).borderRadius === "12px";
    })).toBe(true);
    await button.hover();
    await expect.poll(() => buttonBackground(button)).toBe("rgba(255, 255, 255, 0.1)");
  }

  const play = controls.locator(".play");
  await expect(play.locator(".play-pause-icon")).toHaveClass(/playing/);
  await play.click();
  await expect(play.locator(".play-pause-icon")).not.toHaveClass(/playing/);
  await expect.poll(() => buttonBackground(play)).toBe("rgba(255, 255, 255, 0.1)");
  await page.mouse.move(0, 0);
  await expect.poll(() => buttonBackground(play)).toBe("rgba(0, 0, 0, 0)");
});

test("floating player controls use the same hover-only button style", async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 320 });
  await page.addInitScript(() => {
    let nextCallbackId = 1;
    const callbacks = new Map<number, (payload: unknown) => void>();
    const settings = {
      language: "zh-CN",
      fontId: "misans",
      floatingFillColor: "#08090c",
      floatingUseAlbumColor: false,
      enableMvPlayback: false,
      floatingWindowAlwaysOnTop: false,
      lockFloatingWindow: false,
      enableHdCover: false,
      floatingCircularAlbum: false,
      enablePixelArt: false,
      enableHalftone: false,
      captureHideOnScreenshot: false,
      captureHideOnRecording: false,
      captureHideOnFullscreen: false,
      captureHideOnScreenShare: false,
    };
    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {
        metadata: { currentWindow: { label: "floating" } },
        invoke: async (command: string) => {
          if (command === "get_settings") return settings;
          if (command === "get_media_info_cmd") return {
            title: "Test Track",
            artist: "Test Artist",
            albumArt: "",
            isPlaying: true,
            positionMs: 12_000,
            durationMs: 180_000,
            lastUpdatedTimestamp: Date.now(),
            source: "test",
            sourceDisplay: "Test",
          };
          if (command === "plugin:event|listen") return nextCallbackId++;
          return null;
        },
        transformCallback: (callback: (payload: unknown) => void) => {
          const id = nextCallbackId++;
          callbacks.set(id, callback);
          return id;
        },
        unregisterCallback: (id: number) => callbacks.delete(id),
        convertFileSrc: (path: string) => path,
      },
    });
    Object.defineProperty(window, "__TAURI_EVENT_PLUGIN_INTERNALS__", {
      configurable: true,
      value: { unregisterListener: () => undefined },
    });
  });

  await page.goto("/?window=floating");
  const player = page.locator(".player");
  await expect(player).toBeVisible();
  await player.hover();
  const controls = page.locator(".controls-overlay.visible");
  const buttons = controls.locator(".ctrl-btn, .play-btn");
  await expect(buttons).toHaveCount(3);

  for (const button of await buttons.all()) {
    await expect.poll(() => button.evaluate((element) => {
      const { width, height } = element.getBoundingClientRect();
      return Math.abs(width - 48) < 0.5 && Math.abs(height - 48) < 0.5
        && getComputedStyle(element).borderRadius === "12px"
        && getComputedStyle(element).backgroundColor === "rgba(0, 0, 0, 0)";
    })).toBe(true);
    await button.hover();
    await expect.poll(() => button.evaluate((element) => getComputedStyle(element).backgroundColor))
      .toBe("rgba(255, 255, 255, 0.1)");
  }

  await page.setViewportSize({ width: 180, height: 180 });
  await page.reload();
  const compactPlayer = page.locator(".player.compact-cover");
  await expect(compactPlayer).toBeVisible();
  await compactPlayer.hover();
  const compactPlay = page.locator(".compact-play");
  await expect(compactPlay).toBeVisible();
  await page.mouse.move(10, 10);
  await expect.poll(() => compactPlay.evaluate((element) => {
    const { width, height } = element.getBoundingClientRect();
    return Math.abs(width - 48) < 0.5 && Math.abs(height - 48) < 0.5
      && getComputedStyle(element).borderRadius === "12px"
      && getComputedStyle(element).backgroundColor === "rgba(0, 0, 0, 0)";
  })).toBe(true);
  await compactPlay.hover();
  await expect.poll(() => compactPlay.evaluate((element) => getComputedStyle(element).backgroundColor))
    .toBe("rgba(255, 255, 255, 0.1)");
});

test("timer pause and cancel controls stay clear until hovered and keep their actions", async ({ page }) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.getByRole("button", { name: "Timer", exact: true }).click();

  const main = page.locator(".timer-main-button");
  await main.click();
  await expect(page.locator(".timer-actions")).toHaveClass(/timer-active/);
  await expect.poll(() => main.evaluate((element) => {
    const { width, height } = element.getBoundingClientRect();
    return Math.abs(width - 48) < 0.5 && Math.abs(height - 48) < 0.5
      && getComputedStyle(element).borderRadius === "12px";
  })).toBe(true);
  const mainBackground = () => main.evaluate((element) => getComputedStyle(element).backgroundColor);
  await page.mouse.move(0, 0);
  await expect.poll(mainBackground).toBe("rgba(0, 0, 0, 0)");
  await main.hover();
  await expect.poll(mainBackground).toBe("rgba(255, 255, 255, 0.1)");

  await main.click();
  await expect(main.locator(".play-pause-icon")).not.toHaveClass(/playing/);
  await main.click();
  await expect(main.locator(".play-pause-icon")).toHaveClass(/playing/);

  const reset = page.locator(".timer-reset");
  const resetBackground = () => reset.evaluate((element) => getComputedStyle(element).backgroundColor);
  await expect.poll(() => reset.evaluate((element) => {
    const { width, height } = element.getBoundingClientRect();
    return Math.abs(width - 48) < 0.5 && Math.abs(height - 48) < 0.5
      && getComputedStyle(element).borderRadius === "12px";
  })).toBe(true);
  await page.mouse.move(0, 0);
  await expect.poll(resetBackground).toBe("rgba(0, 0, 0, 0)");
  await reset.hover();
  await expect.poll(resetBackground).toBe("rgba(255, 255, 255, 0.1)");
  await reset.click();
  await expect(page.locator(".timer-actions")).not.toHaveClass(/timer-active/);
});

test("expanded timer readout fits a long 99-hour countdown", async ({ page }, testInfo) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.getByTestId("timer-long").click();
  await page.getByRole("button", { name: "Timer", exact: true }).click();

  const readout = page.locator(".timer-readout");
  const digits = readout.locator(".rolling-number-sr");
  await expect(digits).toHaveText("99:59:00");
  await expect.poll(() => readout.locator("strong").evaluate((element) =>
    element.scrollWidth <= element.clientWidth
  )).toBe(true);
  await expect.poll(() => readout.locator("strong").evaluate((element) =>
    Number.parseFloat(getComputedStyle(element).fontSize) <= 24
  )).toBe(true);
  await expect.poll(() => readout.locator("strong").evaluate((element) =>
    Number.parseFloat(getComputedStyle(element).paddingRight) >= 6
  )).toBe(true);
  await expect.poll(() => readout.locator("strong").evaluate((element) => {
    const safeRight = element.getBoundingClientRect().right - Number.parseFloat(getComputedStyle(element).paddingRight);
    const visualRight = element.querySelector(".rolling-number-visual")!.getBoundingClientRect().right;
    return safeRight - visualRight >= -0.5;
  })).toBe(true);
  await page.screenshot({ path: testInfo.outputPath("timer-long-readout.png"), fullPage: true });
});

test("collapsed countdown keeps a constant progress-line thickness", async ({ page }, testInfo) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.getByTestId("timer-half").click();

  const compactTimer = page.locator(".compact-timer");
  const time = compactTimer.locator(".timer-time");
  await expect(time).toContainText("00:30");
  await expect(compactTimer.locator(".timer-label")).toHaveCount(0);
  const timePosition = await time.evaluate((element) => {
    const summary = element.parentElement!.getBoundingClientRect();
    const bounds = element.getBoundingClientRect();
    return { rightGap: summary.right - bounds.right, isRightAligned: bounds.x > summary.x + summary.width / 2 };
  });
  expect(timePosition.isRightAligned).toBe(true);
  expect(timePosition.rightGap).toBeLessThanOrEqual(1);

  const dimensions = await page.locator(".timer-progress-fill").evaluate((fill) => {
    const track = fill.parentElement!;
    const fillBounds = fill.getBoundingClientRect();
    const trackBounds = track.getBoundingClientRect();
    return {
      widthRatio: fillBounds.width / trackBounds.width,
      fillHeight: fillBounds.height,
      trackHeight: trackBounds.height,
    };
  });
  expect(dimensions.widthRatio).toBeCloseTo(0.5, 1);
  expect(dimensions.fillHeight).toBe(dimensions.trackHeight);
  await page.screenshot({ path: testInfo.outputPath("collapsed-countdown.png"), fullPage: true });
});

test("timer completion opens the island and presents a dismissible notice", async ({ page }, testInfo) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.getByTestId("timer-complete").click();

  const notice = page.getByRole("status");
  await expect(notice).toBeVisible();
  await expect(notice.locator(".timer-complete-copy strong")).not.toBeEmpty();
  const hint = notice.locator(".timer-complete-copy span");
  await expect.poll(() => hint.evaluate((element) => getComputedStyle(element).getPropertyValue("-webkit-line-clamp"))).toBe("2");
  expect(await hint.evaluate((element) => element.scrollHeight <= element.clientHeight)).toBe(true);
  await page.screenshot({ path: testInfo.outputPath("timer-complete-notice.png"), fullPage: true });
  await notice.getByRole("button").click();
  await expect(notice).toBeHidden();
});

async function surfaceMetrics(page: import("@playwright/test").Page) {
  const surface = page.locator(".island-surface");
  return surface.evaluate((element) => {
    const style = getComputedStyle(element);
    const points = [...style.clipPath.matchAll(/(-?[\d.]+)px\s+(-?[\d.]+)px/g)]
      .map((match) => ({ x: Number(match[1]), y: Number(match[2]) }));
    const toolbar = element.querySelector(".function-toolbar")!;
    const lastTool = toolbar.querySelector(".function-icon:last-child")!;
    const bounds = lastTool.getBoundingClientRect();
    const coverBounds = element.querySelector(".expanded-cover")!.getBoundingClientRect();
    const surfaceBounds = element.getBoundingClientRect();
    const inset = 8;
    const hitCorners = [
      [bounds.left + inset, bounds.top + inset],
      [bounds.right - inset, bounds.top + inset],
      [bounds.left + inset, bounds.bottom - inset],
      [bounds.right - inset, bounds.bottom - inset],
    ].map(([x, y]) => document.elementFromPoint(x, y)?.closest(".function-icon") === lastTool);
    return {
      width: Number.parseFloat(style.width),
      height: Number.parseFloat(style.height),
      borderRadius: style.borderRadius,
      clipPath: element.style.clipPath,
      points,
      hitCorners,
      leftContentInset: coverBounds.left - surfaceBounds.left,
      rightContentInset: surfaceBounds.right - bounds.right,
    };
  });
}

test("expanded and collapsed layouts keep a fixed width and configured shoulder", async ({ page }, testInfo) => {
  await page.goto("/ui-tests/island-fixture.html");
  const surface = page.locator(".island-surface");
  const musicPane = page.locator(".music-pane");

  await expect(surface).toBeVisible();
  await expect.poll(() => musicPane.evaluate((element) => getComputedStyle(element).width)).toBe("300px");
  await expect.poll(() => surface.evaluate((element) => getComputedStyle(element).width)).toBe("600px");
  const shortSurfaceBounds = await surface.boundingBox();
  const shortCoverBounds = await page.locator(".expanded-cover").boundingBox();
  expect(shortSurfaceBounds).not.toBeNull();
  expect(shortCoverBounds).not.toBeNull();
  expect(shortCoverBounds!.x - shortSurfaceBounds!.x).toBeGreaterThanOrEqual(32 + 27);
  const spacing = await page.locator(".expanded-shell").evaluate((shell) => {
    const bounds = (selector: string) => shell.querySelector(selector)!.getBoundingClientRect();
    const progress = bounds(".progress-block");
    const controls = bounds(".controls");
    const digits = bounds(".clock-face .digits");
    const forecast = bounds(".forecast-strip");
    return {
      controlsCenterOffset: Math.abs((controls.left + controls.right) / 2 - (progress.left + progress.right) / 2),
      clockForecastGap: forecast.left - digits.right,
    };
  });
  expect(spacing.controlsCenterOffset).toBeLessThanOrEqual(1);
  expect(spacing.clockForecastGap).toBeGreaterThanOrEqual(8);
  await page.screenshot({ path: testInfo.outputPath("fixed-expanded-short-title.png"), fullPage: true });

  await page.getByTestId("long-track").click();
  await expect.poll(() => musicPane.evaluate((element) => getComputedStyle(element).width)).toBe("300px");
  await expect.poll(() => surface.evaluate((element) => getComputedStyle(element).width)).toBe("600px");
  for (const text of await page.locator(".metadata strong, .metadata span").all()) {
    await expect.poll(() => text.evaluate((element) =>
      getComputedStyle(element).textOverflow === "ellipsis"
      && element.getBoundingClientRect().right <= element.parentElement!.getBoundingClientRect().right
    )).toBe(true);
  }

  const expanded = await surfaceMetrics(page);
  expect(expanded.height).toBe(160);
  expect(Number.parseFloat(expanded.borderRadius.split(" ")[2])).toBeCloseTo(45);
  expect(expanded.points[8].x).toBe(32);
  expect(expanded.points[8].y).toBe(32);
  expect(Math.abs(expanded.leftContentInset - expanded.rightContentInset)).toBeLessThanOrEqual(8);
  expect(expanded.hitCorners).toEqual([true, true, true, true]);
  const surfaceBounds = await surface.boundingBox();
  const coverBounds = await page.locator(".expanded-cover").boundingBox();
  expect(surfaceBounds).not.toBeNull();
  expect(coverBounds).not.toBeNull();
  expect(coverBounds!.x - surfaceBounds!.x).toBeGreaterThanOrEqual(expanded.points[8].x + 27);
  await page.screenshot({ path: testInfo.outputPath("fixed-expanded-long-title.png"), fullPage: true });

  await page.getByTestId("compact-mode").click();
  await expect.poll(() => surface.evaluate((element) => getComputedStyle(element).width)).toBe("80px");
  const collapsed = await surfaceMetrics(page);
  expect(collapsed.height).toBe(28);
  expect(collapsed.points[8].x).toBeCloseTo(8);
  expect(collapsed.points[8].y).toBeCloseTo(8);

  await page.getByTestId("expanded-mode").click();
  await expect.poll(() => musicPane.evaluate((element) => getComputedStyle(element).width)).toBe("300px");
});

test("Studio preview renders the attached concave shoulder and manual radius controls", async ({ page }, testInfo) => {
  await page.goto("/studio.html");
  const surface = page.locator(".stage .island-surface");
  await expect(surface).toBeVisible();

  await page.locator('[data-preview-target="shape"] .segmented button').nth(1).click();
  await expect.poll(() => surface.evaluate((element) => element.style.clipPath)).toContain("polygon(");
  const surfaceBounds = await surface.boundingBox();
  const toplineBounds = await page.locator(".stage-topline").boundingBox();
  expect(surfaceBounds).not.toBeNull();
  expect(toplineBounds).not.toBeNull();
  expect(toplineBounds!.y).toBeGreaterThan(surfaceBounds!.y + surfaceBounds!.height);
  const points = await surface.evaluate((element) => [...getComputedStyle(element).clipPath.matchAll(/(-?[\d.]+)px\s+(-?[\d.]+)px/g)]
    .map((match) => ({ x: Number(match[1]), y: Number(match[2]) })));
  expect(points[8].x).toBe(32);
  expect(points[8].y).toBe(32);
  await page.screenshot({ path: testInfo.outputPath("studio-attached-shoulder.png"), fullPage: true });
});
