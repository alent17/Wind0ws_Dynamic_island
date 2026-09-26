import { expect, test } from "@playwright/test";

test("hierarchy, Escape, direct actions, and reopening", async ({ page }, info) => {
  await page.goto("/ui-tests/island-fixture.html");
  const menu = page.locator(".feature-menu");
  await expect(menu.locator("button")).toHaveCount(7);
  await expect(menu.locator("button").first()).toHaveAttribute("aria-label", "Timer");
  await page.screenshot({path:info.outputPath("function-menu.png")});
  for (const [index, action] of [[2,"floating"],[3,"settings"],[4,"hide"]] as const) {
    await menu.locator("button").nth(index).click();
    await expect(page.locator("main")).toHaveAttribute("data-action",action);
    await expect(menu).toBeVisible();
  }
  await menu.locator("button").nth(1).click();
  await expect(page.locator(".volume-ruler")).toBeVisible();
  await expect(page.locator(".music-pane")).toHaveCount(0);
  await page.keyboard.press("Escape");
  await expect(menu).toBeVisible();
  await expect(page.locator(".music-pane")).toBeVisible();
  await menu.locator("button").first().click();
  await page.getByTestId("compact-mode").click();
  await page.getByTestId("expanded-mode").click();
  await expect(page.locator(".music-pane")).toBeVisible();
  await expect(page.locator(".timer-panel")).toHaveCount(0);
  await page.getByTestId("no-tools").click();
  await expect(page.locator(".music-pane")).toBeVisible();
  await expect(page.locator(".expand-functions")).toHaveCount(0);
});

for (const count of [0,1,5]) test(`menu enabled count ${count}`, async ({page}) => {
  await page.goto(`/ui-tests/island-fixture.html?count=${count}`);
  if (!count) { await expect(page.locator(".expand-functions")).toHaveCount(0); return; }
  await expect(page.locator(".feature-menu button")).toHaveCount(count);
  expect(await page.locator(".feature-menu").evaluate(el => el.scrollWidth <= el.clientWidth + 1)).toBe(true);
});

test("overflow rail scrolls with drag, wheel and keyboard without accidental activation", async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html?overflow=1");
  const fixture = page.locator(".overflow-fixture");
  const rail = fixture.locator(".feature-menu");
  const box = (await rail.boundingBox())!;
  const visible = await rail.locator("button").evaluateAll(buttons => {
    const bounds = buttons[0].parentElement!.getBoundingClientRect();
    return buttons.filter(button => { const r = button.getBoundingClientRect(); return r.left >= bounds.left && r.right <= bounds.right + 1; }).length;
  });
  expect(visible).toBe(5);
  await page.mouse.move(box.x + 130, box.y + 20);
  await page.mouse.down();
  await page.mouse.move(box.x + 20, box.y + 20, {steps:10});
  await page.mouse.up();
  await expect(fixture).toHaveAttribute("data-selected", "");
  expect(await rail.evaluate(el => el.scrollLeft)).toBeGreaterThan(0);
  await rail.locator("button").last().focus();
  await page.keyboard.press("Home");
  await expect(rail.locator("button").first()).toBeFocused();
  await rail.hover();
  await page.mouse.wheel(0, 150);
  await expect.poll(() => rail.evaluate(el => el.scrollLeft)).toBeGreaterThan(0);
  await page.keyboard.press("End");
  await expect(rail.locator("button").last()).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(fixture).toHaveAttribute("data-selected","7");
});

for (const style of ["edge", "floating"]) for (const edge of ["top","right","bottom","left"]) {
  test(`${style} ${edge} navigation uses the same anchor and settled hit geometry`, async ({page}, info) => {
    await page.goto(`/ui-tests/island-fixture.html?style=${style}&edge=${edge}`);
    const surface = page.locator(".island-surface");
    const before = (await surface.boundingBox())!;
      const after = (await surface.boundingBox())!;
    if (["top","bottom"].includes(edge)) expect(Math.abs(before.x + before.width/2 - after.x - after.width/2)).toBeLessThan(1);
    else expect(Math.abs(before.y + before.height/2 - after.y - after.height/2)).toBeLessThan(1);
    const region = JSON.parse((await page.locator("main").getAttribute("data-region"))!);
    expect(region.settled).toBe(true);
    expect(region.geometry.width).toBeCloseTo(after.width);
    expect(region.geometry.height).toBeCloseTo(after.height);
    for (const button of await page.locator(".feature-menu button").all().then(buttons => buttons.slice(0,5))) {
      expect(await button.evaluate(el => {
        const r=el.getBoundingClientRect(); const hit=document.elementFromPoint(r.x+r.width/2,r.y+20);
        return hit === el || el.contains(hit);
      })).toBe(true);
    }
    await page.locator(".feature-menu button").first().click();
    await expect(page.locator(".timer-panel")).toBeVisible();
    await page.screenshot({path:info.outputPath(`${style}-${edge}-detail.png`)});
  });
}


test("blank menu and detail space collapses without stopping timer or activating controls", async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html");
  const rail = page.locator(".feature-menu");
  // Gap between the first two 44px buttons.
  await page.locator(".top-tools").click({position:{x:150,y:5}});
  await expect(page.locator("main")).toHaveAttribute("data-mode","compact");
  await page.getByTestId("expanded-mode").click();
  await page.locator(".feature-menu button").first().click();
  await page.locator(".timer-main-button").click();
  await expect(page.locator("main")).toHaveAttribute("data-mode","expanded");
  await page.locator(".page-header").click({position:{x:180,y:14}});
  await expect(page.locator("main")).toHaveAttribute("data-mode","compact");
  await expect(page.locator(".compact-timer")).toBeVisible();
  await page.getByTestId("expanded-mode").click();
  await page.locator(".feature-menu button").nth(1).click();
  await page.locator(".device-trigger").click();
  await page.getByRole("option",{name:"Headphones"}).click();
  await expect(page.locator("main")).toHaveAttribute("data-mode","expanded");
  await page.locator(".volume-label").click();
  await expect(page.locator("main")).toHaveAttribute("data-mode","compact");
});

test("interrupted springs converge and keep the reported shape aligned with rendering", async ({page}) => {
  await page.emulateMedia({reducedMotion:"no-preference"});
  await page.goto("/ui-tests/island-fixture.html?animate=1");
  await expect(page.locator(".feature-menu")).toBeVisible();
  const samples = await page.evaluate(async () => {
    const main = document.querySelector("main")!;
    const surface = document.querySelector<HTMLElement>(".island-surface")!;
    const result: {width:number; reported:number; settled:boolean}[] = [];
    for (let frame=0;frame<90;frame++) {
      if (frame===2) (document.querySelector('[data-testid="compact-mode"]') as HTMLElement).click();
      if (frame===6) (document.querySelector('[data-testid="expanded-mode"]') as HTMLElement).click();
      if (frame===10) (document.querySelector('[data-testid="shape-change"]') as HTMLElement).click();
      if (frame===14) (document.querySelector('[data-testid="style-change"]') as HTMLElement).click();
      await new Promise<void>(resolve => requestAnimationFrame(() => resolve()));
      const region=JSON.parse(main.getAttribute("data-region")!);
      result.push({width:parseFloat(surface.style.width),reported:region.geometry.width,settled:region.settled});
    }
    return result;
  });
  expect(samples.some(s => !s.settled)).toBe(true);
  expect(samples.every(s => s.width>0 && Math.abs(s.width-s.reported)<1)).toBe(true);
  await expect.poll(async () => JSON.parse((await page.locator("main").getAttribute("data-region"))!).settled).toBe(true);
  await expect(page.locator(".island-surface")).toHaveCSS("width","300px");
  await expect(page.locator(".island-surface")).toHaveCSS("border-radius","12px");
});

test("system reduced motion bypasses springs", async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html?animate=1");
  await expect.poll(async () => JSON.parse((await page.locator("main").getAttribute("data-region"))!).settled).toBe(true);
  await page.locator(".top-tools").click({position:{x:150,y:5}});
  await expect(page.locator(".island-surface")).toHaveCSS("width","80px");
});

test("clock and weather pages replace music and release DOM on collapse", async ({page}, info) => {
  await page.goto('/ui-tests/island-fixture.html?weather=1');
  const tools = page.locator('.feature-menu button');
  await tools.nth(5).click();
  await expect(page.locator('.clock-face time')).toHaveAttribute('aria-label','23:59');
  await expect(page.locator('.music-pane')).toHaveCount(0);
  await tools.nth(6).click();
  await expect(page.locator('.clock-face')).toHaveCount(0);
  await expect(page.locator('.current-weather')).toContainText('Shanghai');
  await expect(page.locator('.forecast > div')).toHaveCount(3);
  await page.screenshot({path:info.outputPath('weather.png')});
  await page.keyboard.press('Escape');
  await expect(page.locator('.info-panel')).toHaveCount(0);
  for(let i=0;i<12;i++) {
    await tools.nth(1).click();
    await tools.first().click();
    await page.keyboard.press('Escape');
  }
  await expect(page.locator('.page-body')).toHaveCount(1);
  await expect(page.locator('.volume-ruler, .timer-panel')).toHaveCount(0);
  await page.getByTestId('compact-mode').click();
  await expect(page.locator('.expanded, .feature-menu, .music-pane, .page-body')).toHaveCount(0);
  await page.getByTestId('expanded-mode').click();
  await expect(page.locator('.music-pane')).toBeVisible();
});

test("weather without a city offers settings",async({page})=>{
  await page.goto('/ui-tests/island-fixture.html');
  await page.locator('.feature-menu button').nth(6).click();
  await page.locator('.info-panel button').click();
  await expect(page.locator('main')).toHaveAttribute('data-action','settings');
});

test("overflow title animation is cancelled when music is unmounted",async({page})=>{
  await page.emulateMedia({reducedMotion:'no-preference'});
  await page.goto('/ui-tests/island-fixture.html?animate=1');
  await page.getByTestId('long-track').click();
  const title=page.locator('.marquee-title');
  await expect(title).toHaveClass(/overflow/);
  await expect.poll(()=>title.locator('span').evaluate(el=>el.getAnimations().length)).toBe(1);
  const handle=await title.locator('span').elementHandle();
  await page.locator('.feature-menu button').nth(1).click();
  await expect(title).toHaveCount(0);
  expect(await handle!.evaluate(el=>el.getAnimations().length)).toBe(0);
  await page.keyboard.press('Escape');
  await expect(title).toBeVisible();
  await page.emulateMedia({reducedMotion:'reduce'});
  await expect.poll(()=>title.locator('span').evaluate(el=>el.getAnimations().length)).toBe(0);
});
