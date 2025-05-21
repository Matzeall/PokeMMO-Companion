mod app;
mod backend;
mod gui;

use app::OverlayApp;
use eframe::{CreationContext, NativeOptions, run_native};

fn main() {
    // Configure eframe (window title, transparency will be set native-side per-platform)
    let mut native_opts = NativeOptions::default();
    // configure the winit WindowBuilder directly:
    native_opts.window_builder = Some(Box::new(|builder| {
        builder
            // remove window decorations (titlebar, borders):
            .with_decorations(false)
            // allow per-pixel transparency:
            .with_transparent(true)
            // force the window above all others:
            .with_always_on_top()
    }));

    run_native(
        "My Overlay", // window/app title
        native_opts,
        Box::new(|cc: &CreationContext| {
            // This is called once at startup. Wrap your App in Ok(...)
            Ok(Box::new(OverlayApp::new(cc)))
        }),
    )
    .expect("failed to start eframe");
}
