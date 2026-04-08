use super::async_manager::AsyncManager;
use crate::utils::{download_to_path, find_asset_folder};
use serde::{Deserialize, de::DeserializeOwned};
use std::{
    collections::HashMap,
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, RwLock},
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

/// a definition of what locales there are (supposed to be), and what full display-name they have
#[derive(Debug, Deserialize)]
pub struct LocalesDefinition {
    pub version: u8,
    pub locales: HashMap<String, String>,
}

/// the type of list a localized text is from (and what category of thing it is)
#[derive(Debug, Clone, Copy)]
pub enum TextCategory {
    Monster,
    Move,
    Location,
    PokedexLocation, // contains time-information inseparable from main name
    Item,
    ItemDescription,
    Miscellaneous,
}

#[derive(Debug)]
pub struct LocalizedText {
    pub key: String,
    pub text: String,
    pub category: TextCategory,
}

/// contains all the key-translation pairs for each catagory
#[derive(Debug)]
pub struct Locale {
    #[allow(dead_code)]
    pub locale_name: String,
    pub localized_texts: Vec<Arc<LocalizedText>>,
    pub monsters: HashMap<String, usize>,
    pub moves: HashMap<String, usize>,
    pub locations: HashMap<String, usize>,
    pub locations_pokedex: HashMap<String, usize>,
    pub items: HashMap<String, usize>,
    pub item_descriptions: HashMap<String, usize>,
    pub miscellaneous: HashMap<String, usize>,
}

impl Locale {
    /// looks through any dictionary in order of importance and returns key back when it doesn't find anything
    pub fn find_localized_text(&self, key: &str) -> String {
        let mut index: usize = 0;
        let mut found: bool = false;

        if let Some(i) = self.monsters.get(key) {
            index = *i;
            found = true;
        } else if let Some(i) = self.moves.get(key) {
            index = *i;
            found = true;
        } else if let Some(i) = self.locations.get(key) {
            index = *i;
            found = true;
        } else if let Some(i) = self.locations_pokedex.get(key) {
            index = *i;
            found = true;
        } else if let Some(i) = self.items.get(key) {
            index = *i;
            found = true;
        } else if let Some(i) = self.item_descriptions.get(key) {
            index = *i;
            found = true;
        } else if let Some(i) = self.miscellaneous.get(key) {
            index = *i;
            found = true;
        }

        if found
            && let Some(loc_text) = self.localized_texts.get(index)
            && !loc_text.text.is_empty()
        {
            return loc_text.text.clone();
        }

        key.into()
    }

    /// errors when the parse function for any file fails
    fn parse_from_dir(dir: impl AsRef<Path>, locale_name: String) -> io::Result<Self> {
        let dir_path = dir.as_ref();

        // parse files into HashMap<key,text>
        let monsters: HashMap<String, String> = parse_json_file(dir_path.join("monsters.json"))?;
        let moves: HashMap<String, String> = parse_json_file(dir_path.join("skills.json"))?;
        let locations: HashMap<String, String> = parse_json_file(dir_path.join("locations.json"))?;
        let locations_pokedex: HashMap<String, String> =
            parse_json_file(dir_path.join("locations_pokedex.json"))?;
        let items: HashMap<String, String> = parse_json_file(dir_path.join("item_names.json"))?;
        let item_descriptions: HashMap<String, String> =
            parse_json_file(dir_path.join("item_descriptions.json"))?;
        let miscellaneous: HashMap<String, String> =
            parse_json_file(dir_path.join("miscellaneous.json"))?;

        // build flat localized_texts list and build hashmaps<key -> index>
        let mut localized_texts = Vec::new();

        let mut monster_indices = HashMap::new();
        for (key, text) in monsters {
            monster_indices.insert(key.clone(), localized_texts.len());
            localized_texts.push(Arc::new(LocalizedText {
                key,
                text,
                category: TextCategory::Monster,
            }));
        }

        let mut moves_indices = HashMap::new();
        for (key, text) in moves {
            moves_indices.insert(key.clone(), localized_texts.len());
            localized_texts.push(Arc::new(LocalizedText {
                key,
                text,
                category: TextCategory::Move,
            }));
        }

        let mut locations_indices = HashMap::new();
        for (key, text) in locations {
            locations_indices.insert(key.clone(), localized_texts.len());
            localized_texts.push(Arc::new(LocalizedText {
                key,
                text,
                category: TextCategory::Location,
            }));
        }

        let mut pokedex_location_indices = HashMap::new();
        for (key, text) in locations_pokedex {
            pokedex_location_indices.insert(key.clone(), localized_texts.len());
            localized_texts.push(Arc::new(LocalizedText {
                key,
                text,
                category: TextCategory::PokedexLocation,
            }));
        }

        let mut item_indices = HashMap::new();
        for (key, text) in items {
            item_indices.insert(key.clone(), localized_texts.len());
            localized_texts.push(Arc::new(LocalizedText {
                key,
                text,
                category: TextCategory::Item,
            }));
        }

        let mut item_description_indices = HashMap::new();
        for (key, text) in item_descriptions {
            item_description_indices.insert(key.clone(), localized_texts.len());
            localized_texts.push(Arc::new(LocalizedText {
                key,
                text,
                category: TextCategory::ItemDescription,
            }));
        }

        let mut miscellaneous_indices = HashMap::new();
        for (key, text) in miscellaneous {
            miscellaneous_indices.insert(key.clone(), localized_texts.len());
            localized_texts.push(Arc::new(LocalizedText {
                key,
                text,
                category: TextCategory::Miscellaneous,
            }));
        }

        Ok(Self {
            locale_name,
            localized_texts,
            monsters: monster_indices,
            moves: moves_indices,
            locations: locations_indices,
            locations_pokedex: pokedex_location_indices,
            items: item_indices,
            item_descriptions: item_description_indices,
            miscellaneous: miscellaneous_indices,
        })
    }
}

pub struct LocaleData {
    pub locale_definition: LocalesDefinition,
    pub locales: HashMap<String, Locale>, // all dictionaries loaded in memory
}

pub struct LocaleSubsystem {
    // core data
    pub data: Arc<RwLock<Option<LocaleData>>>,
    pub init_counter: Arc<RwLock<usize>>, // incremented when data is re-initialized

    async_manager: Rc<AsyncManager>,
}

impl LocaleSubsystem {
    pub fn new(async_manager: Rc<AsyncManager>) -> LocaleSubsystem {
        let subsystem = LocaleSubsystem {
            data: Arc::new(RwLock::new(None)),
            init_counter: Arc::new(RwLock::new(0)),
            async_manager: async_manager.clone(),
        };

        subsystem.trigger_initialization();

        subsystem
    }

    pub fn trigger_initialization(&self) {
        println!("LocaleSubsystem - begin asynchronous initialization");

        let data_ref = self.data.clone();
        let counter_ref = self.init_counter.clone();
        // spawn_unique prevents multiple inits at once
        self.async_manager
            .spawn_unique("LocaleSubsystem_Init", async move {
                reload_subsystem_data(data_ref, counter_ref).await;
            });
    }

    pub fn trigger_locale_update(&self) {
        let cur_version = self.get_locale_definition_version();
        println!(
            "LocaleSubsystem - begin asynchronous locale update (current version: {cur_version})"
        );

        let data_ref = self.data.clone();
        let counter_ref = self.init_counter.clone();
        self.async_manager
            .spawn_unique("LocaleSubsystem_Update", async move {
                match update_locales(cur_version).await {
                    Ok(()) => reload_subsystem_data(data_ref, counter_ref).await,
                    Err(e) => eprintln!("LocaleSubsystem - Update failed because, {e}"),
                };
            });
    }

    // GETTERS ///////////////////////////////////////////////////////////
    pub fn is_initialized(&self) -> bool {
        let guard = self.data.read().unwrap();
        (*guard).is_some()
    }

    pub fn get_available_locales(&self) -> Vec<String> {
        let guard = self.data.read().unwrap();
        match &*guard {
            None => Vec::new(),
            Some(data) => data.locales.keys().cloned().collect(),
        }
    }

    pub fn get_locale_display_name(&self, key: &str) -> String {
        let guard = self.data.read().unwrap();
        match &*guard {
            None => String::new(),
            Some(data) => match data.locale_definition.locales.get(key) {
                Some(n) => n.clone(),
                None => String::new(),
            },
        }
    }

    pub fn with_locale<R>(&self, locale_key: &str, f: impl FnOnce(&Locale) -> R) -> Option<R> {
        let guard = self.data.read().unwrap();

        if let Some(data) = &*guard
            && let Some(locale) = data.locales.get(locale_key)
        {
            return Some(f(locale));
        }

        None
    }

    pub fn get_locale_definition_version(&self) -> u8 {
        let guard = self.data.read().unwrap();

        match &*guard {
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

async fn reload_subsystem_data(
    data_ref: Arc<RwLock<Option<LocaleData>>>,
    counter_ref: Arc<RwLock<usize>>,
) {
    let result = load_data().await;
    let mut data_guard = data_ref.write().unwrap();
    let mut counter_guard = counter_ref.write().unwrap();
    match result {
        Ok(data) => {
            *data_guard = Some(data);
            *counter_guard += 1;
            println!("LocaleSubsystem - Initialization successful");
        }
        Err(e) => {
            *data_guard = None;
            *counter_guard += 1;
            eprintln!("LocaleSubsystem - error during initialization: {e}");
        }
    };
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
