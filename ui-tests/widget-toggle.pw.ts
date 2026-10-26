import { test, expect } from "@playwright/test";
test.use({viewport:{width:1800,height:900}});

test("toggle before floating button hides and restores configured cards", async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => (window as any).configureIsland({mode:"expanded",offset:225,scale:.65,widgets:{left:["clock"],right:["system"]}}));
  const toggle = page.locator(".layout-toggle");
  await expect(toggle).toHaveAttribute("aria-pressed","true");
  expect(await toggle.evaluate(button => button.nextElementSibling?.getAttribute("aria-label"))).toBe("Toggle floating player");
  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-pressed","false");
  await expect(page.locator(".widget-card")).toHaveCount(0);
  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-pressed","true");
  await expect(page.locator('.side[data-widget-side="left"] .widget-card')).toHaveAttribute("data-widget-kind","clock");
  await expect(page.locator('.side[data-widget-side="right"] .widget-card')).toHaveAttribute("data-widget-kind","system");
  await page.evaluate(() => (window as any).configureIsland({saveFails:true}));
  await toggle.click();
  await expect(page.locator(".layout-toggle-error")).toBeVisible();
  await expect(toggle).toHaveAttribute("aria-pressed","true");
});

test("empty layout opens editing and saving from disabled enables the layout", async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => (window as any).configureIsland({mode:"expanded",widgetsEnabled:false}));
  await page.locator(".layout-toggle").click();
  await expect(page.locator(".editor-bar")).toBeVisible();
  await page.locator(".catalog-item").first().click();
  await page.locator(".edit-actions .done").click();
  await expect(page.locator(".layout-toggle")).toHaveAttribute("aria-pressed","true");
  await expect(page.locator(".widget-card")).toHaveCount(1);
});

test("desktop saves only the switch and keeps its native host fixed", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const f = window as any;
    f.settingsOverrides = {islandWidgets:{left:["clock"],right:["weather"]},enableAnimations:false,reduceAnimations:true};
    f.togglePatches = [];
    f.boundsChanges = [];
    const invoke = f.__TAURI_INTERNALS__.invoke;
    f.__TAURI_INTERNALS__.invoke = (command:any,args:any,options:any) => {
      if(command === "update_settings") f.togglePatches.push(args.patch);
      if(command === "animate_window_bounds") f.boundsChanges.push(args);
      return invoke(command,args,options);
    };
    f.mountApp();
  });
  await page.evaluate(() => (window as any).expand());
  await expect(page.locator(".widget-card")).toHaveCount(2);
  await page.waitForTimeout(350);
  const boundsCount = await page.evaluate(() => (window as any).boundsChanges.length);
  const toggle = page.locator(".layout-toggle");
  await toggle.click();
  await expect(page.locator(".widget-card")).toHaveCount(0);
  await toggle.click();
  await expect(page.locator(".widget-card")).toHaveCount(2);
  expect(await page.evaluate(() => (window as any).togglePatches)).toEqual([{islandWidgetsEnabled:false},{islandWidgetsEnabled:true}]);
  expect(await page.evaluate(() => (window as any).boundsChanges.length)).toBe(boundsCount);
  await page.evaluate(() => (window as any).destroy());
});
