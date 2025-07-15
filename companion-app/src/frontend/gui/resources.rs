use crate::frontend::style;
use crate::{
    app::OverlayApp,
    backend::{feature_state::Feature, ressources_feature::RessourcesSubsystem},
    frontend::{
        gui_subsystem::GuiSubsystem,
        utils::{self, construct_base_window},
    },
};
use egui::{Color32, Frame, Image, ImageButton, Layout, Margin, Stroke, UiBuilder, Vec2};

////////////////////////////////////////////////////////////////////////////
///  Resources Window
////////////////////////////////////////////////////////////////////////////
pub fn draw_ressources_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    // bind to separate local variables to avoid borrowing state twice
    let window_open = state
        .features
        .get_feature_active_mut_ref(Feature::Resources);

    construct_base_window("Ressources", state.viewport_manager.as_ref())
        .open(window_open)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(5.);
                    state.ressources.render_current_resource(ui);
                    ui.add_space(19.);
                });
            draw_resource_nav_bar(&mut state.ressources, &state.gui, ui);
        });
}

// drawing an overlay over the the same space ui already occupies and creates a navbar on it
fn draw_resource_nav_bar(
    resources_sub: &mut RessourcesSubsystem,
    gui_sub: &GuiSubsystem,
    ui: &mut egui::Ui,
) {
    let content_rect = ui.max_rect();

    let overlay_ui_builder =
        UiBuilder::new()
            .max_rect(content_rect)
            .layout(Layout::from_main_dir_and_cross_align(
                egui::Direction::RightToLeft, // right-aligned
                egui::Align::Min,             // top-aligned
            ));
    // setup navbar style
    ui.allocate_new_ui(overlay_ui_builder, |ui| {
        ui.style_mut().spacing.button_padding = Vec2::new(0., 0.);
        ui.style_mut().spacing.item_spacing = Vec2::new(2., 2.);
        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::TRANSPARENT;
        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::TRANSPARENT;
        ui.style_mut().visuals.widgets.hovered.bg_stroke = Stroke::NONE;
        ui.style_mut().visuals.widgets.active.bg_stroke = Stroke::NONE;

        // slight backdrop to stand out from the bg
        Frame::new()
            .corner_radius(20.)
            .fill(Color32::from_black_alpha(150))
            .inner_margin(Margin::symmetric(10, 3))
            .outer_margin(Vec2::new(12., 0.))
            .stroke(Stroke::NONE)
            .show(ui, |ui| {
                // should prevent runtime panics or do nothing
                ui.set_min_size(Vec2::new(20., 10.));

                ui.horizontal(|ui| {
                    let button_size = Vec2::splat(25.);
                    let home_icon_source = gui_sub.get_image_source("generic_home").clone();
                    let back_icon_source = gui_sub.get_image_source("generic_back").clone();

                    // HOME BUTTON
                    let home_image = Image::new(home_icon_source)
                        .alt_text("Home")
                        .tint(style::COLOR_APPLINK_REST);
                    let home_btn = ImageButton::new(home_image).corner_radius(button_size.x / 2.);
                    let home_response = ui.add_sized(button_size, home_btn).on_hover_text("Home");

                    if home_response.hovered() {
                        // draw highlight
                        utils::draw_highlight_underline(ui, &home_response, 2.);
                    }

                    if home_response.clicked() {
                        resources_sub.set_current_resource("ROOT", true);
                    }

                    // BACK BUTTON
                    let back_image = Image::new(back_icon_source)
                        .alt_text("back")
                        .tint(style::COLOR_APPLINK_REST);
                    let back_btn = ImageButton::new(back_image).corner_radius(button_size.x / 2.);
                    let back_response = ui.add_sized(button_size, back_btn);

                    if let Some(last_res) = resources_sub.inspect_last_resource() {
                        // TODO: later check if I really need to clone here
                        back_response.clone().on_hover_text(last_res);
                    }

                    if back_response.hovered() {
                        // draw highlight
                        utils::draw_highlight_underline(ui, &back_response, 0.);
                    }

                    if back_response.clicked() {
                        // back action
                        resources_sub.go_back_visited_resources();
                    }
                });
            });
    });
}
