use super::async_manager::AsyncManager;
use crate::utils::{download_to_path, find_asset_folder};
use serde::{Deserialize, de::DeserializeOwned};
use std::{
    collections::HashMap,
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    rc::Rc,
    sync::mpsc::Receiver,
};

// TODO: global "automatic download" toggle, so it can be disabled
const REPO_LOCALE_URL: &str = "https://raw.githubusercontent.com/Matzeall/PokeMMO-Companion/main/companion-app/assets/locales/";
const LOCALE_FILES: [&str; 7] = [
    "monsters.json",
    "skills.json",
    "locations.json",
    "locations_pokedex.json",
    "item_names.json",
    "item_descriptions.json",
    "miscellaneous.json",
];

// TODO: add version string & info_getter methods (e.g. for UI)
/// a definition of what locales there are (supposed to be), and what full display-name they have
#[derive(Debug, Deserialize)]
pub struct LocalesDefinition {
    pub version: u8,
    pub locales: HashMap<String, String>,
}

/// contains all the key-translation pairs for each catagory
#[derive(Debug)]
pub struct Locale {
    locale_name: String,
    monsters: HashMap<String, String>,
    moves: HashMap<String, String>,
    locations: HashMap<String, String>,
    locations_pokedex: HashMap<String, String>,
    items: HashMap<String, String>,
    item_descriptions: HashMap<String, String>,
    miscellaneous: HashMap<String, String>,
}

impl Locale {
    // TODO: get_full_dictionary(), get_*category*_dictionary() ...

    /// errors when the parse function for any file fails
    fn parse_from_dir(dir: impl AsRef<Path>, locale_name: String) -> io::Result<Self> {
        let dir_path = dir.as_ref();

        Ok(Self {
            locale_name,
            monsters: parse_json_file(dir_path.join("monsters.json"))?,
            moves: parse_json_file(dir_path.join("skills.json"))?,
            locations: parse_json_file(dir_path.join("locations.json"))?,
            locations_pokedex: parse_json_file(dir_path.join("locations_pokedex.json"))?,
            items: parse_json_file(dir_path.join("item_names.json"))?,
            item_descriptions: parse_json_file(dir_path.join("item_descriptions.json"))?,
            miscellaneous: parse_json_file(dir_path.join("miscellaneous.json"))?,
        })
    }
}

pub struct LocaleData {
    locale_definition: LocalesDefinition,
    locales: HashMap<String, Locale>, // all dictionaries loaded in memory
}

pub struct LocaleSubsystem {
    pub is_initialized: bool,
    pub data: Option<LocaleData>,
    init_rx: Option<Receiver<anyhow::Result<LocaleData>>>,
    update_rx: Option<Receiver<anyhow::Result<()>>>,
    async_manager: Rc<AsyncManager>,
}

impl LocaleSubsystem {
    pub fn new(async_manager: Rc<AsyncManager>) -> LocaleSubsystem {
        let mut subsystem = LocaleSubsystem {
            is_initialized: false,
            data: None,
            init_rx: None,
            update_rx: None,
            async_manager: async_manager.clone(),
        };

        subsystem.trigger_initialization();

        subsystem
    }

    pub fn trigger_initialization(&mut self) {
        if self.is_initializing() || self.is_updating() {
            println!("LocaleSubsystem - something weird or init already running, do nothing ...");
            return;
        }

        println!("LocaleSubsystem - begin asynchronous initialization");
        self.is_initialized = false;
        self.init_rx = Some(
            self.async_manager
                .spawn_with_response(async { load_data().await }),
        );
    }

    pub fn trigger_locale_update(&mut self) {
        if self.is_initializing() || self.is_updating() {
            println!("LocaleSubsystem - something weird or update already running, aborting...");
            return;
        }

        let cur_version = self.get_locale_definition_version();
        println!(
            "LocaleSubsystem - begin asynchronous locale update (current version: {cur_version})"
        );
        self.update_rx = Some(
            self.async_manager
                .spawn_with_response(async move { update_locales(cur_version).await }),
        );
    }

    pub fn is_initializing(&mut self) -> bool {
        self.init_rx.is_some()
    }

    pub fn is_updating(&mut self) -> bool {
        self.update_rx.is_some()
    }

    // TODO: refactor update tick out into common subsystem interface
    pub fn update_subsystem(&mut self) {
        self.poll_init_finished();
        self.poll_update_finished();
    }

    fn poll_init_finished(&mut self) {
        let Some(rx) = &self.init_rx else {
            return; // no initialization running 
        };

        match rx.try_recv() {
            Ok(Ok(data)) => {
                self.data = Some(data);
                self.is_initialized = true;
                self.init_rx = None;
                println!("LocaleSubsystem - Initialization successful");
            }
            Ok(Err(err)) => {
                eprintln!("LocaleSubsystem - locale init failed, because : {err:?}");
                self.init_rx = None;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // not done yet
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                eprintln!("LocaleSubsystem - locale init task disconnected");
                self.init_rx = None;
            }
        }
    }

    fn poll_update_finished(&mut self) {
        let Some(rx) = &self.update_rx else {
            return; // no update running 
        };

        match rx.try_recv() {
            Ok(Ok(())) => {
                self.update_rx = None;
                println!("LocaleSubsystem - locale updated finished, starting re-initialization");
                self.trigger_initialization(); // load new data into LocaleSubsystem
            }
            Ok(Err(err)) => {
                eprintln!("LocaleSubsystem - locale update failed, because : {err:?}");
                self.update_rx = None;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // not done yet
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                eprintln!("LocaleSubsystem - locale update task disconnected");
                self.update_rx = None;
            }
        }
    }

    // GETTERS ///////////////////////////////////////////////////////////

    pub fn get_available_locales(&self) -> Vec<String> {
        match &self.data {
            None => Vec::new(),
            Some(data) => data.locales.keys().cloned().collect(),
        }
    }

    pub fn get_locale(&self, locale_key: String) -> Option<&Locale> {
        if let Some(data) = &self.data {
            data.locales.get(&locale_key)
        } else {
            None
        }
    }

    pub fn get_locale_definition_version(&self) -> u8 {
        match &self.data {
            None => 0,
            Some(data) => data.locale_definition.version,
        }
    }
}

fn get_locale_definition_path() -> io::Result<PathBuf> {
    let asset_folder = find_asset_folder()?;
    let locale_def_file = asset_folder.join("locales/locale_definition.json");

    Ok(locale_def_file)
}
fn get_locale_dir_path() -> io::Result<PathBuf> {
    let asset_folder = find_asset_folder()?;
    let locale_dir_path = asset_folder.join("locales/");

    Ok(locale_dir_path)
}

fn load_locale_definition_from_disk() -> io::Result<LocalesDefinition> {
    let path = get_locale_definition_path()?;
    if !path.exists() {
        return Err(io::Error::new(
            ErrorKind::NotFound,
            "definition not found".to_string(),
        ));
    }

    let definition = parse_json_file(&path)?;

    Ok(definition)
}

async fn load_data() -> anyhow::Result<LocaleData> {
    let def = match load_locale_definition_from_disk() {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            println!(
                "LocaleSubsystem - locale_definition.json not found locally, starting automatic download ..."
            );
            // download locale definition from github and try again
            let destination = get_locale_definition_path()?;
            let url = REPO_LOCALE_URL.to_owned() + "locale_definition.json";
            download_to_path(&url, destination).await?;

            // try loading from disk again
            load_locale_definition_from_disk()?
        }
        Err(e) => return Err(e.into()),
    };

    println!("LocaleSubsystem - Successfully loaded locale_definition.json");

    let locales_dir = get_locale_dir_path()?;
    let mut locales = HashMap::new();

    for (key, name) in &def.locales {
        let directory = locales_dir.join(key);
        if let Ok(locale) = Locale::parse_from_dir(&directory, name.clone()) {
            locales.insert(key.clone(), locale);
        } else {
            // downloads each file from the repo again, overwriting existing ones
            if let Err(e) = download_locale(key).await {
                eprintln!(
                    "LocaleSubsystem - couldn't download locale ({key} - {name}), because {e}"
                );
                continue;
            };
            // try parsing again, print error when necessary, but don't bubble up error
            match Locale::parse_from_dir(&directory, name.clone()) {
                Ok(locale) => {
                    locales.insert(key.clone(), locale);
                }
                Err(e) => {
                    eprintln!(
                        "LocaleSubsystem - couldn't load locale ({name}) even after re-downloading it, because {e}"
                    );
                }
            };
        };

        println!("LocaleSubsystem - {name} was loaded successfully");
    }

    println!("LocaleSubsystem - finished loading locales");

    Ok(LocaleData {
        locale_definition: def,
        locales,
    })
}

async fn update_locales(current_version: u8) -> anyhow::Result<()> {
    // download most current locale_definition, overwriting
    let destination = get_locale_definition_path()?;
    let url = REPO_LOCALE_URL.to_owned() + "locale_definition.json";
    download_to_path(&url, destination.clone()).await?;

    // parse downloaded definition
    let downloaded_definition: LocalesDefinition = parse_json_file(&destination)?;

    if current_version >= downloaded_definition.version {
        println!("LocaleSubsystem - already up-to-date locales");
        return Ok(()); // everything up-to-date
    }

    // if version was bumped, then re-download all locales
    for (key, name) in downloaded_definition.locales {
        if let Err(e) = download_locale(&key).await {
            eprintln!("LocaleSubsystem - couldn't download locale ({key} - {name}), because {e}");
            continue;
        };
    }

    Ok(())
}

async fn download_locale(locale_key: &str) -> anyhow::Result<()> {
    println!("LocaleSubsystem - downloading locale ({locale_key})");
    let base_dir = get_locale_dir_path()?;
    let locale_dir = base_dir.join(locale_key);
    let locale_url = REPO_LOCALE_URL.to_owned() + locale_key + "/";

    for filename in LOCALE_FILES {
        let file_url: String = locale_url.clone() + filename;
        download_to_path(&file_url, locale_dir.join(filename)).await?;
    }

    Ok(())
}

fn parse_json_file<T>(path: impl AsRef<Path>) -> io::Result<T>
where
    T: DeserializeOwned,
{
    let content = fs::read_to_string(path)?;
    let value = serde_json::from_str(&content)?;
    Ok(value)
}
