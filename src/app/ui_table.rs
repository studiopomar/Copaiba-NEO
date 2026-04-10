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
                let mut play_sound = false;
                let _is_rtl = self.is_rtl();
                let (current_sel, current_focus_col, multi_sel, filtered) = {
                    let tab = self.cur_mut();
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{}  ({}/{})", tr!("table.params"), tab.filtered.len(), tab.entries.len())).strong().size(11.0));
                    });
                    (tab.selected, tab.focus_col, tab.multi_selection.clone(), tab.filtered.clone())
                };

                ui.add_space(2.0);
                ui.separator();

                let mut new_sel = None;

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
                        header.col(|ui| { ui.strong(tr!("table.col.offset")).on_hover_text("Offset (OFS) - Tempo de corte inicial (ms)"); });
                        header.col(|ui| { ui.strong(tr!("table.col.overlap")).on_hover_text("Overlap (OVL) - Sobreposição com a nota anterior (ms)"); });
                        header.col(|ui| { ui.strong(tr!("table.col.preutter")).on_hover_text("Preutterance (PRE) - Duração da consoante (ms)"); });
                        header.col(|ui| { ui.strong(tr!("table.col.consonant")).on_hover_text("Consonant (CON) - Região fixa não-esticável (ms)"); });
                        header.col(|ui| { ui.strong(tr!("table.col.cutoff")).on_hover_text("Cutoff (CUT) - Ponto de corte final do ruído (ms)"); });
                        header.col(|ui| { ui.strong(tr!("table.col.notes")).on_hover_text("Notas sobre a gravação"); });
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
                                    let resp = ui.add(egui::TextEdit::singleline(&mut temp_alias).id(id).frame(false));
                                    if resp.changed() { 
                                        if temp_alias.len() > entry.alias.len() { play_sound = true; }
                                        entry.alias = temp_alias; 
                                        tab.dirty = true; 
                                    }
                                    if resp.clicked() { new_sel = Some(fi); tab.focus_col = 2; }
                                });

                                macro_rules! num_col {
                                    ($ui:expr, $val:expr, $col_idx:expr) => {
                                        let id = egui::Id::new(("cell", fi, $col_idx));
                                        let ir = $ui.push_id(id, |ui| ui.add(egui::DragValue::new($val).speed(1.0)));
                                        if ir.response.changed() { 
                                            tab.dirty = true; 
                                            play_sound = true;
                                        }
                                        if ir.response.clicked() { new_sel = Some(fi); tab.focus_col = $col_idx; }
                                    }
                                }

                                row.col(|ui| { num_col!(ui, &mut entry.offset, 3); });
                                row.col(|ui| { num_col!(ui, &mut entry.overlap, 4); });
                                row.col(|ui| { num_col!(ui, &mut entry.preutter, 5); });
                                row.col(|ui| { num_col!(ui, &mut entry.consonant, 6); });
                                row.col(|ui| {
                                    let id = egui::Id::new(("cell", fi, 7));
                                    let ir = ui.push_id(id, |ui| ui.add(egui::DragValue::new(&mut entry.cutoff).speed(1.0)));
                                    if ir.response.changed() { 
                                        tab.dirty = true; 
                                        play_sound = true;
                                    }
                                    if ir.response.clicked() { new_sel = Some(fi); tab.focus_col = 7; }
                                    ir.response.context_menu(|ui| {
                                        if ui.button(tr!("table.cutoff.invert")).clicked() {
                                            if entry.cutoff < 0.0 {
                                                entry.cutoff = 0.0; 
                                            } else {
                                                entry.cutoff = -1.0;
                                            }
                                            tab.dirty = true;
                                            ui.close_menu();
                                        }
                                    });
                                });

                                row.col(|ui| {
                                    let id = egui::Id::new(("cell", fi, 8));
                                    let resp = ui.add(egui::TextEdit::singleline(&mut entry.notes).hint_text("📝").id(id).frame(false));
                                    if resp.changed() { 
                                        tab.dirty = true; 
                                        play_sound = true;
                                    }
                                    if resp.clicked() { new_sel = Some(fi); tab.focus_col = 8; }
                                });
                            }
                        });
                    });

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
