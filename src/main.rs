mod app;
mod backend;
mod color_palette;
mod gui;

use app::OverlayApp;
use eframe::{CreationContext, NativeOptions, run_native};

fn main() {
    // Configure eframe (window title, transparency will be set native-side per-platform)
    let native_opts: NativeOptions = NativeOptions {
        window_builder: Some(Box::new(|builder| {
            builder
                .with_maximized(true)
                // remove window decorations (titlebar, borders):
                .with_decorations(true)
                // allow per-pixel transparency:
                .with_transparent(true)
                .with_mouse_passthrough(false)
                // force the window above all others:
                .with_always_on_top()
        })),
        ..Default::default()
    };

    run_native(
        "PokeMMO-Companion", // window/app title
        native_opts,
        Box::new(|cc: &CreationContext| {
            // This is called once at startup. Wrap your App in Ok(...)
            Ok(Box::new(OverlayApp::new(cc)))
        }),
    )
    .expect("failed to start eframe");
}
