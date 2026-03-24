use egui_i18n::tr;
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn show_tab_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").min_height(32.0).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(4.0);
                if ui.button(format!("➕ {}", tr!("tabs.btn.new_tab"))).clicked() {
                    self.tabs.push(super::state::TabState::default());
                    self.current_tab = self.tabs.len() - 1;
                }
                ui.separator();

                let mut to_remove = None;
                for i in 0..self.tabs.len() {
                    let is_active = self.current_tab == i;
                    let is_renaming = self.ui.renaming_tab == Some(i);

                    ui.style_mut().spacing.item_spacing.x = 2.0;
                    ui.horizontal(|ui| {
                        if is_renaming {
                            let resp = ui.add(egui::TextEdit::singleline(&mut self.tabs[i].name).desired_width(80.0));
                            if resp.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                self.ui.renaming_tab = None;
                            }
                        } else {
                            let name = format!("{}{} ", self.tabs[i].name, if self.tabs[i].dirty { "*" } else { "" });
                            let resp = ui.selectable_label(is_active, name);
                            if resp.clicked() { 
                                if self.current_tab != i {
                                    self.stop_playback();
                                    self.current_tab = i;
                                }
                            }
                            if resp.double_clicked() { self.ui.renaming_tab = Some(i); }
                        }

                        if self.tabs.len() > 1 && !is_renaming {
                            if ui.small_button("x").on_hover_text(tr!("tabs.btn.close_tab")).clicked() { to_remove = Some(i); }
                        }
                    });
                    ui.add_space(8.0);
                }
                if let Some(idx) = to_remove {
                    self.tabs.remove(idx);
                    self.current_tab = self.current_tab.min(self.tabs.len() - 1);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(format!("📁 {}", tr!("tabs.btn.open_folder"))).clicked() { self.open_voicebank_dir(); }
                });
            });
        });
    }
}
