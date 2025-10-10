use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter},
    path::{Path, PathBuf},
};

use anyhow::bail;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub fn normalize_name(mut name: String) -> String {
    name = name.to_lowercase();
    name
}

/// Validate a directory exists, is a directory, and contains all `required_files`.
pub fn validate_dir(path: &Path, label: &str, required_files: &[&str]) -> anyhow::Result<()> {
    if !path.exists() {
        bail!("{} path does not exist: {}", label, path.display());
    }
    if !path.is_dir() {
        bail!("{} is not a directory: {}", label, path.display());
    }

    for &file in required_files {
        let fpath = path.join(file);
        if !fpath.exists() {
            bail!(
                "{} is missing required file '{}' (checked: {})",
                label,
                file,
                fpath.display()
            );
        }
    }

    Ok(())
}

pub fn parse_json_items_from_file<T>(file_path: &Path) -> Vec<T>
where
    T: DeserializeOwned,
{
    println!("Parsing items from file : {:?}", &file_path);

    let Ok(file) = File::open(file_path) else {
        println!("File coult not be opened: {:?}", file_path);
        return Vec::new();
    };

    let Ok(items) = serde_json::from_reader(file) else {
        println!("Couldn't parse file {:?}", file_path);
        return Vec::new();
    };

    items
}

pub fn parse_json_items_from_file_to_index_map<T, E>(file_path: &Path) -> IndexMap<T, E>
where
    T: DeserializeOwned + std::cmp::Eq + std::hash::Hash,
    E: DeserializeOwned,
{
    println!("Parsing items from file : {:?}", &file_path);

    let Ok(file) = File::open(file_path) else {
        println!("File coult not be opened: {:?}", file_path);
        return IndexMap::new();
    };

    let Ok(items) = serde_json::from_reader(file) else {
        println!("Couldn't parse file {:?}", file_path);
        return IndexMap::new();
    };

    items
}

#[derive(Debug, Deserialize)]
struct Strings {
    // child elements named <string>
    #[serde(rename = "string")]
    items: Vec<StringEntry>,
}

#[derive(Debug, Deserialize)]
struct StringEntry {
    // attribute `id="..."`
    #[serde(rename = "@id")]
    id: u32,

    // element text content (the text inside <string>...</string>)
    #[serde(rename = "$text")]
    #[serde(default)]
    value: String,
}

pub fn parse_string_items_from_xml_file(file_path: &Path) -> IndexMap<u32, String> {
    println!("Parsing items from file : {:?}", &file_path);

    let Ok(file) = File::open(file_path) else {
        println!("File coult not be opened: {:?}", file_path);
        return IndexMap::new();
    };

    let buf_reader = BufReader::new(file);
    let items = match quick_xml::de::from_reader::<BufReader<File>, Strings>(buf_reader) {
        Ok(items) => items,
        Err(e) => {
            println!("Couldn't parse file {:?}, because {e}", file_path);
            return IndexMap::new();
        }
    };

    let mut id_map: IndexMap<u32, String> = IndexMap::new();
    for entry in items.items {
        id_map.insert(entry.id, entry.value);
    }

    id_map
}

pub fn write_locale_lookup_to_disk(
    lookup: IndexMap<String, String>,
    file_path: impl Into<PathBuf>,
) -> anyhow::Result<()> {
    let path: PathBuf = file_path.into();

    println!("Writing locale lookup to disk : {:?}", path);

    let file = File::create(path)?;

    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &lookup)?;

    Ok(())
}

pub fn write_base_data_to_disk<T>(
    location_base_data: Vec<T>,
    file_path: PathBuf,
) -> anyhow::Result<()>
where
    T: Serialize,
{
    println!("Writing locale lookup to disk : {:?}", file_path);

    let file = File::create(file_path)?;

    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &location_base_data)?;

    Ok(())
}

pub fn copy_file_overwriting(src: &PathBuf, dst: &PathBuf) -> anyhow::Result<()> {
    // ensure destination directory exists
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    // copy will overwrite the destination file if it already exists
    let _ = fs::copy(src, dst);

    Ok(())
}

pub fn rewrite_file_lower_case(file_path: &PathBuf) -> anyhow::Result<()> {
    let tmp_path = &file_path.with_file_name("tmp.tmp");
    copy_file_overwriting(file_path, tmp_path)?;

    let original_text = fs::read_to_string(tmp_path)?;
    let lower_text = original_text.to_lowercase();
    if lower_text == original_text {
        // nothing to change — remove tmp and keep original untouched
        let _ = fs::remove_file(tmp_path);
        return Ok(());
    }

    fs::write(tmp_path, lower_text)?;

    // try atomic replace using rename -> no data-loss on crash etc.
    match fs::rename(tmp_path, file_path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            // non-atomic fallback for Windows (o.O), where rename can't replace existing files
            fs::remove_file(file_path)?;
            fs::rename(tmp_path, file_path)?;
            Ok(())
        }
        Err(e) => {
            // on error try to cleanup temp file and let error bubble up
            let _ = fs::remove_file(tmp_path);
            Err(e.into())
        }
    }
}
pub fn convert_string_to_normal_case(s: &str) -> String {
    let mut name: String = String::with_capacity(s.len());

    let mut prev: char = ' ';
    for c in s.chars() {
        // ignore letters that start a word or the first
        if prev.is_alphanumeric() && c.is_alphabetic() {
            // some chars become multiple chars when .uppercase / .lowercase
            for l in c.to_lowercase() {
                name.push(l);
            }
        } else {
            name.push(c);
        }

        prev = c;
    }

    name
}

#[cfg(test)]
mod tests {
    use super::convert_string_to_normal_case;

    #[test]
    fn test_convert_string_to_normal_case() {
        assert_eq!(convert_string_to_normal_case("ROCK TUNNEL"), "Rock Tunnel");
        assert_eq!(
            convert_string_to_normal_case("the ROCK tunnel"),
            "the Rock tunnel"
        );
        assert_eq!(convert_string_to_normal_case("O'NEIL"), "O'Neil");
        assert_eq!(
            convert_string_to_normal_case("SMITH-JOHNSON"),
            "Smith-Johnson"
        );
        assert_eq!(
            convert_string_to_normal_case("already lower"),
            "already lower"
        );
        assert_eq!(convert_string_to_normal_case("MiXeD Case"), "Mixed Case");
        assert_eq!(convert_string_to_normal_case("123 456"), "123 456"); // no alphabetic -> unchanged
    }
}
