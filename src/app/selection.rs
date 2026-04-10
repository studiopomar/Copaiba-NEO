use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn save_undo_state(&mut self) {
        let tab = self.cur_mut();
        if tab.undo_stack.last() != Some(&tab.entries) {
            tab.undo_stack.push(tab.entries.clone());
            tab.redo_stack.clear();
            if tab.undo_stack.len() > 50 { tab.undo_stack.remove(0); }
        }
    }

    pub fn undo(&mut self, ctx: &egui::Context) {
        {
            let tab = self.cur_mut();
            if let Some(prev) = tab.undo_stack.pop() {
                tab.redo_stack.push(tab.entries.clone());
                tab.entries = prev;
                tab.dirty = true;
                ctx.memory_mut(|m| {
                    if let Some(id) = m.focused() { m.surrender_focus(id); }
                });
                tab.wave_view.drag_target = crate::waveform::DragTarget::None;
                tab.wave_view.scroll_accum = 0.0;
            } else {
                return;
            }
        }
        self.rebuild_filter();
        self.ensure_wav_loaded();
        self.cur_mut().wave_view.spec_cache = crate::waveform::SpecCache::default();
    }

    pub fn redo(&mut self, ctx: &egui::Context) {
        {
            let tab = self.cur_mut();
            if let Some(next) = tab.redo_stack.pop() {
                tab.undo_stack.push(tab.entries.clone());
                tab.entries = next;
                tab.dirty = true;
                ctx.memory_mut(|m| {
                    if let Some(id) = m.focused() { m.surrender_focus(id); }
                });
                tab.wave_view.drag_target = crate::waveform::DragTarget::None;
            } else {
                return;
            }
        }
        self.rebuild_filter();
        self.ensure_wav_loaded();
        self.cur_mut().wave_view.spec_cache = crate::waveform::SpecCache::default();
    }

    pub fn rebuild_filter(&mut self) {
        let tab = self.cur_mut();
        let q = tab.filter.to_lowercase();
        tab.filtered = tab.entries.iter().enumerate()
            .filter(|(_, e)| q.is_empty() || e.alias.to_lowercase().contains(&q) || e.filename.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect();
        tab.selected = tab.selected.min(tab.filtered.len().saturating_sub(1));
    }

    pub fn select_raw_row(&mut self, raw_idx: usize) {
        let fi = {
            let tab = self.cur();
            tab.filtered.iter().position(|&i| i == raw_idx)
        };
        if let Some(fi) = fi {
            self.select_multi(fi, false, false);
        }
    }

    pub fn select_multi(&mut self, fi: usize, ctrl: bool, shift: bool) {
        {
            let prev_selected = self.cur().selected;
            let pivot_opt = self.shift_pivot;
            let tab = self.cur_mut();

            if shift {
                let pivot = pivot_opt.unwrap_or(prev_selected);
                let start = pivot.min(fi);
                let end = pivot.max(fi);
                tab.multi_selection.clear();
                for i in start..=end { tab.multi_selection.insert(i); }
            } else if ctrl {
                if tab.multi_selection.contains(&fi) {
                    tab.multi_selection.remove(&fi);
                } else {
                    tab.multi_selection.insert(fi);
                }
                self.shift_pivot = Some(fi);
            } else {
                tab.multi_selection.clear();
                tab.multi_selection.insert(fi);
                self.shift_pivot = Some(fi);
            }
        }
        self.select_filtered(fi);
    }

    pub fn select_filtered(&mut self, fi: usize) {
        let (prev_fname, idx_opt) = {
            let tab = self.cur_mut();
            if fi != tab.selected {
                if let Some(&curr_idx) = tab.filtered.get(tab.selected) {
                    if let Some(entry) = tab.entries.get_mut(curr_idx) {
                        if !entry.done {
                            entry.done = true;
                            tab.dirty = true;
                        }
                    }
                }
            }
            let fname_opt: Option<String> = tab.filtered.get(tab.selected).and_then(|&i| tab.entries.get(i)).map(|e| e.filename.clone());
            tab.selected = fi;
            (fname_opt, tab.filtered.get(fi).copied())
        };

        self.ensure_wav_loaded();

        if let Some(idx) = idx_opt {
            let (fname, off, dir_opt) = {
                let tab = self.cur();
                (tab.entries[idx].filename.clone(), tab.entries[idx].offset, tab.oto_dir.clone())
            };

            let full_path_key = dir_opt.as_ref().map(|d| d.join(&fname).to_string_lossy().to_string()).unwrap_or_else(|| fname.clone());

            let mut new_wav = false;
            {
                let tab = self.cur_mut();
                if prev_fname.as_deref() != Some(fname.as_str()) {
                    // WAV file changed: clear ALL caches unconditionally.
                    tab.wave_view.spec_cache    = crate::waveform::SpecCache::default();
                    tab.wave_view.wave_cache    = crate::waveform::WaveCache::default();
                    tab.wave_view.minimap_cache = crate::waveform::MinimapCache::default();
                    new_wav = true;
                } else {
                    // Same WAV file, but viewport will move to new entry's offset.
                    // The spec and wave caches embed their rendered view_start/range,
                    // so they must be invalidated here or they'll show the old
                    // position until a zoom event triggers a rebuild.
                    tab.wave_view.spec_cache = crate::waveform::SpecCache::default();
                    tab.wave_view.wave_cache = crate::waveform::WaveCache::default();
                }
            }

            let wav_duration = self.wav_cache.get(&full_path_key).map(|w| w.duration_ms);
            if let Some(dur) = wav_duration {
                let persistent = self.visual.persistent_zoom;
                let persistent_y = self.visual.persistent_y_zoom;
                let tab = self.cur_mut();
                if new_wav && !persistent {
                    tab.wave_view.reset_to(dur, persistent_y);
                }
                
                // Centering logic (Request 7 fix for cache hits)
                tab.wave_view.target_view_start_ms = (off - tab.wave_view.target_view_range_ms * 0.3)
                    .clamp(0.0, (dur - tab.wave_view.target_view_range_ms).max(0.0));
                
                // Snap view immediately for navigation if not animating yet
                tab.wave_view.view_start_ms = tab.wave_view.target_view_start_ms;
            }
        }
    }
}
