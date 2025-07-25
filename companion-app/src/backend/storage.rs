// has functions for reading/writing JSON (or syncing with cloud?)

use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::app::OverlayApp;

/////////////////////////////////////////////////////////////////////
// Save State
/////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct SaveState {
    version: u8,
    disable_overlay: bool,
    transparent_background_always: bool,
    personal_notes: String,
    type_matrix_scale: f32,
}

// default save values
impl Default for SaveState {
    fn default() -> Self {
        Self {
            version: 0,
            disable_overlay: false,
            transparent_background_always: false,
            personal_notes: "".to_string(),
            type_matrix_scale: 1.0,
        }
    }
}

// deserialization from full OverlayApp
// -> bind relevant OverlayApp fields to their SaveState counterparts
// used to create/derive a SaveState from a full OverlayApp object
impl From<&OverlayApp> for SaveState {
    fn from(app: &OverlayApp) -> Self {
        SaveState {
            version: app.settings.version,
            disable_overlay: app.settings.disable_overlay,
            transparent_background_always: app.settings.transparent_background_always,
            personal_notes: app.notes.text.clone(),
            type_matrix_scale: app.settings.type_matrix_scale,
        }
    }
}

// replace values in overlay app with loaded save_state
pub fn push_save_state_into_app(save_state: SaveState, app: &mut OverlayApp) {
    app.settings.version = save_state.version;
    app.settings.disable_overlay = save_state.disable_overlay;
    app.settings.transparent_background_always = save_state.transparent_background_always;
    app.notes.text = save_state.personal_notes.clone();
    app.settings.type_matrix_scale = save_state.type_matrix_scale;
}

/////////////////////////////////////////////////////////////////////
// Persistent Storage
/////////////////////////////////////////////////////////////////////

pub trait PersistentStorage {
    fn save_state_to_storage(&self, _app: &OverlayApp) -> io::Result<()>;
    fn load_state_from_storage(&self) -> io::Result<SaveState>;
    fn _reset_save_state(&self) -> io::Result<()>;
}

// File Storage ///////////////////////////////////
pub struct FileStorage {}

impl FileStorage {
    pub fn new() -> Self {
        Self {}
    }

    // save-file is guaranteed to exist after calling this function, if not preset yet it will be
    // created. TODO: make sure there cannot be any weird bugs by permissions etc.
    // TODO: In regards to the Todo above and also because it's generally better, change
    // save location to the OS's data dir (.local/share, AppData/Roaming)
    pub fn save_file_path() -> io::Result<PathBuf> {
        match eframe::storage_dir(crate::app::APP_ID) {
            Some(storage_dir) => {
                let save_file = storage_dir.join("save_state").with_extension("toml");
                match save_file.exists() {
                    true => Ok(save_file),
                    false => match fs::write(&save_file, "") {
                        Ok(_) => Ok(save_file),
                        Err(e) => Err(e),
                    },
                }
            }
            None => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "there is no data/storage directory on your device (or eframe::storage_dir() doesn't support one on your platform)",
            )),
        }
    }
}

impl PersistentStorage for FileStorage {
    fn save_state_to_storage(&self, app: &OverlayApp) -> io::Result<()> {
        println!("Saving state to disk....");
        let path = FileStorage::save_file_path()?;
        let save_state: SaveState = app.into(); // pulls everything save-related from app
        match toml::to_string_pretty(&save_state) {
            Ok(serialized) => fs::write(path.with_extension("toml"), serialized),
            // wrap parsing error in io error
            Err(e) => Err(io::Error::other(e)),
        }
    }

    fn load_state_from_storage(&self) -> io::Result<SaveState> {
        println!("Loading state from disk....");
        let path = FileStorage::save_file_path()?;
        match fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str::<SaveState>(&contents) {
                Ok(save_state) => Ok(save_state),
                Err(e) => Err(io::Error::other(e)),
            },
            Err(e) => Err(e),
        }
    }

    fn _reset_save_state(&self) -> io::Result<()> {
        println!("Resetting save state....");
        match FileStorage::save_file_path() {
            Ok(path) => fs::remove_file(path),
            Err(e) => Err(e),
        }
    }
}

// TODO: implement some wannabe cloud storage (personal GoogleDrive etc) for cross-device sync
