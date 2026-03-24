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

        egui::TopBottomPanel::top("voicebank_header").show(ctx, |ui| {
            let (char_tex, char_name, oto_dir, readme, license) = {
                let tab = &self.tabs[tab_idx];
                (tab.character_texture.as_ref().map(|t| t.id()), tab.character_name.clone(), tab.oto_dir.clone(), tab.readme_text.clone(), tab.license_text.clone())
            };

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                // Character Image (50x50)
                let (rect, _resp) = ui.allocate_at_least(Vec2::new(50.0, 50.0), egui::Sense::hover());
                if let Some(tex_id) = char_tex {
                    ui.painter().image(tex_id, rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                } else {
                    ui.painter().rect_filled(rect, 6.0, Color32::from_rgb(30, 30, 46));
                    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "👤", egui::FontId::proportional(24.0), Color32::GRAY);
                }

                ui.add_space(8.0);

                ui.vertical(|ui| {
                    let name = if char_name.is_empty() {
                        oto_dir.as_ref()
                            .and_then(|p: &PathBuf| p.file_name())
                            .map(|s: &std::ffi::OsStr| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Voicebank".to_string())
                    } else { 
                        char_name
                    };
                    ui.label(RichText::new(name).strong().size(16.0));
                    ui.label(RichText::new(oto_dir.as_ref().map(|p: &PathBuf| p.to_string_lossy().to_string()).unwrap_or_default()).color(ui.visuals().weak_text_color()).size(9.0));
                });

                ui.add_space(16.0);

                // Readme / License preview
                if !readme.is_empty() || !license.is_empty() {
                    ui.vertical(|ui| {
                        ui.set_max_width(300.0);
                        egui::ScrollArea::vertical().max_height(45.0).show(ui, |ui| {
                            if !readme.is_empty() {
                                ui.label(RichText::new(&readme).size(10.0).color(Color32::from_rgb(200, 200, 220)));
                            }
                            if !license.is_empty() {
                                ui.separator();
                                ui.label(RichText::new(format!("⚖ {}", license)).size(10.0).color(Color32::from_rgb(150, 200, 150)));
                            }
                        });
                    });
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    
                    // Pitch Selection
                    let pitches = [
                        "C1","C#1","D1","D#1","E1","F1","F#1","G1","G#1","A1","A#1","B1",
                        "C2","C#2","D2","D#2","E2","F2","F#2","G2","G#2","A2","A#2","B2",
                        "C3","C#3","D3","D#3","E3","F3","F#3","G3","G#3","A3","A#3","B3",
                        "C4","C#4","D4","D#4","E4","F4","F#4","G4","G#4","A4","A#4","B4",
                        "C5","C#5","D5","D#5","E5","F5","F#5","G5","G#5","A5","A#5","B5",
                        "C6","C#6","D6","D#6","E6","F6","F#6","G6","G#6","A6","A#6","B6",
                        "C7","C#7","D7","D#7","E7","F7","F#7","G7","G#7","A7","A#7","B7",
                    ];
                    
                    ComboBox::from_id_salt("pitch_select")
                        .selected_text(RichText::new(&self.config.test_pitch).color(Color32::from_rgb(137, 180, 250)).strong())
                        .width(60.0)
                        .show_ui(ui, |ui| {
                            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                                for p in pitches {
                                    ui.selectable_value(&mut self.config.test_pitch, p.to_string(), p);
                                }
                            });
                        });
                    ui.label(tr!("header.pitch.label"));

                    ui.separator();

                    // Resampler Selection
                    if ui.button(RichText::new(format!("⚙ {}", tr!("header.resampler.select"))).strong()).clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.config.resampler_path = Some(path);
                        }
                    }
                    if let Some(res) = &self.config.resampler_path {
                        ui.label(RichText::new(res.file_name().unwrap_or_default().to_string_lossy()).size(10.0).color(ui.visuals().weak_text_color()));
                    } else {
                        ui.label(RichText::new(tr!("header.resampler.none")).size(10.0).color(Color32::from_rgb(243, 139, 168)));
                    }

                    ui.separator();
                    let has_resampler = self.config.resampler_path.is_some();
                    let mut btn = ui.button(RichText::new(format!("🧪 {}", tr!("header.resampler.test"))).strong());
                    if !has_resampler {
                        btn = btn.on_hover_text(tr!("header.resampler.hover"));
                    }
                    if btn.clicked() {
                        if has_resampler {
                            self.resample_current();
                        } else {
                            self.ui.status = tr!("header.resampler.status").to_string();
                        }
                    }
                });
            });
            ui.add_space(4.0);
        });
    }
}
