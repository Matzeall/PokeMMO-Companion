use std::{cell::RefCell, collections::HashMap, io, path::PathBuf};

use crate::{
    style,
    utils::{self, find_asset_folder},
};
use egui::{Label, Sense, Ui, Vec2};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use regex::Regex;

pub struct RessourcesSubsystem {
    available_ressources: HashMap<String, Box<dyn Resource>>,

    current_resource: Option<String>,

    visited_resources: Vec<String>,
}

impl RessourcesSubsystem {
    pub fn new() -> Self {
        let mut ressources_subsystem = Self {
            available_ressources: HashMap::new(),
            current_resource: None,
            visited_resources: Vec::new(),
        };

        ressources_subsystem.load_resources();

        ressources_subsystem.set_root_resource();

        ressources_subsystem
    }

    fn load_resources(&mut self) {
        // TODO: actual load from disk/storage behaviour

        // // Mock resources
        // self.add_link_resource("PokeMMO Info", "https://pokemmo.info");
        // self.add_link_resource(
        //     "EV Hordes",
        //     "https://forums.pokemmo.com/index.php?/topic/108705-2025-all-horde-locations-ev-and-shiny/",
        // );
        // self.add_app_link("AppLink", "AppLink_2");
        // self.add_app_link("AppLink_2", "Root");
        //
        // self.add_markdown_resource("Markdown", "dnlfgidlfkgndlkfngldkngldnl dlkfjgbdljfbg");
        //
        // self.add_resource_collection(
        //     "Root",
        //     vec!["PokeMMO Info", "EV Hordes", "Markdown", "AppLink"],
        // );

        let resources_dir = self.resource_folder_path();

        let md_file_list = match resources_dir {
            Ok(dir) => utils::read_in_all_markdown_files(dir),
            Err(e) => Err(e),
        };

        match md_file_list {
            Ok(list) => {
                // DEBUG Output
                println!("Files found:");
                list.iter().for_each(|(file, _cont)| println!("  - {file}"));

                self.parse_into_resources(list);
            }
            Err(err) => {
                // add one "error" resource, which explains what went wrong to the user
                self.add_markdown_resource(
                    "ROOT",
                    format!("Error during .md file read: \n{0}", err),
                );
            }
        };
    }

    fn resource_folder_path(&mut self) -> Result<PathBuf, io::Error> {
        match find_asset_folder() {
            Ok(assets_folder) => {
                let resource_candidate = assets_folder.join("resources");
                if !resource_candidate.is_dir() {
                    Err(io::Error::new(
                        io::ErrorKind::NotADirectory,
                        format!("no folder called \"resources\" in {:?}", assets_folder),
                    ))
                } else {
                    Ok(resource_candidate)
                }
            }
            Err(e) => Err(e),
        }
    }

    fn parse_into_resources(&mut self, md_file_list: Vec<(String, String)>) {
        // regex building should never fail
        let regex_search_for_res_tag = Regex::new(r"<([^=]+)=([^,>]+)(?:,([^,>]+))?>").unwrap();

        for entry in md_file_list {
            let file_name = entry.0;
            let file_contents = entry.1;

            println!("\n============= Parsing File: {file_name} ==============================");

            let mut ordered_res_keys: Vec<String> = Vec::new();
            let mut md_content: String = String::default();
            let mut md_res_count: i32 = 0; // used to create unique resource names

            // needs to be a local fn with parameters, declaring as closure creates borrow issues
            // markdown is collected line by line and added as one resource whenever this fn is called
            fn add_collected_markdown(
                md_content: &mut String,
                file_name: &String,
                md_res_count: &mut i32,
                ordered_res_keys: &mut Vec<String>,
                this: &mut RessourcesSubsystem,
            ) {
                if !md_content.is_empty() {
                    let md_res_key =
                        format!("inner_{0}_md_{1}", file_name.to_owned(), md_res_count);
                    *md_res_count += 1;
                    this.add_markdown_resource(&md_res_key, md_content.clone());
                    ordered_res_keys.push(md_res_key.to_owned());
                    println!(
                        "- Markdown Section (key: {0})\n------------------------------------------\n{1}\n------------------------------------------",
                        &md_res_key, &md_content
                    );
                    md_content.clear();
                }
            }

            // parse line by line, checking for reserved resource tags or collecting as generic markdown
            for line in file_contents.lines() {
                let mut inserted_res: Option<String> = None;
                // check for resource tag
                if let Some(res_tag) = regex_search_for_res_tag.captures(line) {
                    let res_type: &str = res_tag[1].trim(); // are guaranteed to exist
                    let res_text: &str = res_tag[2].trim(); // otherwise would regex not match
                    let res_data: &str = res_tag.get(3).map(|m| m.as_str()).unwrap_or("").trim();

                    println!(
                        "- {1} : \"{2}\" » \"{3}\" from : \"{0}\"",
                        &res_tag[0], &res_type, &res_text, &res_data
                    );

                    match res_type {
                        "AppLink" => {
                            inserted_res = Some(self.add_app_link(res_text, res_data.to_owned()));
                        }
                        "WebLink" => {
                            inserted_res =
                                Some(self.add_web_link_resource(res_text, res_data.to_owned()));
                        }
                        _ => {
                            inserted_res = None;
                        }
                    }
                }

                if let Some(inserted_res_key) = inserted_res {
                    // add accumulated markdown resource before
                    add_collected_markdown(
                        &mut md_content,
                        &file_name,
                        &mut md_res_count,
                        &mut ordered_res_keys,
                        self,
                    );

                    // now add inserted resource
                    ordered_res_keys.push(inserted_res_key.to_owned());
                    continue;
                }

                // no resource tag => treat as regular markdown & collect
                md_content.push_str(line); // just add to next markdown resource
                md_content.push('\n'); // line does not include the original line break
            }

            // last element could be a collected but uncreated markdown resource
            add_collected_markdown(
                &mut md_content,
                &file_name,
                &mut md_res_count,
                &mut ordered_res_keys,
                self,
            );

            self.add_resource_collection(file_name.to_owned(), ordered_res_keys);
            println!("=================================================================")
        }
    }

    // creates and adds a resource collection
    // then returns the key it was added under for convenience
    fn add_resource_collection(
        &mut self,
        title: impl Into<String>,
        resource_list: Vec<impl Into<String>>,
    ) -> String {
        let col_title = title.into();
        let resource_collection = ResourceCollection::new(
            &col_title,
            resource_list
                .into_iter()
                .map(|r| -> String { Into::<String>::into(r) })
                .collect(),
        );

        self.available_ressources.insert(
            resource_collection.get_title(),
            Box::new(resource_collection),
        );
        col_title
    }
    // creates and adds a web link resource
    // then returns the key it was added under for convenience
    fn add_web_link_resource(&mut self, text: impl Into<String>, url: impl Into<String>) -> String {
        let title = text.into();
        let link = WebLinkResource::new(&title, url);
        self.available_ressources
            .insert(title.to_owned(), Box::new(link));
        title
    }

    // creates and adds a app link resource
    // then returns the key it was added under for convenience
    // This key can be different if a resource name collision was fixed!
    fn add_app_link(&mut self, text: impl Into<String>, link_to: impl Into<String>) -> String {
        let mut title = text.into();
        let link: String = link_to.into();

        if title.eq(&link) {
            // fix up name collision between resource link_to and this link
            title.push_str("_AppLink");
            println!(
                "---> Resource Name Collision Fixup: {0} <-- +_AppLink",
                &title
            );
        }

        let app_link = AppLinkResource::new(&title, &link);
        self.available_ressources
            .insert(app_link.get_title(), Box::new(app_link));
        title
    }

    // creates and adds a markdown resource
    // then returns the key it was added under for convenience
    fn add_markdown_resource(
        &mut self,
        res_title: impl Into<String>,
        markdown_text: impl Into<String>,
    ) -> String {
        let title = res_title.into();
        let markdown = MarkdownResource::new(&title, markdown_text);
        self.available_ressources
            .insert(markdown.get_title(), Box::new(markdown));
        title
    }

    fn set_root_resource(&mut self) {
        self.set_current_resource("ROOT", false);
    }

    pub fn set_current_resource(&mut self, name: impl Into<String>, save_last_in_history: bool) {
        let name_string: String = name.into();
        if self
            .current_resource
            .as_ref()
            .is_none_or(|cur_res| !cur_res.eq(&name_string)) // is not current and is valid
            && self.available_ressources.contains_key(&name_string)
        {
            // commit to switching resource
            if save_last_in_history {
                self.current_resource.as_ref().inspect(|cur_res| {
                    let old_res = (*cur_res).clone();
                    // maybe check for loops?
                    self.visited_resources.push(old_res);
                });
            }
            self.current_resource = Some(name_string);
        }
    }

    pub fn go_back_visited_resources(&mut self) {
        let last_res_possible = self.visited_resources.pop();
        if let Some(last_res) = last_res_possible {
            self.set_current_resource(last_res, false);
        };
    }

    pub fn inspect_last_resource(&self) -> Option<String> {
        self.visited_resources.last().cloned()
    }

    pub fn get_resource(&self, key: impl Into<String>) -> Option<&dyn Resource> {
        self.available_ressources
            .get(&key.into())
            .map(|boxed| boxed.as_ref())
    }

    pub fn get_current_resource(&self) -> Option<&dyn Resource> {
        self.current_resource
            .as_ref()
            .and_then(|key| self.get_resource(key))
    }

    pub fn render_current_resource(&mut self, ui: &mut Ui) {
        if let Some(cur_res) = self.get_current_resource() {
            // `self.get_current_resource()` is effectively
            //      (&mut self).get_current_resource()
            if let Some(new_res) = cur_res.render_resource(self, ui) {
                // now that the `&self` borrow is gone, we can
                // safely do a mutable operation on `self`.
                self.set_current_resource(&new_res, true);
            }
        }
    }
}

pub trait Resource {
    fn get_title(&self) -> String;

    // renders the resource to the provided Ui and returns an optional new current resource
    fn render_resource(
        &self,
        resource_subsystem: &RessourcesSubsystem,
        ui: &mut Ui,
    ) -> Option<String>;
}

////////////////////////////////////////
/// Resource Collection
////////////////////////////////////////
/// basically the backbone of each resource page
/// manages the rendering of all specific resource components present on that page
pub struct ResourceCollection {
    resource_title: String,

    resource_list: Vec<String>,
}

impl ResourceCollection {
    pub fn new(resource_title: impl Into<String>, ordered_res_keys: Vec<String>) -> Self {
        Self {
            resource_title: resource_title.into(),
            resource_list: ordered_res_keys,
        }
    }
}

impl Resource for ResourceCollection {
    fn get_title(&self) -> String {
        self.resource_title.clone()
    }

    fn render_resource(
        &self,
        resource_subsystem: &RessourcesSubsystem,
        ui: &mut Ui,
    ) -> Option<String> {
        let mut clicked_link = None;

        for res_key in &self.resource_list {
            match resource_subsystem.get_resource(res_key) {
                Some(resource) => {
                    if let Some(clicked) = resource.render_resource(resource_subsystem, ui) {
                        clicked_link = Some(clicked);
                    }
                }
                None => println!("no resource for res_key ({res_key}) found"),
            }
        }
        clicked_link // if one was clicked, return requested resource
    }
}

//////////////////////////////////
/// Web Link Resource
//////////////////////////////////
pub struct WebLinkResource {
    link_text: String,
    link_url: String,
}

impl WebLinkResource {
    fn new(text: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            link_text: text.into(),
            link_url: url.into(),
        }
    }
}

impl Resource for WebLinkResource {
    fn get_title(&self) -> String {
        self.link_text.clone()
    }

    fn render_resource(
        &self,
        _resource_subsystem: &RessourcesSubsystem,
        ui: &mut Ui,
    ) -> Option<String> {
        ui.hyperlink_to(&self.link_text, &self.link_url);
        None
    }
}

//////////////////////////////////
/// In App Link to another resource
//////////////////////////////////
pub struct AppLinkResource {
    link_text: String,
    link_to: String,
}

impl AppLinkResource {
    fn new(resource_title: impl Into<String>, link_to: impl Into<String>) -> Self {
        Self {
            link_text: resource_title.into(),
            link_to: link_to.into(),
        }
    }
}

impl Resource for AppLinkResource {
    fn get_title(&self) -> String {
        self.link_text.clone()
    }

    fn render_resource(
        &self,
        _resource_subsystem: &RessourcesSubsystem,
        ui: &mut Ui,
    ) -> Option<String> {
        let mut link_to = None;
        ui.horizontal(|ui| {
            // setup custom interactable widget style, because buttons and selectable labels share a style
            let style = ui.style_mut();
            style.visuals.widgets.inactive.fg_stroke.color = style::COLOR_APPLINK_REST; // normal
            style.visuals.widgets.hovered.fg_stroke.color = style::COLOR_APPLINK_HOVER; // hover
            style.visuals.widgets.active.fg_stroke.color = style::COLOR_APPLINK_REST; // active
            style.spacing.item_spacing = Vec2::new(3., style.spacing.item_spacing.y);

            // actual rendering
            let text = self.link_text.trim_end_matches("_AppLink"); // meh ... I kinda wish I
            // hadn't double used the link_text as resource key and display text right now.

            let title = Label::new(text).sense(Sense::click());
            let response = ui.add(title);

            let mut indicator_clicked = false;
            if response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);

                utils::draw_highlight_underline(ui, &response, 2.);

                // draw indicator arrow
                let indicator_label = Label::new("»").sense(Sense::click());
                indicator_clicked = ui.add(indicator_label).clicked();
            }

            match response.clicked() || indicator_clicked {
                true => link_to = Some(self.link_to.clone()),
                false => link_to = None,
            }
        });
        link_to
    }
}

////////////////////////////////////////////
/// Markdown Resource
////////////////////////////////////////////
/// any plain text (or md) that should just be displayed non-interactively
pub struct MarkdownResource {
    resource_title: String,
    markdown_text: String,
    markdown_cache: RefCell<CommonMarkCache>,
}

impl MarkdownResource {
    pub fn new(resource_title: impl Into<String>, markdown_text: impl Into<String>) -> Self {
        Self {
            resource_title: resource_title.into(),
            markdown_text: markdown_text.into(),
            markdown_cache: RefCell::new(CommonMarkCache::default()),
        }
        // DEBUG
        // let title: String = resource_title.into();
        // Self {
        //     resource_title: title.clone(),
        //     markdown_text: title.clone(),
        //     markdown_cache: RefCell::new(CommonMarkCache::default()),
        // }
    }
}

impl Resource for MarkdownResource {
    fn get_title(&self) -> String {
        self.resource_title.clone()
    }

    fn render_resource(
        &self,
        _resource_subsystem: &RessourcesSubsystem,
        ui: &mut Ui,
    ) -> Option<String> {
        let markdown_viewer = CommonMarkViewer::new();
        markdown_viewer.show(
            ui,
            &mut self.markdown_cache.borrow_mut(),
            &self.markdown_text,
        );
        None
    }
}

/////////////////////////////////////////////
///  Tests
/////////////////////////////////////////////
#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_something() {
        assert!(true == true);
    }
}
