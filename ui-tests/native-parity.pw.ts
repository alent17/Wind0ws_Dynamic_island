import { test, expect } from '@playwright/test';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

// Reference evidence for the native port. These render the production Svelte
// components with fixed data, bundled MiSans, and the application's locale.
test.use({ locale:'zh-CN' });
test('capture native migration reference states', async ({page}) => {
  const out=resolve('native/artifacts/parity');
  mkdirSync(out,{recursive:true});
  const measurements:Record<string,unknown>={};
  await page.goto('/ui-tests/island-fixture.html?weather=1');
  await page.addStyleTag({content:`
    @font-face{font-family:MiSans;src:url('/fonts/MiSans-Regular.ttf');font-weight:400}
    @font-face{font-family:MiSans;src:url('/fonts/MiSans-Medium.ttf');font-weight:500}
    @font-face{font-family:MiSans;src:url('/fonts/MiSans-Bold.ttf');font-weight:700}
    :root{--app-font:MiSans!important}body{font-family:MiSans;line-height:1.5}
    .island-stage{background:#383838}
  `});
  await page.evaluate(()=>document.fonts.ready);
  const capture=async(name:string)=>{
    const surface=page.locator('.island-surface');
    await surface.screenshot({path:resolve(out,`reference-${name}.png`)});
    measurements[name]=await surface.evaluate(el=>{
      const outer=el.getBoundingClientRect();
      const selectors=['.navigation-page','.page-header','.page-content','.timer-ruler','.timer-footer','.timer-readout','.timer-readout .rolling-number','.timer-readout small','.timer-main-button','.timer-complete-card','.volume-ruler','.device-trigger','.device-menu','.volume-readout','.volume-readout .rolling-number','.volume-label','.clock-face','.caption','.digits','.current-weather','.current-weather strong','.current-weather span','.forecast','.forecast>div','.updated'];
      return Object.fromEntries(selectors.flatMap(selector=>{
        const item=el.querySelector(selector);if(!item)return[];
        const r=item.getBoundingClientRect();
        return [[selector,{x:r.x-outer.x,y:r.y-outer.y,width:r.width,height:r.height}]];
      }));
    });
  };
  const tools=page.locator('.feature-menu button');
  await tools.nth(0).click();
  await expect(page.locator('.timer-ruler')).toBeVisible();
  await capture('timer-idle');
  await page.locator('.timer-main-button').click();
  await capture('timer-running');
  await page.locator('.timer-main-button').click();
  await capture('timer-paused');
  await page.getByTestId('timer-complete').click();
  await capture('timer-complete');
  await page.locator('.timer-complete-dismiss').click();
  await tools.nth(1).click();
  await capture('volume');
  await page.locator('.device-trigger').click();
  await capture('devices');
  await page.keyboard.press('Escape');
  await tools.nth(5).click();
  await capture('clock');
  await tools.nth(6).click();
  await capture('weather');
  writeFileSync(resolve(out,'reference-layout.json'),JSON.stringify(measurements,null,2));
});
