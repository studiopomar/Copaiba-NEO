use egui::{Color32, RichText, Vec2, Frame};
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
                        let title_size = (ui.available_width() * 0.09).clamp(28.0, 48.0);
                        ui.label(RichText::new(tr!("home.title")).size(title_size).strong().color(Color32::from_rgb(140, 100, 200)));
                        ui.label(RichText::new(tr!("home.subtitle")).size(16.0).color(Color32::GRAY));
                        ui.add_space(40.0);
                        
                        crate::app::layout::horizontal(ui, self.is_rtl(), |ui| {
                            let button_width = ui.available_width().min(360.0).max(220.0);
                            ui.add_space((ui.available_width() - button_width) / 2.0);
                            if ui.add(egui::Button::new(RichText::new(format!("📁 {}",tr!("home.btn.open_voicebank"))).size(18.0)).min_size(Vec2::new(button_width, 50.0))).clicked() {
                                self.open_voicebank_dir();
                            }
                        });
                        
                        ui.add_space(40.0);
                        
                        if !self.config.recent_voicebanks.is_empty() {
                            crate::app::layout::horizontal(ui, self.is_rtl(), |ui| {
                                ui.label(RichText::new(tr!("home.label.recent")).size(24.0).strong());
                                ui.add_space(10.0);
                                if ui.button(RichText::new("🗑").color(Color32::RED)).on_hover_text("Limpar Histórico").clicked() {
                                    self.config.recent_voicebanks.clear();
                                    self.save_prefs();
                                }
                            });
                            ui.add_space(20.0);
                            
                            // Grouping logic
                            struct RecentGroup {
                                name: String,
                                root: std::path::PathBuf,
                                image: Option<std::path::PathBuf>,
                                items: Vec<crate::app::state::RecentVoicebank>,
                            }
                            
                            let mut groups: Vec<RecentGroup> = Vec::new();
                            for r in &self.config.recent_voicebanks {
                                let root = r.root_path.clone().unwrap_or_else(|| {
                                    r.path.parent().unwrap_or(&r.path).to_path_buf()
                                });
                                
                                if let Some(group) = groups.iter_mut().find(|g| g.root == root) {
                                    group.items.push(r.clone());
                                } else {
                                    groups.push(RecentGroup {
                                        name: r.name.clone(),
                                        root,
                                        image: r.image_path.clone(),
                                        items: vec![r.clone()],
                                    });
                                }
                            }

                            let mut remove_root = None;
                            ui.vertical(|ui| {
                                for (g_idx, group) in groups.iter().enumerate() {
                                    let _id = ui.id().with("group").with(g_idx);
                                    let frame = Frame::new()
                                        .fill(Color32::from_rgb(20, 20, 30))
                                        .corner_radius(egui::CornerRadius::same(12))
                                        .inner_margin(egui::Margin::same(16))
                                        .outer_margin(egui::Margin::symmetric(0, 8));

                                    ui.horizontal(|ui| {
                                        // "X" to remove this voicebank from history (Outside the frame)
                                        ui.vertical(|ui| {
                                            ui.add_space(38.0); // Center with card height
                                            let btn = egui::Button::new(RichText::new("✖").color(Color32::from_rgb(220, 100, 100)).size(18.0).strong())
                                                .fill(Color32::TRANSPARENT)
                                                .stroke(egui::Stroke::new(1.0, Color32::from_rgba_premultiplied(200, 80, 80, 60)))
                                                .corner_radius(egui::CornerRadius::same(50));
                                            
                                            if ui.add(btn).clicked() {
                                                remove_root = Some(group.root.clone());
                                            }
                                        });

                                        ui.add_space(6.0);

                                        let inner_res = frame.show(ui, |ui| {
                                            crate::app::layout::horizontal(ui, self.is_rtl(), |ui| {
                                                // Thumbnail
                                                let (rect, _) = ui.allocate_at_least(Vec2::new(80.0, 80.0), egui::Sense::hover());
                                                let mut painted = false;
                                                if let Some(img_path) = &group.image {
                                                    if let Ok(data) = std::fs::read(img_path) {
                                                        if let Ok(image) = image::load_from_memory(&data) {
                                                            let size = [image.width() as usize, image.height() as usize];
                                                            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, image.to_rgba8().as_flat_samples().as_slice());
                                                            let tex = ui.ctx().load_texture(format!("group_img_{}", g_idx), color_image, Default::default());
                                                            ui.painter().image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                                                            painted = true;
                                                        }
                                                    }
                                                }
                                                if !painted {
                                                    ui.painter().rect_filled(rect, 8.0, Color32::from_rgb(35, 35, 50));
                                                    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "🎵", egui::FontId::proportional(32.0), Color32::GRAY);
                                                }
                                                
                                                ui.add_space(16.0);
                                                
                                                ui.vertical(|ui| {
                                                    ui.label(RichText::new(&group.name).size(24.0).strong().color(Color32::WHITE));
                                                    ui.label(RichText::new(group.root.to_string_lossy()).size(12.0).color(Color32::GRAY));
                                                    ui.add_space(8.0);
                                                    
                                                    for (i, item) in group.items.iter().enumerate() {
                                                        let sub_name = if let Some(parent) = item.path.parent() {
                                                            if parent == group.root { "main".to_string() } else { parent.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_else(|| "sub".into()) }
                                                        } else { "root".into() };

                                                        crate::app::layout::horizontal(ui, self.is_rtl(), |ui| {
                                                            ui.add_space(8.0);
                                                            let btn_text = format!("  ↳  {}", sub_name);
                                                            let btn = egui::Button::new(RichText::new(btn_text).size(13.0)).fill(Color32::TRANSPARENT).stroke(egui::Stroke::NONE).sense(egui::Sense::click());
                                                            let resp = ui.add(btn);
                                                            if resp.clicked() { self.load_oto_in_new_tab(item.path.clone()); self.ui.show_home = false; }
                                                            if resp.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
                                                        });
                                                        if i < group.items.len() - 1 { ui.add_space(2.0); }
                                                    }
                                                });
                                            });
                                        });

                                        let response = ui.interact(inner_res.response.rect, _id, egui::Sense::click());
                                        if response.hovered() {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                            ui.painter().rect_stroke(response.rect, 12.0, egui::Stroke::new(1.0, Color32::from_rgb(100, 100, 255)), egui::StrokeKind::Middle);
                                        }
                                        if response.clicked() && remove_root != Some(group.root.clone()) {
                                            for item in &group.items { self.load_oto_in_new_tab(item.path.clone()); }
                                            self.ui.show_home = false;
                                        }
                                    });
                                    ui.add_space(8.0);
                                }
                            });

                            if let Some(r) = remove_root {
                                self.config.recent_voicebanks.retain(|item| {
                                    item.root_path.as_ref().unwrap_or(&item.path.parent().unwrap().to_path_buf()) != &r
                                });
                                self.save_prefs();
                            }
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
