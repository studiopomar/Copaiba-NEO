use egui_extras::{TableBuilder, Column};
use egui_i18n::tr;
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn show_alias_table(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("alias_sidebar")
            .resizable(true)
            .default_width(420.0)
            .min_width(200.0)
            .show(ctx, |ui| {
                let mut play_sound = false;
                let (current_sel, current_focus_col, multi_sel, filtered) = {
                    let tab = self.cur_mut();
                    ui.horizontal(|ui| {
                        ui.strong(format!("{} ({}/{})", tr!("table.params"), tab.filtered.len(), tab.entries.len()));
                    });
                    ui.separator();
                    (tab.selected, tab.focus_col, tab.multi_selection.clone(), tab.filtered.clone())
                };

                let mut new_sel = None;
                let panel_width = ui.available_width();

                // Adaptive column widths based on panel width
                let use_compact = panel_width < 380.0;
                let (w_chk, w_file, w_alias, w_num) = if use_compact {
                    (20.0, 60.0, 50.0, 42.0)
                } else {
                    (24.0, 90.0, 80.0, 55.0)
                };

                // Track batch delta edits: (field_index, delta)
                // field_index: 0=offset, 1=overlap, 2=preutter, 3=consonant, 4=cutoff
                let mut batch_delta: Option<(usize, f64)> = None;
                let has_multi = multi_sel.len() > 1;

                egui::ScrollArea::vertical().id_salt("alias_vscroll").show(ui, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::exact(w_chk))                       // Done ✔
                        .column(Column::initial(w_file).at_least(40.0).clip(true))  // Filename
                        .column(Column::initial(w_alias).at_least(35.0).clip(true)) // Alias
                        .column(Column::initial(w_num).at_least(35.0))      // Offset
                        .column(Column::initial(w_num).at_least(35.0))      // Overlap
                        .column(Column::initial(w_num).at_least(35.0))      // Preutterance
                        .column(Column::initial(w_num).at_least(35.0))      // Consonant
                        .column(Column::initial(w_num).at_least(35.0))      // Cutoff
                        .header(22.0, |mut header| {
                            header.col(|ui| { ui.strong("✔"); });
                            header.col(|ui| { let h = tr!("table.col.file"); ui.strong(if use_compact { "File".to_string() } else { h }); });
                            header.col(|ui| { let h = tr!("table.col.alias"); ui.strong(if use_compact { "Alias".to_string() } else { h }); });
                            header.col(|ui| { ui.strong("OFS"); });
                            header.col(|ui| { ui.strong("OVL"); });
                            header.col(|ui| { ui.strong("PRE"); });
                            header.col(|ui| { ui.strong("CON"); });
                            header.col(|ui| { ui.strong("CUT"); });
                        })
                        .body(|body| {
                            body.rows(22.0, filtered.len(), |mut row| {
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
                                        let resp = ui.push_id(id, |ui| ui.checkbox(&mut entry.done, "")).response;
                                        if resp.changed() { 
                                            tab.dirty = true;
                                            play_sound = true;
                                        }
                                        if resp.clicked() { new_sel = Some(fi); tab.focus_col = 0; }
                                    });

                                    row.col(|ui| {
                                        let is_focused = is_selected && current_focus_col == 1;
                                        let resp = ui.selectable_label(is_focused, &entry.filename);
                                        if resp.clicked() { new_sel = Some(fi); tab.focus_col = 1; }
                                    });

                                    row.col(|ui| {
                                        let id = egui::Id::new(("cell", fi, 2));
                                        let mut temp_alias = entry.alias.clone();
                                        let resp = ui.add(egui::TextEdit::singleline(&mut temp_alias).id(id).frame(false).desired_width(ui.available_width()));
                                        if resp.changed() { 
                                            if temp_alias.len() > entry.alias.len() { play_sound = true; }
                                            entry.alias = temp_alias; 
                                            tab.dirty = true; 
                                        }
                                        if resp.clicked() { new_sel = Some(fi); tab.focus_col = 2; }
                                    });

                                    // Macro for numeric columns with multi-edit delta tracking
                                    macro_rules! num_col_multi {
                                        ($ui:expr, $val:expr, $col_idx:expr, $field_idx:expr) => {
                                            let id = egui::Id::new(("cell", fi, $col_idx));
                                            let old_val = *$val;
                                            let ir = $ui.push_id(id, |ui| ui.add(egui::DragValue::new($val).speed(1.0)));
                                            if ir.response.changed() {
                                                let delta = *$val - old_val;
                                                tab.dirty = true;
                                                play_sound = true;
                                                if has_multi && in_multi {
                                                    batch_delta = Some(($field_idx, delta));
                                                }
                                            }
                                            if ir.response.clicked() { new_sel = Some(fi); tab.focus_col = $col_idx; }
                                        }
                                    }

                                    row.col(|ui| { num_col_multi!(ui, &mut entry.offset, 3, 0); });
                                    row.col(|ui| { num_col_multi!(ui, &mut entry.overlap, 4, 1); });
                                    row.col(|ui| { num_col_multi!(ui, &mut entry.preutter, 5, 2); });
                                    row.col(|ui| { num_col_multi!(ui, &mut entry.consonant, 6, 3); });
                                    row.col(|ui| { num_col_multi!(ui, &mut entry.cutoff, 7, 4); });
                                }
                            });
                        });
                });

                // Apply batch delta to all other selected entries
                if let Some((field_idx, delta)) = batch_delta {
                    let tab = self.cur_mut();
                    for &fi in &multi_sel {
                        if let Some(&idx) = filtered.get(fi) {
                            if let Some(entry) = tab.entries.get_mut(idx) {
                                match field_idx {
                                    0 => entry.offset += delta,
                                    1 => entry.overlap += delta,
                                    2 => entry.preutter += delta,
                                    3 => entry.consonant += delta,
                                    4 => entry.cutoff += delta,
                                    _ => {}
                                }
                            }
                        }
                    }
                    tab.dirty = true;
                }

                if let Some(fi) = new_sel {
                    let ctrl = ctx.input(|i| i.modifiers.ctrl);
                    let shift = ctx.input(|i| i.modifiers.shift);
                    let old_sel = self.cur().selected;
                    self.select_multi(fi, ctrl, shift);
                    
                    if self.config.play_on_select && self.cur().selected != old_sel {
                        self.play_current_segment(false);
                    }
                }
                if play_sound { self.play_key_sound(); }
            });
    }
}
