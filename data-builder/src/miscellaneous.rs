use anyhow::{Result, bail};
use indexmap::IndexMap;

use crate::utils::{convert_string_to_normal_case, normalize_name};

// definition of string_dump inclusive ranges, which should be taken over into the miscellaneous lookup
//
// <string id="500">HP</string>
// <string id="501">Attack</string>
// <string id="502">Defense</string>
// <string id="503">Speed</string>
// <string id="504">Sp. Attack</string>
// <string id="505">Sp. Defense</string>
//
// <string id="510">Physical</string>
// <string id="511">Special</string>
// <string id="512">Status</string>
//
// <string id="1750">EGG</string>
// <string id="1751">TUTOR</string>
// <string id="1752">SPECIAL</string>
// <string id="1753">PREVO</string>
// <string id="1754">TM??</string>
// <string id="1755">EGG &amp; ITEM</string>
// <string id="1756">EVOLVE</string>
//
// <string id="1770">Morning</string>
// <string id="1771">Day</string>
// <string id="1772">Night</string>
//
// <string id="1773">Very Rare</string>
// <string id="1774">Very Common</string>
//
// <string id="1781">Rarity</string>
// <string id="1782">Common</string>
// <string id="1783">Uncommon</string>
// <string id="1784">Rare</string>
// <string id="1785">Grass</string>
// <string id="1786">Water</string>
// <string id="1787">Rocks</string>
// <string id="1788">Fishing</string>
// <string id="1789">Dark Grass</string>
// <string id="1790">Swarm</string>
//
// <string id="1792">Cave</string>
// <string id="1793">Inside</string>
// <string id="1794">Lure</string>
// <string id="1795">Special</string>
// <string id="1796">Shadow</string>
// <string id="1797">Dust Cloud</string>
// <string id="1798">Honey Tree</string>
// <string id="1799">Horde</string>
//
// <string id="181001">Monster</string>
// <string id="181002">Water A</string>
// <string id="181003">Bug</string>
// <string id="181004">Flying</string>
// <string id="181005">Field</string>
// <string id="181006">Fairy</string>
// <string id="181007">Plant</string>
// <string id="181008">Humanoid</string>
// <string id="181009">Water C</string>
// <string id="181010">Mineral</string>
// <string id="181011">Chaos</string>
// <string id="181012">Water B</string>
// <string id="181013">{STRING_150132}</string> // -> DITTO
// <string id="181014">Dragon</string>
// <string id="181015">Cannot Breed</string>
// <string id="181016">Genderless</string>
//
// <string id="180000">Hardy</string>
// <string id="180001">Lonely</string>
// <string id="180002">Brave</string>
// <string id="180003">Adamant</string>
// <string id="180004">Naughty</string>
// <string id="180005">Bold</string>
// <string id="180006">Docile</string>
// <string id="180007">Relaxed</string>
// <string id="180008">Impish</string>
// <string id="180009">Lax</string>
// <string id="180010">Timid</string>
// <string id="180011">Hasty</string>
// <string id="180012">Serious</string>
// <string id="180013">Jolly</string>
// <string id="180014">Naive</string>
// <string id="180015">Modest</string>
// <string id="180016">Mild</string>
// <string id="180017">Quiet</string>
// <string id="180018">Bashful</string>
// <string id="180019">Rash</string>
// <string id="180020">Calm</string>
// <string id="180021">Gentle</string>
// <string id="180022">Sassy</string>
// <string id="180023">Careful</string>
// <string id="180024">Quirky</string>
//
// <string id="250000">Kanto</string>
// <string id="250001">Hoenn</string>
// <string id="250002">Unova</string>
// <string id="250003">Sinnoh</string>
// <string id="250004">Johto</string>

const IDS_TO_TRANSLATE: &[(u32, u32)] = &[
    (500, 505),
    (510, 512),
    (1750, 1756),
    (1770, 1772),
    (1773, 1774),
    (1781, 1790),
    (1792, 1799),
    (180000, 180024),
    (181001, 181016),
    (250000, 250004),
];

pub fn build_miscellaneous_locale_lookup(
    en_string_dump: IndexMap<u32, String>,
    locale_string_dump: IndexMap<u32, String>,
    locale_additional_translations: IndexMap<String, String>,
) -> IndexMap<String, String> {
    println!("Building miscellaneous locale lookup...");

    let mut translations = locale_additional_translations;

    for (first, last) in IDS_TO_TRANSLATE {
        for i in *first..=*last {
            let Some(english) = en_string_dump.get(&i) else {
                continue;
            };
            let Some(locale) = locale_string_dump.get(&i) else {
                continue;
            };

            let key = match resolve_string_dump_references(english, &en_string_dump) {
                Ok(s) => s,
                Err(e) => {
                    println!("{e}");
                    continue;
                }
            };
            let value = match resolve_string_dump_references(locale, &locale_string_dump) {
                Ok(s) => s,
                Err(e) => {
                    println!("{e}");
                    continue;
                }
            };

            translations.insert(normalize_name(key), convert_string_to_normal_case(&value));
        }
    }
    // TODO: Types are still missing (not present in dumps)

    translations
}

fn resolve_string_dump_references(
    string: &String,
    string_dump: &IndexMap<u32, String>,
) -> Result<String> {
    let mut resolved = string.to_owned();
    if resolved.starts_with("{STRING_") {
        // only leaves ID + parse into u32
        let Ok(index) = string
            .chars()
            .filter(|c| c.is_numeric())
            .collect::<String>()
            .parse::<u32>()
        else {
            bail!("couln't parse the index of the string link, skipping this entry ({string}) ...") // something must've went wrong, just ignore this entry
        };

        resolved = match string_dump.get(&index) {
            Some(s) => s.to_string(),
            _ => bail!(
                "index {index} could not be resolved in string_dump, skipping entry ({string}) ..."
            ),
        };

        resolved = resolve_string_dump_references(&resolved, string_dump)?;
    }

    Ok(resolved)
}
