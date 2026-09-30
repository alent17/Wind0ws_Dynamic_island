import { test, expect } from '@playwright/test';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

test.use({ locale: 'zh-CN' });

for (const window of [
  { kind: 'floating', selector: '.player', viewport: { width: 360, height: 430 } },
  { kind: 'timer', selector: '.timer-window', viewport: { width: 390, height: 430 } },
] as const) {
  test(`capture ${window.kind} window reference`, async ({ page }) => {
    const out = resolve('native/artifacts/parity');
    mkdirSync(out, { recursive: true });
    await page.setViewportSize(window.viewport);
    await page.goto(`/?window=${window.kind}`);
    const root = page.locator(window.selector);
    await expect(root).toBeVisible();
    await page.evaluate(() => document.fonts.ready);
    await root.screenshot({ path: resolve(out, `reference-${window.kind}-window.png`) });
    const layout = await root.evaluate((el) => {
      const selectors = ['.drag-bar', '.album-cover', '.cover-art', '.player-controls', '.timer-shell', '.timer-content', '.timer-presets', '.timer-copy'];
      const bounds = (node: Element) => { const r = node.getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height }; };
      return { root: bounds(el), children: Object.fromEntries(selectors.flatMap((s) => {
        const node = el.querySelector(s);
        return node ? [[s, bounds(node)]] : [];
      })) };
    });
    writeFileSync(resolve(out, `reference-${window.kind}-layout.json`), JSON.stringify(layout, null, 2));
  });
}
