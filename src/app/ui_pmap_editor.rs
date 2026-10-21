use egui::{RichText, Color32, Vec2};
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn show_pmap_editor(&mut self, ctx: &egui::Context) {
        let is_rtl = self.is_rtl();
        let mut window_open = self.ui.show_pmap_editor;
        if !window_open { return; }

        let mut save_clicked = false;
        let mut cancel_clicked = false;
        let mut close_window = false;

        egui::Window::new("🗺️ Prefix Map Editor")
            .open(&mut window_open)
            .default_size(Vec2::new(450.0, 500.0))
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                let tab_idx = self.current_tab;
                if tab_idx >= self.tabs.len() { return; }

                // Search and Batch controls
                ui.group(|ui| {
                    crate::app::layout::horizontal(ui, is_rtl, |ui| {
                        ui.label("🔍");
                        ui.add(egui::TextEdit::singleline(&mut self.ui.pmap_search_query)
                            .hint_text("Search pitch...")
                            .desired_width(120.0));
                        
                        if ui.button("✖").on_hover_text("Clear search").clicked() {
                            self.ui.pmap_search_query.clear();
                        }

                        ui.separator();

                        ui.label("⚡ Batch:");
                        ui.add(egui::TextEdit::singleline(&mut self.ui.pmap_batch_pre).hint_text("Pre").desired_width(40.0));
                        ui.add(egui::TextEdit::singleline(&mut self.ui.pmap_batch_suf).hint_text("Suf").desired_width(40.0));
                        
                        if ui.button("Apply").on_hover_text("Apply to selected").clicked() {
                            let b_pre = self.ui.pmap_batch_pre.clone();
                            let b_suf = self.ui.pmap_batch_suf.clone();
                            let query = self.ui.pmap_search_query.to_lowercase();
                            let tab = self.cur_mut();
                            let has_selection = tab.prefix_map.iter().any(|e| e.selected);
                            
                            for e in &mut tab.prefix_map {
                                let matches = query.is_empty() || e.pitch.to_lowercase().contains(&query);
                                if matches && (!has_selection || e.selected) {
                                    if !b_pre.is_empty() { e.prefix = b_pre.clone(); }
                                    if !b_suf.is_empty() { e.suffix = b_suf.clone(); }
                                }
                            }
                        }
                    });

                    ui.add_space(4.0);
                    crate::app::layout::horizontal(ui, is_rtl, |ui| {
                        if ui.button("Select All").clicked() {
                            let query = self.ui.pmap_search_query.to_lowercase();
                            for e in &mut self.cur_mut().prefix_map {
                                if query.is_empty() || e.pitch.to_lowercase().contains(&query) {
                                    e.selected = true;
                                }
                            }
                        }
                        if ui.button("Deselect All").clicked() {
                            for e in &mut self.cur_mut().prefix_map { e.selected = false; }
                        }
                        if ui.button("Invert").clicked() {
                            let query = self.ui.pmap_search_query.to_lowercase();
                            for e in &mut self.cur_mut().prefix_map {
                                if query.is_empty() || e.pitch.to_lowercase().contains(&query) {
                                    e.selected = !e.selected;
                                }
                            }
                        }
                    });
                });

                ui.add_space(8.0);

                // Table with Prefix Map
                let query = self.ui.pmap_search_query.to_lowercase();
                let tab = &mut self.tabs[tab_idx];
                let pmap_indices: Vec<usize> = tab.prefix_map.iter().enumerate()
                    .filter(|(_, e)| query.is_empty() || e.pitch.to_lowercase().contains(&query))
                    .map(|(i, _)| i)
                    .collect();

                let is_dirty = tab.prefix_map != tab.original_prefix_map;

                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(if is_rtl { egui::Layout::right_to_left(egui::Align::Center) } else { egui::Layout::left_to_right(egui::Align::Center) })
                    .column(egui_extras::Column::auto()) // ✔
                    .column(egui_extras::Column::initial(60.0)) // Pitch
                    .column(egui_extras::Column::remainder()) // Prefix
                    .column(egui_extras::Column::remainder()) // Suffix
                    .header(30.0, |mut header| {
                        header.col(|ui| { ui.strong("✔"); });
                        header.col(|ui| { ui.strong("Pitch"); });
                        header.col(|ui| { ui.strong("Prefix"); });
                        header.col(|ui| { ui.strong("Suffix"); });
                    })
                    .body(|body| {
                        body.rows(30.0, pmap_indices.len(), |mut row| {
                            let idx = pmap_indices[row.index()];
                            let tab = &mut self.tabs[tab_idx];
                            if let Some(entry) = tab.prefix_map.get_mut(idx) {
                                let original = tab.original_prefix_map.get(idx);
                                let changed = original.map(|o| o != entry).unwrap_or(true);
                                
                                row.col(|ui| { ui.checkbox(&mut entry.selected, ""); });
                                row.col(|ui| {
                                    let mut label = RichText::new(&entry.pitch).strong();
                                    if changed { label = label.color(Color32::from_rgb(255, 180, 100)); }
                                    ui.label(label);
                                });
                                row.col(|ui| {
                                    ui.add(egui::TextEdit::singleline(&mut entry.prefix).font(egui::FontId::monospace(12.0)));
                                });
                                row.col(|ui| {
                                    ui.add(egui::TextEdit::singleline(&mut entry.suffix).font(egui::FontId::monospace(12.0)));
                                });
                            }
                        });
                    });

                // Bottom buttons
                ui.separator();
                crate::app::layout::horizontal(ui, is_rtl, |ui| {
                    if ui.add_enabled(is_dirty, egui::Button::new(RichText::new("💾 Save Changes").strong())).clicked() {
                        save_clicked = true;
                    }
                    if ui.add_enabled(is_dirty, egui::Button::new("✖ Cancel")).clicked() {
                        cancel_clicked = true;
                    }
                    ui.with_layout(if is_rtl { egui::Layout::left_to_right(egui::Align::Center) } else { egui::Layout::right_to_left(egui::Align::Center) }, |ui| {
                        if ui.button("Close").clicked() {
                            close_window = true;
                        }
                    });
                });
            });

        self.ui.show_pmap_editor = window_open && !close_window;

        if save_clicked {
            self.save_pmap_changes();
            self.ui.show_pmap_editor = false;
        }
        if cancel_clicked {
            let tab = self.cur_mut();
            tab.prefix_map = tab.original_prefix_map.clone();
            self.ui.show_pmap_editor = false;
        }
    }

    fn save_pmap_changes(&mut self) {
        let tab_idx = self.current_tab;
        let tab = &mut self.tabs[tab_idx];
        if let Some(root) = &tab.root_path {
            let pmap_path = root.join("prefix.map");
            
            let mut content = String::new();
            for entry in &tab.prefix_map {
                content.push_str(&format!("{}\t{}\t{}\n", entry.pitch, entry.prefix, entry.suffix));
            }
            
            if std::fs::write(&pmap_path, content).is_ok() {
                tab.original_prefix_map = tab.prefix_map.clone();
                self.ui.toast_manager.success("prefix.map saved!");
            } else {
                self.ui.toast_manager.error("Failed to save prefix.map");
            }
        }
    }
}
