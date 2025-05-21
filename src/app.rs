// defines runtime data struct -> holds mutable application state
// is instantiated by main.rs

use eframe::{App, CreationContext, Frame};
use egui::Context;
use std::sync::mpsc::{Receiver, Sender};

use crate::gui;

pub struct OverlayApp {
    // your state here
    pub counter: u32,
    // optional channel for platform messages (hotkeys, toggles…)
    // plat_tx: Sender<PlatformMessage>,
    // plat_rx: Receiver<PlatformMessage>,
}

impl OverlayApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        // Here you could spawn a background thread to register
        // RegisterHotKey (windows), XGrabKey (X11), or layer-shell listener (Wayland).
        //
        // Send events back via a channel that you poll in update().
        //
        Self {
            counter: 0, // , plat_tx, plat_rx
        }
    }
}

impl App for OverlayApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        // Poll any platform messages:
        // while let Ok(msg) = self.plat_rx.try_recv() { … }

        gui::side_panel(ctx, self);
        gui::main_panel(ctx, self);

        // request repaint if you want a live overlay:
        ctx.request_repaint();
    }
}
