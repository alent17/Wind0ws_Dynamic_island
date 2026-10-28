import { test, expect } from "@playwright/test";

test("folder drop waits for confirmation; file drops run immediately", async ({ page }) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const w = window as any;
    w.isTauri = true;
    w.settingsOverrides = { fileSortDirectory: "C:\\Output", showCustomFunctionPanel: true };
    w.sortRequests = [];
    const invoke = w.__TAURI_INTERNALS__.invoke;
    w.__TAURI_INTERNALS__.invoke = (command: string, args: any, options: any) => {
      if (command === "inspect_sort_inputs") return Promise.resolve(args.paths.filter((p: string) => p.endsWith("folder")));
      if (command === "sort_dropped_files") { w.sortRequests.push(args); return Promise.resolve([]); }
      return invoke(command, args, options);
    };
    w.mountApp();
  });
  await page.waitForTimeout(1000);
  const drop = async (paths: string[]) => page.evaluate(async (paths) => {
    await (window as any).__TAURI_INTERNALS__.invoke("plugin:event|emit", {event:"tauri://drag-drop",payload:{paths,position:{x:0,y:0}}});
  }, paths);
  await drop(["C:\\folder", "C:\\file.txt"]);
  await expect(page.getByText("确认分类文件夹？")).toBeVisible();
  expect(await page.evaluate(() => (window as any).sortRequests.length)).toBe(0);
  await page.getByRole("button", { name: "取消", exact: true }).click();
  expect(await page.evaluate(() => (window as any).sortRequests.length)).toBe(0);
  await drop(["C:\\folder"]);
  await page.getByRole("button", { name: "确认分类" }).click();
  await expect.poll(() => page.evaluate(() => (window as any).sortRequests.length)).toBe(1);
  expect(await page.evaluate(() => (window as any).sortRequests[0].confirmedFolders)).toEqual(["C:\\folder"]);
  await drop(["C:\\file.txt"]);
  await expect.poll(() => page.evaluate(() => (window as any).sortRequests.length)).toBe(2);
  expect(await page.evaluate(() => (window as any).sortRequests[1].confirmedFolders)).toEqual([]);
});
