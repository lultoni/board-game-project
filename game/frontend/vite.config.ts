import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Output to `dist/`, served statically by GitHub Pages (web build) or
// loaded by Tauri 2 from the same path (desktop build).
export default defineConfig({
  plugins: [svelte()],
  base: "./",
  build: {
    target: "es2022",
    outDir: "dist",
  },
  server: {
    port: 5173,
    strictPort: true,
  },
});
