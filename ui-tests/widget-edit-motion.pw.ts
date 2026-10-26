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
