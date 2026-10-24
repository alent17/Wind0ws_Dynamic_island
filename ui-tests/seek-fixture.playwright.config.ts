import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: ".",
  testMatch: "ui-tests/seek-main-island.pw.ts",
  fullyParallel: false,
  workers: 1,
  reporter: "list",
  use: {
    baseURL: "http://127.0.0.1:1427",
    browserName: "chromium",
    launchOptions: { channel: "chrome", args: ["--no-proxy-server"] },
    viewport: { width: 1280, height: 760 },
    reducedMotion: "reduce",
    screenshot: "off",
    trace: "off",
  },
  webServer: {
    command: "npm run dev -- --host 127.0.0.1 --port 1427",
    url: "http://127.0.0.1:1427/ui-tests/seek-main-island.html",
    reuseExistingServer: false,
    timeout: 30_000,
  },
});
