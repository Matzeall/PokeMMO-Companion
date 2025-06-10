use std::collections::HashMap;

use eframe::CreationContext;
// contains all helper functions to render UI to keep the update function in app.rs clean
use crate::{
    app::OverlayApp,
    backend::{self, feature_state::Feature, ressources_feature::RessourcesSubsystem},
    style, utils,
};
use egui::{
    include_image, widgets::Image, Align2, Color32, Frame, Id, ImageButton, ImageSource, Label, Layout, ScrollArea, Stroke, TextEdit, UiBuilder, Vec2, Window
};
use strum::IntoEnumIterator;

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
        // feature icons
        self.icon_lookup.insert(
            Feature::TypeMatrix.get_name(),
            include_image!("../assets/icons/feature_type_matrix.png"),
        );
        self.icon_lookup.insert(
            Feature::BreedingCalculator.get_name(),
            include_image!("../assets/icons/feature_breeding_calculator.png"),
        );
        self.icon_lookup.insert(
            Feature::Ressources.get_name(),
            include_image!("../assets/icons/feature_resources.png"),
        );
        self.icon_lookup.insert(
            Feature::Notes.get_name(),
            include_image!("../assets/icons/feature_notes.png"),
        );

        self.icon_lookup.insert(
            "shutdown".to_string(),
            include_image!("../assets/icons/shutdown_button.png"),
        );

        self.icon_lookup.insert(
            "generic_home".to_string(),
            include_image!("../assets/icons/generic_home.png"),
        );

        self.icon_lookup.insert(
            "generic_back".to_string(),
            include_image!("../assets/icons/generic_back.png"),
        );
        self.icon_lookup.insert(
            "type_chart".to_string(),
            include_image!("../assets/images/PokeMMO_TypeChart.png"),
        );
    }

    /// utility to dynamically get an image source, which loses the compile time checking of the
    /// images actually being present, but allows for dynamic image names (in loops with iterable
    /// source names etc).
    /// It also eliminates the resulting optional ambiguity by returning a "missing_icon" icon in
    /// case of error
    pub fn get_image_source(&self, by_name: impl Into<String>) -> ImageSource<'static> {
        match self.icon_lookup.get(&by_name.into()) {
            Some(source) => source.clone(),
            None => include_image!("../assets/icons/missing_icon.png"), // needs to exist so i
                                                                        // can rely on this function always returning me some kind of image without panicing
        }
    }
}

pub fn draw_gui(ctx: &egui::Context, frame: &mut eframe::Frame, state: &mut OverlayApp) {
    draw_perf_panel(ctx, frame);

    // draw UI based on AppState
    draw_control_panel(ctx, state);

    draw_notes_panel(ctx, state);

    draw_ressources_panel(ctx, state);

    draw_type_matrix_panel(ctx, state);

    draw_breeding_calculator_panel(ctx, state);
}

////////////////////////////////////////////////////////////////////////////
///  Control Bar
////////////////////////////////////////////////////////////////////////////
pub fn draw_control_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    let feature_shortcuts: HashMap<Feature, &str> = HashMap::from([
        (Feature::Notes, "(Alt+N)"),
        (Feature::Ressources, "(Alt+R)"),
        (Feature::TypeMatrix, "(Alt+T)"),
        (Feature::BreedingCalculator, "(Alt+B)"),
    ]);

    Window::new("ControlPanel")
        .frame(if state.app_focus.is_focused() {
            style::CUSTOM_FRAME_FOCUSSED
        } else {
            style::CUSTOM_FRAME
        })
        .anchor(Align2::CENTER_BOTTOM, Vec2::new(0., 10.))
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

                //create one control button for each feature
                for feature in Feature::iter() {
                    let feature_image = Image::new(state.gui.get_image_source(feature.get_name()))
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
                    Image::new(state.gui.get_image_source("shutdown")).alt_text("Close");
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
pub fn draw_perf_panel(ctx: &egui::Context, frame: &mut eframe::Frame) {
    Window::new("Perf")
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

fn construct_base_window<'open>(
    window_name: impl Into<egui::WidgetText>,
    application_focused: bool,
) -> Window<'open> {
    Window::new(window_name)
        .frame(if application_focused {
            style::CUSTOM_FRAME_FOCUSSED
        } else {
            style::CUSTOM_FRAME
        })
        .resizable(true)
        .default_size(egui::vec2(300.0, 200.0))
        .min_size(egui::vec2(0., 0.))
        .fade_in(true)
        .fade_out(true)
}

//////////////////////////////////////////////////////////////////////////////////////
///// Feature Windows ////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////
///  Notes Window
////////////////////////////////////////////////////////////////////////////
pub fn draw_notes_panel(ctx: &egui::Context, state: &mut OverlayApp) {

    // immediately focus text edit up notes window open
    let notes_open_key ="Notes_Window_Open_Last_Frame";
    // read last open state
    let was_open = ctx.memory(|mem| {
        let id = Id::new(notes_open_key);
        // get returns Option<&Box<dyn Any>>, so unwrap to false
        mem.data.get_temp::<bool>(id).unwrap_or(false)
    });
    // insert current open state
    let notes_open = *state.features.get_feature_active_mut_ref(Feature::Notes);
    ctx.memory_mut(|mem| {
        let id = Id::new(notes_open_key);
        mem.data.insert_temp(
            id,
            notes_open,
        );
    });

    construct_base_window("Notes", state.app_focus.is_focused())
        .open(state.features.get_feature_active_mut_ref(Feature::Notes))
        .show(ctx, |ui| {
            // ui.style_mut().spacing.item_spacing = Vec2::ZERO; // FRAME PADDING is the culprit

            egui::ScrollArea::both()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.style_mut().visuals.widgets.active.bg_stroke=Stroke{ width: 0.3, color: Color32::TRANSPARENT };
                    let text_respone = ui.add(
                        TextEdit::multiline(&mut state.notes.text)
                            .frame(false)
                            .hint_text("Type personal notes and TODOs in here to keep track of them.\n....")
                            .clip_text(false)//does nothing
                            .desired_width(ui.available_width())
                            .font(egui::TextStyle::Body)
                            .layouter(&mut backend::notes_feature::get_notes_textedit_layouter(ctx))
                            .desired_rows(10)
                            .lock_focus(true),
                    );

                    if !was_open && notes_open {
                        text_respone.request_focus(); // first frame open
                    }
                    
                        let content_rect = ui.max_rect();

                    let overlay_ui_builder = UiBuilder::new().max_rect(content_rect)
                        .layout(Layout::from_main_dir_and_cross_align(
                            egui::Direction::RightToLeft, // right-aligned
                            egui::Align::Min,             // top-aligned
                        ));

                    ui.allocate_new_ui(overlay_ui_builder, |ui|{
                        ui.style_mut().interaction.tooltip_delay=0.0; // somehow doesn't work?
                        ui.style_mut().interaction.show_tooltips_only_when_still=false;

                        Frame::new().corner_radius(20.).fill(Color32::from_black_alpha(180)).inner_margin(2.)
                            .stroke(Stroke{width:0.5, color: Color32::from_white_alpha(180)})
                            .show(ui, |ui| {
                                ui.add_sized(Vec2::splat(20.),Label::new("?"));                        
                        }).response.on_hover_text("Simple notes styling:\n# Heading 1\n## Heading 2 etc.\n_underlined_\n*italic*");
                    });

                });
        });

}


////////////////////////////////////////////////////////////////////////////
///  Resources Window
////////////////////////////////////////////////////////////////////////////
pub fn draw_ressources_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    // bind to separate local variables to avoid borrowing state twice
    let window_open = state
        .features
        .get_feature_active_mut_ref(Feature::Ressources);

        
    construct_base_window("Ressources", state.app_focus.is_focused())
        .open(window_open)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    state.ressources.render_current_resource(ui);
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

        // should prevent runtime panics or do nothing
        ui.set_min_size(Vec2::new(20., 10.));

        ui.horizontal(|ui| {
            ui.add_space(15.);

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
                utils::draw_highlight_underline(ui, &back_response, 2.);
            }

            if back_response.clicked() {
                // back action
                resources_sub.go_back_visited_resources();
            }
        });
    });
}

////////////////////////////////////////////////////////////////////////////
///  Type Matrix
////////////////////////////////////////////////////////////////////////////
pub fn draw_type_matrix_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    let type_chart_source = state.gui.get_image_source("type_chart");
    let open_handle = state
        .features
        .get_feature_active_mut_ref(Feature::TypeMatrix);

    construct_base_window("Type Matrix", state.app_focus.is_focused())
        .open(open_handle)
        .show(ctx, |ui| {
            ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add(
                        Image::new(type_chart_source)
                            .fit_to_original_size(0.75)
                            .tint(Color32::from_white_alpha(40)),
                    );
                });
        });
}

////////////////////////////////////////////////////////////////////////////
///  Control Bar
////////////////////////////////////////////////////////////////////////////
pub fn draw_breeding_calculator_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    let open_handle = state
        .features
        .get_feature_active_mut_ref(Feature::BreedingCalculator);
    construct_base_window("Breeding Calculator", state.app_focus.is_focused())
        .open(open_handle)
        .show(ctx, |_ui| {
            ScrollArea::both()
                .auto_shrink([false, false])
                .show(_ui, |_ui| {});
        });
}
