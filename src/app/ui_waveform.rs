use crate::waveform::draw_waveform;
use super::state::{CopaibaApp, ShortcutProfile};

impl CopaibaApp {
    pub fn show_waveform_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (has_entries, tab_selected, _filtered_len) = {
                let tab = self.cur();
                (tab.entries.len(), tab.selected, tab.filtered.len())
            };

            if has_entries == 0 {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new("🎵  Copaiba NEO").size(28.0).color(egui::Color32::from_rgb(140, 100, 200)));
                });
                return;
            }

            let idx_opt = self.cur().filtered.get(tab_selected).copied();
            if let Some(idx) = idx_opt {
                let full_path_key = {
                    let tab = self.cur();
                    let fname = tab.entries[idx].filename.clone();
                    tab.oto_dir.as_ref().map(|d| d.join(&fname).to_string_lossy().to_string()).unwrap_or(fname)
                };

                let (wav_opt, sd_opt, pd_opt) = {
                    let wav = self.wav_cache.get(&full_path_key).cloned();
                    let sd = if self.visual.show_spectrogram { self.spec_data_cache.get(&full_path_key).cloned() } else { None };
                    let pd = if self.visual.wave.show_pitch { self.pitch_data_cache.get(&full_path_key).cloned() } else { None };
                    (wav, sd, pd)
                };

                if let Some(wav) = wav_opt {
                    let mut playback_cursor_val = self.audio.playback_start.map(|s| {
                        let elapsed = s.elapsed().as_secs_f64() * 1000.0;
                        self.audio.playback_offset_ms + (elapsed * self.audio.playback_speed as f64)
                    });

                    if let (Some(cur), Some(limit)) = (playback_cursor_val, self.audio.playback_limit_ms) {
                        if cur >= limit {
                            self.audio.playback_start = None;
                            self.audio.playback_limit_ms = None;
                            playback_cursor_val = None;
                        }
                    }

                    let spec_set = self.visual.spec.clone();
                    let wave_set = self.visual.wave.clone();
                    let show_min = self.visual.show_minimap;
                    let profile = self.config.shortcut_profile;
                    let customs = self.config.custom_shorts.clone();

                    let mut curr_do_undo = false;
                    let mut curr_do_dirty = false;
                    let mut curr_nav_sel = None;
                    let mut curr_trigger_play = false;

                    {
                        let tab = self.cur_mut();
                        if let Some(entry) = tab.entries.get_mut(idx) {
                            let mut do_undo = false;
                            let mut do_dirty = false;
                            let mut new_sel = None;

                            tab.wave_view.show_minimap = show_min;
                            let res = draw_waveform(ui, &wav, sd_opt.as_ref(), pd_opt.as_ref(), &mut tab.wave_view, entry, playback_cursor_val, &spec_set, &wave_set);
                            if res.drag_started { do_undo = true; }
                            if res.modified { do_dirty = true; }

                            let trigger_play = res.clicked || res.drag_released;
                            let nav_delta = res.nav_delta;

                            if nav_delta != 0 {
                                if nav_delta < 0 && tab.selected > 0 { new_sel = Some(tab.selected - 1); }
                                else if nav_delta > 0 && tab.selected + 1 < tab.filtered.len() { new_sel = Some(tab.selected + 1); }
                            }

                            let (k_off, k_ove, k_pre, k_con, k_cut) = match profile {
                                ShortcutProfile::Copaiba => (egui::Key::Q, egui::Key::W, egui::Key::E, egui::Key::R, egui::Key::T),
                                ShortcutProfile::SetParam => (egui::Key::F1, egui::Key::F2, egui::Key::F3, egui::Key::F4, egui::Key::F5),
                                ShortcutProfile::Utau => (egui::Key::Q, egui::Key::W, egui::Key::E, egui::Key::R, egui::Key::T), // Placeholder
                                ShortcutProfile::VLabeler => (egui::Key::Q, egui::Key::W, egui::Key::E, egui::Key::R, egui::Key::T), // Placeholder
                                ShortcutProfile::Custom => (customs.off, customs.ove, customs.pre, customs.con, customs.cut),
                            };

                            let has_focus = ctx.memory(|m| m.focused().is_some());
                            if !has_focus || tab.wave_view.mouse_ms.is_some() {
                                let keys = [k_off, k_ove, k_pre, k_con, k_cut];
                                let down: Vec<bool> = keys.iter().map(|k| ctx.input(|i| i.key_down(*k))).collect();
                                let pressed: Vec<bool> = keys.iter().map(|k| ctx.input(|i| i.key_pressed(*k))).collect();

                                if pressed.iter().any(|&p| p) { do_undo = true; }

                                if down.iter().any(|&d| d) && tab.wave_view.mouse_ms.is_some() {
                                    let ms = tab.wave_view.mouse_ms.unwrap_or(0.0);
                                    let dur = wav.duration_ms;
                                    let curr_c_ms = (dur - entry.cutoff).max(0.0);

                                    if down[0] { // Offset
                                        let old_off = entry.offset;
                                        let new_off = ms.max(0.0);
                                        let delta = new_off - old_off;
                                        if tab.wave_view.srp || tab.wave_view.sro {
                                            entry.offset = new_off;
                                            if entry.cutoff >= 0.0 { entry.cutoff = (entry.cutoff - delta).max(0.0); }
                                        } else {
                                            entry.offset = new_off;
                                            entry.overlap = entry.overlap - delta;
                                            entry.preutter = entry.preutter - delta;
                                            entry.consonant = entry.consonant - delta;
                                            if entry.cutoff < 0.0 { entry.cutoff += delta; }
                                        }
                                        do_dirty = true;
                                    }
                                    if down[1] { // Overlap
                                        if tab.wave_view.sro {
                                            let old_abs = entry.offset + entry.overlap;
                                            let delta = ms - old_abs;
                                            let old_off = entry.offset;
                                            entry.offset = (entry.offset + delta).max(0.0);
                                            let off_real_delta = entry.offset - old_off;
                                            if entry.cutoff >= 0.0 { entry.cutoff = (entry.cutoff - off_real_delta).max(0.0); }
                                        } else {
                                            let o_ms = ms.min(curr_c_ms);
                                            entry.overlap = o_ms - entry.offset;
                                        }
                                        do_dirty = true;
                                    }
                                    if down[2] { // Preutterance
                                        if tab.wave_view.srp {
                                            let old_abs = entry.offset + entry.preutter;
                                            let delta = ms - old_abs;
                                            let old_off = entry.offset;
                                            entry.offset = (entry.offset + delta).max(0.0);
                                            let off_real_delta = entry.offset - old_off;
                                            if entry.cutoff >= 0.0 { entry.cutoff = (entry.cutoff - off_real_delta).max(0.0); }
                                        } else {
                                            let p_ms = ms.max(entry.offset).min(curr_c_ms);
                                            entry.preutter = p_ms - entry.offset;
                                        }
                                        do_dirty = true;
                                    }
                                    if down[3] { // Consonant
                                        let pos = ms.max(entry.offset).min(curr_c_ms);
                                        entry.consonant = pos - entry.offset;
                                        do_dirty = true;
                                    }
                                    if down[4] { // Cutoff
                                        let max_rel = entry.consonant.max(entry.preutter).max(entry.overlap);
                                        let min_ms = entry.offset + max_rel;
                                        if entry.cutoff < 0.0 {
                                            let pos = ms.max(min_ms + 1.0);
                                            entry.cutoff = -(pos - entry.offset);
                                        } else {
                                            let pos = ms.max(min_ms);
                                            entry.cutoff = (dur - pos).max(0.0);
                                        }
                                        do_dirty = true;
                                    }
                                    ctx.request_repaint();
                                }
                            }
                            curr_do_undo = do_undo;
                            curr_do_dirty = do_dirty;
                            curr_nav_sel = new_sel;
                            curr_trigger_play = trigger_play;
                        }
                    }

                    if curr_do_undo { self.save_undo_state(); }
                    if curr_do_dirty { self.cur_mut().dirty = true; }
                    if let Some(fi) = curr_nav_sel {
                        let old_sel = self.cur().selected;
                        self.select_multi(fi, false, false);
                        if self.config.play_on_select && self.cur().selected != old_sel { self.play_current_segment(false); }
                    } else if curr_trigger_play && self.config.play_on_select {
                        self.play_current_segment(false);
                    }
                }
            }

            if ui.input(|i| i.pointer.any_pressed()) && ui.rect_contains_pointer(ui.max_rect()) {
                if let Some(id) = ctx.memory(|m| m.focused()) { ctx.memory_mut(|m| m.surrender_focus(id)); }
            }
        });
    }

    /// Handle global keyboard shortcuts (called before drawing panels)
    pub fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let ctrl = ctx.input(|i| i.modifiers.ctrl);
        let shift = ctx.input(|i| i.modifiers.shift);

        let sc1 = egui::KeyboardShortcut::new(egui::Modifiers::SHIFT, egui::Key::Num1);
        let sc2 = egui::KeyboardShortcut::new(egui::Modifiers::SHIFT, egui::Key::Num2);
        let a1 = egui::KeyboardShortcut::new(egui::Modifiers::ALT, egui::Key::Num1);
        let a2 = egui::KeyboardShortcut::new(egui::Modifiers::ALT, egui::Key::Num2);

        let s1 = ctx.input_mut(|i| i.consume_shortcut(&sc1) || i.consume_shortcut(&a1));
        let s2 = ctx.input_mut(|i| i.consume_shortcut(&sc2) || i.consume_shortcut(&a2));

        if s1 {
            let tab = self.cur_mut();
            tab.wave_view.srp = !tab.wave_view.srp;
            if tab.wave_view.srp { tab.wave_view.sro = false; }
            self.play_key_sound();
        }
        if s2 {
            let tab = self.cur_mut();
            tab.wave_view.sro = !tab.wave_view.sro;
            if tab.wave_view.sro { tab.wave_view.srp = false; }
            self.play_key_sound();
        }

        // Preset shortcuts Ctrl+1..5
        if ctrl && !shift {
            let keys = [egui::Key::Num1, egui::Key::Num2, egui::Key::Num3, egui::Key::Num4, egui::Key::Num5];
            for (i, key) in keys.iter().enumerate() {
                if ctx.input_mut(|inp| inp.consume_key(egui::Modifiers::CTRL, *key)) {
                    let p = self.presets[i].clone();
                    // Collect all indices to apply preset to (multi-selection or single)
                    let indices: Vec<usize> = {
                        let tab = self.cur();
                        if tab.multi_selection.len() > 1 {
                            tab.multi_selection.iter()
                                .filter_map(|&fi| tab.filtered.get(fi).copied())
                                .collect()
                        } else {
                            tab.filtered.get(tab.selected).copied().into_iter().collect()
                        }
                    };
                    if !indices.is_empty() {
                        self.save_undo_state();
                        let tab = self.cur_mut();
                        for idx in indices {
                            if let Some(entry) = tab.entries.get_mut(idx) {
                                entry.consonant = p.consonant;
                                entry.cutoff = p.cutoff;
                                entry.preutter = p.preutter;
                                entry.overlap = p.overlap;
                                tab.dirty = true;
                            }
                        }
                        self.play_key_sound();
                    }
                }
            }
        }

        // Ctrl+A — select all
        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::A)) && ctx.memory(|m| m.focused().is_none()) {
            let tab = self.cur_mut();
            tab.multi_selection.clear();
            for fi in 0..tab.filtered.len() { tab.multi_selection.insert(fi); }
            self.play_key_sound();
        }

        // Save
        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::S)) {
            if shift { self.save_as(); } else { self.save_oto(); }
            self.play_key_sound();
        }

        // Open
        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::O)) { 
            if shift { self.open_oto(); } else { self.open_voicebank_dir(); }
            self.play_key_sound(); 
        }

        // Home
        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::H)) {
            self.ui.show_home = true;
            self.play_key_sound();
        }

        // Settings
        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::Comma)) { self.ui.show_settings = !self.ui.show_settings; self.play_key_sound(); }

        // Undo / Redo
        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::Z)) { self.undo(ctx); self.play_key_sound(); }
        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::Y)) { self.redo(ctx); self.play_key_sound(); }

        // Help
        if ctx.input(|i| i.key_pressed(egui::Key::F1)) { self.ui.show_help = !self.ui.show_help; self.play_key_sound(); }

        // Recorder
        if ctx.input(|i| i.key_pressed(egui::Key::F9)) { self.ui.show_recorder = !self.ui.show_recorder; self.play_key_sound(); }

        // Mark done
        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::M)) {
            let idx = { let tab = self.cur(); tab.filtered.get(tab.selected).copied() };
            if let Some(idx) = idx {
                self.save_undo_state();
                let tab = self.cur_mut();
                if let Some(entry) = tab.entries.get_mut(idx) { entry.done = !entry.done; tab.dirty = true; }
                self.play_key_sound();
            }
        }

        // Delete selection
        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::D)) && ctx.memory(|m| m.focused().is_none()) {
            let to_del = {
                let tab = self.cur();
                let mut td: Vec<usize> = tab.multi_selection.iter()
                    .filter_map(|&fi| tab.filtered.get(fi).copied())
                    .collect();
                td.sort_by(|a, b| b.cmp(a));
                td
            };
            if !to_del.is_empty() {
                self.save_undo_state();
                let tab = self.cur_mut();
                for idx in to_del { tab.entries.remove(idx); }
                tab.dirty = true;
                tab.multi_selection.clear();
                let sel = tab.selected;
                self.rebuild_filter();
                let f_len = self.cur().filtered.len();
                self.select_multi(sel.min(f_len.saturating_sub(1)), false, false);
                self.play_key_sound();
            }
        }

        // Duplicate entry
        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::I)) && ctx.memory(|m| m.focused().is_none()) {
            let idx = { let tab = self.cur(); tab.filtered.get(tab.selected).copied() };
            if let Some(idx) = idx {
                self.save_undo_state();
                let (dup, fi) = {
                    let tab = self.cur_mut();
                    let mut d = tab.entries[idx].clone();
                    d.alias = format!("{}_copy", d.alias);
                    (d, tab.selected)
                };
                let tab = self.cur_mut();
                tab.entries.insert(idx + 1, dup);
                tab.dirty = true;
                self.rebuild_filter();
                self.select_multi(fi + 1, false, false);
                self.play_key_sound();
            }
        }

        // Copy parameters
        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::C)) && ctx.memory(|m| m.focused().is_none()) {
            let tab = self.cur();
            if let Some(entry) = tab.filtered.get(tab.selected).copied().and_then(|idx| tab.entries.get(idx)) {
                let csv = format!("{},{},{},{},{}", entry.offset, entry.consonant, entry.cutoff, entry.preutter, entry.overlap);
                ctx.copy_text(csv);
                self.play_key_sound();
            }
        }

        // Open folder in explorer
        if ctrl && ctx.input(|i| i.key_pressed(egui::Key::P)) {
            #[allow(unused_variables)]
            if let Some(ref d) = self.cur().oto_dir {
                #[cfg(target_os = "windows")] let _ = std::process::Command::new("explorer").arg(d).spawn();
                #[cfg(target_os = "macos")] let _ = std::process::Command::new("open").arg(d).spawn();
                #[cfg(target_os = "linux")] let _ = std::process::Command::new("xdg-open").arg(d).spawn();
                self.play_key_sound();
            }
        }

        // Arrow / Tab navigation
        let up = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
        let down = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));
        let left = ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft));
        let right = ctx.input(|i| i.key_pressed(egui::Key::ArrowRight));
        let tab_key = ctx.input(|i| i.key_pressed(egui::Key::Tab));
        let shift_mod = ctx.input(|i| i.modifiers.shift);

        let (sel, f_len, col) = {
            let tab = self.cur();
            (tab.selected, tab.filtered.len(), tab.focus_col)
        };

        if f_len > 0 {
            if up && sel > 0 { self.select_multi(sel - 1, false, shift_mod); self.play_key_sound(); }
            if down && sel + 1 < f_len { self.select_multi(sel + 1, false, shift_mod); self.play_key_sound(); }

            let mut new_col = col;
            let mut trigger_nav = false;

            if right || (tab_key && !shift_mod) {
                if new_col < 8 { new_col += 1; trigger_nav = true; }
                else if sel + 1 < f_len { self.select_multi(sel + 1, false, false); new_col = 0; trigger_nav = true; }
            }
            if left || (tab_key && shift_mod) {
                if new_col > 0 { new_col -= 1; trigger_nav = true; }
                else if sel > 0 { self.select_multi(sel - 1, false, false); new_col = 8; trigger_nav = true; }
            }

            if trigger_nav {
                self.cur_mut().focus_col = new_col;
                let id = egui::Id::new(("cell", self.cur().selected, new_col));
                ctx.memory_mut(|m| m.request_focus(id));
                self.play_key_sound();
            } else if up || down {
                let id = egui::Id::new(("cell", self.cur().selected, col));
                ctx.memory_mut(|m| m.request_focus(id));
            }
        }

        // Enter → Clique
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.play_ui_sound("enter");
        }

        // Space → play
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) && ctx.memory(|m| m.focused().is_none()) {
            if ctrl && shift { self.resample_current(); }
            else if !ctrl { self.play_current_segment(shift); }
        }

        // Ctrl+R → focus alias field
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::R)) {
            let id = egui::Id::new(("cell", self.cur().selected, 2));
            ctx.memory_mut(|m| m.request_focus(id));
            self.cur_mut().focus_col = 2;
        }
    }
}
