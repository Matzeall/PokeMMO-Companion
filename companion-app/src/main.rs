#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows" // treat as pure gui app, omit the console
)]

mod app;
mod backend;
mod frontend;
mod utils;

use std::sync::Arc;

use app::{APP_ID, OverlayApp};
use eframe::{CreationContext, NativeOptions, run_native};

fn main() {
    let icon_data = eframe::icon_data::from_png_bytes(include_bytes!(
        "../assets/icons/pokemmo-companion-main.png"
    ))
    .expect("The icon data must be valid");

    let mut native_opts: NativeOptions = NativeOptions {
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
            // .with_icon(icon_data) // doesn't work like it should
        })),
        ..Default::default()
    };
    native_opts.viewport.icon = Some(Arc::new(icon_data)); // fix for .with_icon issue

    run_native(
        APP_ID,
        native_opts,
        Box::new(|cc: &CreationContext| Ok(Box::new(OverlayApp::new(cc)))),
    )
    .expect("failed to start eframe");
}
