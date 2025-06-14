// contains small and common helper functions

use std::{
    fs,
    io::{self, ErrorKind, Result},
    path::PathBuf,
};

use crate::style;

pub fn find_asset_folder() -> io::Result<PathBuf> {
    const ASSETS_DIR_NAME: &str = "assets";

    let mut base_folder = std::env::current_exe();

    // prefer project root when in the dev environment
    if let Ok(current_dir) = std::env::current_dir() {
        if current_dir.join(ASSETS_DIR_NAME).is_dir() {
            base_folder = Ok(current_dir);
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

pub fn draw_highlight_underline(
    ui: &mut egui::Ui,
    hover_response: &egui::response::Response,
    bottom_offset: f32,
) {
    let y = hover_response.rect.bottom() - 1.0 + bottom_offset;
    ui.painter().line_segment(
        [
            egui::Pos2::new(hover_response.rect.min.x, y),
            egui::Pos2::new(hover_response.rect.max.x, y),
        ],
        egui::Stroke {
            width: 1.,
            color: style::COLOR_APPLINK_HOVER,
        },
    );
}
