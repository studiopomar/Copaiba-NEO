use egui::{RichText, Color32};
use egui_i18n::tr;
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn show_tools_panel(&mut self, ctx: &egui::Context) {
        if !self.ui.show_tools_panel { return; }

        egui::TopBottomPanel::bottom("tools_panel")
            .resizable(false)
            .exact_height(56.0)
            .show(ctx, |ui| {
                egui::ScrollArea::horizontal().id_salt("tools_hscroll").show(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;

                        // --- PRESETS ---
                        ui.label(RichText::new(tr!("tools.label.presets")).strong().size(10.0).color(Color32::from_rgb(137, 180, 250)));
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
                        ui.label(RichText::new(tr!("tools.label.edit_modes")).strong().size(10.0).color(Color32::from_rgb(137, 180, 250)));
                        {
                            let tab = self.cur_mut();
                            ui.checkbox(&mut tab.wave_view.srp, tr!("tools.label.srp"))
                                .on_hover_text(tr!("tools.label.srp_desc"));
                            ui.checkbox(&mut tab.wave_view.sro, tr!("tools.label.sro"))
                                .on_hover_text(tr!("tools.label.sro_desc"));
                            ui.checkbox(&mut tab.wave_view.snap_to_peaks, tr!("tools.label.auto_oto"))
                                .on_hover_text(tr!("tools.label.auto_oto_desc"));
                        }
                        ui.checkbox(&mut self.visual.persistent_zoom, tr!("tools.label.zoom"))
                            .on_hover_text(tr!("tools.label.zoom_desc"));

                        ui.separator();

                        // --- STATUS ---
                        {
                            let tab = self.cur();
                            ui.label(RichText::new(format!("Aliases: {}  |  {}: {}", tab.entries.len(), tr!("tools.label.filtered"), tab.filtered.len())).size(10.0));
                        }
                    });
                });
            });
    }
}
