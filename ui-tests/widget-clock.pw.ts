import { test, expect } from "@playwright/test";

test.use({ viewport: { width: 1800, height: 900 }, timezoneId: "Asia/Shanghai" });

test("widget clock advances seconds across midnight and updates its calendar", async ({ page }) => {
  await page.clock.install({ time: new Date("2026-10-26T23:59:59+08:00") });
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => (window as any).configureIsland({ mode: "expanded", widgets: { left: ["clock"], right: [] } }));
  await expect(page.locator(".widget-time")).toHaveAttribute("aria-label", "23:59:59");
  const previousDate = await page.locator(".clock-calendar").textContent();
  await page.clock.runFor(1000);
  await expect(page.locator(".widget-time")).toHaveAttribute("aria-label", "00:00:00");
  await expect(page.locator(".clock-calendar")).not.toHaveText(previousDate!);
  await page.clock.runFor(1000);
  await expect(page.locator(".widget-time")).toHaveAttribute("aria-label", "00:00:01");
});

test("widget clock fills its card and keeps the calendar clear at short and wide lengths", async ({ page }) => {
  await page.goto("/ui-tests/island-fixture.html");
  for (const expandedLength of [480, 600, 900]) {
    await page.evaluate(length => (window as any).configureIsland({ mode: "expanded", expandedLength: length, scale: .65, offset: 200, widgets: { left: ["clock"], right: ["volume"] } }), expandedLength);
    const clock = page.locator(".widget-clock");
    await expect(clock).toBeVisible();
    await expect.poll(() => page.locator(".stage").getAttribute("data-settled")).toBe("true");
    const bounds = await clock.evaluate(root => {
      const rect = (selector: string) => root.querySelector(selector)!.getBoundingClientRect();
      const card = root.getBoundingClientRect();
      const time = rect(".rolling-number");
      const heading = rect(".clock-heading");
      const calendar = rect(".clock-calendar");
      return { fill: time.width / card.width, contained: time.x >= card.x && time.right <= card.right, clear: heading.bottom <= time.top && time.bottom <= calendar.top, font: parseFloat(getComputedStyle(root.querySelector(".widget-time")!).fontSize) };
    });
    expect(bounds.fill).toBeGreaterThan(expandedLength === 900 ? .4 : .6);
    expect(bounds.contained).toBe(true);
    expect(bounds.clear).toBe(true);
    expect(bounds.font).toBeGreaterThan(60);
  }
  await page.evaluate(() => (window as any).configureIsland({ expandedLength: 600 }));
  await page.screenshot({ path: "build/widget-clock-redesign.png" });
  await page.locator(".island-surface").focus();
  await page.keyboard.press("F2");
  const heading = await page.locator(".clock-heading").boundingBox();
  const tools = await page.locator('[data-widget-kind="clock"] .card-tools').boundingBox();
  expect(tools!.y + tools!.height).toBeLessThanOrEqual(heading!.y);
});
