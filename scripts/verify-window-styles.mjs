// Run after npm run build: development mode does not reproduce CSS preload bugs.
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile, mkdir } from 'node:fs/promises';
import { resolve, sep, extname } from 'node:path';
import { chromium } from '@playwright/test';

const root = resolve('build');
const server = createServer(async (req, res) => {
  try {
    const pathname = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
    const path = resolve(root, '.' + (pathname === '/' ? '/index.html' : pathname));
    if (!path.startsWith(root + sep)) throw new Error('Invalid path');
    const mime = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml' };
    res.setHeader('Content-Type', mime[extname(path)] || 'application/octet-stream');
    res.end(await readFile(path));
  } catch {
    res.writeHead(404);
    res.end();
  }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
let browser;
try {
  browser = await chromium.launch({ channel: 'chrome', headless: true });
  await mkdir('dist/verification', { recursive: true });
  for (const kind of ['floating', 'timer']) {
    const page = await browser.newPage({ viewport: { width: 360, height: 430 } });
    await page.goto(`http://127.0.0.1:${server.address().port}/?window=${kind}`);
    const selector = kind === 'floating' ? '.player' : '.timer-window';
    await page.locator(selector).waitFor({ state: 'visible' });
    const result = await page.evaluate(selector => {
      const element = document.querySelector(selector);
      const style = getComputedStyle(element);
      return {
        position: style.position,
        background: style.backgroundColor,
        width: element.getBoundingClientRect().width,
        stylesheets: [...document.querySelectorAll('link[rel="stylesheet"]')].map(link => link.href),
      };
    }, selector);
    const ownStyles = kind === 'floating' ? 'FloatingWindow-' : 'TimerWindow-';
    const otherStyles = kind === 'floating' ? 'TimerWindow-' : 'FloatingWindow-';
    assert(result.stylesheets.some(url => url.includes(ownStyles)), `${kind}: missing CSS`);
    assert(!result.stylesheets.some(url => url.includes(otherStyles)), `${kind}: wrong window CSS`);
    if (kind === 'floating') {
      assert.equal(result.position, 'relative');
      assert.equal(result.background, 'rgb(18, 18, 18)');
      assert.equal(result.width, 360);
    }
    await page.screenshot({ path: `dist/verification/${kind}-production.png` });
    console.log(`${kind}: production stylesheet and layout passed`);
    await page.close();
  }
} finally {
  await browser?.close();
  await new Promise(resolve => server.close(resolve));
}
