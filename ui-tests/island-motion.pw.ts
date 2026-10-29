import { test, expect } from "@playwright/test";

test.use({ reducedMotion: "no-preference" });

for (const widgets of [false, true]) {
  test(`style transitions animate content in both directions ${widgets ? "with multiple widgets" : "without widgets"}`, async ({ page }) => {
    await page.goto("/ui-tests/island-fixture.html");
    await page.evaluate((widgets) => (window as any).configureIsland({
      mode: "expanded", edge: "top", style: "floating",
      widgets: widgets ? { left: ["timer"], right: ["clock"] } : { left: [], right: [] },
    }), widgets);
    await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true");
    for (const style of ["edge", "floating"]) {
      const samples = await page.evaluate(async (style) => {
        const shell = document.querySelector<HTMLElement>(".expanded-shell")!;
        const inset = () => parseFloat(shell.style.getPropertyValue("--shoulder-content-inset"));
        const values = [inset()];
        (window as any).configureIsland({ style });
        for (let frame = 0; frame < 35; frame++) {
          await new Promise(requestAnimationFrame);
          values.push(inset());
        }
        return values;
      }, style);
      const start = samples[0], end = samples.at(-1)!;
      expect(Math.abs(end - start)).toBeGreaterThan(5);
      expect(samples.some(value => value > Math.min(start, end) + 1 && value < Math.max(start, end) - 1)).toBe(true);
      await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true");
    }
  });
}

for (const edge of ["top", "right", "bottom", "left"]) {
  test(`spectrum moves with artwork on expansion and collapse at ${edge}`, async ({ page }) => {
    await page.goto("/ui-tests/island-fixture.html");
    await page.evaluate(edge => (window as any).configureIsland({ edge, animate: true, scale: .8, toolbar: true }), edge);
    await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true");
    for (const mode of ["expanded", "compact"]) {
      const sample = await page.evaluate(async mode => {
        (window as any).configureIsland({ mode });
        for (let frame = 0; frame < 24; frame++) {
          await new Promise(requestAnimationFrame);
          const moving = document.querySelector(".moving-spectrum.in-motion");
          if (!moving) continue;
          const rect = moving.getBoundingClientRect();
          const endpoints = [...document.querySelectorAll(".spectrum-endpoint")];
          return { width: rect.width, height: rect.height, coverMoving: !!document.querySelector(".moving-cover.in-motion"), hidden: endpoints.every(el => getComputedStyle(el).visibility === "hidden") };
        }
        return null;
      }, mode);
      expect(sample).not.toBeNull();
      expect(sample!.width).toBeGreaterThan(0);
      expect(sample!.height).toBeGreaterThan(0);
      expect(sample!.coverMoving).toBe(true);
      expect(sample!.hidden).toBe(true);
      await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true");
      await expect(page.locator(".moving-spectrum.in-motion")).toHaveCount(0);
      await expect(page.locator(`${mode === "expanded" ? ".top-row" : ".compact-layer"} .spectrum-endpoint`)).toBeVisible();
    }
  });
}

for (const remembered of [false, true]) test(`long ${remembered ? "remembered" : "current"} title animates only while the music panel is visible`, async ({ page }) => {
  await page.goto("/ui-tests/island-fixture.html");
  const longTitle = "A deliberately long fixture title that must scroll across the visible music panel";
  await page.evaluate(({remembered,longTitle}) => (window as any).configureIsland({
    idle:remembered,
    title:longTitle,
    rememberedTitle:remembered ? longTitle : "",
  }), {remembered,longTitle});
  const marquee = page.locator(".music-pane .marquee-title span");
  await expect(marquee).toHaveCount(1);
  await expect(marquee).toHaveText(longTitle);
  await expect.poll(() => marquee.evaluate(element => element.getAnimations().some(animation => animation.playState === "running"))).toBe(false);
  await page.evaluate(() => (window as any).configureIsland({mode:"expanded"}));
  await expect.poll(() => marquee.evaluate(element => element.getAnimations().some(animation => animation.playState === "running"))).toBe(true);

  await page.evaluate(() => (window as any).configureIsland({mode:"compact"}));
  await expect.poll(() => marquee.evaluate(element => element.getAnimations().some(animation => animation.playState === "running"))).toBe(false);
  await page.evaluate(() => (window as any).configureIsland({mode:"expanded"}));
  await expect.poll(() => marquee.evaluate(element => element.getAnimations().some(animation => animation.playState === "running"))).toBe(true);

  await page.evaluate(() => (window as any).configureIsland({visible:false}));
  await expect(page.locator(".expanded-layer")).toHaveCount(0);
  await page.evaluate(() => (window as any).configureIsland({visible:true,mode:"expanded"}));
  await expect.poll(() => page.locator(".music-pane .marquee-title span").evaluate(element => element.getAnimations().some(animation => animation.playState === "running"))).toBe(true);
  await page.evaluate((selector) => {
    (window as any).marqueeNode = document.querySelector(selector);
    (window as any).unmountIsland();
  }, ".music-pane .marquee-title span");
  expect(await page.evaluate(() => (window as any).marqueeNode.getAnimations().some((animation: Animation) => animation.playState === "running"))).toBe(false);
});

for (const edge of ["top", "right", "bottom", "left"]) {
  test(`cover follows the live ${edge} layout and settles quickly`, async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto("/ui-tests/island-fixture.html");
    await page.evaluate((edge) => (window as any).configureIsland({ edge, style: "edge", length: 240, scale: .8, toolbar: true }), edge);
    await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true");
    // Observe the short-lived moving layer in the browser's own frame loop;
    // a second automation round trip can arrive after the animation settles.
    const sawMovingCover = await page.evaluate(async () => {
      (window as any).configureIsland({ mode: "expanded" });
      for (let frame = 0; frame < 12; frame++) {
        await new Promise(requestAnimationFrame);
        if (document.querySelector(".moving-cover.in-motion")) return true;
      }
      return false;
    });
    expect(sawMovingCover).toBe(true);
    await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true", { timeout: 550 });
    const expanded = await page.locator(".top-row .expanded-cover").boundingBox();
    expect(expanded!.width).toBeCloseTo(84 * .8, 0);
    // Move the host and change compact length while expanded. Collapse must
    // finish at the newly laid-out compact cover, rather than a cached point.
    await page.evaluate(() => (window as any).configureIsland({ length: 170, offset: 90 }));
    const sample = await page.evaluate(async () => {
      (window as any).configureIsland({ mode: "compact" });
      // Sample in the browser's frame loop, avoiding round-trip races with
      // an animation which can finish between two Playwright commands.
      for (let frame = 0; frame < 12; frame++) {
        await new Promise(requestAnimationFrame);
        const moving = document.querySelector(".moving-cover.in-motion");
        if (!moving) continue;
        const a = moving.getBoundingClientRect();
        const compact = document.querySelector(".compact-cover")!.getBoundingClientRect();
        const large = document.querySelector(".top-row .expanded-cover")!.getBoundingClientRect();
        const layer = document.querySelector(".expanded-layer")!;
        const translation = new DOMMatrix(getComputedStyle(layer).transform);
        const p = (a.width - compact.width) / (large.width - compact.width);
        return { width: a.width,
          errorX: a.x - (compact.x + (large.x - translation.m41 - compact.x) * p),
          errorY: a.y - (compact.y + (large.y - translation.m42 - compact.y) * p) };
      }
      return null;
    });
    expect(sample).not.toBeNull();
    expect(sample!.width).toBeLessThan(expanded!.width);
    expect(Math.abs(sample!.errorX)).toBeLessThan(1);
    expect(Math.abs(sample!.errorY)).toBeLessThan(1);
    await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true", { timeout: 550 });
    await expect(page.locator(".moving-cover.in-motion")).toHaveCount(0);
    await expect(page.locator(".compact-cover")).toBeVisible();
    const compact = await page.locator(".compact-cover").boundingBox();
    expect(compact!.width).toBeCloseTo(20, 0);
    expect(errors).toEqual([]);
  });
}

test("reversing a transition preserves the current cover position", async ({ page }) => {
  await page.goto("/ui-tests/island-fixture.html");
  await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true");
  const result = await page.evaluate(async () => {
    const configure = (window as any).configureIsland;
    configure({ mode: "expanded" });
    await new Promise((resolve) => setTimeout(resolve, 70));
    const before = document.querySelector(".moving-cover.in-motion")!.getBoundingClientRect();
    configure({ mode: "compact" });
    // Flush the reactive state without advancing the animation clock.
    await Promise.resolve();
    await Promise.resolve();
    const after = document.querySelector(".moving-cover.in-motion")!.getBoundingClientRect();
    return { dx: after.x - before.x, dy: after.y - before.y };
  });
  expect(Math.abs(result.dx)).toBeLessThan(1);
  expect(Math.abs(result.dy)).toBeLessThan(1);
  await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true", { timeout: 550 });
});

test("reduced motion switches layouts without flying artwork", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => (window as any).configureIsland({ mode: "expanded" }));
  await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true");
  await expect(page.locator(".moving-cover.in-motion")).toHaveCount(0);
  await expect(page.locator(".top-row .expanded-cover")).toBeVisible();
});

test("scaled previews preserve artwork coordinates", async ({ page }) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => (window as any).configureIsland({ hostScale: .65, toolbar: true, scale: .8 }));
  await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true");
  const error = await page.evaluate(async () => {
    (window as any).configureIsland({ mode: "expanded" });
    await new Promise((resolve) => setTimeout(resolve, 80));
    const moving = document.querySelector(".moving-cover.in-motion")!.getBoundingClientRect();
    const compact = document.querySelector(".compact-cover")!.getBoundingClientRect();
    const large = document.querySelector(".top-row .expanded-cover")!.getBoundingClientRect();
    const layer = document.querySelector(".expanded-layer")!;
    const matrix = new DOMMatrix(getComputedStyle(layer).transform);
    const p = (moving.width - compact.width) / (large.width - compact.width);
    return { x: moving.x - (compact.x + (large.x - matrix.m41 * .65 - compact.x) * p),
      y: moving.y - (compact.y + (large.y - matrix.m42 * .65 - compact.y) * p) };
  });
  expect(Math.abs(error.x)).toBeLessThan(1);
  expect(Math.abs(error.y)).toBeLessThan(1);
});

test("expansion leaves time to see the shell and cover move", async ({ page }) => {
  await page.goto("/ui-tests/island-fixture.html");
  await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true");
  const sample = await page.evaluate(async () => {
    const surface = document.querySelector<HTMLElement>(".island-surface")!;
    const startHeight = parseFloat(surface.style.height);
    (window as any).configureIsland({ mode: "expanded" });
    const start = performance.now();
    while (performance.now() - start < 100) await new Promise(requestAnimationFrame);
    return { height: parseFloat(surface.style.height), startHeight,
      settled: document.querySelector(".stage")!.getAttribute("data-settled") };
  });
  expect(sample.height).toBeGreaterThan(sample.startHeight);
  expect(sample.settled).toBe("false");
  await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true", { timeout: 650 });
  const endHeight = await page.locator(".island-surface").evaluate((element) => parseFloat((element as HTMLElement).style.height));
  // A short initial interval must not complete most of the motion.
  expect((sample.height - sample.startHeight) / (endHeight - sample.startHeight)).toBeLessThan(.6);
});

test("an unfinished hover can expand without snapping the compact artwork", async ({ page }) => {
  await page.goto("/ui-tests/island-fixture.html");
  await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true");
  const jump = await page.evaluate(async () => {
    const configure = (window as any).configureIsland;
    configure({ mode: "hover" });
    await new Promise((resolve) => setTimeout(resolve, 60));
    const cover = document.querySelector<HTMLElement>(".compact-cover")!;
    const before = cover.getBoundingClientRect();
    configure({ mode: "expanded" });
    await Promise.resolve();
    await Promise.resolve();
    const after = cover.getBoundingClientRect();
    return { x: after.x - before.x, y: after.y - before.y };
  });
  expect(Math.abs(jump.x)).toBeLessThan(1);
  expect(Math.abs(jump.y)).toBeLessThan(1);
  await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true", { timeout: 650 });
});

test("pointer hover preserves compact dimensions and returns without a position jump", async ({ page }) => {
  await page.goto("/ui-tests/island-fixture.html");
  await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true");
  await page.evaluate(() => {
    const anchor = document.querySelector<HTMLElement>(".surface-anchor")!;
    const surface = document.querySelector<HTMLElement>(".island-surface")!;
    const cover = document.querySelector<HTMLElement>(".compact-cover")!;
    (window as any).leaveSample = new Promise((resolve) => {
      anchor.addEventListener("mouseleave", async () => {
        const width = parseFloat(surface.style.width);
        await Promise.resolve();
        await Promise.resolve();
        resolve({ width, after: parseFloat(surface.style.width) });
      }, { once: true });
    });
    (window as any).hoverSample = new Promise((resolve) => {
      anchor.addEventListener("mouseenter", async () => {
        const start = performance.now();
        const startWidth = parseFloat(surface.style.width);
        const initialCover = cover.getBoundingClientRect();
        await Promise.resolve();
        await Promise.resolve();
        const nextCover = cover.getBoundingClientRect();
        while (performance.now() - start < 60) await new Promise(requestAnimationFrame);
        const width = parseFloat(surface.style.width);
        resolve({ startWidth, width, jumpX: nextCover.x - initialCover.x, jumpY: nextCover.y - initialCover.y });
      }, { once: true });
    });
  });
  await page.locator(".island-surface").hover();
  const sample = await page.evaluate(() => (window as any).hoverSample);
  expect(sample.width).toBeCloseTo(sample.startWidth, 1);
  expect(Math.abs(sample.jumpX)).toBeLessThan(.5);
  expect(Math.abs(sample.jumpY)).toBeLessThan(.5);
  await page.mouse.move(0, 0);
  const leave = await page.evaluate(() => (window as any).leaveSample);
  expect(Math.abs(leave.after - leave.width)).toBeLessThan(.5);
  await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true", { timeout: 650 });
  const width = await page.locator(".island-surface").evaluate((element) => parseFloat((element as HTMLElement).style.width));
  expect(width).toBeCloseTo(sample.startWidth, 1);
});

test("page navigation focuses only the latest request and does not focus after collapse or hide", async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => {
    (window as any).focusAttempts = [];
    const original = HTMLButtonElement.prototype.focus;
    HTMLButtonElement.prototype.focus = function(options?: FocusOptions) {
      const target = this.matches(".page-back") ? "page-back"
        : this.closest(".feature-menu") ? "menu-button"
        : "other-button";
      (window as any).focusAttempts.push(target);
      return original.call(this, options);
    };
    (window as any).configureIsland({mode:"expanded",toolbar:true});
  });
  await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true");

  // Reach a real non-music page through its production toolbar action.
  await page.locator(".feature-menu button").nth(0).click({force:true});
  const back = page.locator(".navigation-page .page-back");
  await expect(back).toBeVisible();
  await expect(back).toBeFocused();
  await expect.poll(() => page.evaluate(() => (window as any).focusAttempts.at(-1))).toBe("page-back");

  // Two production click handlers run in one task: the first asks for Music,
  // the second replaces that request with Timer. The stale Music focus must not run.
  await page.evaluate(() => {
    const backButton = document.querySelector<HTMLButtonElement>(".page-back")!;
    const timerButton = document.querySelectorAll<HTMLButtonElement>(".feature-menu button")[0];
    (window as any).focusAttempts = [];
    backButton.click();
    timerButton.click();
  });
  await expect(back).toBeVisible();
  await expect.poll(() => page.evaluate(() => (window as any).focusAttempts)).toEqual(["page-back"]);

  // Collapse during the pending Music navigation; the request must not attempt focus.
  await page.evaluate(() => {
    (window as any).focusAttempts = [];
    document.querySelector<HTMLButtonElement>(".page-back")!.click();
    (window as any).configureIsland({mode:"compact"});
  });
  await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true", {timeout:650});
  await page.waitForTimeout(50);
  expect(await page.evaluate(() => (window as any).focusAttempts)).toEqual([]);

  // Hidden visibility also leaves no stale focus attempt while the layer is removed.
  await page.evaluate(() => {
    (window as any).configureIsland({mode:"expanded",visible:true});
  });
  await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true", {timeout:650});
  await page.locator(".feature-menu button").nth(0).click({force:true});
  await expect(page.locator(".navigation-page .page-back")).toBeVisible();
  await page.evaluate(() => {
    (window as any).focusAttempts = [];
    document.querySelector<HTMLButtonElement>(".page-back")!.click();
    (window as any).configureIsland({visible:false});
  });
  await expect(page.locator(".expanded-layer")).toHaveCount(0);
  await page.waitForTimeout(50);
  expect(await page.evaluate(() => (window as any).focusAttempts)).toEqual([]);
});

test("neutral control fills preserve existing island button proportions", async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => (window as any).configureIsland({mode:"expanded",toolbar:true}));
  await expect(page.locator(".stage")).toHaveAttribute("data-settled", "true");
  const initial = await page.evaluate(() => {
    const transport = document.querySelector<HTMLButtonElement>(".transport-controls .play")!;
    const tool = document.querySelector<HTMLElement>(".function-glyph")!;
    const transportStyle = getComputedStyle(transport);
    const toolStyle = getComputedStyle(tool);
    return {
      transport: {width: transport.getBoundingClientRect().width, height: transport.getBoundingClientRect().height, background: transportStyle.backgroundColor},
      tool: {width: tool.getBoundingClientRect().width, height: tool.getBoundingClientRect().height, background: toolStyle.backgroundColor},
    };
  });
  expect(initial.transport.width).toBeCloseTo(58, 0);
  expect(initial.transport.height).toBeCloseTo(58, 0);
  expect(initial.transport.background).toBe("rgba(0, 0, 0, 0)");
  expect(initial.tool.width).toBeCloseTo(28, 0);
  expect(initial.tool.height).toBeCloseTo(28, 0);
  expect(initial.tool.background).toBe("rgba(255, 255, 255, 0.1)");

  // File sorting has a deliberately larger back control; use the timer page
  // to verify the shared navigation control.
  await page.locator(".feature-menu").getByRole("button", { name: "Timer", exact: true }).click({force:true});
  const back = page.locator(".navigation-page .page-back");
  await expect(back).toBeVisible();
  const backStyle = await back.evaluate((button) => ({
    width: button.getBoundingClientRect().width,
    height: button.getBoundingClientRect().height,
    background: getComputedStyle(button).backgroundColor,
  }));
  expect(backStyle.width).toBeCloseTo(28, 0);
  expect(backStyle.height).toBeCloseTo(28, 0);
  expect(backStyle.background).toBe("rgba(255, 255, 255, 0.08)");
});
