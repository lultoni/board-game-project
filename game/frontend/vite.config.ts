import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

// SvelteKit in SPA mode (adapter-static + ssr=false + prerender=true).
// Output is a relative-path static site that ships to GitHub Pages (web)
// and is bundled by Tauri 2 (desktop). Same build, two targets.
export default defineConfig({
  plugins: [sveltekit()],
  server: {
    port: 5173,
    strictPort: true,
    fs: {
      // Allow Vite to serve the wasm-pack output that lives outside
      // the frontend root (../crates/wasm_wrapper/pkg/).
      allow: [".."],
    },
  },
  // The wasm module ships its own glue + .wasm; treat it as an asset boundary.
  optimizeDeps: {
    exclude: ["wasm_wrapper"],
  },
  worker: {
    format: "es",
  },
});
