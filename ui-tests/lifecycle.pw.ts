import { expect, test } from "@playwright/test";

for (const stage of ["exit", "enter"]) test(`App cancels WAAPI on unmount during ${stage}, no late layout IPC`, async ({page}) => {
  const errors: string[] = []; page.on("pageerror", error => errors.push(String(error)));
  await page.emulateMedia({reducedMotion:"no-preference"});
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => (window as any).mountApp());
  await expect.poll(() => page.evaluate(() => (window as any).calls.includes("show_main_window"))).toBe(true);
  await page.evaluate(() => (window as any).changeEdge());
  await page.waitForFunction(opacity => (window as any).animations.at(-1)?.firstOpacity === opacity, stage === "exit" ? 1 : 0);
  const before = await page.evaluate(async () => {
    await (window as any).destroy();
    return (window as any).calls.filter((cmd: string) => cmd === "animate_window_bounds").length;
  });
  await page.waitForTimeout(800);
  expect(await page.evaluate(() => (window as any).calls.filter((cmd: string) => cmd === "animate_window_bounds").length)).toBe(before);
  expect(await page.evaluate(() => (window as any).animations.at(-1).cancelled)).toBeGreaterThan(0);
  await expect(page.locator(".fixed-island-host")).toHaveCount(0);
  expect(errors).toEqual([]);
});

for (const frames of [0,1]) test(`RollingDigit cancels pending frame after ${frames} recenter frames`, async ({page}) => {
  await page.emulateMedia({reducedMotion:"no-preference"});
  await page.goto("/ui-tests/lifecycle-fixture.html"); await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => (window as any).mountDigit());
  await expect(page.locator(".digit-strip")).toHaveClass(/ready/);
  await page.evaluate(() => (window as any).setDigit("9"));
  await expect(page.locator(".digit-strip")).toHaveCSS("transform", /matrix/);
  const result = await page.evaluate(async count => {
    let next = 100; const queue = new Map<number,FrameRequestCallback>(), cancelled: number[] = [];
    window.requestAnimationFrame = callback => { queue.set(++next,callback); return next; };
    window.cancelAnimationFrame = id => { cancelled.push(id); queue.delete(id); };
    document.querySelector(".digit-strip")!.dispatchEvent(new TransitionEvent("transitionend",{propertyName:"transform"}));
    if (count) { const [id,callback] = [...queue][0]; queue.delete(id); callback(0); }
    await (window as any).destroy();
    return {cancelled,pending:queue.size};
  }, frames);
  expect(result.pending).toBe(0); expect(result.cancelled.length).toBeGreaterThan(0);
  await expect(page.locator(".digit-strip")).toHaveCount(0);
});
