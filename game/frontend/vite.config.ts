import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

// SvelteKit in SPA mode (adapter-static + ssr=false + prerender=true).
// Output is a relative-path static site bundled by Tauri 2 (desktop only).
export default defineConfig({
  plugins: [sveltekit()],
  server: {
    port: 5173,
    strictPort: true,
  },
  worker: {
    format: "es",
  },
});
