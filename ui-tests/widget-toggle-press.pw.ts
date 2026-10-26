import { test, expect } from "@playwright/test";
test.use({viewport:{width:1800,height:900},reducedMotion:"no-preference"});
for(const style of ["floating","edge"])test(`${style} layout toggle does not press or shake neighboring controls and spectrum`,async({page})=>{
  await page.goto('/ui-tests/island-fixture.html');
  await page.evaluate(style=>(window as any).configureIsland({mode:'expanded',style,offset:225,scale:.65,animate:true,widgets:{left:['clock'],right:['weather']}}),style);
  await expect(page.locator('.stage')).toHaveAttribute('data-settled','true');
  await page.locator('.music-pane .spectrum-endpoint canvas').evaluate(canvas=>{
    (canvas as HTMLElement).dataset.identity='stable-spectrum';
    (window as any).canvasResizes=0;
    new MutationObserver(records=>(window as any).canvasResizes+=records.length).observe(canvas,{attributes:true,attributeFilter:['width','height']});
  });
  const sample=()=>page.evaluate(()=>[...document.querySelectorAll('.transport-controls button,.music-pane .spectrum-endpoint,.music-pane .progress-block')].map(node=>{
    const r=node.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};
  }));
  const before=await sample();
  const button=await page.locator('.layout-toggle').boundingBox();
  await page.mouse.move(button!.x+button!.width/2,button!.y+button!.height/2);await page.mouse.down();
  await page.waitForTimeout(180);
  const pressed=await sample();
  for(let i=0;i<before.length;i++)for(const key of ['x','y','width','height'] as const)expect(Math.abs(pressed[i][key]-before[i][key])).toBeLessThan(.5);
  await page.mouse.up();
  const frames=await page.evaluate(async()=>{
    const values=[];
    for(let i=0;i<32;i++){
      await new Promise(requestAnimationFrame);
      values.push([...document.querySelectorAll('.transport-controls button,.music-pane .spectrum-endpoint,.music-pane .progress-block')].map(node=>{
        const r=node.getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height};
      }));
    }
    return values;
  });
  for(const frame of frames)for(let i=0;i<frame.length;i++)for(const key of ['x','y','width','height'] as const)expect(Math.abs(frame[i][key]-before[i][key])).toBeLessThan(.6);
  await expect(page.locator('.music-pane .spectrum-endpoint canvas')).toHaveAttribute('data-identity','stable-spectrum');
  expect(await page.evaluate(()=>(window as any).canvasResizes)).toBe(0);
});
