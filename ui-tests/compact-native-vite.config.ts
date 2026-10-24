import { defineConfig } from "vite";
import base from "../vite.config.js";
export default defineConfig(async () => ({
  ...await base(),
  publicDir: false,
  cacheDir: "dist/performance/compact-native-vite-cache",
  server: {host:"127.0.0.1",port:1426,strictPort:true},
  build: {outDir:"dist/performance/compact-native",rollupOptions:{input:"ui-tests/compact-native-fixture.html"}},
}));
