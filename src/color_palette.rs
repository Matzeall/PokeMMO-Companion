use egui::{Color32, CornerRadius, Margin, Shadow, Stroke};

pub const COLOR_BUTTON_REST: Color32 = Color32::from_rgb(191, 67, 22);
pub const COLOR_BUTTON_HOVER: Color32 = Color32::from_rgb(242, 89, 33);
pub const COLOR_BUTTON_PRESSED: Color32 = Color32::from_rgb(191, 67, 22);
pub const COLOR_BUTTON_SELECTED: Color32 = Color32::from_rgb(89, 31, 10);
pub const COLOR_PANEL_BACKGROUND: Color32 = Color32::from_rgba_premultiplied(12, 12, 12, 196);
pub const COLOR_TEXT: Color32 = Color32::from_rgb(242, 242, 242);

pub const FRAME_PADDING: i8 = 15;
pub const CUSTOM_FRAME: egui::containers::Frame = egui::containers::Frame {
    // inner and outer padding:
    inner_margin: Margin::same(FRAME_PADDING),
    outer_margin: Margin::same(0),
    // corner radius for all corners:
    corner_radius: CornerRadius::same(20 + FRAME_PADDING.cast_unsigned()),
    // background fill:
    fill: COLOR_PANEL_BACKGROUND,
    // optional border stroke:
    stroke: Stroke::NONE,
    shadow: Shadow::NONE,
};
