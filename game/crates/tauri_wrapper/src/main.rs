fn main() {
    // On Linux/webkit2gtk, set env vars before the webview is created.
    // WEBKIT_DISABLE_DMABUF_RENDERER: fixes CSS animation lag on wlroots
    //   compositors (Hyprland, Sway) where webkit2gtk's DMABUF path fails to
    //   negotiate buffer formats, silently falling back to a slow software path.
    // WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS: the bubblewrap sandbox blocks
    //   the PipeWire socket, causing AudioContext to silently fail. Upstream
    //   WebKit bug #239682; no landed fix as of 2026-06.
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        std::env::set_var("WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS", "1");
    }

    tauri_wrapper::run();
}
