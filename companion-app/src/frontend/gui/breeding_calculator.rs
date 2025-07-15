use egui::ScrollArea;

use crate::{
    app::OverlayApp, backend::feature_state::Feature, frontend::utils::construct_base_window,
};

////////////////////////////////////////////////////////////////////////////
///  BreedingCalculator
////////////////////////////////////////////////////////////////////////////
pub fn draw_breeding_calculator_panel(ctx: &egui::Context, state: &mut OverlayApp) {
    let open_handle = state
        .features
        .get_feature_active_mut_ref(Feature::BreedingCalculator);
    construct_base_window("Breeding Calculator", state.viewport_manager.as_ref())
        .open(open_handle)
        .show(ctx, |_ui| {
            ScrollArea::both()
                .auto_shrink([false, false])
                .show(_ui, |_ui| _ui.label("not implemented yet"));
        });
}
