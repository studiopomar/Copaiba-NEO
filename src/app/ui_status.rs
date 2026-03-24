use egui::RichText;
use egui_i18n::tr;
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn show_status_bar(&mut self, ctx: &egui::Context, now: f64) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                let tab = self.cur();

                // Display project path or translated "Novo Arquivo"/"New File"
                if let Some(p) = &self.project_path {
                    let dirty_marker = if self.tabs.iter().any(|t| t.dirty) { "*" } else { "" };
                    ui.label(egui::RichText::new(format!("{}{dirty_marker}", p.display())).color(egui::Color32::from_rgb(140, 140, 160)).size(11.0));
                } else {
                    ui.label(RichText::new(tr!("status.label.new_file")).color(egui::Color32::LIGHT_GRAY).small());
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let total = tab.entries.len();
                    let sel = tab.selected;
                    let sel_display = if sel == usize::MAX { 0 } else { sel + 1 };
                    ui.label(RichText::new(format!("{} {}/{}", tr!("status.label.line"), sel_display, total)).small().color(egui::Color32::GRAY));
                    ui.add_space(16.0);

                    let done_count = tab.entries.iter().filter(|e| e.done).count();
                    let pct_done = if total > 0 { done_count as f32 / total as f32 } else { 0.0 };
                    ui.horizontal(|ui| {
                        ui.add(egui::ProgressBar::new(pct_done).desired_width(100.0));
                        ui.label(RichText::new(format!("{:.0}%", pct_done * 100.0)).small().color(egui::Color32::GRAY));
                        ui.label(RichText::new(format!("({}/{})", done_count, total)).small().color(egui::Color32::GRAY));
                    });

                    ui.add_space(16.0);
                    let elapsed = now - self.session_start_time;
                    let mins = (elapsed / 60.0).floor();
                    let secs = (elapsed % 60.0).floor();
                    ui.label(RichText::new(format!("{} {:02}:{:02}", tr!("status.label.time_elapsed"), mins as u32, secs as u32)).small().color(egui::Color32::GRAY));
                    ui.add_space(16.0);
                    ui.label(RichText::new(if tab.dirty { tr!("status.label.not_saved") } else { tr!("status.label.saved") }).small().color(egui::Color32::GRAY));
                    ctx.request_repaint_after(std::time::Duration::from_millis(500));

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add_space(16.0);
                        ui.label(RichText::new(&self.ui.status).small());
                    });
                });
            });
        });
    }
}
