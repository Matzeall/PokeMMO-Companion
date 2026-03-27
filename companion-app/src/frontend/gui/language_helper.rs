use egui::ScrollArea;

use crate::{
    app::OverlayApp, backend::feature_state::Feature, frontend::utils::construct_base_window,
};

////////////////////////////////////////////////////////////////////////////
///  LanguageHelper
////////////////////////////////////////////////////////////////////////////
pub fn draw_language_helper_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    let open_handle = state
        .features
        .get_feature_active_mut_ref(Feature::LanguageHelper);
    construct_base_window("LanguageHelper", state.viewport_manager.as_ref())
        .open(open_handle)
        .show(ctx, |ui| {
            ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {});
        });
}
