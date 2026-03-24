use egui::{Color32, RichText, Vec2, Frame, Margin, CornerRadius};
use egui_i18n::tr;
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn show_home_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(Color32::from_rgb(10, 10, 18)))
            .show(ctx, |ui| {
            
            egui::ScrollArea::vertical()
                .id_salt("home_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.set_max_width(800.0);
                        
                        ui.add_space(60.0);
                        ui.label(RichText::new(tr!("home.title")).size(48.0).strong().color(Color32::from_rgb(140, 100, 200)));
                        ui.label(RichText::new(tr!("home.subtitle")).size(16.0).color(Color32::GRAY));
                        ui.add_space(40.0);
                        
                        ui.horizontal(|ui| {
                            ui.add_space(ui.available_width() / 2.0 - 150.0);
                            if ui.add(egui::Button::new(RichText::new(format!("📁 {}",tr!("home.btn.open_voicebank"))).size(18.0)).min_size(Vec2::new(300.0, 50.0))).clicked() {
                                self.open_voicebank_dir();
                            }
                        });
                        
                        ui.add_space(40.0);
                        
                        if !self.config.recent_voicebanks.is_empty() {
                            ui.label(RichText::new(tr!("home.label.recent")).size(24.0).strong());
                            ui.add_space(20.0);
                            
                            ui.vertical(|ui| {
                                for i in 0..self.config.recent_voicebanks.len() {
                                    let recent = self.config.recent_voicebanks[i].clone();
                                    let mut frame = Frame::new()
                                        .fill(Color32::from_rgb(20, 20, 30))
                                        .corner_radius(CornerRadius::same(8))
                                        .inner_margin(Margin::same(12))
                                        .outer_margin(Margin::symmetric(0, 4));
                                    
                                    let id = ui.id().with("recent").with(i);
                                    let resp = ui.interact(ui.available_rect_before_wrap(), id, egui::Sense::click());
                                    if resp.hovered() {
                                        frame = frame.fill(Color32::from_rgb(30, 30, 45));
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }
                                    
                                    frame.show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            // Thumbnail
                                            let (rect, _) = ui.allocate_at_least(Vec2::new(60.0, 60.0), egui::Sense::hover());
                                            let mut painted = false;
                                            if let Some(img_path) = &recent.image_path {
                                                if let Ok(data) = std::fs::read(img_path) {
                                                    if let Ok(image) = image::load_from_memory(&data) {
                                                        let size = [image.width() as usize, image.height() as usize];
                                                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                                            size,
                                                            image.to_rgba8().as_flat_samples().as_slice(),
                                                        );
                                                        let tex = ui.ctx().load_texture(format!("recent_{}", i), color_image, Default::default());
                                                        ui.painter().image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                                                        painted = true;
                                                    }
                                                }
                                            }
                                            
                                            if !painted {
                                                ui.painter().rect_filled(rect, 4.0, Color32::from_rgb(35, 35, 50));
                                                ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "🎵", egui::FontId::proportional(24.0), Color32::GRAY);
                                            }
                                            
                                            ui.add_space(12.0);
                                            
                                            ui.vertical(|ui| {
                                                let path_str = recent.path.to_string_lossy();
                                                ui.label(RichText::new(&recent.name).size(18.0).strong().color(Color32::WHITE));
                                                ui.label(RichText::new(path_str).size(12.0).color(Color32::GRAY));
                                                
                                                // Branch representation
                                                if let Some(parent) = recent.path.parent() {
                                                    if let Some(gp) = parent.parent() {
                                                        let branch = format!("{} > {}", gp.file_name().unwrap_or_default().to_string_lossy(), parent.file_name().unwrap_or_default().to_string_lossy());
                                                        ui.label(RichText::new(format!(" {}", branch)).size(10.0).color(Color32::from_rgb(100, 100, 150)));
                                                    }
                                                }
                                            });
                                            
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.button(tr!("home.btn.open")).clicked() || resp.clicked() {
                                                    self.load_oto(recent.path.clone());
                                                    self.ui.show_home = false;
                                                }
                                            });
                                        });
                                    });
                                    ui.add_space(8.0);
                                }
                            });
                        } else {
                            ui.add_space(20.0);
                            ui.label(RichText::new(tr!("home.label.no_recent")).color(Color32::GRAY));
                        }
                        
                        ui.add_space(60.0);
                    });
                });
            });
    }
}
