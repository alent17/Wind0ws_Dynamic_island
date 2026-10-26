import { test, expect } from "@playwright/test";
const snapshot = {cpuPercent:24,memoryPercent:62,memoryUsedBytes:10*1024**3,memoryTotalBytes:16*1024**3,gpuPercent:35,downloadBytesPerSecond:3*1024**2,uploadBytesPerSecond:128*1024};
test.use({viewport:{width:1800,height:900}});

test("system monitoring can be added, saved and opened as a function page", async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(snapshot => (window as any).configureIsland({mode:"expanded",expandedLength:480,offset:200,scale:.7,toolbar:true,monitorSnapshot:snapshot}),snapshot);
  await page.locator(".island-surface").focus(); await page.keyboard.press("F2");
  await page.locator(".catalog-item").last().click();
  await expect(page.locator('[data-widget-kind="system"]')).toHaveCount(1);
  await page.locator(".edit-actions .done").click();
  await expect(page.locator(".metric-number").first()).toHaveText("24%");
  await expect(page.locator(".network-rate").first()).toHaveText("3.0 MB/s");
  for (const expandedLength of [480,900]) {
    await page.evaluate(expandedLength => (window as any).configureIsland({expandedLength}),expandedLength);
    const fits = await page.locator(".system-monitor").evaluate(root => {
      const r = root.getBoundingClientRect();
      return [...root.querySelectorAll(".system-metric,.system-status")].every(el => {const a=el.getBoundingClientRect();return a.left>=r.left-1 && a.right<=r.right+1 && a.bottom<=r.bottom+1;});
    });
    expect(fits).toBe(true);
  }
  await page.evaluate(() => (window as any).configureIsland({expandedLength:600}));
  await page.screenshot({path:"build/system-monitor-widget.png"});
  await page.locator('.function-icon[aria-label="System monitor"]').click();
  await expect(page.locator(".navigation-page .system-monitor")).toBeVisible();
  await expect(page.locator(".metric-number").nth(1)).toHaveText("35%");
  await page.evaluate(snapshot => (window as any).configureIsland({monitorSnapshot:{...snapshot,gpuPercent:null}}),snapshot);
  await expect(page.locator(".metric-number").nth(1)).toHaveText("--");
});

test("native polling updates live and stops while collapsed", async ({page}) => {
  await page.addInitScript(snapshot => {
    (window as any).isTauri = true;
    (window as any).monitorCalls = 0;
    (window as any).__TAURI_INTERNALS__ = {invoke:async (command:string) => {
      if(command !== "get_system_snapshot") throw Error(command);
      (window as any).monitorCalls++;
      return snapshot;
    }};
  },snapshot);
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => (window as any).configureIsland({mode:"expanded",widgets:{left:["system"],right:[]}}));
  await expect(page.locator(".metric-number").first()).toHaveText("24%");
  await expect.poll(() => page.evaluate(() => (window as any).monitorCalls)).toBeGreaterThan(1);
  await page.evaluate(() => (window as any).configureIsland({mode:"compact"}));
  const calls = await page.evaluate(() => (window as any).monitorCalls);
  await page.waitForTimeout(1200);
  expect(await page.evaluate(() => (window as any).monitorCalls)).toBe(calls);
});
