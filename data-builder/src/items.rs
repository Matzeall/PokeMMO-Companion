use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Item {
    id: u32,
    name: String,
    desc: String,
}

pub fn build_item_locale_lookups(
    en_items: Vec<Item>,
    locale_items: Vec<Item>,
) -> (IndexMap<String, String>, IndexMap<String, String>) {
    println!("Building locale lookups...");
    let mut name_translations = IndexMap::new();
    let mut desc_translations = IndexMap::new();

    if en_items.len() != locale_items.len() {
        println!("Item lists don't have the same lengths, aborting ...");
        return (IndexMap::new(), IndexMap::new());
    }

    for i in 0..en_items.len() {
        if en_items[i].id != locale_items[i].id {
            println!(
                "item entry {i} did not match ids with locale entry ({} & {})",
                en_items[i].id, locale_items[i].id
            );
            continue;
        }

        let normalize_key = |mut key: String| -> String {
            key = key.to_lowercase();
            key
        };

        name_translations.insert(
            normalize_key(en_items[i].name.clone()),
            locale_items[i].name.clone(),
        );
        desc_translations.insert(
            normalize_key(en_items[i].desc.clone()),
            locale_items[i].desc.clone(),
        );
    }

    (name_translations, desc_translations)
}
