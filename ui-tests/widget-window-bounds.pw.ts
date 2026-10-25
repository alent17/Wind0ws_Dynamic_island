import { test, expect } from "@playwright/test";

test.use({viewport:{width:960,height:540}});

test("a delayed native resize cannot clip the first animated editor frames", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const f=window as any;
    f.settingsOverrides={islandStyle:"edge",islandEdge:"top",edgeExpandedLength:660,enableAnimations:true,reduceAnimations:false,islandWidgets:{left:["weather"],right:["volume"]}};
    f.mountApp();
  });
  await expect(page.locator(".fixed-host")).toBeVisible();
  await page.evaluate(() => (window as any).expand());
  await page.locator(".island-surface").focus(); await page.keyboard.press("F2");
  await expect(page.locator(".editor-bar")).toBeVisible();
  await page.waitForTimeout(450);
  await page.setViewportSize({width:600,height:250});
  const samples=await page.evaluate(async () => {
    const samples=[];
    for(let i=0;i<8;i++){
      await new Promise(requestAnimationFrame);
      const rect=document.querySelector(".island-surface")!.getBoundingClientRect();
      samples.push({left:rect.x,right:rect.right,bottom:rect.bottom});
    }
    return samples;
  });
  for(const rect of samples){
    expect(rect.left).toBeGreaterThanOrEqual(-1);
    expect(rect.right).toBeLessThanOrEqual(601);
    expect(rect.bottom).toBeLessThanOrEqual(251);
  }
  await page.evaluate(() => (window as any).destroy());
});

for (const edge of ["top", "right", "bottom", "left"]) test(`native ${edge} editor stays within a high-DPI work area`, async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(edge => {
    const f=window as any;
    f.settingsOverrides={islandStyle:"edge",islandEdge:edge,edgeExpandedLength:900,enableAnimations:false,reduceAnimations:true,islandWidgets:{left:[],right:["timer"]}};
    f.monitorList=[{width:1920,height:1080,x:0,y:0,workX:0,workY:0,workWidth:1920,workHeight:1080,scaleFactor:2}];
    f.availableMonitorList=[{name:"test",position:{x:0,y:0},size:{width:1920,height:1080},scaleFactor:2}];
    f.boundsRequests=[];
    f.regionRequests=[];
    const invoke=f.__TAURI_INTERNALS__.invoke;
    f.__TAURI_INTERNALS__.invoke=(command:any,args:any,options:any)=>{
      if(command==="animate_window_bounds")f.boundsRequests.push(args);
      if(command==="set_island_interaction_region")f.regionRequests.push(args);
      return invoke(command,args,options);
    };
    f.mountApp();
  }, edge);
  await expect.poll(() => page.evaluate(() => (window as any).boundsRequests.length)).toBeGreaterThan(0);
  await page.evaluate(() => (window as any).expand());
  await page.locator(".island-surface").focus(); await page.keyboard.press("F2");
  await expect(page.locator(".editor-bar")).toBeVisible();
  const bounds=await page.evaluate(() => (window as any).boundsRequests.at(-1));
  expect(bounds.width).toBeLessThanOrEqual(1920);
  expect(bounds.height).toBeLessThanOrEqual(1080);
  const host=await page.locator(".fixed-host").boundingBox();
  const surface=await page.locator(".island-surface").boundingBox();
  expect(surface!.x).toBeGreaterThanOrEqual(host!.x-1);
  expect(surface!.x+surface!.width).toBeLessThanOrEqual(host!.x+host!.width+1);
  expect(surface!.y).toBeGreaterThanOrEqual(host!.y-1);
  expect(surface!.y+surface!.height).toBeLessThanOrEqual(host!.y+host!.height+1);
  await expect.poll(() => page.evaluate(({x,y}) => {
    const region=(window as any).regionRequests.at(-1);
    return !!region && Math.abs(region.x-x)<1.5 && Math.abs(region.y-y)<1.5;
  }, {x:surface!.x-host!.x,y:surface!.y-host!.y})).toBe(true);
  for (const selector of [".widget-card", ".edit-actions .done", ".edit-actions button:first-child"]) {
    const box=await page.locator(selector).boundingBox();
    expect(box!.x).toBeGreaterThanOrEqual(0);
    expect(box!.x+box!.width).toBeLessThanOrEqual(960);
    expect(box!.y+box!.height).toBeLessThanOrEqual(540);
  }
  // The native command is mocked, so make the real browser viewport lag behind
  // its requested window size, matching the clipped-window report.
  await page.setViewportSize({width:600,height:300});
  for (const selector of [".island-surface", ".widget-card", ".edit-actions .done"]) {
    await expect.poll(async () => {
      const box=await page.locator(selector).boundingBox();
      return !!box && box.x>=-1 && box.y>=-1 && box.x+box.width<=601 && box.y+box.height<=301;
    }).toBe(true);
  }
  await expect.poll(() => page.evaluate(() => {
    const region=(window as any).regionRequests.at(-1);
    const surface=document.querySelector(".island-surface")!.getBoundingClientRect();
    return !!region && Math.abs(region.x-surface.x)<1.5 && Math.abs(region.y-surface.y)<1.5
      && Math.abs(region.width-surface.width)<1.5 && Math.abs(region.height-surface.height)<1.5;
  })).toBe(true);
  const done=await page.locator(".edit-actions .done").boundingBox();
  await page.mouse.click(done!.x+done!.width/2,done!.y+done!.height/2);
  await expect(page.locator(".editor-bar")).toHaveCount(0);
  await page.evaluate(() => (window as any).destroy());
});

test("Studio reserves space for edited side panels and restores the saved layout", async ({page}) => {
  await page.setViewportSize({width:1280,height:900});
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const f=window as any;
    f.settingsOverrides={islandStyle:"edge",islandEdge:"top",enableAnimations:false,reduceAnimations:true,islandWidgets:{left:[],right:["timer"]}};
    f.mountStudio();
  });
  await expect(page.locator(".widget-card")).toHaveCount(1);
  await page.locator(".island-surface").focus(); await page.keyboard.press("F2");
  await expect(page.locator(".editor-bar")).toBeVisible();
  const stage=await page.locator(".stage").boundingBox();
  const host=await page.locator(".preview-host").boundingBox();
  const surface=await page.locator(".island-surface").boundingBox();
  expect(surface!.x).toBeGreaterThanOrEqual(stage!.x-1);
  expect(surface!.x+surface!.width).toBeLessThanOrEqual(stage!.x+stage!.width+1);
  expect(surface!.y+surface!.height).toBeLessThanOrEqual(stage!.y+stage!.height+1);
  expect(surface!.width).toBeLessThanOrEqual(host!.width);
  const done=await page.locator(".edit-actions .done").boundingBox();
  await page.mouse.click(done!.x+done!.width/2,done!.y+done!.height/2);
  await expect(page.locator(".editor-bar")).toHaveCount(0);
  await expect(page.locator(".widget-card")).toHaveCount(1);
  await page.evaluate(() => (window as any).destroy());
});
