use super::viewport::ViewportManager;
use crate::frontend::style;
use egui::{Ui, Vec2, Window};
use egui_extras::{Size, StripBuilder};

pub fn construct_base_window<'open>(
    window_name: impl Into<egui::WidgetText>,
    // application_focused: bool,
    viewport_manager: &dyn ViewportManager,
) -> Window<'open> {
    Window::new(window_name)
        // .frame(if application_focused {
        .frame(if viewport_manager.current_focus_state().is_focused() {
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

/// useful to make some child ui adhere to a given max size and put the cursor after
#[allow(dead_code)]
pub fn size_boxed<F>(ui: &mut Ui, size: Vec2, add_contents: F) -> egui::Response
where
    F: FnOnce(&mut Ui),
{
    // let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    //
    // ui.put(rect, |ui: &mut Ui| {
    //     ui.set_min_size(ui.available_size());
    //     ui.set_max_size(ui.available_size());
    //     // prohibit text to expand past the allowed boundary
    //     ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
    //
    //     add_contents(ui);
    //     response
    // })

    StripBuilder::new(ui)
        .size(Size::exact(size.x))
        .horizontal(|mut strip| {
            strip.strip(|strip_builder| {
                strip_builder
                    .size(Size::exact(size.y))
                    .vertical(|mut strip| {
                        strip.cell(add_contents);
                    });
            })
        })
}

/// retrieve a dev_slider's value from the memory of the provided ui
/// if not present (wrong id etc) or in release build it will always return 1.0
#[allow(dead_code)]
pub fn get_dev_slider_value(_id: impl Into<String>, _ui: &Ui) -> f32 {
    #[cfg(debug_assertions)]
    {
        use egui::Id;
        let id = Id::new(_id.into());

        _ui.ctx()
            .memory(|mem| -> f32 { mem.data.get_temp(id).unwrap_or(1.0) })
    }
    #[cfg(not(debug_assertions))]
    1.0
}

pub fn draw_highlight_underline(
    ui: &mut egui::Ui,
    hover_response: &egui::response::Response,
    bottom_offset: f32,
) {
    let y = hover_response.rect.bottom() - 1.0 + bottom_offset;
    ui.painter().line_segment(
        [
            egui::Pos2::new(hover_response.rect.min.x, y),
            egui::Pos2::new(hover_response.rect.max.x, y),
        ],
        egui::Stroke {
            width: 1.,
            color: style::COLOR_APPLINK_HOVER,
        },
    );
}
