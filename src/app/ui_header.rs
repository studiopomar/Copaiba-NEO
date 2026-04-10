use egui::{Color32, RichText, Vec2, ComboBox};
use egui_i18n::tr;
use std::path::PathBuf;
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn show_voicebank_header(&mut self, ctx: &egui::Context) {
        let tab_idx = self.current_tab;
        
        // Ensure character texture is loaded
        {
            let tab = &mut self.tabs[tab_idx];
            if tab.character_texture.is_none() {
                if let Some(path) = &tab.character_image_path {
                    if let Ok(data) = std::fs::read(path) {
                        if let Ok(image) = image::load_from_memory(&data) {
                            let size = [image.width() as usize, image.height() as usize];
                            let image_buffer = image.to_rgba8();
                            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                size,
                                image_buffer.as_flat_samples().as_slice(),
                            );
                            tab.character_texture = Some(ctx.load_texture("char_img", color_image, Default::default()));
                        }
                    }
                }
            }
        }

        egui::TopBottomPanel::top("voicebank_header").resizable(false).show(ctx, |ui| {
            let (char_tex, char_name, oto_dir, license, readme_path, root_path) = {
                let tab = &self.tabs[tab_idx];
                (tab.character_texture.as_ref().map(|t| t.id()), tab.character_name.clone(), tab.oto_dir.clone(), tab.license_text.clone(), tab.readme_path.clone(), tab.root_path.clone())
            };

            ui.add_space(2.0);
            ui.vertical(|ui| {
                // Primary Row
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

                    // Close Tab Button (X)
                    if self.tabs.len() > 1 {
                        if ui.add(egui::Button::new(RichText::new("✖").size(12.0).color(Color32::from_rgb(255, 100, 100))).small().fill(Color32::TRANSPARENT)).on_hover_text(tr!("tabs.btn.close_tab")).clicked() {
                            self.tabs.remove(tab_idx);
                            self.current_tab = self.current_tab.min(self.tabs.len() - 1);
                        }
                    }

                    // Character Image (60x60)
                    let (rect, _resp) = ui.allocate_at_least(Vec2::new(60.0, 60.0), egui::Sense::hover());
                    if let Some(tex_id) = char_tex {
                        ui.painter().image(tex_id, rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                    } else {
                        ui.painter().rect_filled(rect, 8.0, Color32::from_rgb(30, 30, 46));
                        ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "👤", egui::FontId::proportional(28.0), Color32::GRAY);
                    }

                    ui.add_space(8.0);

                    // Character Info + Readme Buttons
                    ui.vertical(|ui| {
                        let name = if char_name.is_empty() {
                            oto_dir.as_ref()
                                .and_then(|p: &PathBuf| p.file_name())
                                .map(|s: &std::ffi::OsStr| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Voicebank".to_string())
                        } else { 
                            char_name
                        };
                        ui.label(RichText::new(name).strong().size(14.0));
                        let path_str = oto_dir.as_ref().map(|p: &std::path::PathBuf| p.to_string_lossy().to_string()).unwrap_or_default();
                        ui.label(RichText::new(&path_str).color(ui.visuals().weak_text_color()).size(10.0))
                            .on_hover_text(path_str);

                        ui.horizontal(|ui| {
                            if readme_path.is_some() {
                                if ui.button(RichText::new("📄 Readme").size(9.0)).clicked() {
                                    self.ui.show_readme = true;
                                }
                            } else if root_path.is_some() {
                                if ui.button(RichText::new("➕ Readme").size(9.0)).clicked() {
                                    let root = root_path.as_ref().unwrap();
                                    let new_path = root.join("readme.txt");
                                    if std::fs::write(&new_path, "...\n").is_ok() {
                                        let tab = &mut self.tabs[tab_idx];
                                        tab.readme_path = Some(new_path);
                                        tab.readme_text = "...\n".to_string();
                                        tab.original_readme_text = "...\n".to_string();
                                    }
                                }
                            }
                            if !license.is_empty() {
                                if ui.button(RichText::new("⚖ License").size(9.0).color(Color32::from_rgb(150, 200, 150))).clicked() {
                                    self.ui.show_license = true;
                                }
                            }
                        });
                    });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);

                    // Alias Search / Filter
                    let filter_changed = {
                        let tab = &mut self.tabs[tab_idx];
                        let mut changed = false;
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🔍").size(10.0).color(Color32::from_rgb(180, 180, 200)));
                            let resp = ui.add(egui::TextEdit::singleline(&mut tab.filter)
                                .hint_text(tr!("table.filter.hint"))
                                .desired_width(180.0)
                                .margin(egui::Margin::same(4))
                            );
                            if resp.changed() { changed = true; }
                            
                            if !tab.filter.is_empty() {
                                if ui.button(RichText::new("✖").size(10.0).color(Color32::from_rgb(255, 100, 100))).on_hover_text("Clear filter").clicked() {
                                    tab.filter.clear();
                                    changed = true;
                                }
                            }
                        });
                        changed
                    };
                    if filter_changed {
                        self.rebuild_filter();
                        self.play_key_sound();
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);

                    // Resampler controls
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let has_resampler = self.config.resampler_path.is_some();
                        let test_btn = egui::Button::new(RichText::new(format!("🧪 {}", tr!("header.resampler.test"))).strong().color(Color32::from_rgb(20, 20, 30)))
                            .fill(Color32::from_rgb(137, 180, 250))
                            .min_size(egui::vec2(100.0, 24.0));
                        if ui.add(test_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            if has_resampler { self.resample_current(); }
                        }
                        
                        ui.add(egui::DragValue::new(&mut self.config.test_duration_ms).suffix("ms").range(50.0..=2000.0));
                        ui.separator();

                        ComboBox::from_id_salt("pitch_select")
                            .selected_text(RichText::new(&self.config.test_pitch).color(Color32::from_rgb(137, 180, 250)).strong())
                            .width(60.0)
                            .show_ui(ui, |ui| {
                                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                                    for p in &["C1","C#1","D1","D#1","E1","F1","F#1","G1","G#1","A1","A#1","B1",
                                            "C2","C#2","D2","D#2","E2","F2","F#2","G2","G#2","A2","A#2","B2",
                                            "C3","C#3","D3","D#3","E3","F3","F#3","G3","G#3","A3","A#3","B3",
                                            "C4","C#4","D4","D#4","E4","F4","F#4","G4","G#4","A4","A#4","B4",
                                            "C5","C#5","D5","D#5","E5","F5","F#5","G5","G#5","A5","A#5","B5",
                                            "C6","C#6","D6","D#6","E6","F6","F#6","G6","G#6","A6","A#6","B6",
                                            "C7","C#7","D7","D#7","E7","F7","F#7","G7","G#7","A7","A#7","B7"] {
                                        ui.selectable_value(&mut self.config.test_pitch, p.to_string(), *p);
                                    }
                                });
                            });
                        ui.label(tr!("header.pitch.label"));
                        ui.separator();

                        ui.add(egui::TextEdit::singleline(&mut self.config.test_flags)
                            .hint_text("Flags")
                            .desired_width(60.0)
                            .margin(egui::Margin::symmetric(4, 2))
                        );
                        ui.label("Flags");
                        ui.separator();

                        if let Some(res) = &self.config.resampler_path {
                            ui.label(RichText::new(res.file_name().unwrap_or_default().to_string_lossy()).size(9.0).color(ui.visuals().weak_text_color()));
                        }
                        if ui.add(egui::Button::new(RichText::new(format!("⚙ {}", tr!("header.resampler.select"))).strong())).clicked() {
                            #[cfg(any(windows, target_os = "linux", target_os = "macos"))]
                            if let Some(path) = rfd::FileDialog::new().pick_file() { self.config.resampler_path = Some(path); }
                        }
                    });
                });

                ui.add_space(2.0);
                ui.separator();

                // Secondary Row (Volume, Device) - Balanced
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new("🔊").size(10.0));
                    ui.add(egui::Slider::new(&mut self.config.test_volume, 0.0..=1.0).show_value(false).trailing_fill(true));
                    
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    if ui.add(egui::Button::new(RichText::new("🎧").size(10.0))).on_hover_text("Refresh audio devices").clicked() {
                        self.refresh_audio_devices();
                    }
                    ui.add_space(2.0);
                    ComboBox::from_id_salt("audio_device_select")
                        .selected_text(RichText::new(self.config.audio_device.as_deref().unwrap_or("Default Device")).size(9.0))
                        .width(120.0)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(self.config.audio_device.is_none(), "Default Device").clicked() {
                                self.set_audio_device(None);
                            }
                            for dev in self.ui.available_devices.clone() {
                                if ui.selectable_label(self.config.audio_device.as_ref() == Some(&dev), &dev).clicked() {
                                    self.set_audio_device(Some(dev));
                                }
                            }
                        });
                });
            });
            ui.add_space(2.0);
        });
    }
}
