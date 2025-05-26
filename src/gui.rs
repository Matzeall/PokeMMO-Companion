// contains all helper functions to render UI to keep the update function in app.rs clean

use egui::{Align2, Button, Color32, Vec2};
use strum::IntoEnumIterator;

use crate::{
    app::{Feature, OverlayApp},
    color_palette,
};

pub fn draw_control_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    egui::Window::new("ControlPanel")
        .frame(color_palette::CUSTOM_FRAME)
        .anchor(Align2::CENTER_BOTTOM, Vec2::new(0., 10.))
        .title_bar(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // // temporarily override spacing
                // let spacing = &mut ui.spacing_mut().item_spacing;
                // *spacing = Vec2::new(20.0, 8.); // 20 px between items horizontally

                //create one control button for each feature
                for feature in Feature::iter() {
                    let feature_name = format!("{:?}", feature);
                    let mut feature_button = Button::new(feature_name).corner_radius(20);

                    if state.is_feature_active(feature) {
                        feature_button = feature_button.selected(true);
                    }

                    if ui.add(feature_button).clicked() {
                        state.set_feature_active(feature, !state.is_feature_active(feature));
                    }
                }
                let close_btn = Button::new("close")
                    .fill(Color32::from_rgb(186, 50, 50))
                    .corner_radius(20);

                if ui.add(close_btn).clicked() {
                    println!("close");
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
}

pub fn draw_perf_panel(ctx: &egui::Context, frame: &mut eframe::Frame) {
    egui::Window::new("Perf")
        .title_bar(false)
        .anchor(Align2::RIGHT_TOP, Vec2::new(10., 10.))
        .resizable(false)
        .auto_sized()
        .movable(false)
        .show(ctx, |ui| {
            let delta_time: f32 = frame.info().cpu_usage.unwrap_or_default() * 1000.;
            ui.label(format!("deltaTime: {:02.2}ms", delta_time));
        });
}

fn add_base_window(
    ctx: &egui::Context,
    window_name: impl Into<egui::WidgetText>,
    open_state: &mut bool,
    add_contents: impl Fn(&mut egui::Ui), // I am sure only the rust compiler knows what type that is
) {
    egui::Window::new(window_name)
        .frame(color_palette::CUSTOM_FRAME)
        .resizable(true)
        .default_size(egui::vec2(300.0, 200.0))
        .min_size(egui::vec2(0., 0.))
        .fade_in(true)
        .fade_out(true)
        .open(open_state)
        .show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, add_contents);
        });
}

///// Feature Windows ////////////////////////////////////////////////////////////////

pub fn draw_notes_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    add_base_window(
        ctx,
        "Notes",
        state.get_feature_active_mut_ref(Feature::Notes),
        |_ui| {},
    );
}

pub fn draw_ressources_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    add_base_window(
        ctx,
        "Ressources",
        state.get_feature_active_mut_ref(Feature::Ressources),
        |ui| {
            ui.label("Hello Ressources");
            ui.hyperlink_to("PokeMMO Info", "https://pokemmo.info");
            ui.hyperlink_to("EV Hordes","https://forums.pokemmo.com/index.php?/topic/108705-2025-all-horde-locations-ev-and-shiny/");
            ui.label("Hello Ressources");
            ui.hyperlink_to("PokeMMO Info", "https://pokemmo.info");
            ui.hyperlink_to("EV Hordes","https://forums.pokemmo.com/index.php?/topic/108705-2025-all-horde-locations-ev-and-shiny/");
            ui.label("Hello Ressources");
            ui.hyperlink_to("PokeMMO Info", "https://pokemmo.info");
            ui.hyperlink_to("EV Hordes","https://forums.pokemmo.com/index.php?/topic/108705-2025-all-horde-locations-ev-and-shiny/");
            ui.label("Hello Ressources");
            ui.hyperlink_to("PokeMMO Info", "https://pokemmo.info");
            ui.hyperlink_to("EV Hordes","https://forums.pokemmo.com/index.php?/topic/108705-2025-all-horde-locations-ev-and-shiny/");
            ui.label("Hello Ressources");
            ui.hyperlink_to("PokeMMO Info", "https://pokemmo.info");
            ui.hyperlink_to("EV Hordes","https://forums.pokemmo.com/index.php?/topic/108705-2025-all-horde-locations-ev-and-shiny/");
        },
    );
}

pub fn draw_type_matrix_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    add_base_window(
        ctx,
        "Type Matrix",
        state.get_feature_active_mut_ref(Feature::TypeMatrix),
        |_ui| {},
    );
}

pub fn draw_breeding_calculator_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    add_base_window(
        ctx,
        "Breeding Calculator",
        state.get_feature_active_mut_ref(Feature::BreedingCalculator),
        |_ui| {},
    );
}
