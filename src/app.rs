// defines runtime data struct -> holds mutable application state
// is instantiated by main.rs

// TODO:
// 1. Make notes layouter more robust with overlapping tags
// 2. update regex patterns in notes layouter
// 3. different Heading, italic & underlined colors?
// 4. persist notes -> serde or manual?
// 5. Finally get to the main app focus issue -> mouse passthrough & unfocussed input events
// 6. Bring some nice notes layout things over to resources
use eframe::{App, CreationContext, Frame};

use crate::{
    backend::{
        feature_state::FeatureSubsystem, notes_feature::NotesSubsystem,
        ressources_feature::RessourcesSubsystem,
    },
    gui::{self, GuiSubsystem},
};
use egui::Context;

/// Focused when interacting with any of the windows
/// Unfocused when seeing the overlay but interacting with something underneath
/// Hidden when the whole overlay is hidden and needs to be manually unhidden before seeing anything
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum FocusState {
    Focused,
    Unfocused,
    Hidden,
}

impl FocusState {
    pub fn is_focused(&self) -> bool {
        *self == FocusState::Focused
    }
    pub fn is_unfocused(&self) -> bool {
        *self == FocusState::Unfocused
    }
    pub fn is_hidden(&self) -> bool {
        *self == FocusState::Hidden
    }
}

pub struct OverlayApp {
    pub app_focus: FocusState,

    pub features: FeatureSubsystem,

    pub gui: GuiSubsystem,

    pub ressources: RessourcesSubsystem,

    pub notes: NotesSubsystem,
}

impl OverlayApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        // Here I could spawn a background thread to register
        // RegisterHotKey (windows), XGrabKey (X11), or layer-shell listener (Wayland).
        //
        // Send events back via a channel that are polled in update().

        Self {
            app_focus: FocusState::Unfocused,
            features: FeatureSubsystem::new(),
            gui: GuiSubsystem::new(cc),
            ressources: RessourcesSubsystem::new(),
            notes: NotesSubsystem::new(),
        }
    }

    fn update_app_focus(&mut self, ctx: &Context) {
        let aspired_state: FocusState = if ctx.is_pointer_over_area() {
            FocusState::Focused
        } else {
            FocusState::Unfocused
        };

        match (self.app_focus, aspired_state) {
            (FocusState::Focused, FocusState::Unfocused) => self.app_focus = aspired_state,
            (FocusState::Unfocused, FocusState::Focused) => self.app_focus = aspired_state,
            (_, _) => {}
        }
    }
}

// main egui update loop
impl App for OverlayApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        // Poll any platform messages:
        // while let Ok(msg) = self.plat_rx.try_recv() { … }

        self.update_app_focus(ctx);

        self.features
            .handle_feature_state_input(ctx.input(|i| i.clone()));

        if ctx.input(|i| i.key_down(egui::Key::Num0)) {
            println!("reset window positions and rects");
            ctx.memory_mut(|m| m.reset_areas());
        }
        gui::draw_gui(ctx, frame, self);

        // request repaint if you want a live overlay:
        ctx.request_repaint();
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }
}
