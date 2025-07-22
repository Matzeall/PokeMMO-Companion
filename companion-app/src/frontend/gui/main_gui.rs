use crate::frontend::style;
use crate::{app::OverlayApp, backend::feature_state::Feature};
use egui::{Align2, Color32, Image, ImageButton, Vec2, Window};
use std::collections::HashMap;
use strum::IntoEnumIterator;

use super::{
    breeding_calculator::draw_breeding_calculator_panel, notes::draw_notes_panel,
    resources::draw_ressources_panel, settings::draw_options_panel,
    type_matrix::draw_type_matrix_panel,
};

pub fn draw_gui(ctx: &egui::Context, _frame: &mut eframe::Frame, state: &mut OverlayApp) {
    #[cfg(debug_assertions)]
    draw_perf_panel(ctx, _frame);

    // draw UI based on AppState
    if state.viewport_manager.current_focus_state().is_focused() {
        draw_control_panel(ctx, state);
    }

    draw_notes_panel(ctx, state);

    draw_ressources_panel(ctx, state);

    draw_type_matrix_panel(ctx, state);

    draw_breeding_calculator_panel(ctx, state);

    draw_options_panel(ctx, state);
}

////////////////////////////////////////////////////////////////////////////
///  Control Bar
////////////////////////////////////////////////////////////////////////////
pub fn draw_control_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    let feature_shortcuts: HashMap<Feature, &str> = HashMap::from([
        (Feature::Notes, "(Alt+N)"),
        (Feature::Resources, "(Alt+R)"),
        (Feature::TypeMatrix, "(Alt+T)"),
        (Feature::BreedingCalculator, "(Alt+B)"),
        (Feature::Settings, "(Alt+O)"),
    ]);

    Window::new("ControlPanel")
        .frame(
            if state.viewport_manager.current_focus_state().is_focused() {
                style::CUSTOM_FRAME_FOCUSSED
            } else {
                style::CUSTOM_FRAME
            },
        )
        .anchor(Align2::CENTER_BOTTOM, Vec2::new(0., -20.))
        .title_bar(false)
        .resizable(false)
        .auto_sized() // necessary so the remembered window size doesn't stop up-sizing
        // for new content
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // temporarily override innner button padding because images need more
                let button_padding = &mut ui.spacing_mut().button_padding;
                *button_padding = Vec2::new(7., 7.);

                let control_bar_height = 40.;
                ui.set_height(control_bar_height);

                // create one control button for each feature
                for feature in Feature::iter() {
                    let feature_image = Image::new(
                        state
                            .gui
                            .get_image_source(format!("feature_{}", feature.get_name())),
                    )
                    .alt_text(feature.get_name());

                    let mut feature_button =
                        ImageButton::new(feature_image).corner_radius(control_bar_height / 2.);

                    if state.features.is_feature_active(feature) {
                        feature_button = feature_button.selected(true);
                    }

                    let mut hover_text = feature.get_name();
                    hover_text.push_str(
                        format!(" {}", feature_shortcuts.get(&feature).unwrap_or(&" - ")).as_str(),
                    );
                    if ui.add(feature_button).on_hover_text(hover_text).clicked() {
                        state.features.set_feature_active(
                            feature,
                            !state.features.is_feature_active(feature),
                        );
                    }
                }

                let shutdown_image =
                    Image::new(state.gui.get_image_source("shutdown_button")).alt_text("Close");
                ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                    Color32::from_rgb(110, 32, 32);
                ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                    Color32::from_rgb(160, 32, 32);
                let close_btn =
                    ImageButton::new(shutdown_image).corner_radius(control_bar_height / 2.);

                if ui
                    .add(close_btn)
                    .on_hover_text("Close Companion App (Ctrl+D)")
                    .clicked()
                    || ctx.input(|i| i.key_pressed(egui::Key::D) && i.modifiers.ctrl)
                {
                    println!("shutdown!");
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
}

////////////////////////////////////////////////////////////////////////////
///  Debug Perf Panel
////////////////////////////////////////////////////////////////////////////
#[cfg(debug_assertions)]
pub fn draw_perf_panel(ctx: &egui::Context, frame: &mut eframe::Frame) {
    use egui::Frame;
    use egui::Label;

    Window::new("Perf")
        .frame(Frame {
            shadow: egui::Shadow::NONE,
            ..Frame::window(&ctx.style())
        })
        .title_bar(false)
        .anchor(Align2::RIGHT_TOP, Vec2::new(-10., 10.))
        .movable(false)
        .resizable(false)
        .auto_sized()
        .min_width(0.0)
        .default_size(Vec2::ZERO)
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                let delta_time: f32 = frame.info().cpu_usage.unwrap_or(0.);
                ui.add(
                    Label::new(format!("deltaTime: {:>4.2}ms", delta_time * 1000.))
                        .wrap_mode(egui::TextWrapMode::Extend),
                );
                ui.add(
                    Label::new(format!("fps: {:>4.1}hz", 1. / delta_time))
                        .wrap_mode(egui::TextWrapMode::Extend),
                );
            });
        });
}
