use egui::RichText;
use egui_i18n::tr;
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn show_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button(tr!("menu.file"), |ui| {
                    if ui
                        .button(format!("🏠 {}\tCtrl+H", tr!("menu.file.home")))
                        .clicked()
                    {
                        self.ui.show_home = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button(format!("📁 {}\tCtrl+O", tr!("menu.file.open.vb")))
                        .clicked()
                    {
                        self.open_voicebank_dir();
                        ui.close_menu();
                    }
                    if ui
                        .button(format!("📄 {}\tCtrl+Shift+O", tr!("menu.file.open.oto")))
                        .clicked()
                    {
                        self.open_oto();
                        ui.close_menu();
                    }
                    ui.separator();
                    let save_enabled = self.cur().dirty || self.cur().oto_path.is_some();
                    if ui
                        .add_enabled(
                            save_enabled,
                            egui::Button::new(format!("💾 {}\tCtrl+S", tr!("menu.file.save"))),
                        )
                        .clicked()
                    {
                        self.save_oto();
                        ui.close_menu();
                    }
                    if ui
                        .button(format!("💾 {}\tCtrl+Shift+S", tr!("menu.file.save_as")))
                        .clicked()
                    {
                        self.save_as();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button(format!("📂 {}\tCtrl+P", tr!("menu.file.open.explorer")))
                        .clicked()
                    {
                        if let Some(ref d) = self.cur().oto_dir {
                            #[cfg(target_os = "windows")]
                            let _ = std::process::Command::new("explorer").arg(d).spawn();
                            #[cfg(target_os = "linux")]
                            let _ = std::process::Command::new("xdg-open").arg(d).spawn();
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(format!("🚪 {}", tr!("menu.file.exit"))).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button(tr!("menu.edit"), |ui| {
                    if ui
                        .button(format!("↩ {}\tCtrl+Z", tr!("menu.edit.undo")))
                        .clicked()
                    {
                        self.undo(ctx);
                        ui.close_menu();
                    }
                    if ui
                        .button(format!("↪ {}\tCtrl+Y", tr!("menu.edit.redo")))
                        .clicked()
                    {
                        self.redo(ctx);
                        ui.close_menu();
                    }
                    ui.separator();

                    let tab = self.cur_mut();
                    let mut snap = tab.wave_view.snap_to_peaks;
                    if ui
                        .checkbox(&mut snap, format!("🪄 {}", tr!("menu.edit.auto_oto")))
                        .on_hover_text(tr!("menu.edit.auto_oto.hover"))
                        .changed()
                    {
                        tab.wave_view.snap_to_peaks = snap;
                    }

                    ui.add_space(8.0);
                    ui.menu_button(format!("📍 {}", tr!("menu.edit.snap_mode")), |ui| {
                        let tab = self.cur_mut();
                        if ui
                            .button(format!("{}\tShift+1", tr!("menu.edit.snap_mode.srp")))
                            .clicked()
                        {
                            tab.wave_view.srp = !tab.wave_view.srp;
                            if tab.wave_view.srp {
                                tab.wave_view.srna = false;
                            }
                            ui.close_menu();
                        }
                        if ui
                            .button(format!("{}\tShift+2", tr!("menu.edit.snap_mode.srna")))
                            .clicked()
                        {
                            tab.wave_view.srna = !tab.wave_view.srna;
                            if tab.wave_view.srna {
                                tab.wave_view.srp = false;
                            }
                            ui.close_menu();
                        }
                    });
                    ui.separator();
                    if ui
                        .button(format!("📝 {}\tCtrl+R", tr!("menu.edit.alias.rename")))
                        .clicked()
                    {
                        self.ui.is_renaming = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!("🗑️ {}\tCtrl+D", tr!("menu.edit.alias.delete")))
                        .clicked()
                    {
                        ui.close_menu();
                    }
                });

                ui.menu_button(tr!("menu.view"), |ui| {
                    ui.checkbox(&mut self.visual.show_spectrogram, tr!("menu.view.spectrogram"));
                    ui.checkbox(&mut self.visual.show_minimap, tr!("menu.view.minimap"));
                    ui.separator();
                    ui.checkbox(
                        &mut self.ui.auto_scroll_to_selected,
                        tr!("menu.view.auto_scroll"),
                    );
                    ui.separator();
                    if ui
                        .button(tr!("menu.view.reset"))
                        .clicked()
                    {
                        let tab = self.cur_mut();
                        tab.wave_view.scroll_accum = 0.0;
                        tab.wave_view.mouse_ms = None;
                        ui.close_menu();
                    }
                });

                ui.menu_button(tr!("menu.play"), |ui| {
                    if ui
                        .button(format!("▶ {}\tSpace", tr!("menu.play.segment")))
                        .clicked()
                    {
                        self.play_current_segment(false);
                        ui.close_menu();
                    }
                    if ui
                        .button(format!("▶ {}\tShift+Space", tr!("menu.play.audio")))
                        .clicked()
                    {
                        self.play_current_segment(true);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button(format!("🧪 {}\tCtrl+Shift+Space", tr!("menu.play.test")))
                        .clicked()
                    {
                        self.resample_current();
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.checkbox(&mut self.config.play_on_select, tr!("menu.play.on_select"));
                });

                ui.menu_button(tr!("menu.config"), |ui| {
                    if ui
                        .button(format!("⚙ {}\tCtrl+,", tr!("menu.config.general")))
                        .clicked()
                    {
                        self.ui.show_settings = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.config.auto_save_enabled, tr!("menu.config.auto_save"));
                        if self.config.auto_save_enabled {
                            ui.add(
                                egui::DragValue::new(&mut self.config.auto_save_interval_mins)
                                    .suffix(tr!("menu.config.auto_save.time_unit"))
                                    .range(1..=60),
                            );
                        }
                    });
                });

                ui.menu_button(tr!("menu.plugins"), |ui| {
                    if ui
                        .button(format!("🔍 {}", tr!("menu.plugins.consistency_checker")))
                        .clicked()
                    {
                        self.ui.show_consistency_checker = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!("✂ {}", tr!("menu.plugins.duplicate_detector")))
                        .clicked()
                    {
                        self.ui.show_duplicate_detector = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!("🎵 {}", tr!("menu.plugins.pitch_analyzer")))
                        .clicked()
                    {
                        self.ui.show_pitch_analyzer = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button(format!("↕ {}", tr!("menu.plugins.alias_sorter")))
                        .clicked()
                    {
                        self.ui.show_alias_sorter = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui
                        .button(format!("📝 {}", tr!("menu.plugins.batch_rename")))
                        .clicked()
                    {
                        self.ui.show_batch_rename = true;
                        ui.close_menu();
                    }
                    if ui
                        .button(format!("📊 {}", tr!("menu.plugins.batch_edit")))
                        .clicked()
                    {
                        self.ui.show_batch_edit = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button(tr!("menu.help"), |ui| {
                    if ui
                        .button(format!("⌨ {}\tF1", tr!("menu.help.shortcuts")))
                        .clicked()
                    {
                        self.ui.show_help = true;
                        ui.close_menu();
                    }
                });

                ui.separator();
                if ui
                    .button(
                        RichText::new(format!("{} (F9)", tr!("menu.others.re_record")))
                            .color(egui::Color32::from_rgb(100, 200, 100))
                            .strong(),
                    )
                    .clicked()
                {
                    self.ui.show_recorder = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("NEO")
                            .color(egui::Color32::from_rgb(140, 100, 200))
                            .strong(),
                    );
                    ui.label(RichText::new("Copaiba").strong());
                });
            });
        });
    }
}
