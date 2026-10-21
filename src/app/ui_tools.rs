use egui::{RichText, Color32};
use egui_i18n::tr;
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn show_tools_panel(&mut self, ctx: &egui::Context) {
        if !self.ui.show_tools_panel { return; }
        let is_rtl = self.is_rtl();

        egui::TopBottomPanel::bottom("tools_panel")
            .resizable(false)
            .exact_height(72.0)
            .frame(egui::Frame::new()
                .fill(ctx.style().visuals.faint_bg_color)
                .inner_margin(egui::Margin::symmetric(10, 7)))
            .show(ctx, |ui| {
                egui::ScrollArea::horizontal().id_salt("tools_hscroll").show(ui, |ui| {
                    ui.with_layout(if is_rtl { egui::Layout::right_to_left(egui::Align::Center) } else { egui::Layout::left_to_right(egui::Align::Center) }, |ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;

                        // --- PRESETS ---
                        ui.label(RichText::new(tr!("tools.label.presets")).strong().size(12.0).color(Color32::from_rgb(195, 157, 226)));
                        let display_presets = self.presets.clone();
                        for (i, preset) in display_presets.iter().enumerate() {
                            let shortcut = format!("Ctrl+{}", i + 1);
                            if ui.small_button(format!("{} ({shortcut})", preset.name)).clicked() {
                                let idx = {
                                    let tab = self.cur();
                                    tab.filtered.get(tab.selected).copied()
                                };
                                if let Some(idx) = idx {
                                    self.save_undo_state();
                                    let tab = self.cur_mut();
                                    if let Some(entry) = tab.entries.get_mut(idx) {
                                        entry.consonant = preset.consonant;
                                        entry.cutoff = preset.cutoff;
                                        entry.preutter = preset.preutter;
                                        entry.overlap = preset.overlap;
                                        tab.dirty = true;
                                    }
                                }
                            }
                        }
                        if ui.small_button(tr!("tools.btn.edit_presets")).clicked() { self.ui.show_preset_editor = true; }
                        if ui.small_button("Prefix Map").clicked() { self.ui.show_pmap_editor = true; }

                        ui.separator();

                        // --- EDIT MODES ---
                        ui.label(RichText::new(tr!("tools.label.edit_modes")).strong().size(12.0).color(Color32::from_rgb(195, 157, 226)));
                        {
                            let tab = self.cur_mut();
                            ui.checkbox(&mut tab.wave_view.srp, tr!("tools.label.srp"))
                                .on_hover_text(tr!("tools.label.srp_desc"));
                            ui.checkbox(&mut tab.wave_view.sro, tr!("tools.label.sro"))
                                .on_hover_text(tr!("tools.label.sro_desc"));
                        }
                        ui.checkbox(&mut self.visual.persistent_zoom, tr!("tools.label.zoom"))
                            .on_hover_text(tr!("tools.label.zoom_desc"));

                        ui.separator();

                        // --- SELECTION & PROGRESS ---
                        {
                            let tab = self.cur();
                            if !tab.multi_selection.is_empty() {
                                ui.separator();
                                ui.label(RichText::new(format!("{} selecionados", tab.multi_selection.len())).size(12.0).color(Color32::from_rgb(240, 195, 110)));
                            }
                            
                            ui.separator();
                            let done = tab.entries.iter().filter(|e| e.done).count();
                            let total = tab.entries.len();
                            let percent = if total > 0 { (done as f32 / total as f32) * 100.0 } else { 0.0 };
                            ui.label(RichText::new(format!("Progresso: {}/{} ({:.1}%)", done, total, percent)).size(12.0).color(Color32::from_rgb(150, 220, 180)));
                        }

                        ui.separator();

                        // --- SHORTCUTS HINTS ---
                        crate::app::layout::vertical(ui, is_rtl, |ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.label(RichText::new("⌨ Atalhos").size(11.0).weak());
                            ui.label(RichText::new("[Space] Play | [Ctrl+S] Salvar | [Q/W/E/R/T] Parâmetros").size(10.0).weak());
                        });
                    });
                });
            });
    }
}
