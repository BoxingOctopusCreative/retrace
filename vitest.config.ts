import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    // Tell Vite to use browser exports so Svelte compiles in DOM mode
    // (not SSR mode), which is required for onMount to run in tests.
    conditions: ["browser"],
  },
  test: {
    include: ["src/**/*.test.ts"],
    environment: "jsdom",
    globals: true,
    setupFiles: ["src/test-setup.ts"],
  },
});
