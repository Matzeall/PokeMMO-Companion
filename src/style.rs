use eframe::CreationContext;
use egui::{Color32, CornerRadius, Margin, Shadow, Stroke};

pub const COLOR_BUTTON_REST: Color32 = Color32::from_rgb(191, 67, 22);
pub const COLOR_BUTTON_HOVER: Color32 = Color32::from_rgb(242, 89, 33);
pub const COLOR_BUTTON_PRESSED: Color32 = Color32::from_rgb(191, 67, 22);
pub const COLOR_BUTTON_SELECTED: Color32 = Color32::from_rgb(89, 31, 10);
pub const COLOR_PANEL_BACKGROUND: Color32 = Color32::from_rgba_premultiplied(12, 12, 12, 196);
pub const COLOR_TEXT: Color32 = Color32::from_rgb(242, 242, 242);
pub const COLOR_HYPERLINK: egui::Color32 = Color32::from_rgb(56, 203, 232);
pub const COLOR_APPLINK_REST: egui::Color32 = Color32::from_rgb(242, 89, 33);
pub const COLOR_APPLINK_HOVER: egui::Color32 = Color32::from_rgb(242, 89, 33);

pub const FRAME_PADDING: i8 = 10;
pub const CUSTOM_FRAME: egui::containers::Frame = egui::containers::Frame {
    // inner and outer padding:
    inner_margin: Margin::same(FRAME_PADDING),
    outer_margin: Margin::same(0),
    // corner radius for all corners:
    corner_radius: CornerRadius::same(20 + FRAME_PADDING.cast_unsigned()),
    // background fill:
    fill: COLOR_PANEL_BACKGROUND,
    // optional border stroke:
    stroke: Stroke {
        width: 0.1,
        color: Color32::TRANSPARENT,
    },
    shadow: Shadow::NONE,
};
pub const CUSTOM_FRAME_FOCUSSED: egui::containers::Frame = egui::containers::Frame {
    stroke: Stroke {
        width: 0.1,
        color: Color32::WHITE,
    },
    ..CUSTOM_FRAME
};

pub fn setup_global_application_style(cc: &CreationContext<'_>) {
    let mut style = (*cc.egui_ctx.style()).clone();

    // spacing, padding
    style.spacing.item_spacing = egui::Vec2::new(15.0, 8.0);
    style.spacing.button_padding = egui::Vec2::new(12.0, 6.0);

    // TEXT SIZES
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(18.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(8.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(14.0, egui::FontFamily::Monospace),
    );
    // TEXT COLORS
    style.visuals.widgets.noninteractive.fg_stroke.color = COLOR_TEXT; // normal
    style.visuals.widgets.inactive.fg_stroke.color = COLOR_TEXT; // normal
    style.visuals.widgets.hovered.fg_stroke.color = COLOR_TEXT; // hover
    style.visuals.widgets.active.fg_stroke.color = COLOR_TEXT; // press
    style.visuals.selection.stroke.color = COLOR_TEXT; // selected
    style.visuals.hyperlink_color = COLOR_HYPERLINK;
    // style.debug.show_expand_height  // TODO: investigate debug options

    // just for my sanity to override nothing
    style.override_text_style = None;
    style.override_font_id = None;
    style.override_text_valign = None;
    style.visuals.override_text_color = None;

    // BUTTON BG
    style.visuals.widgets.inactive.weak_bg_fill = COLOR_BUTTON_REST; // normal
    style.visuals.widgets.hovered.weak_bg_fill = COLOR_BUTTON_HOVER; // hover
    style.visuals.widgets.active.weak_bg_fill = COLOR_BUTTON_PRESSED; // press
    style.visuals.selection.bg_fill = COLOR_BUTTON_SELECTED; // selected

    // Other
    style.url_in_tooltip = true;
    style.visuals.window_stroke.color = egui::Color32::from_white_alpha(180);
    style.visuals.window_stroke.width = 0.35;
    style.visuals.window_fill = COLOR_PANEL_BACKGROUND.to_opaque();

    // apply it back to the context
    cc.egui_ctx.set_style(style);
}
