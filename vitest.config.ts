import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { resolve } from "node:path";

export default defineConfig({
  plugins: [svelte()],
  resolve: { alias: { $lib: resolve(import.meta.dirname, "src/lib") } },
  test: { include: ["src/**/*.test.ts"] },
});
