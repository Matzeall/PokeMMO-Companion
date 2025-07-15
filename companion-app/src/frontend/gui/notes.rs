use egui::{Color32, Frame, Id, Key, Label, Layout, Modifiers, Stroke, TextEdit, UiBuilder, Vec2};

use crate::{
    app::OverlayApp,
    backend::{self, feature_state::Feature},
    frontend::utils::construct_base_window,
};

////////////////////////////////////////////////////////////////////////////
///  Notes Window
////////////////////////////////////////////////////////////////////////////
pub fn draw_notes_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    // immediately focus text edit up notes window open
    let notes_open_key = "Notes_Window_Open_Last_Frame";
    // read last open state
    let was_open = ctx.memory(|mem| {
        let id = Id::new(notes_open_key);
        // get returns Option<&Box<dyn Any>>, so unwrap to false
        mem.data.get_temp::<bool>(id).unwrap_or(false)
    });
    let is_alt_down= ctx.input(|r| r.modifiers.alt);
    // insert current open state
    let notes_open = *state.features.get_feature_active_mut_ref(Feature::Notes);
    ctx.memory_mut(|mem| {
        let id = Id::new(notes_open_key);
        mem.data.insert_temp(id, notes_open);
    });

    construct_base_window("Notes", state.viewport_manager.as_ref())
        .open(state.features.get_feature_active_mut_ref(Feature::Notes))
        .show(ctx, |ui| {

            // {
            //     ui.memory_mut(|w| w.stop_text_input());
            //     // text_respone.surrender_focus();
            // }

            egui::ScrollArea::both()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    let text_respone = ui.add(
                        TextEdit::multiline(&mut state.notes.text)
                            .frame(false)
                            .interactive(!is_alt_down)
                            .hint_text("...\nType personal notes and TODOs in here to keep track of them.\n...")
                            .clip_text(false)//does nothing
                            .desired_width(ui.available_width())
                            .font(egui::TextStyle::Body)
                            .layouter(&mut backend::notes_feature::get_notes_textedit_layouter(ctx))
                            .desired_rows(10)
                            .lock_focus(true),
                    );

                    if !was_open && notes_open || 
                    ctx.input_mut(| r|r.consume_key(Modifiers::NONE, Key::N)){
                        state.notes.requests_focus=true;
                    }

                    if state.notes.requests_focus && !is_alt_down {
                        text_respone.request_focus(); // first frame open after alt was released
                        state.notes.requests_focus=false;
                    } 
                });

            // overlay helper icon on top
            let content_rect = ui.max_rect();

            let overlay_ui_builder = UiBuilder::new().max_rect(content_rect)
                .layout(Layout::from_main_dir_and_cross_align(
                    egui::Direction::RightToLeft, // right-aligned
                    egui::Align::Min,             // top-aligned
                ));
            ui.allocate_new_ui(overlay_ui_builder, |ui|{
                ui.style_mut().interaction.tooltip_delay=0.0; // somehow doesn't work?
                ui.style_mut().interaction.show_tooltips_only_when_still=false;

                Frame::new().corner_radius(20.).fill(Color32::from_black_alpha(150)).inner_margin(2.).outer_margin(Vec2::new(15.,0.))
                    .stroke(Stroke{width:0.5, color: Color32::from_white_alpha(150)})
                    .show(ui, |ui| {
                        ui.add_sized(Vec2::splat(20.),Label::new("?"));                        
                    }).response.on_hover_text("Simple notes styling:\n# Heading 1\n## Heading 2 ...\n_underlined_\n*italic*");
            });

        });
}
