import { expect, test } from "@playwright/test";

test.beforeEach(async ({page}) => {
  await page.addInitScript(() => {
    const fixture = window as any;
    fixture.coverRequests = [];
    // Only new Image() preloaders are replaced; DOM img slots remain real.
    fixture.Image = class {
      src = "";
      crossOrigin = "";
      onload: (() => void) | null = null;
      onerror: (() => void) | null = null;
      constructor() { fixture.coverRequests.push(this); }
      removeAttribute() { this.src = ""; }
    };
  });
  await page.goto("/ui-tests/cover-lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).setCover);
});

test("rapid track changes cancel obsolete preloaders and unmount releases the last request", async ({page}) => {
  for (let index = 0; index < 100; index++) {
    await page.evaluate(index => (window as any).setCover(`cover-${index}`), index);
  }
  const live = () => page.evaluate(() => (window as any).coverRequests.filter((image:any) => image.src).length);
  await expect.poll(live).toBe(1);
  await page.evaluate(() => (window as any).destroyCover());
  expect(await live()).toBe(0);
  expect(await page.evaluate(() => (window as any).coverRequests.every((image:any) => image.onload === null && image.onerror === null))).toBe(true);
});

test("success releases the preloader, preserves crossfade, and failure keeps the last cover", async ({page}) => {
  await page.evaluate(() => (window as any).setCover("/icons/spotify.svg"));
  await page.evaluate(() => (window as any).coverRequests.at(-1).onload());
  await expect(page.locator("img.active")).toHaveAttribute("src", "/icons/spotify.svg");
  await page.evaluate(() => (window as any).setCover("/icons/next.svg"));
  await page.evaluate(() => (window as any).coverRequests.at(-1).onload());
  await expect(page.locator("img")).toHaveCount(2);
  await expect(page.locator("img.active")).toHaveAttribute("src", "/icons/next.svg");
  await expect(page.locator("img")).toHaveCount(1);
  await page.evaluate(() => (window as any).setCover("broken-cover"));
  await page.evaluate(() => (window as any).coverRequests.at(-1).onerror());
  await expect(page.locator("img.active")).toHaveAttribute("src", "/icons/next.svg");
  expect(await page.evaluate(() => (window as any).coverRequests.every((image:any) => image.src === ""))).toBe(true);
});

test("stalled preloading expires after five seconds without retaining handlers or source", async ({page}) => {
  await page.clock.install();
  await page.evaluate(() => (window as any).setCover("stalled-cover"));
  await page.clock.runFor(5000);
  expect(await page.evaluate(() => {
    const image = (window as any).coverRequests.at(-1);
    return {src:image.src, load:image.onload, error:image.onerror};
  })).toEqual({src:"",load:null,error:null});
});
