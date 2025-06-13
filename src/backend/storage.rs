// has functions for reading/writing JSON (or syncing with cloud?)

use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{app::OverlayApp, utils};

/////////////////////////////////////////////////////////////////////
// Save State
/////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct SaveState {
    personal_notes: String,
}

// default save values
impl Default for SaveState {
    fn default() -> Self {
        Self {
            personal_notes: "".to_string(),
        }
    }
}

// deserialization from full OverlayApp
// -> bind relevant OverlayApp fields to their SaveState counterparts
// used to create/derive a SaveState from a full OverlayApp object
impl From<&OverlayApp> for SaveState {
    fn from(app: &OverlayApp) -> Self {
        SaveState {
            personal_notes: app.notes.text.clone(),
        }
    }
}

// replace values in overlay app with loaded save_state
pub fn push_save_state_into_app(save_state: SaveState, app: &mut OverlayApp) {
    app.notes.text = save_state.personal_notes.clone();
}

/////////////////////////////////////////////////////////////////////
// Persistent Storage
/////////////////////////////////////////////////////////////////////

pub trait PersistentStorage {
    fn save_state_to_storage(&self, _app: &OverlayApp);
    fn load_state_from_storage(&self) -> SaveState;
    fn _reset_save_state(&self);
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
    pub fn save_file_path() -> PathBuf {
        let base_path = utils::get_base_folder();
        let mut save_file = base_path.join("save_state");
        save_file = save_file.with_extension("toml");
        if !save_file.exists() {
            if let Ok(()) = fs::write(&save_file, "") {
                return save_file;
            }
        }
        if !save_file.exists() {
            panic!("Save file could not be created!! Panicking now...")
        }
        save_file
    }
}

impl PersistentStorage for FileStorage {
    fn save_state_to_storage(&self, app: &OverlayApp) {
        println!("Saving state to disk....");
        let path = FileStorage::save_file_path();
        let save_state: SaveState = app.into(); // pulls everything save-related from app
        match toml::to_string_pretty(&save_state) {
            Ok(serialized) => {
                fs::write(path.with_extension("toml"), serialized).ok();
            }
            Err(e) => {
                println!("Error serializing SaveState : {e}");
            }
        }
    }

    fn load_state_from_storage(&self) -> SaveState {
        println!("Loading state from disk....");
        let path = FileStorage::save_file_path();
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(save_state) = toml::from_str::<SaveState>(&contents) {
                return save_state;
            } else {
                panic!("Could not deserialize SaveState");
            }
        }
        panic!(
            "Could not read save_file from {:?}! Save file should exist at this point.",
            path
        );
    }

    fn _reset_save_state(&self) {
        println!("Resetting save state....");
        let _ = fs::remove_file(FileStorage::save_file_path());
    }
}

// TODO: implement some wannabe cloud storage (personal GoogleDrive etc) for cross-device sync
