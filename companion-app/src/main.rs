#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows" // treat as pure gui app, omit the console
)]

mod app;
mod backend;
mod frontend;
mod utils;

use app::{APP_ID, OverlayApp};
use eframe::{CreationContext, NativeOptions, run_native};

fn main() {
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
                // .with_icon( )
                .with_taskbar(true)
                .with_title(APP_ID)
                .with_app_id(APP_ID)
        })),
        ..Default::default()
    };

    run_native(
        APP_ID,
        native_opts,
        Box::new(|cc: &CreationContext| Ok(Box::new(OverlayApp::new(cc)))),
    )
    .expect("failed to start eframe");
}
