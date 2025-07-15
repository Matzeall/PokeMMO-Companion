use crate::frontend::style;
use eframe::CreationContext;
use egui::{ImageSource, include_image};
use std::collections::HashMap;

mod icons {
    include!("gen_icon_includes.rs");
}

pub struct GuiSubsystem {
    pub icon_lookup: HashMap<String, ImageSource<'static>>,
}

impl GuiSubsystem {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut gui = Self {
            icon_lookup: HashMap::new(),
        };

        gui.fetch_image_source_data();

        style::setup_global_application_style(cc);

        gui
    }

    fn fetch_image_source_data(&mut self) {
        // fill from generated icon import list (gen_icon_includes.rs)
        self.icon_lookup = icons::get_icon_map();

        println!("\nGuiSubsystem - registering icons ...");
        for icon in self.icon_lookup.iter() {
            println!("registered icon => \"{}\"", icon.0);
        }
    }

    /// utility to dynamically get an image source, which loses the compile time checking of the
    /// images actually being present, but allows for dynamic image names (in loops with iterable
    /// source names etc).
    /// It also eliminates the resulting optional ambiguity by returning a "missing_icon" icon in
    /// case of error
    /// TODO: improve the accessibiltiy of this function with lazy-get/lazy-statics singleton
    pub fn get_image_source(&self, by_name: impl Into<String>) -> ImageSource<'static> {
        let id: String = by_name.into().to_lowercase();

        match self.icon_lookup.get(&id) {
            Some(source) => source.clone(),
            None => {
                eprintln!(
                    "icon with id \"{id}\" was requested, but doesn't exist => returning missing-icon icon"
                );
                // needs to exist so i
                // can rely on this function always returning me some kind of image without panicing
                include_image!("../../assets/icons/missing_icon.png")
            }
        }
    }
}
