import { test, expect } from "@playwright/test";

const cover = (color: string) => `data:image/svg+xml,${encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" width="2" height="2"><rect width="2" height="2" fill="${color}"/></svg>`)}`;
const red = cover("red");
const blue = cover("blue");

test("actual DOM remounts release Canvas backing stores over 100 empty-cover cycles", async ({ page }) => {
  await page.goto("/ui-tests/canvas-artwork-fixture.html");
  for (let index = 0; index < 100; index++) {
    await page.evaluate(url => (window as any).configureArtwork({url,track:'track',pixelated:false}), red);
    await expect.poll(() => page.locator("canvas").evaluate(canvas => (canvas as HTMLCanvasElement).width)).toBe(2);
    await page.evaluate(() => (window as any).configureArtwork(null));
    await expect(page.locator("canvas")).toHaveCount(0);
  }
  await page.evaluate(url => (window as any).configureArtwork({url,track:'latest',pixelated:false}), blue);
  await expect.poll(() => page.locator("canvas").evaluate(canvas => [...(canvas as HTMLCanvasElement).getContext("2d")!.getImageData(0,0,1,1).data])).toEqual([0,0,255,255]);
  const releases = await page.evaluate(() => (window as any).releasedCanvases);
  expect(releases).toHaveLength(100);
  expect(releases.every((item: any) => item.width === 0 && item.height === 0)).toBe(true);
});

test("late backend work cannot replace the latest cover or revive a removed node", async ({ page }) => {
  await page.goto("/ui-tests/canvas-artwork-fixture.html");
  await page.evaluate(url => (window as any).configureArtwork({url,track:'old',pixelated:true}), red);
  await expect(page.locator("canvas")).toHaveCount(1);
  await page.evaluate(url => (window as any).configureArtwork({url,track:'new',pixelated:false}), blue);
  await expect.poll(() => page.locator("canvas").evaluate(canvas => [...(canvas as HTMLCanvasElement).getContext("2d")!.getImageData(0,0,1,1).data])).toEqual([0,0,255,255]);
  await page.evaluate(() => (window as any).resolveArtwork());
  await page.waitForTimeout(100);
  expect(await page.locator("canvas").evaluate(canvas => [...(canvas as HTMLCanvasElement).getContext("2d")!.getImageData(0,0,1,1).data])).toEqual([0,0,255,255]);
  await page.evaluate(url => (window as any).configureArtwork({url,track:'pending',pixelated:true}), red);
  await page.evaluate(() => (window as any).configureArtwork(null));
  await expect(page.locator("canvas")).toHaveCount(0);
  await page.evaluate(() => (window as any).resolveArtwork());
  expect(await page.evaluate(() => (window as any).releasedCanvases.at(-1))).toEqual({width:0,height:0});
});
