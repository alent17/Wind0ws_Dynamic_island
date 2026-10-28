import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: ".",
  testMatch: ["netease-connection.pw.ts", "lifecycle.pw.ts", "music-controls-layout.pw.ts"],
  grep: /NetEase|short music panes/,
  workers: 1,
  timeout: 60_000,
  use: { baseURL: "http://127.0.0.1:1430", launchOptions: { channel: "chrome", args: ["--no-proxy-server"] } },
  webServer: {
    command: "node node_modules/vite/bin/vite.js --host 127.0.0.1 --port 1430",
    cwd: "..",
    url: "http://127.0.0.1:1430/ui-tests/lifecycle-fixture.html",
    reuseExistingServer: false,
    timeout: 30_000,
  },
});
