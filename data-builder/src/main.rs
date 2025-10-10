mod generic_entry;
mod items;
mod locations;
mod miscellaneous;
mod utils;

use clap::{ArgAction, Parser};
use generic_entry::Entry;
use indexmap::IndexMap;
use items::Item;
use locations::MonsterLocations;
use std::path::PathBuf;
use utils::{
    parse_json_items_from_file, parse_json_items_from_file_to_index_map,
    parse_string_items_from_xml_file, validate_dir, write_locale_lookup_to_disk,
};

/// Example call: ./data-builder "EN_DIR" "DE_DIR" --out "OUT_DIR"
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "After validating both directories exist and contain valid files it will try to build the locale lookup files by diff-ing the source locale_dir with the source en_dir + some additional magic"
)]
struct CLIArgs {
    /// Path to the EN directory (required)
    en_dir: PathBuf,

    /// Path to the new locale directory (required)
    locale_dir: PathBuf,

    /// Optional output directory
    #[arg(short, long)]
    out: Option<PathBuf>,

    /// Optional instruction to also build the base-data (location lookup etc.) and give it an output directory
    #[arg(short, long)]
    base_data_out: Option<PathBuf>,

    #[arg(short, long, action = ArgAction::SetTrue)]
    normal_case_base_data: bool,
}

fn main() -> anyhow::Result<()> {
    let args = CLIArgs::parse();

    println!(
        "-------------------------------------------------------------------\nStarting data-builder...\nLooking for source data at :\n    {:?}\n    and\n    {:?}",
        args.en_dir, args.locale_dir
    );

    const REQUIRED_FILES: &[&str] = &[
        "monsters.json",
        "items.json",
        "skills.json",
        "dump_strings.xml",
    ];
    validate_dir(&args.en_dir, "EN Directory", REQUIRED_FILES)?;
    validate_dir(&args.locale_dir, "Locale Directory", REQUIRED_FILES)?;

    let out_dir = args.out.unwrap_or(
        std::env::current_dir()
            .expect("Could not locate current directory and none was specified. Aborting ..."),
    );
    println!("Using out dir : {:?}", &out_dir);

    println!("\n---------- Monsters ----------");
    {
        let mut en_entries =
            parse_json_items_from_file::<Entry>(&args.en_dir.join("monsters.json"));
        let mut locale_entries =
            parse_json_items_from_file::<Entry>(&args.locale_dir.join("monsters.json"));
        // id's over 1000 are event specifics and stuff -> remove
        en_entries.retain(|e| e.id < 1000);
        locale_entries.retain(|e| e.id < 1000);
        let name_lookup = generic_entry::build_generic_locale_lookup(en_entries, locale_entries);
        write_locale_lookup_to_disk(name_lookup, out_dir.clone().join("monsters.json"))?;
    }

    println!("\n---------- ITEMS -------------");
    {
        let en_items = parse_json_items_from_file::<Item>(&args.en_dir.join("items.json"));
        let locale_items = parse_json_items_from_file::<Item>(&args.locale_dir.join("items.json"));
        let (name_lookup, desc_lookup) = items::build_item_locale_lookups(en_items, locale_items);
        write_locale_lookup_to_disk(name_lookup, out_dir.clone().join("item_names.json"))?;
        write_locale_lookup_to_disk(desc_lookup, out_dir.clone().join("item_descriptions.json"))?;
    }

    println!("\n---------- Skills ----------");
    {
        let en_entries = parse_json_items_from_file::<Entry>(&args.en_dir.join("skills.json"));
        let locale_entries =
            parse_json_items_from_file::<Entry>(&args.locale_dir.join("skills.json"));
        let name_lookup = generic_entry::build_generic_locale_lookup(en_entries, locale_entries);
        write_locale_lookup_to_disk(name_lookup, out_dir.clone().join("skills.json"))?;
    }

    println!("\n---------- Locations ----------");
    {
        let en_monster_locations =
            parse_json_items_from_file::<MonsterLocations>(&args.en_dir.join("monsters.json"));
        let en_string_dump =
            parse_string_items_from_xml_file(&args.en_dir.join("dump_strings.xml"));
        let locale_string_dump =
            parse_string_items_from_xml_file(&args.locale_dir.join("dump_strings.xml"));
        let locale_additional_translations: IndexMap<String, String> =
            parse_json_items_from_file_to_index_map::<String, String>(
                &args.locale_dir.join("additional_translations.json"),
            )
            .into_iter()
            .collect();
        let (unique_location_lookup, pokedex_location_lookup) =
            locations::build_location_locale_lookup(
                en_monster_locations,
                en_string_dump,
                locale_string_dump,
                locale_additional_translations,
            );
        write_locale_lookup_to_disk(
            unique_location_lookup,
            out_dir.clone().join("locations.json"),
        )?;
        write_locale_lookup_to_disk(
            pokedex_location_lookup,
            out_dir.clone().join("locations_pokedex.json"),
        )?;
    }

    println!("\n---------- Miscellaneous ----------");
    {
        // let en_monster_locations =
        //     parse_json_items_from_file::<MonsterLocations>(&args.en_dir.join("monsters.json"));
        let en_string_dump =
            parse_string_items_from_xml_file(&args.en_dir.join("dump_strings.xml"));
        let locale_string_dump =
            parse_string_items_from_xml_file(&args.locale_dir.join("dump_strings.xml"));
        let locale_additional_translations: IndexMap<String, String> =
            parse_json_items_from_file_to_index_map::<String, String>(
                &args.locale_dir.join("additional_translations.json"),
            )
            .into_iter()
            .collect();
        let miscellaneous_locale_lookup = miscellaneous::build_miscellaneous_locale_lookup(
            en_string_dump,
            locale_string_dump,
            locale_additional_translations,
        );
        write_locale_lookup_to_disk(
            miscellaneous_locale_lookup,
            out_dir.clone().join("miscellaneous.json"),
        )?;
    }

    if let Some(base_data_out) = &args.base_data_out {
        println!("\n---------- Base Data ----------");
        let en_monster_locations =
            parse_json_items_from_file::<MonsterLocations>(&args.en_dir.join("monsters.json"));
        let location_base_data = locations::build_location_base_data(en_monster_locations);
        utils::write_base_data_to_disk(location_base_data, base_data_out.join("locations.json"))?;
        // hack to make all keys lower_case() for easier lookup into locales
        // this might not be desired, if the data only will be used in english
        if !args.normal_case_base_data {
            utils::rewrite_file_lower_case(&base_data_out.join("locations.json"))?;
        }

        // copy pokedex dump date to base-data directory
        const POKEDEX_DUMP_FILES: &[&str] = &["monsters.json", "items.json", "skills.json"];
        for filename in POKEDEX_DUMP_FILES {
            utils::copy_file_overwriting(
                &args.en_dir.join(filename),
                &base_data_out.join(filename),
            )?;

            // same hack for lowercase keys
            if !args.normal_case_base_data {
                utils::rewrite_file_lower_case(&base_data_out.join(filename))?;
            }
        }
    }
    println!("\n-------------------------------------------------------------------\nFinished !");
    Ok(())
}
