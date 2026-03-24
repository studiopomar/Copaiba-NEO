use egui::RichText;
use egui_plot::{Plot, Line, PlotPoints};
use egui::Stroke;
use egui_i18n::tr;
use crate::plugins;
use crate::spectrogram::ColormapKind;
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn show_modals(&mut self, ctx: &egui::Context) {
        self.modal_exit_dialog(ctx);
        self.modal_preset_editor(ctx);
        self.modal_settings(ctx);
        self.modal_help(ctx);
        self.modal_batch_rename(ctx);
        self.modal_batch_edit(ctx);
        self.modal_alias_sorter(ctx);
        self.modal_consistency_checker(ctx);
        self.modal_duplicate_detector(ctx);
        self.modal_pitch_analyzer(ctx);
        self.modal_recorder(ctx);
    }

    fn modal_exit_dialog(&mut self, ctx: &egui::Context) {
        if !self.ui.show_exit_dialog { return; }
        egui::Window::new(tr!("modal.exit.window.name"))
            .id(egui::Id::new("exit_dialog"))
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(tr!("modal.exit.label.unsaved_changes"));
                ui.add_space(8.0);
                egui::ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
                    let mut has_diff = false;
                    let tab = self.cur();
                    for (i, entry) in tab.entries.iter().enumerate() {
                        if let Some(orig) = tab.original_entries.get(i) {
                            if entry != orig {
                                has_diff = true;
                                let mut changes = Vec::new();
                                if orig.alias != entry.alias { changes.push(format!("Alias '{}' -> '{}'", orig.alias, entry.alias)); }
                                if (orig.offset - entry.offset).abs() > 0.001 { changes.push(format!("Offset {} -> {}", orig.offset, entry.offset)); }
                                if (orig.overlap - entry.overlap).abs() > 0.001 { changes.push(format!("Overlap {} -> {}", orig.overlap, entry.overlap)); }
                                if (orig.preutter - entry.preutter).abs() > 0.001 { changes.push(format!("Preutter {} -> {}", orig.preutter, entry.preutter)); }
                                if (orig.consonant - entry.consonant).abs() > 0.001 { changes.push(format!("Consonant {} -> {}", orig.consonant, entry.consonant)); }
                                if (orig.cutoff - entry.cutoff).abs() > 0.001 { changes.push(format!("Cutoff {} -> {}", orig.cutoff, entry.cutoff)); }
                                ui.label(RichText::new(format!("[{}] {}: {}", orig.filename, orig.alias, changes.join(", "))).color(egui::Color32::from_rgb(220, 200, 100)));
                            }
                        }
                    }
                    if !has_diff { ui.label(tr!("modal.exit.label.unchanged_but_marked")); }
                });
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.button(tr!("btn.save_exit")).clicked() {
                        self.save_oto();
                        if !self.cur().dirty { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                    }
                    if ui.button(tr!("btn.not_save_exit")).clicked() {
                        self.cur_mut().dirty = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button(tr!("btn.cancel")).clicked() { self.ui.show_exit_dialog = false; }
                });
            });
    }

    fn modal_preset_editor(&mut self, ctx: &egui::Context) {
        if !self.ui.show_preset_editor { return; }
        let mut close = false;
        egui::Window::new(tr!("modal.preset.window.name"))
            .id(egui::Id::new("preset_editor"))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                egui::Grid::new("preset_grid").striped(true).show(ui, |ui| {
                    ui.heading(tr!("modal.preset.grid.shortcut")); ui.heading(tr!("modal.preset.grid.name")); ui.heading(tr!("modal.preset.grid.consonant"));
                    ui.heading(tr!("modal.preset.grid.cutoff")); ui.heading(tr!("modal.preset.grid.preutter")); ui.heading(tr!("modal.preset.grid.overlap"));
                    ui.end_row();
                    for (i, preset) in self.presets.iter_mut().enumerate() {
                        ui.label(format!("Ctrl+{}", i + 1));
                        ui.add(egui::TextEdit::singleline(&mut preset.name).desired_width(50.0));
                        ui.add(egui::DragValue::new(&mut preset.consonant).speed(1.0));
                        ui.add(egui::DragValue::new(&mut preset.cutoff).speed(1.0));
                        ui.add(egui::DragValue::new(&mut preset.preutter).speed(1.0));
                        ui.add(egui::DragValue::new(&mut preset.overlap).speed(1.0));
                        ui.end_row();
                    }
                });
                ui.add_space(8.0);
                if ui.button(tr!("btn.close")).clicked() { close = true; }
            });
        if close { self.ui.show_preset_editor = false; }
    }

    fn modal_settings(&mut self, ctx: &egui::Context) {
        if !self.ui.show_settings { return; }
        let mut open = true;
        egui::Window::new(format!("⚙ {}", tr!("modal.settings.window.name")))
            .id(egui::Id::new("settings"))
            .open(&mut open)
            .default_size([400.0, 600.0])
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading(format!("🌍 {}", tr!("modal.settings.general.heading")));
                    ui.horizontal(|ui| {
                        ui.label(tr!("modal.settings.general.label.language"));
                        egui::ComboBox::from_id_salt("language_selector")
                            .selected_text(&self.config.language)
                            .show_ui(ui, |ui| {
                                if ui.selectable_value(&mut self.config.language, "en-US".to_string(), "en-US").clicked() {
                                    egui_i18n::set_language("en-US");
                                }
                                if ui.selectable_value(&mut self.config.language, "pt-BR".to_string(), "pt-BR").clicked() {
                                    egui_i18n::set_language("pt-BR");
                                }
                            });
                    });
                    ui.separator();

                    ui.heading(format!("🎬 {}", tr!("modal.settings.waveform.heading")));
                    ui.checkbox(&mut self.visual.show_minimap, tr!("modal.settings.waveform.ckb.show_minimap"));
                    ui.checkbox(&mut self.visual.persistent_zoom, tr!("modal.settings.waveform.ckb.persistent_zoom"));
                    ui.horizontal(|ui| {
                        ui.label(tr!("modal.settings.waveform.label.color"));
                        if ui.color_edit_button_srgba(&mut self.visual.wave.top_color).changed() { self.clear_wave_cache(); }
                        ui.label(tr!("modal.settings.waveform.label.positive"));
                        if ui.color_edit_button_srgba(&mut self.visual.wave.bot_color).changed() { self.clear_wave_cache(); }
                        ui.label(tr!("modal.settings.waveform.label.negative"));
                    });
                    ui.horizontal(|ui| {
                        ui.label(tr!("modal.settings.waveform.label.spline"));
                        if ui.color_edit_button_srgba(&mut self.visual.wave.line_color).changed() { self.clear_wave_cache(); }
                        ui.add(egui::Slider::new(&mut self.visual.wave.thickness, 0.5..=5.0).step_by(0.1));
                    });
                    ui.separator();

                    ui.heading(format!("🌐 {}", tr!("modal.settings.encoding.heading")));
                    egui::ComboBox::from_label(tr!("modal.settings.encoding.label"))
                        .selected_text(format!("{:?}", self.encoding))
                        .show_ui(ui, |ui| {
                            use crate::oto::OtoEncoding;
                            if ui.selectable_value(&mut self.encoding, OtoEncoding::Utf8, "UTF-8").clicked() { self.load_character_metadata(self.current_tab); }
                            if ui.selectable_value(&mut self.encoding, OtoEncoding::ShiftJis, "Shift-JIS (Japonês)").clicked() { self.load_character_metadata(self.current_tab); }
                            if ui.selectable_value(&mut self.encoding, OtoEncoding::Gbk, "GBK (Chinês)").clicked() { self.load_character_metadata(self.current_tab); }
                        });
                    ui.separator();

                    ui.heading(format!("🎨 {}", tr!("modal.settings.spectrogram.heading")));
                    ui.checkbox(&mut self.visual.show_spectrogram, tr!("modal.settings.spectrogram.ckb.hd"));
                    ui.add_space(4.0);

                    let mut fft_changed = false;
                    let mut render_changed = false;

                    ui.horizontal(|ui| {
                        ui.label(tr!("modal.settings.spectrogram.label.fft_size"));
                        for &sz in &[512usize, 1024, 2048, 4096, 8192] {
                            if ui.selectable_label(self.visual.spec.fft_size == sz, sz.to_string()).clicked() {
                                self.visual.spec.fft_size = sz; fft_changed = true;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(tr!("modal.settings.spectrogram.label.hop"));
                        for &sz in &[64usize, 128, 256, 512] {
                            if ui.selectable_label(self.visual.spec.hop_size == sz, sz.to_string()).clicked() {
                                self.visual.spec.hop_size = sz; fft_changed = true;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(tr!("modal.settings.spectrogram.label.freq_min"));
                        if ui.add(egui::DragValue::new(&mut self.visual.spec.min_freq).speed(5.0).range(1.0..=5000.0).suffix(" Hz")).changed() { render_changed = true; }
                    });
                    ui.horizontal(|ui| {
                        ui.label(tr!("modal.settings.spectrogram.label.freq_max"));
                        if ui.add(egui::DragValue::new(&mut self.visual.spec.max_freq).speed(100.0).range(0.0..=24000.0).suffix(" Hz")).changed() { render_changed = true; }
                    });
                    ui.horizontal(|ui| {
                        ui.label(tr!("modal.settings.spectrogram.label.noise"));
                        if ui.add(egui::Slider::new(&mut self.visual.spec.min_db, -120.0_f32..=-20.0).suffix(" dB")).changed() { render_changed = true; }
                    });
                    ui.horizontal(|ui| {
                        ui.label(tr!("modal.settings.spectrogram.label.gama"));
                        if ui.add(egui::Slider::new(&mut self.visual.spec.gamma, 0.1_f32..=1.5).step_by(0.05)).changed() { render_changed = true; }
                    });
                    ui.horizontal(|ui| {
                        ui.label(tr!("modal.settings.spectrofram.label.palette"));
                        for (kind, label) in &[
                            (ColormapKind::Fire, "🔥 Fire"),
                            (ColormapKind::Inferno, "🌋 Inferno"),
                            (ColormapKind::Viridis, "🌿 Viridis"),
                            (ColormapKind::Grayscale, "⬜ Gray"),
                        ] {
                            if ui.selectable_label(self.visual.spec.colormap == *kind, *label).clicked() {
                                self.visual.spec.colormap = kind.clone(); render_changed = true;
                            }
                        }
                    });
                    if ui.checkbox(&mut self.visual.spec.adaptive_norm, tr!("modal.settings.spectrogram.ckb.adaptative_norm")).changed() { render_changed = true; }

                    if fft_changed {
                        self.spec_data_cache.clear();
                        for t in &mut self.tabs {
                            t.wave_view.spec_cache = crate::waveform::SpecCache::default();
                        }
                        self.ensure_wav_loaded();
                    } else if render_changed {
                        for t in &mut self.tabs {
                            t.wave_view.spec_cache = crate::waveform::SpecCache::default();
                        }
                    }

                    ui.separator();
                    if ui.button(tr!("btn.close")).clicked() { self.save_prefs(); self.ui.show_settings = false; }
                });
            });
        if !open { self.save_prefs(); self.ui.show_settings = false; }
    }

    fn modal_help(&mut self, ctx: &egui::Context) {
        if !self.ui.show_help { return; }
        let mut open = true;
        egui::Window::new(tr!("modal.shortcuts.window.name"))
            .id(egui::Id::new("help"))
            .open(&mut open)
            .show(ctx, |ui| {
                egui::Grid::new("shorts").striped(true).show(ui, |ui| {
                    ui.strong(tr!("modal.shortcuts.label.general")); ui.label(""); ui.end_row();
                    ui.label("Ctrl + O"); ui.label(tr!("modal.shortcuts.label.open_oto")); ui.end_row();
                    ui.label("Ctrl + S"); ui.label(tr!("modal.shortcuts.label.save")); ui.end_row();
                    ui.label("Ctrl + Z"); ui.label(tr!("modal.shortcuts.label.undo")); ui.end_row();
                    ui.label("Ctrl + Y"); ui.label(tr!("modal.shortcuts.label.redo")); ui.end_row();
                    ui.label("Ctrl + ,"); ui.label(tr!("modal.shortcuts.label.config")); ui.end_row();
                    ui.label("F1"); ui.label(tr!("modal.shortcuts.label.shortcuts")); ui.end_row();

                    ui.strong(tr!("modal.shortcuts.label.table")); ui.label(""); ui.end_row();
                    ui.label("Ctrl + A"); ui.label(tr!("modal.shortcuts.label.select_all")); ui.end_row();
                    ui.label("Ctrl + D"); ui.label(tr!("modal.shortcuts.label.del_selection")); ui.end_row();
                    ui.label("Ctrl + I"); ui.label(tr!("modal.shortcuts.label.duplicate")); ui.end_row();
                    ui.label("Ctrl + R"); ui.label(tr!("modal.shortcuts.label.rename")); ui.end_row();

                    ui.strong(tr!("modal.shortcuts.label.audio")); ui.label(""); ui.end_row();
                    ui.label("Espaço"); ui.label(tr!("modal.shortcuts.label.play_segment")); ui.end_row();
                    ui.label("Shift + Espaço"); ui.label(tr!("modal.shortcuts.label.play_audio")); ui.end_row();
                    ui.label("Ctrl+Shift+Esp"); ui.label(tr!("modal.shortcuts.label.synth_test")); ui.end_row();
                });
            });
        if !open { self.ui.show_help = false; }
    }

    fn modal_batch_rename(&mut self, ctx: &egui::Context) {
        if !self.ui.show_batch_rename { return; }
        let mut open = true;
        egui::Window::new(format!("📝 {}", tr!("modal.batch_rename.window.name")))
            .id(egui::Id::new("batch_rename"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(tr!("modal.batch_rename.label.info"));
                ui.separator();
                ui.label(tr!("modal.batch_rename.label.subst"));
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.rename_find);
                    ui.label("→");
                    ui.text_edit_singleline(&mut self.rename_replace);
                });
                ui.add_space(8.0);
                ui.label(tr!("modal.batch_rename.label.set"));
                ui.horizontal(|ui| { ui.label(tr!("modal.batch_rename.label.prefix")); ui.text_edit_singleline(&mut self.rename_prefix); });
                ui.horizontal(|ui| { ui.label(tr!("modal.batch_rename.label.sufffix")); ui.text_edit_singleline(&mut self.rename_suffix); });
                ui.add_space(8.0);
                if ui.button(tr!("btn.exe")).clicked() {
                    let filtered = self.cur().filtered.clone();
                    let find = self.rename_find.clone();
                    let repl = self.rename_replace.clone();
                    let pref = self.rename_prefix.clone();
                    let suff = self.rename_suffix.clone();
                    self.save_undo_state();
                    let tab = self.cur_mut();
                    for &idx in &filtered {
                        if let Some(entry) = tab.entries.get_mut(idx) {
                            let mut new_name = entry.alias.clone();
                            if !find.is_empty() { new_name = new_name.replace(&find, &repl); }
                            new_name = format!("{}{}{}", pref, new_name, suff);
                            entry.alias = new_name;
                        }
                    }
                    tab.dirty = true;
                    self.ui.show_batch_rename = false;
                    self.rename_find.clear(); self.rename_replace.clear();
                    self.rename_prefix.clear(); self.rename_suffix.clear();
                    self.rebuild_filter();
                }
            });
        if !open { self.ui.show_batch_rename = false; }
    }

    fn modal_batch_edit(&mut self, ctx: &egui::Context) {
        if !self.ui.show_batch_edit { return; }
        let mut open = true;
        egui::Window::new(format!("📊 {}", tr!("modal.batch_edit.window.name")))
            .id(egui::Id::new("batch_edit"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(tr!("modal.batch_edit.label.info"));
                ui.separator();
                let labels = ["Offset", "Preutterance", "Overlap", "Consonant", "Cutoff"];
                for i in 0..5 {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.batch_edit_enabled[i], labels[i]);
                        if self.batch_edit_enabled[i] {
                            ui.add(egui::DragValue::new(&mut self.batch_edit_values[i]).speed(1.0).suffix(" ms"));
                        }
                    });
                }
                if ui.button(tr!("btn.apply")).clicked() {
                    let filtered = self.cur().filtered.clone();
                    let enabled = self.batch_edit_enabled;
                    let values = self.batch_edit_values;
                    self.save_undo_state();
                    let tab = self.cur_mut();
                    for &idx in &filtered {
                        if let Some(entry) = tab.entries.get_mut(idx) {
                            if enabled[0] { entry.offset = values[0]; }
                            if enabled[1] { entry.preutter = values[1]; }
                            if enabled[2] { entry.overlap = values[2]; }
                            if enabled[3] { entry.consonant = values[3]; }
                            if enabled[4] { entry.cutoff = values[4]; }
                        }
                    }
                    tab.dirty = true;
                    self.ui.show_batch_edit = false;
                    self.rebuild_filter();
                }
            });
        if !open { self.ui.show_batch_edit = false; }
    }

    fn modal_alias_sorter(&mut self, ctx: &egui::Context) {
        if !self.ui.show_alias_sorter { return; }
        let mut open = true;
        egui::Window::new(format!("↕ {}", tr!("modal.org_alias.window.name")))
            .id(egui::Id::new("alias_sorter"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(tr!("modal.org_alias.label.info"));
                ui.separator();
                egui::ComboBox::from_label(tr!("modal.org_alias.label.mode"))
                    .selected_text(format!("{:?}", self.sort_settings.mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.sort_settings.mode, plugins::SortMode::Alpha, tr!("modal.org_alias.label.alphabetic"));
                        ui.selectable_value(&mut self.sort_settings.mode, plugins::SortMode::FileName, tr!("modal.org_alias.label.file"));
                        ui.selectable_value(&mut self.sort_settings.mode, plugins::SortMode::Offset, tr!("modal.org_alias.label.offset"));
                    });
                ui.checkbox(&mut self.sort_settings.group_by_file, tr!("modal.org_alias.ckb.group_by_file"));
                if ui.button(tr!("btn.apply")).clicked() {
                    let settings = self.sort_settings.clone();
                    self.save_undo_state();
                    let tab = self.cur_mut();
                    plugins::sort_entries(&mut tab.entries, &settings);
                    tab.dirty = true;
                    self.ui.show_alias_sorter = false;
                    self.rebuild_filter();
                }
            });
        if !open { self.ui.show_alias_sorter = false; }
    }

    fn modal_consistency_checker(&mut self, ctx: &egui::Context) {
        if !self.ui.show_consistency_checker { return; }
        let mut open = true;
        egui::Window::new(format!("🔍 {}", tr!("modal.consistency_checker.window.name")))
            .id(egui::Id::new("consistency_checker"))
            .open(&mut open)
            .default_size([700.0, 500.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(format!("🚀 {}", tr!("modal.consistency_checker.btn.scan"))).clicked() {
                        let tab = self.cur();
                        self.ui.consistency_issues = plugins::check_consistency(&tab.entries, tab.oto_dir.as_deref());
                    }
                    ui.label(format!("{} {}", tr!("modal.consistency_checker.label.issues"), self.ui.consistency_issues.len()));
                });
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("consistency_grid").striped(true).show(ui, |ui| {
                        ui.label(tr!("modal.consistency_checker.label.line")); ui.label(tr!("modal.consistency_checker.label.alias")); ui.label(tr!("modal.consistency_checker.label.message")); ui.end_row();
                        let mut jump_to = None;
                        for issue in &self.ui.consistency_issues {
                            ui.label((issue.row + 1).to_string());
                            ui.label(&issue.alias);
                            if ui.link(&issue.message).clicked() { jump_to = Some(issue.row); }
                            ui.end_row();
                        }
                        if let Some(row) = jump_to { self.select_raw_row(row); }
                    });
                });
            });
        if !open { self.ui.show_consistency_checker = false; }
    }

    fn modal_duplicate_detector(&mut self, ctx: &egui::Context) {
        let mut show_dups = self.ui.show_duplicate_detector;
        if show_dups {
            egui::Window::new(format!("✂ {}", tr!("modal.duplicate_detector.window.name")))
                .id(egui::Id::new("duplicate_detector"))
                .open(&mut show_dups)
                .default_size([700.0, 500.0])
                .show(ctx, |ui| {
                    ui.label(tr!("modal.duplicate_detector.label.info"));
                    ui.separator();
                    if ui.button(format!("🔍 {}", tr!("modal.duplicate_detector.btn.scan"))).clicked() {
                        let tab = self.cur();
                        self.ui.duplicate_results = plugins::detect_duplicates(&tab.entries, true, true, true, false);
                    }
                    ui.add_space(8.0);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("dup_grid").striped(true).show(ui, |ui| {
                            ui.label(tr!("modal.duplicate_detector.label.type")); ui.label(tr!("modal.duplicate_detector.label.alias_a")); ui.label(tr!("modal.duplicate_detector.label.alias_b")); ui.label(tr!("modal.duplicate_detector.label.action")); ui.end_row();
                            let mut delete_row = None;
                            let mut jump_to = None;
                            for dup in &self.ui.duplicate_results {
                                ui.label(match dup.match_type.as_str() {
                                    "exact" => tr!("modal.duplicate_detector.label.exact"), "case" => tr!("modal.duplicate_detector.label.case"), "functional" => tr!("modal.duplicate_detector.label.functional"), _ => tr!("modal.duplicate_detector.label.simmilar"),
                                });
                                ui.label(RichText::new(&dup.alias1).strong());
                                ui.label(RichText::new(&dup.alias2).strong());
                                ui.horizontal(|ui| {
                                    if ui.button(tr!("modal.duplicate_detector.btn.goto_1")).clicked() { jump_to = Some(dup.row1); }
                                    if ui.button(tr!("modal.duplicate_detector.btn.goto_2")).clicked() { jump_to = Some(dup.row2); }
                                    if ui.button(tr!("modal.duplicate_detector.btn.del_2")).clicked() { delete_row = Some(dup.row2); }
                                });
                                ui.end_row();
                            }
                            if let Some(row) = jump_to { self.select_raw_row(row); }
                            if let Some(row) = delete_row {
                                self.save_undo_state();
                                let tab = self.cur_mut();
                                tab.entries.remove(row);
                                tab.dirty = true;
                                self.rebuild_filter();
                                self.ui.duplicate_results.clear();
                            }
                        });
                    });
                });
        }
        self.ui.show_duplicate_detector = show_dups;
    }

    fn modal_pitch_analyzer(&mut self, ctx: &egui::Context) {
        let mut show_pitch = self.ui.show_pitch_analyzer;
        if show_pitch {
            egui::Window::new(format!("🎵 {}", tr!("modal.pitch_analyzer.window.name")))
                .id(egui::Id::new("pitch_analyzer"))
                .open(&mut show_pitch)
                .default_size([700.0, 450.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(tr!("modal.pitch_analyzer.label.window"));
                        ui.add(egui::Slider::new(&mut self.pitch_window_ms, 5.0..=40.0));
                        if ui.button(format!("🎵 {}", tr!("modal.pitch_analyzer.btn.analyze"))).clicked() {
                            let tab = self.cur();
                            if let Some(&idx) = tab.filtered.get(tab.selected) {
                                let entry = &tab.entries[idx];
                                if let Some(wav) = self.wav_cache.get(&entry.filename) {
                                    let dur_ms = wav.samples.len() as f64 * 1000.0 / wav.sample_rate as f64;
                                    let start_ms = entry.offset;
                                    let end_ms = if entry.cutoff < 0.0 { (entry.offset - entry.cutoff).min(dur_ms) } else { (dur_ms - entry.cutoff).max(0.0) };
                                    let start_idx = ((start_ms * wav.sample_rate as f64) / 1000.0) as usize;
                                    let end_idx = ((end_ms * wav.sample_rate as f64) / 1000.0) as usize;
                                    if start_idx < wav.samples.len() && end_idx <= wav.samples.len() && start_idx < end_idx {
                                        let (t, v) = plugins::analyze_pitch(&wav.samples[start_idx..end_idx], wav.sample_rate, self.pitch_window_ms);
                                        self.pitch_times = t.into_iter().map(|t| t + start_ms).collect();
                                        self.pitch_values = v;
                                    }
                                }
                            }
                        }
                        if !self.pitch_values.is_empty() {
                            let valid: Vec<f64> = self.pitch_values.iter().filter(|&&v| v > 0.0).copied().collect();
                            if !valid.is_empty() {
                                let avg = valid.iter().sum::<f64>() / valid.len() as f64;
                                ui.label(RichText::new(format!("{} {:.1} Hz ({})", tr!("modal.pitch_analyzer.label.avg"), avg, plugins::freq_to_note(avg))).color(egui::Color32::from_rgb(100, 255, 100)).strong());
                            }
                        }
                    });
                    ui.separator();
                    let points: Vec<[f64; 2]> = self.pitch_times.iter()
                        .zip(self.pitch_values.iter())
                        .filter(|(_, &v)| v > 0.0)
                        .map(|(&t, &v)| [t, v])
                        .collect();
                    Plot::new("pitch_plot")
                        .view_aspect(2.0)
                        .x_axis_label("ms")
                        .y_axis_label("Hz")
                        .include_y(50.0)
                        .include_y(600.0)
                        .show(ui, |plot_ui| {
                            for octave in 2..=5 {
                                for note_h in 0..12 {
                                    let freq = 440.0 * 2.0_f64.powf((octave as f64 - 4.0) + (note_h as f64 - 9.0) / 12.0);
                                    let color = if note_h == 0 { egui::Color32::from_gray(60) } else { egui::Color32::from_gray(30) };
                                    plot_ui.hline(egui_plot::HLine::new(freq).stroke(Stroke::new(1.0, color)));
                                }
                            }
                            plot_ui.line(Line::new(PlotPoints::from(points)).color(egui::Color32::from_rgb(100, 255, 100)).width(2.0));
                        });
                });
        }
        self.ui.show_pitch_analyzer = show_pitch;
    }
    pub fn clear_wave_cache(&mut self) {
        for tab in &mut self.tabs {
            tab.wave_view.wave_cache = crate::waveform::WaveCache::default();
            tab.wave_view.minimap_cache = crate::waveform::MinimapCache::default();
        }
    }
}
