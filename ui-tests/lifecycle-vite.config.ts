import { defineConfig } from "vite";
import base from "../vite.config.js";
export default defineConfig(async () => ({
  ...await base(),
  cacheDir: "dist/performance/m5-vite-cache",
  build: {outDir:"dist/performance/m5-fixture",rollupOptions:{input:"ui-tests/lifecycle-fixture.html"}},
  server: {host:"127.0.0.1",port:1424,strictPort:true},
}));
