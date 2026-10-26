import { test, expect } from "@playwright/test";
test.use({viewport:{width:1800,height:900},reducedMotion:"no-preference"});
for (const layout of [{left:["clock"],right:["weather"]},{left:[],right:["clock","weather"]}]) {
  test(`layout reveals continuously from the center with ${layout.left.length} left cards`, async ({page}) => {
    await page.goto("/ui-tests/island-fixture.html");
    await page.evaluate(widgets => (window as any).configureIsland({mode:"expanded",offset:225,scale:.65,widgets}),layout);
    await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
    const sample = () => page.evaluate(async () => {
      const frames=[];
      for(let i=0;i<28;i++) {
        await new Promise(requestAnimationFrame);
        const s=document.querySelector(".island-surface")!.getBoundingClientRect();
        const m=document.querySelector(".music-pane")!.getBoundingClientRect();
        frames.push({center:s.x+s.width/2,width:s.width,musicCenter:m.x+m.width/2,musicWidth:m.width});
      }
      return frames;
    });
    for (const enabling of [false,true]) {
      await page.locator(".layout-toggle").click();
      const frames=await sample();
      expect(new Set(frames.map(frame=>Math.round(frame.width))).size).toBeGreaterThan(6);
      for(let i=1;i<frames.length;i++) {
        expect(Math.abs(frames[i].center-frames[0].center)).toBeLessThan(1);
        expect(Math.abs(frames[i].musicWidth-frames[0].musicWidth)).toBeLessThan(1);
        expect(Math.abs(frames[i].musicCenter-frames[i-1].musicCenter)).toBeLessThan(32);
        if(layout.left.length) expect(Math.abs(frames[i].musicCenter-frames[0].musicCenter)).toBeLessThan(1);
      }
      await expect(page.locator(".widget-card")).toHaveCount(enabling ? 2 : 0);
    }
  });
}
