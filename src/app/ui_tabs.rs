use egui_i18n::tr;
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn show_tab_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").min_height(32.0).show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.spacing_mut().button_padding.y = 2.0;

            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add_space(4.0);
                let btn_text = format!("➕ {}", tr!("tabs.btn.new_tab"));
                if ui.add_sized([0.0, 24.0], egui::Button::new(btn_text)).clicked() {
                    self.tabs.push(super::state::TabState::default());
                    self.current_tab = self.tabs.len() - 1;
                }
                ui.separator();

                // Right-side button: reserve space first so tabs scroll in the remaining area
                let right_width = 100.0;
                let avail = ui.available_width() - right_width;

                let mut to_remove = None;

                egui::ScrollArea::horizontal()
                    .id_salt("tab_bar_scroll")
                    .max_width(avail.max(100.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for i in 0..self.tabs.len() {
                                let is_active = self.current_tab == i;
                                let is_renaming = self.ui.renaming_tab == Some(i);

                                ui.spacing_mut().item_spacing.x = 2.0;
                                ui.spacing_mut().interact_size.y = 24.0;
                                
                                ui.horizontal(|ui| {
                                    if is_renaming {
                                        let resp = ui.add(egui::TextEdit::singleline(&mut self.tabs[i].name).desired_width(80.0));
                                        if resp.changed() { self.play_key_sound(); }
                                        if resp.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                            self.ui.renaming_tab = None;
                                        }
                                    } else {
                                        let mut name = self.tabs[i].name.clone();
                                        if name.is_empty() {
                                            name = tr!("state.tab.default_name").to_string();
                                        }
                                        let label_text = format!("  {} {}  ", name, if self.tabs[i].dirty { "*" } else { "" });
                                        let resp = ui.add_sized([0.0, 24.0], egui::SelectableLabel::new(is_active, label_text));
                                        if resp.clicked() { 
                                            if self.current_tab != i {
                                                self.stop_playback();
                                                self.current_tab = i;
                                            }
                                        }
                                        if resp.double_clicked() { self.ui.renaming_tab = Some(i); }
                                    }

                                    if self.tabs.len() > 1 && !is_renaming {
                                        let close_btn = egui::Button::new("x").small();
                                        if ui.add_sized([0.0, 24.0], close_btn).on_hover_text(tr!("tabs.btn.close_tab")).clicked() { 
                                            to_remove = Some(i); 
                                        }
                                    }
                                });
                                ui.add_space(8.0);
                            }
                        });
                    });

                if let Some(idx) = to_remove {
                    self.tabs.remove(idx);
                    self.current_tab = self.current_tab.min(self.tabs.len() - 1);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let folder_btn = egui::Button::new(format!("📁 {}", tr!("tabs.btn.open_folder")));
                    if ui.add_sized([0.0, 24.0], folder_btn).clicked() { 
                        self.open_voicebank_dir(); 
                    }
                });
            });
        });
    }
}
