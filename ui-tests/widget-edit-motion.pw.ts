import { test, expect } from "@playwright/test";
test.use({viewport:{width:1800,height:900},reducedMotion:"no-preference"});
for (const side of ["left","right"]) test(`adding and removing ${side} widgets animates continuously`, async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => (window as any).configureIsland({mode:"expanded",offset:225,scale:.65}));
  await page.locator(".island-surface").focus();await page.keyboard.press("F2");
  await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
  const samples = () => page.evaluate(async()=>{
    const result=[];
    for(let i=0;i<85;i++) {
      const time = await new Promise<number>(resolve=>requestAnimationFrame(resolve));
      const s=document.querySelector(".island-surface")!.getBoundingClientRect();
      const m=document.querySelector(".music-pane")!.getBoundingClientRect();
      const cell=document.querySelector(".widget-cell")?.getBoundingClientRect();
      result.push({time,center:s.x+s.width/2,width:s.width,musicCenter:m.x+m.width/2,musicWidth:m.width,cellWidth:cell?.width ?? 0});
    }
    return result;
  });
  for(const remove of [false,true]) {
    const capture = samples();
    if(remove) await page.locator(".remove").click();
    else if(side === "right") await page.locator(".catalog-item").first().click();
    else {
      const source=await page.locator(".catalog-item").first().boundingBox();
      const target=await page.locator(".music-slot").boundingBox();
      await page.mouse.move(source!.x+source!.width/2,source!.y+20);await page.mouse.down();
      await page.mouse.move(target!.x+20,target!.y+90,{steps:10});await page.mouse.up();
    }
    const frames=await capture;
    expect(new Set(frames.map(frame=>Math.round(frame.width))).size).toBeGreaterThan(6);
    expect(new Set(frames.map(frame=>Math.round(frame.cellWidth))).size).toBeGreaterThan(6);
    for(let i=1;i<frames.length;i++) {
      expect(Math.abs(frames[i].center-frames[0].center)).toBeLessThan(1);
      expect(Math.abs(frames[i].musicWidth-frames[0].musicWidth)).toBeLessThan(1);
      expect(Math.abs(frames[i].musicCenter-frames[i-1].musicCenter),JSON.stringify(frames)).toBeLessThan(1.3*Math.max(16,frames[i].time-frames[i-1].time)+2);
    }
    await expect(page.locator(".widget-card")).toHaveCount(remove ? 0 : 1);
  }
});

test("removing during an entrance reverses smoothly and finishes with no stale card", async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => (window as any).configureIsland({mode:"expanded",offset:225,scale:.65}));
  await page.locator(".island-surface").focus();await page.keyboard.press("F2");
  await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
  await page.locator(".catalog-item").first().click();
  await page.waitForTimeout(90);
  await page.locator(".remove").evaluate((button:HTMLButtonElement)=>button.click());
  await expect(page.locator(".widget-cell")).toHaveCount(0);
  await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
  await expect(page.locator(".music-slot")).toHaveCSS("width","600px");
});

test("adding and removing an editor card use the same animation duration", async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => (window as any).configureIsland({mode:"expanded",offset:225,scale:.65}));
  await page.locator(".island-surface").focus();
  await page.keyboard.press("F2");
  await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
  const duration = () => page.locator(".widget-cell").evaluate(el => el.getAnimations().map(animation => animation.effect?.getTiming().duration).find(value => typeof value === "number" && value > 0) ?? 0);
  await page.locator(".catalog-item").first().click();
  await expect.poll(duration).toBe(420);
  await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
  await page.locator(".remove").click();
  await expect.poll(duration).toBe(420);
  await expect(page.locator(".widget-cell")).toHaveCount(0);
});

test("removing one of two cards and completing the editor keeps the music panel continuous", async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => (window as any).configureIsland({mode:"expanded",offset:225,scale:.65,widgets:{left:["timer"],right:["clock"]}}));
  await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
  await page.locator(".island-surface").focus();
  await page.keyboard.press("F2");
  await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");

  const capture = () => page.evaluate(async () => {
    const frames: {time:number;x:number;y:number;height:number}[] = [];
    for (let i = 0; i < 60; i++) {
      const time = await new Promise<number>(resolve => requestAnimationFrame(resolve));
      const rect = document.querySelector(".music-pane")!.getBoundingClientRect();
      frames.push({time,x:rect.x,y:rect.y,height:rect.height});
    }
    return frames;
  });
  const checkContinuity = (frames: {time:number;x:number;y:number;height:number}[]) => {
    for (let i = 1; i < frames.length; i++) {
      const allowance = 8 + 1.4 * (frames[i].time - frames[i-1].time);
      for (const key of ["x","y","height"] as const) {
        expect(Math.abs(frames[i][key] - frames[i-1][key]), `${key}: ${JSON.stringify(frames)}`).toBeLessThan(allowance);
      }
    }
  };

  const removing = capture();
  await page.locator('[data-widget-kind="clock"] .remove').click();
  checkContinuity(await removing);
  await expect(page.locator(".widget-card")).toHaveCount(1);
  await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
  const closing = capture();
  await page.locator(".edit-actions .done").click();
  checkContinuity(await closing);
  await expect(page.locator(".editor-bar")).toHaveCount(0);
  await expect(page.locator('[data-widget-kind="timer"]')).toHaveCount(1);
});

test("opening the editor keeps the music panel continuous and the controls clear", async ({page}) => {
  for (const widgets of [{left:[],right:[]},{left:[],right:["timer"]}]) {
    await page.goto("/ui-tests/island-fixture.html");
    await page.evaluate(widgets => (window as any).configureIsland({mode:"expanded",offset:225,scale:.65,expandedLength:480,widgets}), widgets);
    await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
    const frames = page.evaluate(async () => {
      const result: {time:number;y:number;height:number}[] = [];
      for (let i = 0; i < 65; i++) {
        const time = await new Promise<number>(resolve => requestAnimationFrame(resolve));
        const rect = document.querySelector(".music-pane")!.getBoundingClientRect();
        result.push({time,y:rect.y,height:rect.height});
      }
      return result;
    });
    await page.locator(".island-surface").focus();
    await page.keyboard.press("F2");
    const samples = await frames;
    for (let i = 1; i < samples.length; i++) {
      const allowance = 8 + 1.4 * (samples[i].time - samples[i-1].time);
      expect(Math.abs(samples[i].y - samples[i-1].y)).toBeLessThan(allowance);
      expect(Math.abs(samples[i].height - samples[i-1].height)).toBeLessThan(allowance);
    }
    const bar = await page.locator(".editor-bar").boundingBox();
    const catalog = await page.locator(".catalog").boundingBox();
    const actions = await page.locator(".edit-actions").boundingBox();
    expect(bar && catalog && actions).toBeTruthy();
    expect(catalog!.x + catalog!.width).toBeLessThanOrEqual(actions!.x + 1);
    expect(actions!.x + actions!.width).toBeLessThanOrEqual(bar!.x + bar!.width + 1);
    expect(await page.locator(".catalog").evaluate(el => el.scrollWidth <= el.clientWidth + 1)).toBe(true);
  }
});

for (const scenario of [
  {name:"music to one", initial:{left:[],right:[]}, add:["timer"], remove:[]},
  {name:"music to two", initial:{left:[],right:[]}, add:["timer","clock"], remove:[]},
  {name:"disabled layout to one", initial:{left:[],right:[]}, enabled:false, add:["timer"], remove:[]},
  {name:"disabled saved widget to one", initial:{left:[],right:["timer"]}, enabled:false, add:[], remove:[]},
  {name:"one to two", initial:{left:[],right:["timer"]}, add:["clock"], remove:[]},
  {name:"one to music", initial:{left:[],right:["timer"]}, add:[], remove:["timer"]},
  {name:"two to one", initial:{left:["clock"],right:["timer"]}, add:[], remove:["timer"]},
  {name:"two to music", initial:{left:["clock"],right:["timer"]}, add:[], remove:["clock","timer"]},
]) {
  for (const toolbar of [false, true]) test(`${scenario.name}: confirming with toolbar ${toolbar} does not jump`, async ({page}) => {
    await page.goto("/ui-tests/island-fixture.html");
    await page.evaluate(({initial,toolbar,enabled}) => (window as any).configureIsland({mode:"expanded",offset:225,scale:.65,widgets:initial,widgetsEnabled:enabled,toolbar}), {initial:scenario.initial,toolbar,enabled:"enabled" in scenario ? scenario.enabled : true});
    await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
    await page.locator(".island-surface").focus();
    await page.keyboard.press("F2");
    await expect(page.locator(".editor-bar")).toBeVisible();
    await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
    for (const kind of scenario.add) {
      await page.locator(`.catalog-item`).filter({has:page.locator(`.app-icon.${kind}`)}).click();
      await expect(page.locator(`[data-widget-kind="${kind}"]`)).toBeVisible();
      await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
    }
    for (const kind of scenario.remove) {
      await page.locator(`[data-widget-kind="${kind}"] .remove`).click();
      await expect(page.locator(`[data-widget-kind="${kind}"]`)).toHaveCount(0);
      await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
    }
    const samples = page.evaluate(async () => {
      const frames: {time:number;x:number;y:number;height:number}[] = [];
      for (let i=0;i<80;i++) {
        const time = await new Promise<number>(resolve=>requestAnimationFrame(resolve));
        const rect = document.querySelector(".music-pane")!.getBoundingClientRect();
        frames.push({time,x:rect.x,y:rect.y,height:rect.height});
      }
      return frames;
    });
    await page.locator(".edit-actions .done").click();
    const frames = await samples;
    for (let i=1;i<frames.length;i++) {
      const allowance = 8 + 1.4 * (frames[i].time - frames[i-1].time);
      for (const key of ["x","y","height"] as const) {
        expect(Math.abs(frames[i][key]-frames[i-1][key]), `${scenario.name}, toolbar ${toolbar}, ${key}: ${JSON.stringify(frames)}`).toBeLessThan(allowance);
      }
    }
    await expect(page.locator(".editor-bar")).toHaveCount(0);
    await expect(page.locator(".widget-card")).toHaveCount(scenario.initial.left.length + scenario.initial.right.length + scenario.add.length - scenario.remove.length);
  });
}

for (const edge of ["top", "right", "bottom", "left"] as const) {
  for (const widgets of [{left:[],right:[]},{left:[],right:["timer"]},{left:["clock"],right:["timer"]}]) {
    test(`${edge} editor entry and cancel stay continuous with ${widgets.left.length + widgets.right.length} widgets`, async ({page}) => {
      await page.goto("/ui-tests/island-fixture.html");
      await page.evaluate(({edge,widgets}) => (window as any).configureIsland({mode:"expanded",offset:225,scale:.65,edge,widgets,toolbar:true}), {edge,widgets});
      await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
      const capture = () => page.evaluate(async () => {
        const frames: {time:number;x:number;y:number;height:number}[] = [];
        for (let i=0;i<80;i++) {
          const time = await new Promise<number>(resolve => requestAnimationFrame(resolve));
          const rect = document.querySelector(".music-pane")!.getBoundingClientRect();
          frames.push({time,x:rect.x,y:rect.y,height:rect.height});
        }
        return frames;
      });
      const check = (frames:{time:number;x:number;y:number;height:number}[]) => {
        for (let i=1;i<frames.length;i++) {
          const allowance = 8 + 1.4 * (frames[i].time - frames[i-1].time);
          for (const key of ["x","y","height"] as const) expect(Math.abs(frames[i][key] - frames[i-1][key]), `${edge} ${key}`).toBeLessThan(allowance);
        }
      };
      const entering = capture();
      await page.locator(".island-surface").focus();
      await page.keyboard.press("F2");
      check(await entering);
      await expect(page.locator(".editor-bar")).toBeVisible();
      await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
      const cancelling = capture();
      await page.locator(".edit-actions .cancel").click();
      check(await cancelling);
      await expect(page.locator(".editor-bar")).toHaveCount(0);
      await expect(page.locator(".widget-card")).toHaveCount(widgets.left.length + widgets.right.length);
    });
  }
}
