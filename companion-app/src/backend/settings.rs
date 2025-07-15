use egui::{Checkbox, Color32, Frame, RichText, Slider, Stroke, Ui, Widget};

/// runtime value store for all settings and container for settings behaviour
pub struct SettingsSubsystem {
    pub version: u8,
    pub disable_overlay: bool,
    pub transparent_background_always: bool,
    pub type_matrix_scale: f32,

    // transient request flags
    pub request_viewport_restart: bool,
    pub request_clear_ui_data: bool,

    // dev-only
    _dev_on_hover_diagnostics: bool,
    _dev_hover_shows_next: bool,
    _dev_show_expand_size: bool,
    _dev_show_resize: bool,
    _dev_show_widget_hits: bool,
    _dev_show_interactive_widgets: bool,
    _dev_sliders: Vec<String>,
}

impl SettingsSubsystem {
    pub fn new() -> SettingsSubsystem {
        let dev_sliders = ["1".to_string(), "2".to_string(), "3".to_string()].into();

        SettingsSubsystem {
            version: 0,
            disable_overlay: false,
            transparent_background_always: false,
            type_matrix_scale: 1.0,
            request_viewport_restart: false,
            request_clear_ui_data: false,
            _dev_sliders: dev_sliders,
            _dev_on_hover_diagnostics: false,
            _dev_show_expand_size: false,
            _dev_show_widget_hits: false,
            _dev_hover_shows_next: false,
            _dev_show_resize: false,
            _dev_show_interactive_widgets: false,
        }
    }

    /// will draw nothing in release build
    /// following calls tp get_dev_slider_value() are still allowed and will return 1.0 always
    pub fn draw_dev_options(&mut self, _ui: &mut Ui) {
        #[cfg(debug_assertions)]
        Frame::new()
            .inner_margin(10.0)
            .outer_margin(8.0)
            .stroke(Stroke::new(0.3, Color32::WHITE))
            .corner_radius(8.)
            .show(_ui, |_ui| {
                _ui.label(
                    RichText::new("Dev Options (debug-only)")
                        .heading()
                        .color(Color32::from_white_alpha(200)),
                );

                // draw egui debug options
                // draws height and width of hovered widget
                Checkbox::new(&mut self._dev_on_hover_diagnostics, "On-hover diagnostics").ui(_ui);
                _ui.ctx().style_mut(|s| {
                    s.debug.debug_on_hover = self._dev_on_hover_diagnostics;
                    s.debug.debug_on_hover_with_all_modifiers = self._dev_on_hover_diagnostics;
                });
                // draw additional pointers where the next widget starts in relation to the hovered
                if self._dev_on_hover_diagnostics {
                    Checkbox::new(
                        &mut self._dev_hover_shows_next,
                        "Also mark next cursor on-hover",
                    )
                    .ui(_ui);
                    _ui.ctx().style_mut(|s| {
                        s.debug.hover_shows_next = self._dev_hover_shows_next;
                    });
                }
                // Show which widgets epxand their parents
                Checkbox::new(
                    &mut self._dev_show_expand_size,
                    "Show expand parent-size culprits",
                )
                .ui(_ui);
                _ui.ctx().style_mut(|s| {
                    s.debug.show_expand_height = self._dev_show_expand_size;
                    s.debug.show_expand_width = self._dev_show_expand_size;
                });
                // Show widget hits
                Checkbox::new(&mut self._dev_show_widget_hits, "Show widget hit by cursor").ui(_ui);
                _ui.ctx().style_mut(|s| {
                    s.debug.show_widget_hits = self._dev_show_widget_hits;
                });
                Checkbox::new(&mut self._dev_show_resize, "Show resizeable regions").ui(_ui);
                _ui.ctx().style_mut(|s| {
                    s.debug.show_resize = self._dev_show_resize;
                });
                // Hightlighs all interactable widgets
                Checkbox::new(
                    &mut self._dev_show_interactive_widgets,
                    "Show interactive widgets",
                )
                .ui(_ui);
                _ui.ctx().style_mut(|s| {
                    s.debug.show_interactive_widgets = self._dev_show_interactive_widgets;
                });

                _ui.add_space(10.0);
                // draw custom dev sliders
                for string_id in &self._dev_sliders {
                    let cur_slider_id = egui::Id::new(string_id);

                    let mut value: f32 = _ui
                        .ctx()
                        .memory(|mem| -> f32 { mem.data.get_temp(cur_slider_id).unwrap_or(1.0) });

                    let response = _ui.add(
                        Slider::new::<f32>(&mut value, 0.0..=2.0)
                            .clamping(egui::SliderClamping::Never)
                            .text(string_id),
                    );

                    value = value.max(0.01);

                    if response.changed() {
                        _ui.data_mut(|data| {
                            data.insert_temp(cur_slider_id, value);
                        });
                    }
                }
            });
    }
}
