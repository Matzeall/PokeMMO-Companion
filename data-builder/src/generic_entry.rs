use indexmap::IndexMap;
use serde::Deserialize;

/// generic entries require the two fields id and name
#[derive(Deserialize, Debug)]
pub struct Entry {
    pub id: u32,
    pub name: String,
}

pub fn build_generic_locale_lookup(
    en_entries: Vec<Entry>,
    locale_entries: Vec<Entry>,
) -> IndexMap<String, String> {
    println!("Building locale lookups...");
    let mut name_translations = IndexMap::new();

    if en_entries.len() != locale_entries.len() {
        println!("Entry lists don't have the same lengths, aborting ...");
        return IndexMap::new();
    }

    for i in 0..en_entries.len() {
        if en_entries[i].id != locale_entries[i].id {
            println!(
                "entry {i} did not match ids with locale entry ({} & {})",
                en_entries[i].id, locale_entries[i].id
            );
            continue;
        }

        let normalize_key = |mut key: String| -> String {
            key = key.to_lowercase();
            key
        };

        name_translations.insert(
            normalize_key(en_entries[i].name.clone()),
            locale_entries[i].name.clone(),
        );
    }

    name_translations
}
