use egui::{Ui, InnerResponse};

/// Place controls in reading order while leaving text shaping to `bidi`.
pub fn horizontal<R>(ui: &mut Ui, is_rtl: bool, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let layout = if is_rtl {
        egui::Layout::right_to_left(egui::Align::Center)
    } else {
        egui::Layout::left_to_right(egui::Align::Center)
    };
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
        layout,
        add_contents,
    )
}

pub fn horizontal_wrapped<R>(ui: &mut Ui, is_rtl: bool, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let layout = if is_rtl {
        egui::Layout::right_to_left(egui::Align::Center)
    } else {
        egui::Layout::left_to_right(egui::Align::Center)
    }.with_main_wrap(true);
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
        layout,
        add_contents,
    )
}

pub fn vertical<R>(ui: &mut Ui, is_rtl: bool, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let align = if is_rtl { egui::Align::Max } else { egui::Align::Min };
    ui.with_layout(egui::Layout::top_down(align), add_contents)
}

