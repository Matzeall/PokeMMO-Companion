use egui::{
    Color32, CornerRadius, Direction, FontId, Frame, Id, Image, Label, Layout, Margin, RichText,
    ScrollArea, Sense, Stroke, TextStyle, Vec2, scroll_area::ScrollBarVisibility,
};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use strum::IntoEnumIterator;

use crate::{
    app::OverlayApp,
    backend::{
        feature_state::Feature,
        type_matrix_feature::{PokemonType, attack_effectiveness_single},
    },
    frontend::{gui_subsystem::GuiSubsystem, style, utils::construct_base_window},
};

////////////////////////////////////////////////////////////////////////////
///  Type Matrix
////////////////////////////////////////////////////////////////////////////
pub fn draw_type_matrix_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    let open_handle = state
        .features
        .get_feature_active_mut_ref(Feature::TypeMatrix);

    construct_base_window("Type Matrix", state.viewport_manager.as_ref())
        // override default frame to erase inner margin
        .frame(Frame {
            inner_margin: Margin {
                bottom: 5,
                ..Margin::ZERO
            },
            corner_radius: CornerRadius {
                se: 5,
                sw: 5,
                ..style::FRAME_CORNER_RADIUS
            },
            ..if state.viewport_manager.current_focus_state().is_focused() {
                style::CUSTOM_FRAME_FOCUSSED
            } else {
                style::CUSTOM_FRAME
            }
        })
        .open(open_handle)
        .show(ctx, |ui| {
            ui.set_min_size([1.0, 1.0].into());

            build_type_table(&state.gui, ui, state.settings.type_matrix_scale);
        });
}

fn build_type_table(gui_subsystem: &GuiSubsystem, ui: &mut egui::Ui, matrix_scale: f32) {
    let cell_size = 40.0 * matrix_scale;
    let type_count = PokemonType::iter().count();

    let mut cur_scroll_offset: Vec2 = ui
        .memory(|r| r.data.get_temp(Id::new("type_matrix_scroll_offset")))
        .unwrap_or_default();

    // style setup
    ui.style_mut().spacing.item_spacing = Vec2::splat(0.0);
    let current_text_size = ui
        .style()
        .text_styles
        .get(&TextStyle::Body)
        .map(|f| f.size)
        .unwrap_or(14.0);
    ui.style_mut().override_font_id = Some(FontId::new(
        current_text_size * matrix_scale,
        style::condensed_font(),
    ));

    // actual ui -> strip builds : |left layout| - |right layout|
    StripBuilder::new(ui)
        .size(Size::exact(cell_size + 0.5))
        .size(Size::remainder())
        .horizontal(|mut strip| {
            // left bar - top-down layout for : empty cell + type strip
            strip.strip(|strip_builder| {
                strip_builder
                    .size(Size::exact(cell_size))
                    .size(/*Size::exact(body_size)*/ Size::remainder())
                    .clip(true)
                    .vertical(|mut strip| {
                        // top left cell
                        strip.cell(|ui| {
                            draw_top_left_cell(ui, gui_subsystem);
                        });

                        // left type bar (vertical)
                        strip.cell(|ui| {
                            let new_scroll_offset = ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .enable_scrolling(true)
                                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                                .drag_to_scroll(true)
                                .vertical_scroll_offset(cur_scroll_offset.y)
                                .show(ui, |ui| {
                                    StripBuilder::new(ui)
                                        .sizes(Size::exact(cell_size), type_count)
                                        .cell_layout(Layout::centered_and_justified(
                                            Direction::TopDown,
                                        ))
                                        .vertical(|strip| fill_type_strip(gui_subsystem, strip));
                                })
                                .state
                                .offset
                                .y;
                            // update scroll_offset if changed here
                            if cur_scroll_offset.y.ne(&new_scroll_offset) {
                                cur_scroll_offset.y = new_scroll_offset;
                            }
                        });
                    });
            }); // left layout done

            // right - top-down layout for : horizontal type strip + type table content
            strip.strip(|strip_builder| {
                strip_builder
                    .size(Size::exact(cell_size))
                    .size(Size::remainder())
                    .clip(true)
                    .vertical(|mut strip| {
                        // top type bar (horizontal)
                        strip.cell(|ui| {
                            let new_scroll_offset = ScrollArea::horizontal()
                                .auto_shrink([false, false])
                                .enable_scrolling(true)
                                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                                .drag_to_scroll(true)
                                .horizontal_scroll_offset(cur_scroll_offset.x)
                                .show(ui, |ui| {
                                    StripBuilder::new(ui)
                                        .sizes(Size::exact(cell_size), type_count)
                                        .cell_layout(Layout::centered_and_justified(
                                            Direction::LeftToRight,
                                        ))
                                        .horizontal(|strip| fill_type_strip(gui_subsystem, strip));
                                })
                                .state
                                .offset
                                .x;
                            // update scroll_offset if changed here
                            if cur_scroll_offset.x.ne(&new_scroll_offset) {
                                cur_scroll_offset.x = new_scroll_offset;
                            }
                        });

                        // main cell containing effectiveness combination table
                        strip.cell(|ui| {
                            let new_scroll_offset =
                                draw_type_combination_table(cell_size, ui, cur_scroll_offset);
                            // update scroll_offset if changed here
                            if cur_scroll_offset.ne(&new_scroll_offset) {
                                cur_scroll_offset = new_scroll_offset;
                            }
                        });
                    });
            });
        });

    // write last scroll_offset to memory
    ui.memory_mut(|w| {
        w.data
            .insert_temp(Id::new("type_matrix_scroll_offset"), cur_scroll_offset)
    });
}

fn draw_top_left_cell(ui: &mut egui::Ui, gui_subsystem: &GuiSubsystem) {
    let cell_frame = Frame::new()
        .inner_margin(Margin::same(4))
        .stroke(Stroke::new(0.4, Color32::WHITE));

    cell_frame.show(ui, |ui| {
        ui.add(
            Image::new(gui_subsystem.get_image_source("typematrix_atk_def_icon"))
                .fit_to_fraction(Vec2::splat(1.0)),
        );
    });
}

fn draw_type_combination_table(cell_size: f32, ui: &mut egui::Ui, cur_scroll_offset: Vec2) -> Vec2 {
    let type_count = PokemonType::iter().count();
    let last_hovered: Option<[usize; 2]> = ui
        .memory(|r| r.data.get_temp(Id::new("type_matrix_hovered_cell")))
        .flatten();
    let mut new_hovered_cell: Option<[usize; 2]> = None;

    let scroll_offset = ScrollArea::both()
        .auto_shrink([false, false])
        .enable_scrolling(true)
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
        .drag_to_scroll(true)
        .scroll_offset(cur_scroll_offset)
        .show(ui, |ui| {
            let table_builder = TableBuilder::new(ui)
                .auto_shrink([true, true])
                .vscroll(false)
                .resizable(false)
                .striped(true)
                .cell_layout(Layout::centered_and_justified(Direction::LeftToRight))
                .columns(Column::exact(cell_size), type_count);
            table_builder.reset();
            table_builder.body(|tbody| {
                // type_count -1 doesn't make any sense, right? Why does it work?
                tbody.rows(cell_size, type_count - 1, |mut row| {
                    let r_index = row.index();
                    if let Some(atk_type) = PokemonType::iter().nth(r_index) {
                        for c_index in 0..(type_count - 1) {
                            if let Some(def_type) = PokemonType::iter().nth(c_index) {
                                let highlighted = last_hovered
                                    .is_some_and(|l| l[0].eq(&c_index) || l[1].eq(&r_index));

                                row.col(|ui| {
                                    if draw_effectiveness_cell_content(
                                        ui,
                                        atk_type,
                                        def_type,
                                        highlighted,
                                    ) {
                                        new_hovered_cell = Some([c_index, r_index]);
                                    }
                                });
                            }
                        }
                    };
                });
            });
        })
        .state
        .offset; // return current scroll offset

    ui.memory_mut(|w| {
        w.data
            .insert_temp(Id::new("type_matrix_hovered_cell"), new_hovered_cell)
    });

    scroll_offset
}

fn draw_effectiveness_cell_content(
    ui: &mut egui::Ui,
    atk_type: PokemonType,
    def_type: PokemonType,
    highlighted: bool,
) -> bool {
    let effectiveness = attack_effectiveness_single(atk_type, def_type);
    let color = match effectiveness {
        e if e < 0.75 => Color32::RED,
        e if e > 1.25 => Color32::GREEN,
        _ => Color32::WHITE,
    };

    let mut cell_frame = Frame::new().stroke(Stroke::new(0.2, Color32::WHITE));

    if highlighted {
        cell_frame = cell_frame.fill(Color32::from_white_alpha(5));
    }

    cell_frame
        .show(ui, |ui| {
            // actual efectiveness label (x0.5, x1.0, x2.0)
            let response = ui.add(
                Label::new(RichText::new(format!("x{:>2.1}", effectiveness)).color(color))
                    .truncate()
                    .selectable(false)
                    .sense(Sense::hover()),
            );

            ui.shrink_width_to_current();
            response.hovered()
        })
        .inner
}

fn fill_type_strip(gui_subsystem: &GuiSubsystem, mut strip: egui_extras::Strip<'_, '_>) {
    for i in 0..(PokemonType::iter().count() - 1) {
        if let Some(pokemon_type) = PokemonType::iter().nth(i) {
            strip.cell(|ui| {
                let cell_frame = Frame::new()
                    .outer_margin(Margin::same(1))
                    .inner_margin(Margin::same(3))
                    .fill(Color32::BLACK)
                    .stroke(Stroke::new(0.2, Color32::WHITE));

                cell_frame.show(ui, |ui| {
                    ui.add(
                        Image::new(
                            gui_subsystem.get_image_source(format!(
                                "type_{}",
                                pokemon_type.get_debug_name()
                            )),
                        )
                        .fit_to_fraction(Vec2::splat(1.))
                        .show_loading_spinner(true),
                    )
                    .on_hover_text(pokemon_type.get_debug_name());
                });
            });
        }
    }
}
