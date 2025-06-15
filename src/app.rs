// defines runtime data struct -> holds mutable application state
// is instantiated by main.rs
// TODO:
// - Finally get to the main app focus issue -> mouse passthrough & unfocussed input events
//     - this is how I could swap or patch a crate if I need to fix something in my own fork:
//             egui    = { git = "https://github.com/your‑username/egui.git", branch = "my‑patch‑branch" } # swapping
//             [patch.crates-io] # patching
//             egui = { git = "https://github.com/your‑username/egui.git", branch = "my‑patch‑branch" }
//             egui = { path = "../my‑egui‑fork" } # for development with relative path
//             # After changing Cargo.toml, run : cargo update -p egui # rebuild from fork
//     - I could also try to spawn a thread (with a ref to hwnd) which gets RegisterHotKey commands that then shows the window again (the same for click-through)
// - look into nvim debugging
//      - toggle breakpoint in current line
//      - view contents of variable in scope
//      - step into/over/play until next breakpoint
//      - attaching debugger -> what is a debugger?
//
//
//
//
//
//
use eframe::{App, CreationContext, Frame};
use raw_window_handle::HasWindowHandle;

use crate::{
    backend::{
        self,
        feature_state::FeatureSubsystem,
        notes_feature::NotesSubsystem,
        ressources_feature::RessourcesSubsystem,
        storage::{FileStorage, PersistentStorage, SaveState},
    },
    frontend::viewport::{DefaultViewportManager, NativeViewportManagerWin32, ViewportManager},
    gui::{self, GuiSubsystem},
};
use egui::Context;

pub const APP_ID: &str = "pokemmo-companion";

// INFO: this is part of the egui storage solution which requires an eframe bug fix
// const SAVE_STATE_STORAGE_KEY: &str = "save_state";
// const SAVE_STATE_STORAGE_KEY_BROKEN: &str = "save_state_broken";

pub struct OverlayApp {
    pub features: FeatureSubsystem,

    pub gui: GuiSubsystem,

    pub ressources: RessourcesSubsystem,

    pub notes: NotesSubsystem,

    storage: Box<dyn PersistentStorage>,

    pub viewport_manager: Box<dyn ViewportManager>,
}

impl OverlayApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let mut app = Self {
            features: FeatureSubsystem::new(),
            gui: GuiSubsystem::new(cc),
            ressources: RessourcesSubsystem::new(),
            notes: NotesSubsystem::new(),
            storage: Box::new(FileStorage::new()),
            viewport_manager: Box::new(DefaultViewportManager::default()),
        };

        #[cfg(windows)]
        if let Ok(window_handle) = cc.window_handle() {
            app.viewport_manager = Box::new(NativeViewportManagerWin32::new(window_handle));
        }

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
}

// main egui update loop
impl App for OverlayApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        // Poll any platform messages:
        // while let Ok(msg) = self.plat_rx.try_recv() { … }

        self.viewport_manager.update_viewport(ctx, frame);

        // only handle input when control_bar is also visible
        // and the application is currently meant to be controlled
        if self.viewport_manager.current_focus_state().is_focused() {
            self.features
                .handle_feature_state_input(ctx.input(|i| i.clone()));
        }

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
