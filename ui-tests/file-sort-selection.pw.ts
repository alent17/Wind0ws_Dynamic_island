import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";

test("selected date grouping keeps its blue background in the main island", async ({ page }) => {
  await page.goto("/ui-tests/island-fixture.html");
  await page.addStyleTag({ content: readFileSync("src/app.css", "utf8") });
  await page.evaluate(() => (window as any).configureIsland({ mode: "expanded", toolbar: true, scale: .625 }));
  await page.getByRole("button", { name: /文件分类|File sorting|ファイル整理/ }).click();
  const selected = page.getByRole("button", { name: "不按日期" });
  await expect(selected).toHaveAttribute("aria-pressed", "true");
  await expect(selected).toHaveCSS("background-color", "rgb(0, 47, 167)");
  await expect(selected).toHaveCSS("color", "rgb(255, 255, 255)");
  await selected.hover();
  await expect(selected).toHaveCSS("background-color", "rgb(0, 47, 167)");
  await page.keyboard.press("Tab");
  await selected.focus();
  await expect(selected).toHaveCSS("background-color", "rgb(0, 47, 167)");
});
