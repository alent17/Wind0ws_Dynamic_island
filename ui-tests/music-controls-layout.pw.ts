import { test, expect } from "@playwright/test";

test.use({viewport:{width:1800,height:900}});

for (const source of ["netease", "spotify"]) test(`short music panes keep ${source} controls separated`, async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html");
  for (const expandedLength of [480, 520, 600]) {
    for (const edge of ["top", "right", "bottom", "left"]) {
      for (const shoulder of [18, 64]) {
        await page.evaluate(config => (window as any).configureIsland({
          ...config,mode:"expanded",style:"edge",offset:225,
          neteasePlaybackMode:"shuffle",neteaseFeatures:config.source === "netease" ? {
            progress:{source:"cdp",readable:true,controllable:true},like:{source:"cdp",readable:true,controllable:true},mode:{source:"cdp",readable:true,controllable:true},
          } : undefined,capabilities:{shuffle:true,repeat:true},
          widgets:{left:["weather"],right:["volume"]},scale:.625,
        }),{source,expandedLength,edge,shoulder});
        const row=page.locator(".control-row");
        await expect(page.locator(".stage")).toHaveAttribute("data-settled","true");
        const {boxes,bounds}=await row.evaluate(row=>{
          const r=row.getBoundingClientRect();
          return {bounds:{x:r.x,width:r.width},boxes:Array.from(row.querySelectorAll("button")).map(button=>{
            const r=button.getBoundingClientRect();
            return {label:button.getAttribute("aria-label"),left:r.left,right:r.right};
          }).sort((a,b)=>a.left-b.left)};
        });
        expect(boxes).toHaveLength(7);
        for(let i=0;i<boxes.length;i++){
          expect(boxes[i].left,JSON.stringify({source,expandedLength,edge,shoulder,boxes})).toBeGreaterThanOrEqual(bounds!.x-1);
          expect(boxes[i].right).toBeLessThanOrEqual(bounds!.x+bounds!.width+1);
          if(i)expect(boxes[i].left-boxes[i-1].right,JSON.stringify({source,expandedLength,edge,shoulder,boxes})).toBeGreaterThanOrEqual(2);
        }
      }
    }
  }
  await page.screenshot({path:`build/music-short-controls-${source}.png`});
});
