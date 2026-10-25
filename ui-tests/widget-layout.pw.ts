import { test, expect, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";

test.use({ viewport: {width:1800,height:900} });

test("larger editing controls keep a red circular remove button under production styles", async ({page}) => {
  await openEditor(page);
  await page.addStyleTag({content:readFileSync("src/app.css", "utf8").replace(/^@import[^\n]*\n/, "")});
  await page.evaluate(() => (window as any).configureIsland({widgets:{left:["timer"],right:[]}}));
  // Reopen to populate the editor draft from the saved layout.
  await page.keyboard.press("Escape");
  await page.locator(".island-surface").focus(); await page.keyboard.press("F2");
  const remove = page.locator(".remove");
  await expect(remove).toHaveCSS("width", "36px");
  await expect(remove).toHaveCSS("height", "36px");
  await expect(remove).toHaveCSS("border-top-left-radius", "50%");
  await expect(remove).toHaveCSS("background-color", "rgb(197, 59, 59)");
  await remove.hover();
  await expect(remove).toHaveCSS("background-color", "rgb(226, 75, 69)");
  await remove.focus();
  await expect(remove).toHaveCSS("background-color", "rgb(226, 75, 69)");
  await expect(page.locator(".edit-actions .done")).toHaveCSS("width", "52px");
  await expect(page.locator(".app-icon").first()).toHaveCSS("width", "52px");
  await expect(page.locator(".grip")).toHaveCSS("min-height", "44px");
  await expect(page.locator(".drop-slot").first()).toHaveCSS("width", "36px");
  await expect(page.locator(".editor-bar")).toHaveCSS("background-color", "rgb(0, 0, 0)");
  const tools = await page.locator(".card-tools").boundingBox();
  const content = await page.locator('[data-widget-kind="timer"] .widget-content').boundingBox();
  expect(tools!.y + tools!.height).toBeLessThanOrEqual(content!.y);
  await page.screenshot({path:"build/widget-editor-large-controls.png"});
  await remove.click();
  await expect(page.locator(".widget-card")).toHaveCount(0);
});

test("side functions use larger panels, readouts and rulers without shrinking music", async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => (window as any).configureIsland({mode:"expanded", offset:225, widgets:{left:["timer"],right:["volume"]}}));
  for (const kind of ["timer", "volume"]) {
    const card = page.locator(`[data-widget-kind="${kind}"]`);
    await expect(card).toHaveCSS("width", "480px");
    await expect(card.locator(".widget-content")).toHaveCSS("width", "432px");
    await expect(card.locator(`.${kind}-readout strong`)).toHaveCSS("font-size", "60px");
    await expect(card.locator(`.${kind}-ruler`)).toHaveCSS("height", "96px");
    await expect(card.locator(kind === "timer" ? ".timer-panel" : ".panel-volume")).not.toHaveClass(/compact/);
    await expect(card.locator(`.${kind}-footer`)).toHaveCSS("flex-direction", "row");
  }
  const music = await page.locator(".music-slot").boundingBox();
  const side = await page.locator(".widget-card").first().boundingBox();
  expect(side!.width / music!.width).toBeCloseTo(0.8);
  expect(side!.height).toBeCloseTo(music!.height);
  await page.screenshot({path:"build/widget-reference-size.png"});
});

async function openEditor(page: Page) {
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => (window as any).configureIsland({mode: "expanded", style: "floating", offset:225}));
  await page.locator(".island-surface").focus();
  await page.keyboard.press("F2");
  await expect(page.locator(".editor-bar")).toBeVisible();
}
async function add(page: Page, kind: string, side: string) {
  await page.locator(`.catalog-item`).filter({has: page.locator(`.app-icon.${kind}`)}).click();
  await page.locator(`.side[data-widget-side="${side}"] .drop-slot.last`).click();
}

test("two same-side widgets, limit, reordering, save and cancel with a black toolbar", async ({page}) => {
  await openEditor(page);
  await expect(page.locator(".editor-bar")).toHaveCSS("background-color", "rgb(0, 0, 0)");
  await expect(page.locator(".editor-bar")).toHaveCSS("background-image", "none");
  await add(page, "timer", "left");
  await add(page, "weather", "left");
  await expect(page.locator('.side[data-widget-side="left"] .widget-card')).toHaveCount(2);
  await expect(page.locator('.catalog-item').filter({has: page.locator('.app-icon.clock')})).toBeDisabled();
  await page.locator('[data-widget-kind="timer"] .card-tools button').last().click();
  await expect(page.locator('.side[data-widget-side="left"] .widget-card').first()).toHaveAttribute("data-widget-kind", "weather");
  await page.locator(".edit-actions .done").click();
  await expect(page.locator(".editor-bar")).toHaveCount(0);
  await expect(page.locator(".widget-card")).toHaveCount(2);
  await page.locator(".island-surface").focus(); await page.keyboard.press("F2");
  await page.locator('[data-widget-kind="timer"] .remove').click();
  await page.keyboard.press("Escape");
  await expect(page.locator(".widget-card")).toHaveCount(2);
});

test("drag across music and keep draft after save failure", async ({page}) => {
  await openEditor(page);
  const source = await page.locator('.catalog-item').filter({has: page.locator('.app-icon.timer')}).boundingBox();
  const target = await page.locator('.side[data-widget-side="right"] .drop-slot').boundingBox();
  if (!source || !target) throw new Error("Missing drag bounds");
  await page.mouse.move(source.x + source.width / 2, source.y + 15); await page.mouse.down();
  await page.mouse.move(target.x + target.width / 2, target.y + target.height / 2, {steps: 12}); await page.mouse.up();
  await expect(page.locator('.side[data-widget-side="right"] .widget-card')).toHaveCount(1);
  await page.evaluate(() => (window as any).configureIsland({saveFails: true}));
  await page.locator(".edit-actions .done").click();
  await expect(page.locator(".editor-bar")).toBeVisible();
  await expect(page.locator(".hint")).toContainText(/保存|saved|保存/);
  await page.evaluate(() => (window as any).configureIsland({saveFails: false}));
  await page.locator(".edit-actions .done").click();
  await expect(page.locator(".editor-bar")).toHaveCount(0);
});

test("long press enters after two seconds; early release, movement and controls do not", async ({page}) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.evaluate(() => (window as any).configureIsland({mode: "expanded"}));
  const pane = await page.locator(".music-pane").boundingBox();
  if (!pane) throw new Error("Missing music pane");
  const x = pane.x + pane.width / 2, y = pane.y + 8;
  await page.mouse.move(x, y); await page.mouse.down(); await page.waitForTimeout(200); await page.mouse.up();
  await expect(page.locator(".editor-bar")).toHaveCount(0);
  await page.evaluate(() => (window as any).configureIsland({mode: "expanded"}));
  await page.mouse.move(x, y); await page.mouse.down(); await page.mouse.move(x + 12, y); await page.waitForTimeout(2100); await page.mouse.up();
  await expect(page.locator(".editor-bar")).toHaveCount(0);
  const play = await page.locator(".transport-controls .play").boundingBox();
  if (!play) throw new Error("Missing play control");
  await page.mouse.move(play.x + 20, play.y + 20); await page.mouse.down(); await page.waitForTimeout(2100); await page.mouse.up();
  await expect(page.locator(".editor-bar")).toHaveCount(0);
  await page.mouse.move(x, y); await page.mouse.down();
  await expect(page.locator(".editor-bar")).toBeVisible({timeout: 4000}); await page.mouse.up();
  await expect(page.locator(".editor-bar")).toBeVisible();
});

test("composed panels fit all edges, restore after collapse, and discard hidden drafts", async ({page}) => {
  await openEditor(page);
  await page.locator('.edit-actions button').first().click();
  await page.evaluate(() => (window as any).configureIsland({style:"edge",scale:.625}));
  await page.locator(".island-surface").focus(); await page.keyboard.press("F2");
  for (const selector of ['.catalog-item:has(.app-icon.timer)', '.side[data-widget-side="left"] .drop-slot.last']) {
    const box = await page.locator(selector).boundingBox();
    if (!box) throw new Error("Missing empty-layout editor control");
    await page.mouse.click(box.x + box.width/2, box.y + box.height/2);
  }
  await expect(page.locator(".widget-card")).toHaveCount(1);
  await page.keyboard.press("Escape");
  await expect(page.locator(".widget-card")).toHaveCount(0);
  for (const edge of ["top", "left", "right", "bottom"]) {
    for (const style of ["floating", "edge"]) {
      await page.evaluate(({edge, style}) => (window as any).configureIsland({edge, style, scale: .625, widgets: {left: ["clock"], right: ["volume"]}}), {edge, style});
      const cards = page.locator(".widget-card");
      await expect(cards).toHaveCount(2);
      const surface = await page.locator(".island-surface").boundingBox();
      if (!surface) throw new Error("Missing surface");
      for (const card of await cards.all()) {
        const box = await card.boundingBox();
        if (!box) throw new Error("Missing card");
        expect(box.x).toBeGreaterThanOrEqual(surface.x - 1);
        expect(box.x + box.width).toBeLessThanOrEqual(surface.x + surface.width + 1);
        expect(box.y + box.height).toBeLessThanOrEqual(surface.y + surface.height + 1);
      }
      await page.locator(".island-surface").focus(); await page.keyboard.press("F2");
      for (const remove of await page.locator(".remove").all()) {
        const hit = await remove.evaluate(element => {
          const box = element.getBoundingClientRect();
          const hit = document.elementFromPoint(box.x + box.width / 2, box.y + box.height / 2);
          return {works: element.contains(hit), hit: hit?.outerHTML.slice(0,200), x:box.x,y:box.y};
        });
        expect(hit.works, JSON.stringify({edge,style,hit})).toBe(true);

      }
      const doneHit = await page.locator(".edit-actions .done").evaluate(element => {
        const box = element.getBoundingClientRect();
        return element.contains(document.elementFromPoint(box.x + box.width/2, box.y + box.height/2));
      });
      expect(doneHit, `${style}/${edge} save button`).toBe(true);
      const removeBox = await page.locator(".remove").last().boundingBox();
      if (!removeBox) throw new Error("Missing remove button");
      await page.mouse.click(removeBox.x + removeBox.width/2, removeBox.y + removeBox.height/2);
      await expect(page.locator(".widget-card")).toHaveCount(1);
      await page.keyboard.press("Escape");
    }
  }
  await page.evaluate(() => (window as any).configureIsland({edge:"top", style:"edge", scale:1, widgets:{left:["timer"],right:["clock"]}}));
  await page.locator(".island-surface").focus(); await page.keyboard.press("F2");
  await page.screenshot({path: "build/widget-editor-preview.png"});
  await page.locator('[data-widget-kind="timer"] .remove').click();
  await page.evaluate(() => (window as any).configureIsland({visible:false}));
  await expect(page.locator(".editor-bar")).toHaveCount(0);
  await page.evaluate(() => (window as any).configureIsland({visible:true,mode:"expanded"}));
  await expect(page.locator(".widget-card")).toHaveCount(2);
  await page.evaluate(() => (window as any).configureIsland({mode:"compact"}));
  await page.evaluate(() => (window as any).configureIsland({mode:"expanded"}));
  await expect(page.locator(".widget-card")).toHaveCount(2);
});
