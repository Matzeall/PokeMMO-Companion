// defines runtime data struct -> holds mutable application state
// is instantiated by main.rs
// TODO:
// - refactor viewport structs out into multiple files and use cfg attributes in mod file
// - add resources in-file scroll target links (e.g. cache.add_link_hook("#next");
//      - if self.cache.get_link_hook("#next").unwrap() {
//            self.curr_page = 1;
//       })

use std::sync::Arc;

use eframe::{App, CreationContext, Frame};

use crate::{
    backend::{
        self,
        feature_state::FeatureSubsystem,
        notes_feature::NotesSubsystem,
        ressources_feature::RessourcesSubsystem,
        settings::SettingsSubsystem,
        storage::{FileStorage, PersistentStorage, SaveState},
    },
    frontend::{
        self,
        gui_subsystem::GuiSubsystem,
        viewport::{self, DefaultViewportManager, ViewportManager},
    },
};
use egui::{Context, Modifiers};

pub const APP_ID: &str = "pokemmo-companion";

// INFO: this is part of the egui storage solution which requires an eframe bug fix
// const SAVE_STATE_STORAGE_KEY: &str = "save_state";
// const SAVE_STATE_STORAGE_KEY_BROKEN: &str = "save_state_broken";

pub struct OverlayApp {
    pub features: FeatureSubsystem,

    pub gui: GuiSubsystem,

    pub settings: SettingsSubsystem,

    pub ressources: RessourcesSubsystem,

    pub notes: NotesSubsystem,

    storage: Box<dyn PersistentStorage>,

    pub viewport_manager: Box<dyn ViewportManager>,

    winit_window: Option<Arc<winit::window::Window>>,
}
impl OverlayApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        println!("Creating App ...");
        let mut app = Self {
            features: FeatureSubsystem::new(),
            gui: GuiSubsystem::new(cc),
            settings: SettingsSubsystem::new(),
            ressources: RessourcesSubsystem::new(),
            notes: NotesSubsystem::new(),
            storage: Box::new(FileStorage::new()),
            viewport_manager: Box::new(DefaultViewportManager::default()),
            winit_window: cc.winit_window.clone(),
        };

        // setup storage and load settings
        app.setup_persistent_storage();

        app.setup_native_viewport_manager();

        app
    }

    fn setup_native_viewport_manager(
        &mut self,
        /*cc: &CreationContext<'_>*/
    ) {
        println!("Setup ViewportManager ...");
        self.settings.request_viewport_restart = false;
        let Some(winit_window) = self
            .winit_window
            .clone()
            // also early out if disabled overlay
            .filter(|_| !self.settings.disable_overlay)
        else {
            println!(
                "Couldn't start viewport manager, because winit_window was lost/not valid\n=> Falling back to non-overlay viewport..."
            );
            self.viewport_manager = Box::new(DefaultViewportManager::default());
            return;
        };

        #[cfg(windows)]
        {
            use raw_window_handle::HasWindowHandle;
            if let Ok(window_handle) = winit_window.window_handle() {
                self.viewport_manager =
                    Box::new(viewport::windows::NativeViewportManagerWin32::new(
                        window_handle,
                        winit_window.clone(),
                    ));
            }
        }

        #[cfg(unix)]
        {
            use raw_window_handle::{HandleError, HasDisplayHandle, HasWindowHandle};
            // dynamically switch between X11 and Wayland backends, depending on display compositor
            let native_viewport_manager: Result<Box<dyn ViewportManager>, HandleError> =
                if std::env::var("WAYLAND_DISPLAY")
                    .as_deref()
                    .is_ok_and(|env| env.contains("wayland"))
                {
                    match (winit_window.window_handle(), winit_window.display_handle()) {
                        (Ok(window_handle), Ok(display_handle)) => {
                            let manager: Box<dyn ViewportManager> =
                                viewport::unix::NativeViewportManagerWayland::try_new(
                                    window_handle,
                                    display_handle,
                                    winit_window.clone(),
                                );
                            Ok(manager)
                        }
                        (Err(e), _) => Err(e),
                        (_, Err(e)) => Err(e),
                    }
                } else {
                    match winit_window.window_handle() {
                        Ok(window_handle) => {
                            let manager: Box<dyn ViewportManager> = Box::new(
                                viewport::unix::NativeViewportManagerX11::new(window_handle),
                            );
                            Ok(manager)
                        }
                        Err(e) => Err(e),
                    }
                };

            match native_viewport_manager {
                Ok(manager) => self.viewport_manager = manager,
                Err(e) => {
                    println!(
                        "On unix system, neither X11 nor Wayland could be used as display compositor, because : {e}"
                    );
                    self.viewport_manager = Box::new(DefaultViewportManager::default());
                }
            }
        }
    }

    // replace values in overlay app with loaded save_state
    pub fn push_save_state_into_app(&mut self, save_state: SaveState) {
        // rerouted for editing convenience of SaveState properties
        backend::storage::push_save_state_into_app(save_state, self);
    }

    fn setup_persistent_storage(&mut self) {
        println!("Setup persistent storage ...");
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
        let storage = &self.storage;
        match storage.load_state_from_storage() {
            Ok(save_state) => self.push_save_state_into_app(save_state),
            Err(e) => {
                // TODO: notify user
                println!("Could not load save_state from storage, because:\n{e}")
            }
        }
    }
}

// main egui update loop
impl App for OverlayApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if self.settings.request_viewport_restart {
            self.setup_native_viewport_manager();
        } else {
            self.viewport_manager.update_viewport(ctx, frame);
        }

        // check whether to clear the ui data -> reset window positions and caches etc.
        if ctx
            .input(|r| r.clone())
            .consume_key(Modifiers::CTRL, egui::Key::Num0)
            || self.settings.request_clear_ui_data
        {
            self.settings.request_clear_ui_data = false;
            ctx.memory_mut(|w| {
                w.reset_areas();
                w.data.clear();
            })
        }

        // only handle input when control_bar is also visible
        // and the application is currently meant to be controlled
        if self.viewport_manager.current_focus_state().is_focused() {
            self.features
                .handle_feature_state_input(ctx.input(|i| i.clone()));
        }

        if self.viewport_manager.should_draw_gui() {
            frontend::gui::main_gui::draw_gui(ctx, frame, self);
        }
        // request repaint if you want a live overlay:
        ctx.request_repaint();
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        if self.settings.transparent_background_always {
            egui::Rgba::TRANSPARENT.to_array()
        } else {
            self.viewport_manager.window_background_color().to_array()
        }
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
