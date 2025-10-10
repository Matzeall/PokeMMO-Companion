use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::utils::{convert_string_to_normal_case, normalize_name};

const LOCATION_NAME_REGEX: &str = r"^(?P<name>.*?)\s*(?:\(\s*(?P<inside>[^()]*)\s*\))?\s*$";

/// structure to parse dump json data into
#[derive(Deserialize, Debug)]
pub struct MonsterLocations {
    id: u32,
    locations: Vec<Location>,
}

#[derive(Deserialize, Debug)]
pub struct Location {
    #[serde(rename = "location")]
    name: String,
    #[serde(rename = "type")]
    encounter_type: String,
    region_id: u8,
    region_name: String,
    min_level: u8,
    max_level: u8,
    rarity: String,
}

/// Collected info about one location_name -> will be written to json (base-data)
#[cfg_attr(test, derive(serde::Deserialize))]
#[derive(Serialize, Debug)]
pub struct LocationInfo {
    #[serde(rename = "name")]
    pub base_name: String,
    pub region_id: u8,
    pub region_base_name: String,
    pub encounters: Vec<Encounter>,
}

#[cfg_attr(test, derive(serde::Deserialize))]
#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct Encounter {
    #[serde(rename = "type")]
    pub encounter_type: String,
    pub pokemon_id: u32,
    pub min_level: u8,
    pub max_level: u8,
    pub time_restrictions: Vec<String>,
    pub rarity: String,
}

pub fn build_location_locale_lookup(
    en_monsters: Vec<MonsterLocations>,
    en_string_dump: IndexMap<u32, String>,
    locale_string_dump: IndexMap<u32, String>,
    locale_additional_translations: IndexMap<String, String>,
) -> (IndexMap<String, String>, IndexMap<String, String>) {
    println!("Building locale lookups...");
    let mut pokedex_translations = IndexMap::new();
    let mut unique_name_translations = IndexMap::new();
    let regex = Regex::new(LOCATION_NAME_REGEX)
        .expect("REGEX to parse location names could not be compiled! Panicing now...");

    for monster in &en_monsters {
        for location in &monster.locations {
            let en_location = normalize_name(location.name.clone());

            // parse en_location into components
            let Some((location_name, season, is_day, time)) =
                parse_location_name_into_components(en_location.as_str(), &regex)
            else {
                println!(
                    "Regex couldn't parse location entry \"{en_location}\" into it's components. Skipping it ..."
                );
                continue;
            };
            // early out if key already present
            if unique_name_translations.contains_key(location_name) {
                continue;
            }

            let locale_name = find_translation_for(
                location_name,
                &en_string_dump,
                &locale_string_dump,
                &locale_additional_translations,
            );

            unique_name_translations.insert(
                location_name.to_owned(),
                convert_string_to_normal_case(&locale_name),
            );

            // recombine localized pokedex locations and time if present
            let parts: Vec<&str> = [season, is_day, time].into_iter().flatten().collect();
            let mut locale_time_modifiers = Vec::new(); // translated parts vec

            // try translating the time parts first too
            for name in &parts {
                locale_time_modifiers.push(find_translation_for(
                    name,
                    &en_string_dump,
                    &locale_string_dump,
                    &locale_additional_translations,
                ));
            }

            let pokedex_locale_name = convert_string_to_normal_case(&if parts.is_empty() {
                locale_name.to_string()
            } else {
                format!("{} ({})", locale_name, locale_time_modifiers.join("/"))
            });

            pokedex_translations.insert(en_location.to_owned(), pokedex_locale_name);
        }
    }

    (unique_name_translations, pokedex_translations)
}

//////////////////////////  BASE DATA  /////////////////////////////////////
pub fn build_location_base_data(en_monsters: Vec<MonsterLocations>) -> Vec<LocationInfo> {
    println!("Building location base-data...");
    let mut location_info = Vec::new();

    let regex = Regex::new(LOCATION_NAME_REGEX)
        .expect("REGEX to parse location names could not be compiled! Panicing now...");
    for monster in &en_monsters {
        for location in &monster.locations {
            // bring strings in universal format -> normal_case, to allow for only english use
            // let en_location = normalize_name(location.name.clone());
            let en_location = crate::utils::convert_string_to_normal_case(&location.name);

            // parse en_location into components
            let Some((location_name, season, is_day, time)) =
                parse_location_name_into_components(en_location.as_str(), &regex)
            else {
                println!(
                    "Regex couldn't parse location entry \"{en_location}\" into it's components. Skipping it ..."
                );
                continue;
            };

            // build time_modifier from parts
            let time_modifier: Vec<String> = [season, is_day, time]
                .into_iter()
                .flatten()
                .map(|s| s.to_string())
                .collect();
            // let time_modifier = parts.join("/");

            // assign values of current encounter
            let encounter = Encounter {
                encounter_type: location.encounter_type.clone(),
                pokemon_id: monster.id,
                min_level: location.min_level,
                max_level: location.max_level,
                time_restrictions: time_modifier,
                rarity: location.rarity.clone(),
            };

            // find the same location's info already present in the vec and update it
            if let Some(info) = location_info.iter_mut().find(|i: &&mut LocationInfo| {
                i.base_name == location_name && i.region_id == location.region_id
            }) {
                if !info.encounters.contains(&encounter) {
                    info.encounters.push(encounter);
                }
            }
            // or create and add new location entry
            else {
                location_info.push(LocationInfo {
                    base_name: location_name.to_string(),
                    region_id: location.region_id,
                    region_base_name: location.region_name.clone(),
                    encounters: vec![encounter],
                });
            }
        }
    }
    location_info
}

/// looks through the english string dump (and matches with the locale string dump by id) aswell as
/// the additional translations map
/// If no translation can be found, the original name will be returned -> no empty translations or
/// different sized translation maps.
fn find_translation_for(
    name: &str,
    string_dump: &IndexMap<u32, String>,
    locale_string_dump: &IndexMap<u32, String>,
    additional_translations: &IndexMap<String, String>,
) -> String {
    let mut locale_name = name.to_string();
    let mut id_hit = None;

    if let Some(locale_location_name) = additional_translations.get(name) {
        locale_name = locale_location_name.clone();
    } else {
        // find that location's id in the string dump
        for (id, en_dump_name) in string_dump {
            if name == normalize_name(en_dump_name.clone()) {
                id_hit = Some(*id);
                break;
            }
        }

        if let Some(id) = id_hit {
            if let Some(locale_location_name) = locale_string_dump.get(&id) {
                locale_name = locale_location_name.clone();
            };
        };
    }
    locale_name
}

#[allow(clippy::type_complexity)]
fn parse_location_name_into_components<'a>(
    s: &'a str,
    re: &Regex,
) -> Option<(&'a str, Option<&'a str>, Option<&'a str>, Option<&'a str>)> {
    re.captures(s).map(|caps| {
        let name = caps.name("name").map(|m| m.as_str().trim()).unwrap_or("");
        let inside_opt = caps.name("inside").map(|m| m.as_str().trim());

        // If there's an inside string, split on '/' and trim parts.
        // Right-align: 3 parts -> season/day/time, 2 parts -> day/time, 1 part -> time.
        let (season, day, time) = match inside_opt {
            None => (None, None, None),
            Some(inside) => {
                let parts: Vec<&str> = inside
                    .split('/')
                    .map(|p| p.trim())
                    .filter(|p| !p.is_empty())
                    .collect();
                match parts.len() {
                    3 => (Some(parts[0]), Some(parts[1]), Some(parts[2])),
                    2 => (None, Some(parts[0]), Some(parts[1])),
                    1 => (None, None, Some(parts[0])),
                    0 => (None, None, None),
                    // If there are more than 3 parts, take the last three (right-aligned).
                    n => {
                        let t = parts[n - 1];
                        let d = parts.get(n - 2).copied();
                        let s = parts.get(n - 3).copied();
                        (s, d, Some(t))
                    }
                }
            }
        };

        (name, season, day, time)
    })
}

#[cfg(test)]
mod test {
    use std::{collections::HashSet, path::PathBuf};

    use crate::utils;

    use super::*;

    #[test]
    fn test_location_name_parsing() {
        let re = Regex::new(LOCATION_NAME_REGEX).unwrap();

        // input -> expected (name, season, day, time)
        let cases = [
            (
                "route 29 (season0/day/morning)",
                ("route 29", Some("season0"), Some("day"), Some("morning")),
            ),
            (
                "route 43 (day/morning)",
                ("route 43", None, Some("day"), Some("morning")),
            ),
            ("route 1 (night)", ("route 1", None, None, Some("night"))),
            ("route 5", ("route 5", None, None, None)),
        ];

        for (input, expected) in cases {
            let parsed = parse_location_name_into_components(input, &re)
                .unwrap_or_else(|| panic!("Did not match input {:?}", input));

            assert_eq!(parsed.0, expected.0, "wrong name for input {}", input);
            assert_eq!(parsed.1, expected.1, "wrong season for input {}", input);
            assert_eq!(parsed.2, expected.2, "wrong day for input {}", input);
            assert_eq!(parsed.3, expected.3, "wrong time for input {}", input);
        }
    }

    #[test]
    fn fake_test_write_up_unique_strings() {
        // to list types run $ cargo test -- --nocaptur
        let file_path = PathBuf::from("data/base/locations.json");
        if !file_path.exists() {
            println!("path doesn't exist.");
            return;
        }
        let location_infos = utils::parse_json_items_from_file::<LocationInfo>(&file_path);

        let mut encounter_types = HashSet::<String>::new();
        let mut rarity_levels = HashSet::<String>::new();
        for info in &location_infos {
            for encounter in &info.encounters {
                encounter_types.insert(encounter.encounter_type.to_owned());
                rarity_levels.insert(encounter.rarity.to_owned());
            }
        }

        eprintln!("\nencounter_types:");
        encounter_types
            .iter()
            .for_each(|enc_type| eprintln!("\"{enc_type}\""));
        eprintln!("\nrarity levels:");
        rarity_levels
            .iter()
            .for_each(|rarity| eprintln!("\"{rarity}\""));

        let print_visible = false;
        assert!(!print_visible);
    }
}
