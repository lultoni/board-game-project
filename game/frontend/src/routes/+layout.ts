// SPA mode: render purely client-side. WASM and Tauri IPC both require a
// browser-like runtime, so SSR would only be dead weight.
export const ssr = false;
export const prerender = true;
export const trailingSlash = "always";
