import { test, expect } from "@playwright/test";

test.use({ viewport: { width: 1800, height: 900 } });
test("weather card aligns its current conditions, forecast and editor controls", async ({ page }) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => (window as any).configureIsland({ mode: "expanded", scale: .7, offset: 220, widgets: { left: ["weather"], right: ["volume"] }, weather: {
    weatherCity: "江门市 · 广东 · 中国", idleWeatherTemperature: 31, idleWeatherCode: 2, weatherUpdatedAt: Date.now(),
    idleWeatherForecast: [26,27,28,29].map((day, i) => ({ date: `2026-10-${day}`, weatherCode: i === 2 ? 61 : 2, temperatureMax: 32-i, temperatureMin: 23-i })),
  } }));
  await expect(page.locator(".weather-location strong")).toHaveText("江门市");
  await expect(page.locator(".weather-location span")).toHaveText("广东 · 中国");
  await expect(page.locator(".weather-day")).toHaveCount(3);
  for (const expandedLength of [480,600,900]) {
    await page.evaluate(length => (window as any).configureIsland({ expandedLength: length }), expandedLength);
    await expect.poll(() => page.locator(".stage").getAttribute("data-settled")).toBe("true");
    const valid = await page.locator(".widget-weather").evaluate(root => {
      const card = root.getBoundingClientRect();
      const box = (selector: string) => root.querySelector(selector)!.getBoundingClientRect();
      const hero = box(".weather-hero"), days = box(".weather-days"), status = box(".weather-status");
      return hero.bottom <= days.top && days.bottom <= status.top && status.bottom <= card.bottom + 1 &&
        box(".weather-location").right <= box(".weather-temperature").left && box(".weather-temperature").right <= box(".weather-symbol").left && box(".weather-symbol").right <= card.right + 1;
    });
    expect(valid).toBe(true);
  }
  await page.evaluate(() => (window as any).configureIsland({ expandedLength: 600 }));
  await page.screenshot({path:"build/widget-weather-redesign.png"});
  await page.locator(".island-surface").focus(); await page.keyboard.press("F2");
  const content = await page.locator(".weather-hero").boundingBox();
  const tools = await page.locator('[data-widget-kind="weather"] .card-tools').boundingBox();
  expect(tools!.y + tools!.height).toBeLessThanOrEqual(content!.y);
  await page.evaluate(() => (window as any).configureIsland({weather:{idleWeatherTemperature:null,weatherFailed:true}}));
  await expect(page.locator(".weather-temperature")).toHaveText("--°");
  await expect(page.locator(".weather-status")).not.toBeEmpty();
});
