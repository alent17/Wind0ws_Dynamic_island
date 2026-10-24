import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: ".",
  testMatch: "ui-tests/seek-app.pw.ts",
  fullyParallel: false,
  workers: 1,
  timeout: 60_000,
  reporter: "list",
  use: {
    baseURL: "http://127.0.0.1:1428",
    browserName: "chromium",
    launchOptions: { channel: "chrome", args: ["--no-proxy-server"] },
    viewport: { width: 1280, height: 760 },
    reducedMotion: "reduce",
    screenshot: "off",
    trace: "off",
  },
  webServer: {
    command: "npm run dev -- --host 127.0.0.1 --port 1428",
    url: "http://127.0.0.1:1428/ui-tests/lifecycle-fixture.html",
    reuseExistingServer: false,
    timeout: 30_000,
  },
});
