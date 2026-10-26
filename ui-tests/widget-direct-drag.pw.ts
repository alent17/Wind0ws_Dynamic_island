import { test, expect, type Page, type Locator } from "@playwright/test";
test.use({viewport:{width:1800,height:900}});
const order=(page:Page)=>page.locator('.panels>[data-panel-id]').evaluateAll(nodes=>nodes.map(node=>(node as HTMLElement).dataset.panelId));
async function drag(page:Page,source:Locator,x:number,y:number) {
  const rect=await source.boundingBox();
  await page.mouse.move(rect!.x+rect!.width/2,rect!.y+rect!.height/2);
  await page.mouse.down();await page.mouse.move(x,y,{steps:20});await page.mouse.up();
}
async function open(page:Page,widgets={left:["clock"],right:["system"]}) {
  await page.goto('/ui-tests/island-fixture.html');
  await page.evaluate(widgets=>(window as any).configureIsland({mode:'expanded',offset:225,scale:.65,widgets}),widgets);
  await page.locator('.island-surface').focus();await page.keyboard.press('F2');
  await expect(page.locator('.editor-bar')).toBeVisible();
}
test('music moves directly across cards, saves and restores its order',async({page})=>{
  await open(page);
  await expect(page.locator('.drop-slot')).toHaveCount(0);
  await expect(page.locator('.music-slot .drag-dots i')).toHaveCount(3);
  const target=await page.locator('[data-panel-id="system"]').boundingBox();
  await drag(page,page.locator('.music-slot .grip'),target!.x+target!.width-15,target!.y+65);
  expect(await order(page)).toEqual(['clock','system','music']);
  await expect(page.locator('.drag-preview')).toHaveCount(0);
  await page.locator('.edit-actions .done').click();
  await page.locator('.island-surface').focus();await page.keyboard.press('F2');
  expect(await order(page)).toEqual(['clock','system','music']);
});
test('catalog cards drag directly into the row and Escape cancels a reorder',async({page})=>{
  await open(page,{left:[],right:[]});
  const music=await page.locator('.music-slot').boundingBox();
  await drag(page,page.locator('.catalog-item').first(),music!.x+20,music!.y+90);
  expect(await order(page)).toEqual(['timer','music']);
  const timer=await page.locator('[data-panel-id="timer"]').boundingBox();
  const grip=await page.locator('.music-slot .grip').boundingBox();
  await page.mouse.move(grip!.x+grip!.width/2,grip!.y+grip!.height/2);await page.mouse.down();
  await page.mouse.move(timer!.x+12,timer!.y+65,{steps:20});
  expect(await order(page)).toEqual(['music','timer']);
  await page.keyboard.press('Escape');await page.mouse.up();
  await expect(page.locator('.editor-bar')).toBeVisible();
  expect(await order(page)).toEqual(['timer','music']);
  await expect(page.locator('.drag-preview')).toHaveCount(0);
});
test('clicking a catalog function adds it immediately without a plus target',async({page})=>{
  await open(page,{left:[],right:[]});
  await page.locator('.catalog-item').first().click();
  expect(await order(page)).toEqual(['music','timer']);
  await page.locator('[data-panel-id="timer"] .card-tools button').first().click();
  expect(await order(page)).toEqual(['timer','music']);
});

test('a function card can be grabbed directly and moved across music without remounting',async({page})=>{
  await open(page);
  await page.locator('.widget-clock').evaluate(el=>(el as HTMLElement).dataset.identity='preserved');
  const target=await page.locator('[data-panel-id="system"]').boundingBox();
  await drag(page,page.locator('[data-widget-kind="clock"] .widget-content'),target!.x+target!.width-12,target!.y+110);
  expect(await order(page)).toEqual(['music','system','clock']);
  await expect(page.locator('.widget-clock')).toHaveAttribute('data-identity','preserved');
});
