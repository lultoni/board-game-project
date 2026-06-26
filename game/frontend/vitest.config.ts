import { defineConfig } from "vitest/config";
import { svelte, vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import { fileURLToPath } from "node:url";

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
  resolve: {
    alias: {
      // Mirror SvelteKit's `$lib` alias (declared in .svelte-kit/tsconfig.json)
      // so files that use `import ... from "$lib/..."` resolve under vitest.
      $lib: fileURLToPath(new URL("./src/lib", import.meta.url)),
    },
  },
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
    setupFiles: ["./vitest.setup.ts"],
  },
});
