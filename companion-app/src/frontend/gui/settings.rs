use crate::{
    app::OverlayApp,
    backend::{feature_state::Feature, settings::SettingsSubsystem},
    frontend::utils::construct_base_window,
};
use egui::{
    Align, Button, Checkbox, Frame, Layout, Margin, Response, ScrollArea, Separator, Slider, Vec2,
    Widget,
};

pub fn draw_options_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    construct_base_window("Settings", state.viewport_manager.as_ref())
        .resizable([false, true])
        .open(state.features.get_feature_active_mut_ref(Feature::Settings))
        .show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink([true, false])
                .show(ui, |ui| {
                    Frame::default()
                        .inner_margin(Margin::symmetric(10, 0))
                        .show(ui, |ui| {
                            ui.with_layout(Layout::top_down(Align::Max), |ui| {
                                global_application_scale_slider(ctx, ui);

                                disable_overlay_checkbox(ui, &mut state.settings);

                                transparent_bg_always_checkbox(ui, &mut state.settings);

                                typematrix_scale_slider(ui, &mut state.settings);

                                reset_ui_data(ui, &mut state.settings);
                            });
                            state.settings.draw_dev_options(ui);
                        });
                });
        });
}

fn add_default_sized_setting(ui: &mut egui::Ui, widget: impl Widget) -> Response {
    let response = ui.add_sized(Vec2::new(ui.available_width(), 30.), widget);
    ui.add(Separator::default().grow(5.));

    response
}

fn disable_overlay_checkbox(ui: &mut egui::Ui, settings: &mut SettingsSubsystem) {
    let checkbox = Checkbox::new(&mut settings.disable_overlay, "Disable Overlay");
    if add_default_sized_setting(ui, checkbox).changed() {
        settings.request_viewport_restart = true;
    }
}

fn transparent_bg_always_checkbox(ui: &mut egui::Ui, settings: &mut SettingsSubsystem) {
    let checkbox = Checkbox::new(
        &mut settings.transparent_background_always,
        "Transparent BG\nalways",
    );
    add_default_sized_setting(ui, checkbox);
}

fn reset_ui_data(ui: &mut egui::Ui, settings: &mut SettingsSubsystem) {
    let trigger_button = Button::new("Reset UI Data (e.g. Window positions)");
    if add_default_sized_setting(ui, trigger_button).clicked() {
        settings.request_clear_ui_data = true;
    }
}

fn typematrix_scale_slider(ui: &mut egui::Ui, settings: &mut SettingsSubsystem) {
    let scale_slider = Slider::new(&mut settings.type_matrix_scale, 0.1..=3.0)
        .text("TypeMatrix Scale")
        .prefix("x")
        .fixed_decimals(2);
    add_default_sized_setting(ui, scale_slider);
}

fn global_application_scale_slider(ctx: &egui::Context, ui: &mut egui::Ui) {
    let mut dpi = ctx.pixels_per_point();
    let scale_slider = Slider::new(&mut dpi, 0.25..=3.0)
        .text("Global Application\nSize")
        .prefix("x")
        .fixed_decimals(2);

    let resp = add_default_sized_setting(ui, scale_slider);
    if resp.changed() {
        ctx.set_pixels_per_point(dpi);
    }
}
