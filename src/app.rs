// defines runtime data struct -> holds mutable application state
// is instantiated by main.rs

// TODO:
// - look into nvim debugging
//      - toggle breakpoint in current line
//      - view contents of variable in scope
//      - step into/over/play until next breakpoint
//      - attaching debugger -> what is a debugger?
// - persist notes -> serde or manual?
// - Finally get to the main app focus issue -> mouse passthrough & unfocussed input events
// - Bring some nice notes layout things over to resources
use eframe::{App, CreationContext, Frame};

use crate::{
    backend::{
        self,
        feature_state::FeatureSubsystem,
        notes_feature::NotesSubsystem,
        ressources_feature::RessourcesSubsystem,
        storage::{FileStorage, PersistentStorage, SaveState},
    },
    gui::{self, GuiSubsystem},
};
use egui::Context;

pub const APP_ID: &str = "pokemmo-companion";

// INFO: this is part of the egui storage solution which requires an eframe bug fix
// const SAVE_STATE_STORAGE_KEY: &str = "save_state";
// const SAVE_STATE_STORAGE_KEY_BROKEN: &str = "save_state_broken";

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

    storage: Box<dyn PersistentStorage>,
}

impl OverlayApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        // Here I could spawn a background thread to register
        // RegisterHotKey (windows), XGrabKey (X11), or layer-shell listener (Wayland).
        //
        // Send events back via a channel that are polled in update().

        let mut app = Self {
            app_focus: FocusState::Unfocused,
            features: FeatureSubsystem::new(),
            gui: GuiSubsystem::new(cc),
            ressources: RessourcesSubsystem::new(),
            notes: NotesSubsystem::new(),
            storage: Box::new(FileStorage::new()),
        };

        // if let Some(storage) = cc.storage {
        //     if let Some(save_serialized) = storage.get_string(SAVE_STATE_STORAGE_KEY) {
        //         if let Ok(save_state) = toml::from_str(&save_serialized) {
        //             backend::storage::push_save_state_into_app(&mut app, save_state);
        //         } else {
        //             println!(
        //                 "save_state is broken (not valid .toml formt). It could not be deserialized!\
        //                 You have to fix the save_file manually now. \n\
        //                 Program will continue with a new empty save_state and save the broken one under the key: {SAVE_STATE_STORAGE_KEY_BROKEN}"
        //             );
        //             storage.set_string(SAVE_STATE_STORAGE_KEY_BROKEN, save_serialized);
        //             panic!(
        //                 "save_state is broken (not valid .toml formt). It could not be deserialized!"
        //             );
        //         }
        //     } else {
        //         println!(
        //             "There was no save_state found. If this is the first time the program is run, it's normal.\n\
        //              Otherwise you have to fix the save_file manually now. \n\
        //              Program will continue with a new empty save_state and save the broken one under the key: {SAVE_STATE_STORAGE_KEY_BROKEN}"
        //         )
        //     }
        // } else {
        //     panic!("There was no storage associated with this app!");
        // }

        // TODO: recheck with egui issue (https://github.com/emilk/egui/issues/5689)
        // It prohibits me from using the eframe storage framework
        // Therefore rolling my own for now.
        let storage = &app.storage;
        match storage.load_state_from_storage() {
            Ok(save_state) => app.push_save_state_into_app(save_state),
            Err(e) => {
                // TODO: notify user
                println!("Could not load save_state from storage, because:\n{e}")
            }
        }

        app
    }

    // replace values in overlay app with loaded save_state
    pub fn push_save_state_into_app(&mut self, save_state: SaveState) {
        // rerouted for editing convenience of SaveState properties
        backend::storage::push_save_state_into_app(save_state, self);
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

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // INFO: This would be the eframe idiomatic approach, but since there are some technical
        // limitations I use my custom implementation instead
        //
        // let save_state: backend::storage::SaveState = (&*self).into(); // pulls everything save-related from full OverlayApp
        // match toml::to_string_pretty(&save_state) {
        //     Ok(save_serialized) => {
        //         storage.set_string(SAVE_STATE_STORAGE_KEY, save_serialized);
        //     }
        //     Err(e) => {
        //         println!("Error serializing SaveState : {e}");
        //     }
        // }

        if let Err(e) = self.storage.save_state_to_storage(self) {
            println!("Could not save save_state to storage, because:\n{e}");
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(20)
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, _raw_input: &mut egui::RawInput) {}
}
