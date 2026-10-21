//! Shared visual tokens and small presentation helpers for the egui layer.
//! Keep these independent from editor state and audio processing.

use egui::{Color32, RichText, Ui};

pub const SPACE_2: f32 = 8.0;
pub const TABLE_ROW_HEIGHT: f32 = 34.0;
pub const TABLE_HEADER_HEIGHT: f32 = 36.0;

pub fn accent(ui: &Ui) -> Color32 {
    ui.visuals().hyperlink_color
}

pub fn section_title(ui: &mut Ui, is_rtl: bool, title: impl Into<String>, detail: impl Into<String>) {
    let title = title.into();
    let detail = detail.into();
    crate::app::layout::horizontal(ui, is_rtl, |ui| {
        ui.label(RichText::new(title).strong().size(15.0).color(accent(ui)));
        if !detail.is_empty() {
            ui.label(RichText::new(detail).small().color(ui.visuals().weak_text_color()));
        }
    });
}
