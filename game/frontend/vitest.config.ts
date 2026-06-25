import { defineConfig } from "vitest/config";
import { svelte, vitePreprocess } from "@sveltejs/vite-plugin-svelte";

// Run `.svelte.ts` runes files through the Svelte preprocessor so vitest can
// import them. Without this, `$state`/`$derived`/`$effect` are undefined at
// import time and any test that touches a runed module crashes on load.
// `hot: false` keeps the test runner from spinning up an HMR server.
export default defineConfig({
  plugins: [
    svelte({
      preprocess: vitePreprocess(),
      hot: false,
    }),
  ],
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
    setupFiles: ["./vitest.setup.ts"],
  },
});
