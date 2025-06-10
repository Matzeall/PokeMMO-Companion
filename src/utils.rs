// contains small and common helper functions

use std::{fs, io::Result, path::PathBuf};

use crate::style;

pub fn find_asset_folder() -> PathBuf {
    // 1) next to the running binary:
    let mut exe_folder = std::env::current_exe().expect("current_exe failed");
    exe_folder.pop(); // strip off the executable name

    let candidate = exe_folder.join("assets");
    if candidate.is_dir() {
        return candidate;
    }

    // 2) Fallback: if run with `cargo run` from the project root,
    //    CWD is the project directory (next to Cargo.toml), so look there:
    if let Ok(cwd) = std::env::current_dir() {
        let fallback = cwd.join("assets");
        if fallback.is_dir() {
            return fallback;
        }
    }

    panic!(
        "Could not locate an `assets` folder. Looked at:\n  {:#?}\n  {:#?}",
        candidate,
        std::env::current_dir().unwrap_or_else(|_| "<unknown cwd>".into())
    );
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
