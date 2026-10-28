import {test, expect} from "@playwright/test";

test.use({viewport:{width:1600,height:900},reducedMotion:"no-preference"});
for (const widgets of [[], ["clock"], ["clock", "volume"]]) test(`music opens the feature stack with ${widgets.length} selected widgets`, async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(widgets => {
    const f = window as any;
    f.settingsOverrides = {islandStyle:"edge",islandEdge:"top",enableAnimations:true,reduceAnimations:false,islandWidgets:{left:[],right:widgets},islandWidgetsEnabled:true,wheelVolumeControl:true};
    f.mountApp();
  }, widgets);
  await expect(page.locator(".fixed-host")).toBeVisible();
  await page.evaluate(() => (window as any).expand());
  const music = page.locator(".music-pane");
  await expect(music).toBeVisible();
  await page.waitForTimeout(700);
  const writes = await page.evaluate(() => (window as any).volumeWrites.length);
  await music.locator(".metadata").hover(); await page.mouse.wheel(0,120);
  const stack = page.locator(".feature-stack");
  await expect(stack).toBeVisible();
  expect(await page.evaluate(() => (window as any).volumeWrites.length)).toBe(writes);
  await expect(stack.locator(".card-handle")).toHaveCount(6);
  await stack.locator(".card-handle").last().hover();
  await expect(stack.locator(".card-handle").last()).toHaveAttribute("aria-pressed", "true");
  await expect(stack.locator('[data-stack-kind="system"]')).toBeVisible();
  await stack.locator(".card-handle").nth(3).hover();
  await expect(stack.locator('[data-stack-kind="calendar"]')).toBeVisible();
  await stack.locator(".calendar-widget").hover(); await page.mouse.wheel(0,120);
  await expect(page.locator(".calendar-extension")).toBeVisible();
  await page.locator(".calendar-extension header button").first().click();
  await expect(stack.locator('[data-stack-kind="calendar"]')).toBeVisible();
  await stack.locator("header button").click();
  await expect(stack).toHaveCount(0);
  await expect(music).toBeVisible();
  for (const kind of widgets) await expect(page.locator(`[data-widget-kind="${kind}"]`)).toBeVisible();
  await page.evaluate(() => (window as any).destroy());
});
