import { test, expect } from "@playwright/test";

test("context menu supports keyboard navigation, dismissal and settings", async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html");
  // The desktop host passes transparent areas through to the windows beneath it.
  await page.addStyleTag({content:".stage{pointer-events:none}.stage .island-surface{pointer-events:auto}"});
  const surface = page.locator(".island-surface");
  await surface.click({button:"right"});
  const items = page.getByRole("menuitem");
  await expect(items.first()).toBeFocused();
  await page.keyboard.press("ArrowDown");
  await expect(items.last()).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.locator(".stage")).toHaveAttribute("data-settings-opened","1");
  await expect(page.getByRole("menu")).toHaveCount(0);
  await surface.click({button:"right"});
  await expect(page.getByRole("menu")).toHaveCSS("pointer-events","auto");
  await page.getByRole("menuitem").first().click();
  await expect(page.locator(".editor-bar")).toBeVisible();
  await page.keyboard.press("Escape");
  await surface.focus(); await page.keyboard.press("Shift+F10");
  await expect(page.getByRole("menu")).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("menu")).toHaveCount(0);
  await expect(surface).toBeFocused();
  await surface.click({button:"right"});
  await page.mouse.click(10,10);
  await expect(page.getByRole("menu")).toHaveCount(0);
});

test("menu stays in the native host on every edge and contributes a clickable region", async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html");
  for (const edge of ["top","bottom","left","right"]) {
    await page.evaluate(edge => (window as any).configureIsland({edge,style:"edge",mode:"compact"}),edge);
    await page.locator(".island-surface").focus(); await page.keyboard.press("Shift+F10");
    const menu = page.getByRole("menu");
    await expect(menu).toBeVisible();
    const valid = await menu.evaluate(root => {
      const m = root.getBoundingClientRect(), f = root.closest(".island-frame")!.getBoundingClientRect();
      return m.left >= f.left && m.right <= f.right && m.top >= f.top && m.bottom <= f.bottom;
    });
    expect(valid).toBe(true);
    const region = await page.evaluate(() => (window as any).lastIslandRegion);
    expect(region.extraRects).toHaveLength(1);
    expect(region.extraRects[0].width).toBeGreaterThan(100);
    await page.keyboard.press("Escape");
    await expect.poll(() => page.evaluate(() => (window as any).lastIslandRegion.extraRects.length)).toBe(0);
  }
});
