use egui::{Color32, RichText, Vec2, Frame};
use egui_i18n::tr;
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn show_home_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(ctx.style().visuals.panel_fill))
            .show(ctx, |ui| {
            
            egui::ScrollArea::vertical()
                .id_salt("home_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.set_max_width(800.0);
                        
                        ui.add_space(32.0);
                        let accent = Color32::from_rgb(173, 139, 255);
                        let card_fill = ui.visuals().widgets.noninteractive.bg_fill;
                        Frame::new()
                            .fill(card_fill)
                            .stroke(egui::Stroke::new(1.0, accent.gamma_multiply(0.38)))
                            .corner_radius(egui::CornerRadius::same(20))
                            .inner_margin(egui::Margin::same(28))
                            .show(ui, |ui| {
                                crate::app::layout::horizontal(ui, self.is_rtl(), |ui| {
                                    crate::app::layout::vertical(ui, self.is_rtl(), |ui| {
                                        let title_size = (ui.available_width() * 0.075).clamp(28.0, 42.0);
                                        ui.label(RichText::new(tr!("home.title")).size(title_size).strong().color(accent));
                                        ui.add_space(6.0);
                                        ui.label(RichText::new(tr!("home.subtitle")).size(15.0).color(ui.visuals().weak_text_color()));
                                    });
                                    ui.add_space(24.0);
                                    let button = egui::Button::new(RichText::new(format!("📁  {}", tr!("home.btn.open_voicebank"))).size(16.0).strong())
                                        .fill(accent)
                                        .stroke(egui::Stroke::NONE)
                                        .corner_radius(egui::CornerRadius::same(12))
                                        .min_size(Vec2::new(230.0, 48.0));
                                    if ui.add(button).clicked() { self.open_voicebank_dir(); }
                                });
                            });

                        ui.add_space(30.0);
                        
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
                            let mut group_by_root: std::collections::HashMap<std::path::PathBuf, usize> = std::collections::HashMap::new();
                            for r in &self.config.recent_voicebanks {
                                let root = r.root_path.clone().unwrap_or_else(|| {
                                    r.path.parent().unwrap_or(&r.path).to_path_buf()
                                });
                                
                                if let Some(&group_index) = group_by_root.get(&root) {
                                    let group = &mut groups[group_index];
                                    group.items.push(r.clone());
                                } else {
                                    group_by_root.insert(root.clone(), groups.len());
                                    groups.push(RecentGroup {
                                        name: r.name.clone(),
                                        root,
                                        image: r.image_path.clone(),
                                        items: vec![r.clone()],
                                    });
                                }
                            }

                            let mut remove_root = None;
                            crate::app::layout::vertical(ui, self.is_rtl(), |ui| {
                                for (g_idx, group) in groups.iter().enumerate() {
                                    let _id = ui.id().with("group").with(g_idx);
                                    let frame = Frame::new()
                                        .fill(card_fill)
                                        .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color))
                                        .corner_radius(egui::CornerRadius::same(16))
                                        .inner_margin(egui::Margin::same(16))
                                        .outer_margin(egui::Margin::symmetric(0, 6));

                                    let mut opened_item = false;
                                    crate::app::layout::horizontal(ui, self.is_rtl(), |ui| {
                                        // "X" to remove this voicebank from history (Outside the frame)
                                        crate::app::layout::vertical(ui, self.is_rtl(), |ui| {
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
                                                if let Some(img_path) = &group.image {
                                                    let cache_id = egui::Id::new(("home_group_image", img_path));
                                                    let cached = ui.ctx().data(|data| data.get_temp::<Option<egui::TextureHandle>>(cache_id));
                                                    let texture = cached.unwrap_or_else(|| {
                                                        let texture = std::fs::read(img_path).ok()
                                                            .and_then(|bytes| image::load_from_memory(&bytes).ok())
                                                            .map(|image| {
                                                                let size = [image.width() as usize, image.height() as usize];
                                                                let pixels = egui::ColorImage::from_rgba_unmultiplied(size, image.to_rgba8().as_flat_samples().as_slice());
                                                                ui.ctx().load_texture(format!("home_group_{}", img_path.display()), pixels, egui::TextureOptions::LINEAR)
                                                            });
                                                        ui.ctx().data_mut(|data| data.insert_temp(cache_id, texture.clone()));
                                                        texture
                                                    });
                                                    if let Some(tex) = texture {
                                                        ui.painter().image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                                                    } else {
                                                        ui.painter().rect_filled(rect, 10.0, accent.gamma_multiply(0.16));
                                                    }
                                                }
                                                if group.image.is_none() {
                                                    ui.painter().rect_filled(rect, 10.0, accent.gamma_multiply(0.16));
                                                    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "🎵", egui::FontId::proportional(32.0), Color32::GRAY);
                                                }
                                                
                                                ui.add_space(16.0);
                                                
                                                crate::app::layout::vertical(ui, self.is_rtl(), |ui| {
                                                    ui.label(RichText::new(&group.name).size(20.0).strong());
                                                    ui.label(RichText::new(group.root.to_string_lossy()).size(11.0).color(ui.visuals().weak_text_color()));
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
                                                            if resp.clicked() { opened_item = true; self.load_oto_in_new_tab(item.path.clone()); self.ui.show_home = false; }
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
                                            ui.painter().rect_stroke(response.rect, 16.0, egui::Stroke::new(1.0, accent), egui::StrokeKind::Middle);
                                        }
                                        if response.clicked() && !opened_item && remove_root != Some(group.root.clone()) {
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
