// defines runtime data struct -> holds mutable application state
// is instantiated by main.rs

use eframe::{App, CreationContext, Frame};
use egui::Context;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{color_palette, gui};

// implements default Equals & Hash functions so a HashSet<Feature> can be checked for containment.
// Debug/Display is for printing or str::fmt the enum value name.
// Clone for passing it without a reference to function without trasfering ownership (by copying it)
// EnumIter is from the strum & strum_macros crate, allows iterating over all enum values.
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, EnumIter)]
pub enum Feature {
    Notes,
    Ressources,
    TypeMatrix,
    BreedingCalculator,
}

pub struct OverlayApp {
    // optional channel for platform messages (hotkeys, toggles…)
    // plat_tx: Sender<PlatformMessage>,
    // plat_rx: Receiver<PlatformMessage>,
    active_feature_windows: HashMap<Feature, bool>,
}
/*
<palette>
<color rgb='BF4417' r='191' g='67' b='22' />
<color rgb='59200B' r='89' g='31' b='10' />
<color rgb='F25922' r='242' g='89' b='33' />
<color rgb='F2F2F2' r='242' g='242' b='242' />
<color rgb='0D0D0D' r='12' g='12' b='12' />
</palette>
*/
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

        // init map with all feature values + false
        let active_feature_windows = Feature::iter().map(|f| (f, false)).collect();
        let mut app = Self {
            active_feature_windows,
        };

        app.setup_global_application_style(cc);

        app
    }

    pub fn set_feature_active(&mut self, feature: Feature, enabled: bool) {
        self.active_feature_windows.insert(feature, enabled);
        println!("set {feature:#?} to {enabled}");
    }

    pub fn is_feature_active(&self, feature: Feature) -> bool {
        self.active_feature_windows
            .get(&feature)
            .cloned()
            .unwrap_or(false)
    }

    pub fn get_feature_active_mut_ref(&mut self, feature: Feature) -> &mut bool {
        self.active_feature_windows
            .get_mut(&feature)
            .expect("every feature should be contained in the map after app init")
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

        if self.is_feature_active(Feature::Ressources) {
            gui::draw_ressources_panel(ctx, self);
        }

        if self.is_feature_active(Feature::Notes) {
            gui::draw_notes_panel(ctx, self);
        }

        if self.is_feature_active(Feature::TypeMatrix) {
            gui::draw_type_matrix_panel(ctx, self);
        }

        if self.is_feature_active(Feature::BreedingCalculator) {
            gui::draw_breeding_calculator_panel(ctx, self);
        }

        // request repaint if you want a live overlay:
        ctx.request_repaint();
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }
}
