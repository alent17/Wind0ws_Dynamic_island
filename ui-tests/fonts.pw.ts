import { expect, test } from "@playwright/test";

test("multilingual text loads bundled fonts without loading a full MiSans TTF", async ({ page }) => {
  const failedFonts: string[] = [];
  page.on("response", response => {
    if (response.url().includes("/fonts/official/") && !response.ok()) failedFonts.push(response.url());
  });
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  const fontRequests = await page.evaluate(async () => {
    const text = "音乐播放 Midnight City 日本語 あいうえお العربية தமிழ்";
    const sample = document.createElement("div");
    sample.textContent = text;
    sample.style.cssText = 'font-family: MiSans, sans-serif; font-size: 24px; font-weight: 500';
    document.body.append(sample);
    await Promise.all([400, 500, 700].map(weight => document.fonts.load(`${weight} 24px MiSans`, text)));
    await document.fonts.ready;
    return performance.getEntriesByType("resource").map(entry => entry.name).filter(name => name.includes("/fonts/"));
  });
  expect(failedFonts).toEqual([]);
  expect(fontRequests.some(name => name.includes("/fonts/official/"))).toBe(true);
  expect(fontRequests.filter(name => /MiSans-(Regular|Medium|Bold)\.ttf/.test(name))).toEqual([]);
});
