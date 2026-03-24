use egui_i18n::tr;
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn show_tools_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("tools_panel")
            .resizable(true)
            .default_width(180.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().id_salt("tools_scroll").show(ui, |ui| {
                    ui.add_space(8.0);
                    ui.heading(format!("🛠️ {}", tr!("tools.label.presets")));
                    ui.separator();

                    let display_presets = self.presets.clone();
                    for (i, preset) in display_presets.iter().enumerate() {
                        let shortcut = format!("Ctrl+{}", i + 1);
                        if ui.button(format!("{} ({shortcut})", preset.name)).clicked() {
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

                    ui.add_space(4.0);
                    if ui.button(tr!("tools.btn.edit_presets")).clicked() { self.ui.show_preset_editor = true; }

                    ui.add_space(20.0);
                    ui.heading(format!("🕹️ {}", tr!("tools.label.edit_modes")));
                    ui.separator();
                    {
                        let tab = self.cur_mut();
                        ui.checkbox(&mut tab.wave_view.srp, format!("{} (Shift+1)", tr!("tools.label.srp")));
                        ui.label(egui::RichText::new(tr!("tools.label.srp_desc")).small());
                        ui.add_space(8.0);
                        ui.checkbox(&mut tab.wave_view.srna, format!("{} (Shift+2)", tr!("tools.label.srna")));
                        ui.label(egui::RichText::new(tr!("tools.label.srna_desc")).small());
                        ui.add_space(8.0);
                        ui.checkbox(&mut tab.wave_view.snap_to_peaks, tr!("tools.label.auto_oto"));
                        ui.label(egui::RichText::new(tr!("tools.label.auto_oto_desc")).small());
                    }
                    ui.add_space(8.0);
                    ui.checkbox(&mut self.visual.persistent_zoom, tr!("tools.label.zoom"));
                    ui.label(egui::RichText::new(tr!("tools.label.zoom_desc")).small());

                    ui.add_space(20.0);
                    ui.heading(format!("📊 {}", tr!("tools.label.status")));
                    ui.separator();
                    {
                        let tab = self.cur();
                        ui.label(format!("{} {}", tr!("tools.label.aliases"), tab.entries.len()));
                        ui.label(format!("{} {}", tr!("tools.label.filtered"), tab.filtered.len()));
                    }
                });
            });
    }
}
