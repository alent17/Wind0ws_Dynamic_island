import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./ui-tests",
  testMatch: "**/*.pw.ts",
  fullyParallel: true,
  reporter: "list",
  use: {
    baseURL: "http://127.0.0.1:1422",
    browserName: "chromium",
    launchOptions: { channel: "chrome" },
    viewport: { width: 1280, height: 760 },
    reducedMotion: "reduce",
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
  },
  webServer: {
    command: "pnpm dev --host 127.0.0.1 --port 1422",
    url: "http://127.0.0.1:1422/ui-tests/island-fixture.html",
    reuseExistingServer: !process.env.CI,
    timeout: 30_000,
  },
});
