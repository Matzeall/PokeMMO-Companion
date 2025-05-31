// defines runtime data struct -> holds mutable application state
// is instantiated by main.rs

use std::cell::RefCell;

use eframe::{App, CreationContext, Frame};
use egui_commonmark::CommonMarkCache;

use egui::Context;

use crate::{
    color_palette,
    feature_state::{Feature, FeatureSubsystem},
    gui,
};

pub struct OverlayApp {
    // optional channel for platform messages (hotkeys, toggles…)
    // plat_tx: Sender<PlatformMessage>,
    // plat_rx: Receiver<PlatformMessage>,
    pub features: FeatureSubsystem,
    // markdown stuff
    pub cache: RefCell<CommonMarkCache>, // for runtime borrowing
}

impl OverlayApp {
    fn setup_global_application_style(&mut self, cc: &CreationContext<'_>) {
        let mut style = (*cc.egui_ctx.style()).clone();

        // mutate spacing, padding, fonts…
        style.spacing.item_spacing = egui::Vec2::new(15.0, 8.0);
        style.spacing.button_padding = egui::Vec2::new(12.0, 6.0);
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(14.0, egui::FontFamily::Proportional),
        );

        // BUTTON BG
        style.visuals.widgets.inactive.weak_bg_fill = color_palette::COLOR_BUTTON_REST; // normal
        style.visuals.widgets.hovered.weak_bg_fill = color_palette::COLOR_BUTTON_HOVER; // hover
        style.visuals.widgets.active.weak_bg_fill = color_palette::COLOR_BUTTON_PRESSED; // press
        style.visuals.selection.bg_fill = color_palette::COLOR_BUTTON_SELECTED; // selected

        // TEXTS
        style.visuals.widgets.inactive.fg_stroke.color = color_palette::COLOR_TEXT; // normal
        style.visuals.widgets.hovered.fg_stroke.color = color_palette::COLOR_TEXT; // hover
        style.visuals.widgets.active.fg_stroke.color = color_palette::COLOR_TEXT; // press
        style.visuals.selection.stroke.color = color_palette::COLOR_TEXT; // selected

        // apply it back to the context
        cc.egui_ctx.set_style(style);
    }

    pub fn new(cc: &CreationContext<'_>) -> Self {
        // Here you could spawn a background thread to register
        // RegisterHotKey (windows), XGrabKey (X11), or layer-shell listener (Wayland).
        //
        // Send events back via a channel that you poll in update().

        let mut app = Self {
            features: FeatureSubsystem::new(),
            cache: RefCell::new(CommonMarkCache::default()),
        };

        app.setup_global_application_style(cc);

        app
    }
}

// main egui update loop
impl App for OverlayApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        // Poll any platform messages:
        // while let Ok(msg) = self.plat_rx.try_recv() { … }

        gui::draw_perf_panel(ctx, frame);

        // draw UI based on AppState
        gui::draw_control_panel(ctx, self);

        if self.features.is_feature_active(Feature::Ressources) {
            gui::draw_ressources_panel(ctx, self);
        }

        if self.features.is_feature_active(Feature::Notes) {
            gui::draw_notes_panel(ctx, self);
        }

        if self.features.is_feature_active(Feature::TypeMatrix) {
            gui::draw_type_matrix_panel(ctx, self);
        }

        if self.features.is_feature_active(Feature::BreedingCalculator) {
            gui::draw_breeding_calculator_panel(ctx, self);
        }

        // request repaint if you want a live overlay:
        ctx.request_repaint();
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }
}
