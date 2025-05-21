// contains all helper functions to render UI to keep the update function in app.rs clean

use egui::{CentralPanel, SidePanel, Ui};

use crate::app::OverlayApp;

pub fn side_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    SidePanel::left("side_panel").show(ctx, |ui| {
        ui.heading("Tools");
        if ui.button("Increment").clicked() {
            state.counter += 1;
        }
        ui.separator();
        ui.label(format!("Count: {}", state.counter));
    });
}

pub fn main_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    CentralPanel::default().show(ctx, |ui| {
        ui.heading("Overlay Window");
        ui.label("Press Ctrl+F1 to toggle visibility");
        // … more UI …
    });
}
