#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod oto;
mod waveform;
mod spectrogram;
mod plugins;

use std::path::Path;
use std::sync::Arc;
use egui::{Color32, Stroke, Vec2};
use app::CopaibaApp;

fn main() -> eframe::Result {
    let pt_br = include_str!("assets/pt-BR.egl");
    let en_us = include_str!("assets/en-US.egl");
    egui_i18n::load_translations_from_text("pt-BR", pt_br).unwrap();
    egui_i18n::load_translations_from_text("en-US", en_us).unwrap();
    egui_i18n::set_language("pt-BR");
    egui_i18n::set_fallback("pt-BR");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Copaiba NEO")
            .with_inner_size([1400.0, 860.0])
            .with_min_inner_size([800.0, 500.0])
            .with_maximized(true),
        ..Default::default()
    };
    eframe::run_native(
        "Copaiba NEO",
        options,
        Box::new(|cc| {
            apply_dark_theme(&cc.egui_ctx);
            setup_fonts(&cc.egui_ctx);
            let mut app = CopaibaApp::default();
            app.load_prefs();
            egui_i18n::set_language(&app.config.language);
            Ok(Box::new(app))
        }),
    )
}

// ── Font setup ────────────────────────────────────────────────────────────────

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let system_fonts = [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansCJK.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK.ttc",
    ];
    let mut found = false;
    for path in system_fonts {
        if Path::new(path).exists() {
            if let Ok(data) = std::fs::read(path) {
                fonts.font_data.insert("cjk_font".to_owned(), Arc::new(egui::FontData::from_owned(data)));
                fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().push("cjk_font".to_owned());
                fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("cjk_font".to_owned());
                found = true;
                break;
            }
        }
    }
    if found { ctx.set_fonts(fonts); }
}

// ── eframe::App implementation ─────────────────────────────────────────────────

impl eframe::App for CopaibaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = ctx.input(|i| i.time);
        if self.session_start_time == 0.0 { self.session_start_time = now; }

        // Close confirmation
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.cur().dirty {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.ui.show_exit_dialog = true;
            }
        }

        // ── Panels (order matters: top/bottom before central) ──────────────────
        self.show_menu_bar(ctx);
        if !self.ui.show_home {
            self.show_tab_bar(ctx);
            self.show_voicebank_header(ctx);
            self.show_alias_table(ctx);
            self.show_tools_panel(ctx);
        }
        self.show_status_bar(ctx, now);

        // ── Keyboard shortcuts (before waveform to avoid consuming events) ─────
        self.handle_shortcuts(ctx);

        // ── Central waveform panel / Home Screen ─────────────────────────────
        if self.ui.show_home {
            self.show_home_screen(ctx);
        } else {
            self.show_waveform_panel(ctx);
        }

        // ── Modal windows ──────────────────────────────────────────────────────
        self.show_modals(ctx);

        // Repaint rate
        if self.audio.playback_start.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(32));
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }
    }
}

// ── Dark theme ────────────────────────────────────────────────────────────────

fn apply_dark_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(18, 18, 28);
    visuals.window_fill = Color32::from_rgb(24, 24, 36);
    visuals.extreme_bg_color = Color32::from_rgb(12, 12, 20);
    visuals.faint_bg_color = Color32::from_rgb(24, 24, 36);
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 30, 46);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(38, 38, 56);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(50, 50, 72);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.active.bg_fill = Color32::from_rgb(70, 50, 110);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
    visuals.override_text_color = Some(Color32::from_rgb(205, 214, 244));
    visuals.hyperlink_color = Color32::from_rgb(137, 180, 250);
    visuals.selection.bg_fill = Color32::from_rgb(70, 50, 120);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(137, 180, 250));
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(6.0, 4.0);
    style.spacing.button_padding = Vec2::new(8.0, 4.0);
    style.interaction.selectable_labels = false;
    ctx.set_style(style);
}
