// contains small and common helper functions

use std::{
    fs,
    io::{self, ErrorKind, Result},
    path::PathBuf,
};

pub fn find_asset_folder() -> io::Result<PathBuf> {
    const ASSETS_DIR_NAME: &str = "assets";
    const CRATE_DIR_NAME: &str = "companion-app";
    const POSSIBLE_SUB_PATHS: [&str; 2] = ["", CRATE_DIR_NAME];

    // use folder the executable is in
    let mut base_folder = std::env::current_exe();
    if let Ok(ref mut folder) = base_folder {
        if folder.pop() {
            base_folder = Ok(folder.clone());
        }
    }

    // prefer source code project root when in the dev environment
    #[cfg(debug_assertions)]
    if let Ok(current_dir) = std::env::current_dir() {
        // build possible sub-paths + "assets" and see if it is there
        for sub_path in POSSIBLE_SUB_PATHS {
            let path: String = format!("{sub_path}/{ASSETS_DIR_NAME}");

            if current_dir.join(path).is_dir() {
                base_folder = Ok(PathBuf::from(sub_path));
                break;
            }
        }
    }

    if let Ok(root_folder) = &base_folder {
        let candidate = root_folder.join(ASSETS_DIR_NAME);
        if candidate.is_dir() {
            return Ok(candidate);
        }
        return Err(io::Error::new(
            ErrorKind::NotADirectory,
            format!("no {ASSETS_DIR_NAME} directory in {:?}", base_folder),
        ));
    }

    base_folder // bubble up error
}

pub fn read_in_all_markdown_files(path: PathBuf) -> Result<Vec<(String, String)>> {
    let mut md_file_list = Vec::new();

    let read_dir = fs::read_dir(path)?; // stop and let error bubble up

    for entry in read_dir {
        if entry.is_err() {
            // skip over any single file error and ready the rest
            continue;
        }
        let path = entry?.path();

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext.eq_ignore_ascii_case("md") {
                    // read contents or write error into the resource
                    let contents = fs::read_to_string(&path).unwrap_or_else(|e| e.to_string());
                    // only add resource if filename(used as identifier) is valid, ignore otherwise
                    if let Some(stem) = path.file_stem().and_then(|os| os.to_str()) {
                        md_file_list.push((stem.to_owned(), contents));
                    }
                }
            }
        }
    }
    Ok(md_file_list)
}
