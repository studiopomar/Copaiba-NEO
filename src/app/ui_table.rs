use egui::RichText;
use egui_extras::{TableBuilder, Column};
use egui_i18n::tr;
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn show_alias_table(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("alias_panel")
            .resizable(true)
            .default_height(180.0)
            .min_height(80.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                {
                    let tab = self.cur_mut();
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{}  ({}/{})", tr!("table.params"), tab.filtered.len(), tab.entries.len())).strong());
                    });
                }
                ui.add_space(2.0);
                let f_changed = {
                    let tab = self.cur_mut();
                    ui.add(
                        egui::TextEdit::singleline(&mut tab.filter)
                            .hint_text(tr!("table.filter.hint"))
                            .desired_width(f32::INFINITY)
                    ).changed()
                };
                if f_changed { self.rebuild_filter(); }

                let tab = self.cur_mut();
                ui.add_space(2.0);
                ui.separator();

                let mut new_sel = None;
                let current_sel = tab.selected;
                let current_focus_col = tab.focus_col;
                let multi_sel = tab.multi_selection.clone();
                let filtered = tab.filtered.clone();

                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::auto().at_least(40.0))         // Done ✔
                    .column(Column::initial(180.0).at_least(80.0)) // Filename
                    .column(Column::initial(150.0).at_least(80.0)) // Alias
                    .column(Column::initial(70.0).at_least(60.0))  // Offset
                    .column(Column::initial(70.0).at_least(60.0))  // Overlap
                    .column(Column::initial(90.0).at_least(60.0))  // Preutterance
                    .column(Column::initial(70.0).at_least(60.0))  // Consonant
                    .column(Column::initial(70.0).at_least(60.0))  // Cutoff
                    .column(Column::remainder().at_least(80.0))    // Anotações
                    .header(24.0, |mut header| {
                        header.col(|ui| { ui.strong("✔"); });
                        header.col(|ui| { ui.strong(tr!("table.col.file")); });
                        header.col(|ui| { ui.strong(tr!("table.col.alias")); });
                        header.col(|ui| { ui.strong(tr!("table.col.offset")); });
                        header.col(|ui| { ui.strong(tr!("table.col.overlap")); });
                        header.col(|ui| { ui.strong(tr!("table.col.preutter")); });
                        header.col(|ui| { ui.strong(tr!("table.col.consonant")); });
                        header.col(|ui| { ui.strong(tr!("table.col.cutoff")); });
                        header.col(|ui| { ui.strong(tr!("table.col.notes")); });
                    })
                    .body(|body| {
                        body.rows(24.0, filtered.len(), |mut row| {
                            let fi = row.index();
                            let idx = filtered[fi];
                            let is_selected = fi == current_sel;
                            let in_multi = multi_sel.contains(&fi);

                            let tab = self.cur_mut();
                            if let Some(entry) = tab.entries.get_mut(idx) {
                                row.set_selected(in_multi);

                                row.col(|ui| {
                                    if is_selected { ui.scroll_to_cursor(Some(egui::Align::Center)); }
                                    let id = egui::Id::new(("cell", fi, 0));
                                    if ui.push_id(id, |ui| ui.checkbox(&mut entry.done, "")).response.changed() { tab.dirty = true; }
                                    if ui.memory(|m| m.has_focus(id)) { new_sel = Some(fi); tab.focus_col = 0; }
                                });

                                row.col(|ui| {
                                    let id = egui::Id::new(("cell", fi, 1));
                                    let is_focused = is_selected && current_focus_col == 1;
                                    let resp = ui.selectable_label(is_focused, &entry.filename);
                                    if resp.clicked() { new_sel = Some(fi); tab.focus_col = 1; }
                                    if ui.memory(|m| m.has_focus(id)) { new_sel = Some(fi); tab.focus_col = 1; }
                                });

                                row.col(|ui| {
                                    let id = egui::Id::new(("cell", fi, 2));
                                    let mut temp_alias = entry.alias.clone();
                                    let resp = ui.add(egui::TextEdit::singleline(&mut temp_alias).id(id).frame(false));
                                    if resp.changed() { entry.alias = temp_alias; tab.dirty = true; }
                                    if resp.clicked() { new_sel = Some(fi); tab.focus_col = 2; }
                                    if ui.memory(|m| m.has_focus(id)) { new_sel = Some(fi); tab.focus_col = 2; }
                                });

                                macro_rules! num_col {
                                    ($ui:expr, $val:expr, $col_idx:expr) => {
                                        let id = egui::Id::new(("cell", fi, $col_idx));
                                        let ir = $ui.push_id(id, |ui| ui.add(egui::DragValue::new($val).speed(1.0)));
                                        if ir.response.changed() { tab.dirty = true; }
                                        if ir.response.clicked() { new_sel = Some(fi); tab.focus_col = $col_idx; }
                                        if $ui.memory(|m| m.has_focus(id)) { new_sel = Some(fi); tab.focus_col = $col_idx; }
                                    }
                                }

                                row.col(|ui| { num_col!(ui, &mut entry.offset, 3); });
                                row.col(|ui| { num_col!(ui, &mut entry.overlap, 4); });
                                row.col(|ui| { num_col!(ui, &mut entry.preutter, 5); });
                                row.col(|ui| { num_col!(ui, &mut entry.consonant, 6); });
                                row.col(|ui| { 
                                    num_col!(ui, &mut entry.cutoff, 7); 
                                    ui.interact_bg(egui::Sense::click()).context_menu(|ui| {
                                        if ui.button(tr!("table.cutoff.invert")).clicked() {
                                            if entry.cutoff < 0.0 {
                                                // Convert to positive (relative to end) - approximate
                                                entry.cutoff = 0.0; 
                                            } else {
                                                // Convert to negative (relative to offset)
                                                entry.cutoff = -1.0;
                                            }
                                            tab.dirty = true;
                                            ui.close_menu();
                                        }
                                    });
                                });

                                row.col(|ui| {
                                    let id = egui::Id::new(("cell", fi, 8));
                                    let resp = ui.add(egui::TextEdit::singleline(&mut entry.notes).hint_text("...").id(id).frame(false));
                                    if resp.changed() { tab.dirty = true; }
                                    if resp.clicked() { new_sel = Some(fi); tab.focus_col = 8; }
                                    if ui.memory(|m| m.has_focus(id)) { new_sel = Some(fi); tab.focus_col = 8; }
                                });
                            }
                        });
                    });

                if let Some(fi) = new_sel {
                    let ctrl = ctx.input(|i| i.modifiers.ctrl);
                    let shift = ctx.input(|i| i.modifiers.shift);
                    self.select_multi(fi, ctrl, shift);
                }
            });
    }
}
