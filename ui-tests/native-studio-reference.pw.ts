import { test, expect } from '@playwright/test';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

test.use({ locale: 'zh-CN', viewport: { width: 1440, height: 960 } });

test('capture the authored Studio layout for native migration', async ({ page }) => {
  const out = resolve('native/artifacts/parity');
  mkdirSync(out, { recursive: true });
  await page.goto('/studio.html');
  await page.evaluate(() => document.fonts.ready);
  await expect(page.locator('.stage')).toBeVisible();
  await page.screenshot({ path: resolve(out, 'reference-studio.png'), fullPage: true });
  const rects = await page.locator('main').evaluate((main) => {
    const selectors = ['.studio-header', '.studio-brand', '.studio-status', '.workspace', '.stage', '.stage-topline', '.preview-host', '.stage-caption', '.workspace aside', '.workspace aside>section', '.preferences-section'];
    return Object.fromEntries(selectors.flatMap((selector) => {
      const node = main.querySelector(selector);
      if (!node) return [];
      const r = node.getBoundingClientRect();
      return [[selector, { x: r.x, y: r.y, width: r.width, height: r.height }]];
    }));
  });
  writeFileSync(resolve(out, 'reference-studio-layout.json'), JSON.stringify(rects, null, 2));
});
