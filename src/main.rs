mod app;
mod backend;
mod frontend;
mod gui;
mod style;
mod utils;

use app::{APP_ID, OverlayApp};
use eframe::{CreationContext, NativeOptions, run_native};

fn main() {
    // Configure eframe (window title, transparency will be set native-side per-platform)
    let native_opts: NativeOptions = NativeOptions {
        window_builder: Some(Box::new(|builder| {
            builder
                .with_maximized(true)
                // remove window decorations (titlebar, borders):
                .with_decorations(false)
                // allow per-pixel transparency:
                .with_transparent(true)
                .with_mouse_passthrough(false)
                // force the window above all others:
                .with_always_on_top()
        })),
        ..Default::default()
    };

    run_native(
        APP_ID, // window/app title
        native_opts,
        Box::new(|cc: &CreationContext| {
            // This is called once at startup. Wrap your App in Ok(...)
            Ok(Box::new(OverlayApp::new(cc)))
        }),
    )
    .expect("failed to start eframe");
}
