import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir:".",testMatch:"lifecycle.pw.ts",workers:1,timeout:60000,
  use:{baseURL:"http://127.0.0.1:1424",launchOptions:{channel:"chrome",args:["--no-proxy-server"]}},
  webServer:{command:"node scripts/m5-serve-fixture.mjs",cwd:"..",url:"http://127.0.0.1:1424/ui-tests/lifecycle-fixture.html",timeout:60000,reuseExistingServer:true},
});
