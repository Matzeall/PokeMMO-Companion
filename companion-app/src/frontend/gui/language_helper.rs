use crate::{
    app::OverlayApp,
    backend::{
        feature_state::Feature, language_helper::language_helper_feature::LanguageHelperSubsystem,
    },
    frontend::{gui_subsystem::GuiSubsystem, style, utils::construct_base_window},
};
use egui::{
    Color32, ComboBox, CornerRadius, Frame, Image, ImageButton, Label, Layout, Margin, ScrollArea,
    Stroke, Vec2,
};
use egui_extras::{Size, StripBuilder};

////////////////////////////////////////////////////////////////////////////
///  LanguageHelper
////////////////////////////////////////////////////////////////////////////
pub fn draw_language_helper_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    let swap_image = state.gui.get_image_source("translation_swap");

    let open_handle = state
        .features
        .get_feature_active_mut_ref(Feature::LanguageHelper);

    construct_base_window("Language Helper", state.viewport_manager.as_ref())
        .open(open_handle)
        .show(ctx, |ui| {
            create_locale_select_bar(&mut state.language_helper, swap_image, ui);

            ui.add_space(4.);

            create_searchbar(&mut state.language_helper, ui);

            ui.separator();

            create_translation_list(&mut state.language_helper, &state.gui, ui);
        });
}

fn create_locale_select_bar(
    language_helper: &mut LanguageHelperSubsystem,
    swap_image: egui::ImageSource<'_>,
    ui: &mut egui::Ui,
) {
    // temporarily override innner button padding because images need more
    let button_padding = &mut ui.spacing_mut().button_padding;
    *button_padding = Vec2::new(5., 5.);

    // get current user set values
    let mut locale_source_selected = language_helper.get_translation_source_locale().clone();
    let mut locale_target_selected = language_helper.get_translation_target_locale().clone();

    let available_keys = language_helper.locale_subsystem.get_available_locales();
    let mut available_names = Vec::<String>::new();
    for key in &available_keys {
        available_names.push(
            language_helper
                .locale_subsystem
                .get_locale_display_name(key),
        );
    }

    let mut swap_clicked = false;

    // build ui
    ui.scope(|ui| {
        let select_bar_height = 30.;
        ui.set_height(select_bar_height);

        StripBuilder::new(ui)
            .cell_layout(Layout::centered_and_justified(egui::Direction::TopDown))
            .size(Size::remainder())
            .size(Size::relative(0.2))
            .size(Size::remainder())
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    ComboBox::from_id_salt("Translation_Source_Combobox")
                        .width(ui.available_width())
                        .truncate()
                        .selected_text(
                            language_helper
                                .locale_subsystem
                                .get_locale_display_name(&locale_source_selected)
                                .to_string(),
                        )
                        .show_ui(ui, |ui| {
                            for i in 0..available_keys.len() {
                                ui.selectable_value(
                                    &mut locale_source_selected,
                                    available_keys[i].clone(),
                                    available_names[i].clone(),
                                );
                            }
                        });
                });

                strip.cell(|ui| {
                    let swap_ui_image = Image::new(swap_image);
                    swap_clicked = ui
                        .add(ImageButton::new(swap_ui_image).corner_radius(select_bar_height / 2.))
                        .clicked();
                });

                strip.cell(|ui| {
                    ComboBox::from_id_salt("Translation_Target_Combobox")
                        .width(ui.available_width())
                        .truncate()
                        .selected_text(
                            language_helper
                                .locale_subsystem
                                .get_locale_display_name(&locale_target_selected)
                                .to_string(),
                        )
                        .show_ui(ui, |ui| {
                            for i in 0..available_keys.len() {
                                ui.selectable_value(
                                    &mut locale_target_selected,
                                    available_keys[i].clone(),
                                    available_names[i].clone(),
                                );
                            }
                        });
                });
            });
    });

    // apply user changes back
    language_helper.set_translation_target_locale(locale_target_selected);
    language_helper.set_translation_source_locale(locale_source_selected);
    if swap_clicked {
        language_helper.swap_translation_locales();
    }
}

fn create_translation_list(
    language_helper: &mut LanguageHelperSubsystem,
    gui_subsystem: &GuiSubsystem,
    ui: &mut egui::Ui,
) {
    let row_height: f32 = 30.0;

    let translation_pairs = language_helper.get_translation_pairs_for_search();

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, row_height, translation_pairs.len(), |ui, range| {
            // show_rows optimizes this ScrollArea so much, because only in range things are drawn
            for (category, source_text, translation) in translation_pairs[range].iter() {
                ui.scope(|ui| {
                    ui.set_height(row_height);
                    Frame::new()
                        .corner_radius(4.0)
                        .stroke(Stroke {
                            width: 0.15,
                            color: Color32::WHITE,
                        })
                        .show(ui, |ui| {
                            StripBuilder::new(ui)
                                .size(Size::exact(row_height))
                                .size(Size::remainder())
                                .size(Size::remainder())
                                .horizontal(|mut strip| {
                                    strip.cell(|ui| {
                                        let type_image = Image::new(
                                            gui_subsystem.get_image_source(
                                                format!("text_category_{:?}", category)
                                                    .to_lowercase(),
                                            ),
                                        );
                                        ui.centered_and_justified(|ui| {
                                            ui.add_sized(Vec2::splat(row_height - 6.), type_image)
                                                .on_hover_text(format!("Type: {:?}", category));
                                        });
                                    });

                                    strip.cell(|ui| {
                                        let loc_label = Label::new(source_text.clone()).truncate();
                                        ui.add_sized(
                                            Vec2::new(ui.available_width(), row_height),
                                            loc_label,
                                        );
                                    });

                                    strip.cell(|ui| {
                                        let trans_label =
                                            Label::new(translation.clone()).truncate();
                                        ui.add_sized(
                                            Vec2::new(ui.available_width(), row_height),
                                            trans_label,
                                        );
                                    });
                                });
                        });
                });
            }
        });
}

fn create_searchbar(language_helper: &mut LanguageHelperSubsystem, ui: &mut egui::Ui) {
    let search_focus_id = egui::Id::new("language_search_has_focus");
    let search_focused = ui.memory(|r| r.data.get_temp(search_focus_id).unwrap_or(false));

    let corner_radius = CornerRadius::same(4);
    let search_frame = if search_focused {
        egui::Frame {
            fill: Color32::from_rgba_premultiplied(20, 20, 20, 220),
            corner_radius,
            ..style::CUSTOM_FRAME_FOCUSSED
        }
    } else {
        egui::Frame {
            corner_radius,
            ..style::CUSTOM_FRAME_FOCUSSED
        }
    };

    let mut search_prompt = language_helper.get_search_prompt();
    let response = search_frame
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut search_prompt)
                    .frame(false) // disable default frame!
                    .hint_text("search pokemon / items / moves / locations")
                    .clip_text(false) //does nothing
                    .desired_width(ui.available_width())
                    .margin(Margin::symmetric(6, 2)),
            )
        })
        .inner;

    language_helper.set_search_prompt(search_prompt);

    // has focus update for next frame
    let search_focussed = response.has_focus();
    ui.memory_mut(|w| w.data.insert_temp(search_focus_id, search_focussed));
}
