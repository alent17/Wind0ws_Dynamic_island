import { test, expect } from "@playwright/test";

test.use({ reducedMotion: "no-preference" });
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
