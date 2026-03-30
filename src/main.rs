// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod oto;
mod waveform;
mod spectrogram;
mod plugins;

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
use egui::{Color32, Stroke, Vec2};
use app::CopaibaApp;
use app::state::AppTheme;
use app::bidi;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

fn main() -> eframe::Result {
    // Load translations at compile time
    let _ = egui_i18n::load_translations_from_text("en-US", include_str!("assets/en-US.egl"));
    let _ = egui_i18n::load_translations_from_text("pt-BR", include_str!("assets/pt-BR.egl"));

    // Arabic translations need Unicode Bidi + shaping before being fed to egui,
    // because egui has no built-in RTL text engine.  We pre-process each value
    // in the .egl file through `bidi::reshape` so that `tr!()` already returns
    // visually-ordered, shaped text.
    {
        let raw_ar = include_str!("assets/ar-SA.egl");
        let reshaped = reshape_egl_arabic(raw_ar);
        let _ = egui_i18n::load_translations_from_text("ar-SA", &reshaped);
    }

    println!("Starting Copaiba NEO...");
    
    // Load Icon
    #[cfg(not(target_arch = "wasm32"))]
    let icon_data = if let Ok(img) = image::open("favicon_mori.png") {
        use image::GenericImageView;
        let (width, height) = img.dimensions();
        let rgba = img.to_rgba8().into_raw();
        Some(egui::IconData { rgba, width, height })
    } else {
        None
    };

    #[cfg(not(target_arch = "wasm32"))]
    {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Copaiba NEO")
                .with_inner_size([1280.0, 720.0])
                .with_min_inner_size([800.0, 500.0])
                .with_icon(icon_data.unwrap_or_default()),
            ..Default::default()
        };
        println!("Options initialized. Running native...");
        eframe::run_native(
            "Copaiba NEO",
            options,
            Box::new(|cc| {
                egui_extras::install_image_loaders(&cc.egui_ctx);
                println!("Initializing app...");
                let mut app = CopaibaApp::default();
                println!("Loading preferences...");
                app.load_prefs();
                
                apply_theme(&cc.egui_ctx, app.config.theme);
                let lang = app.config.language.clone();
                app.set_language(&lang);
                
                if app.tabs.len() == 1 && (app.tabs[0].name.is_empty() || app.tabs[0].name == "Novo Set") {
                    app.tabs[0].name = egui_i18n::tr!("state.tab.default_name").to_string();
                }

                println!("Loading UI sounds...");
                app.load_ui_sounds();
                setup_fonts(&cc.egui_ctx);

                println!("App started!");
                Ok(Box::new(app))
            }),
        )
    }

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();

        let web_options = eframe::WebOptions::default();

        wasm_bindgen_futures::spawn_local(async {
            let document = web_sys::window()
                .expect("No window")
                .document()
                .expect("No document");
            let canvas = document
                .get_element_by_id("the_canvas_id")
                .expect("Failed to find the_canvas_id")
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .expect("the_canvas_id is not a HtmlCanvasElement");

            let runner = eframe::WebRunner::new();
            runner.start(
                canvas,
                web_options,
                Box::new(|cc| {
                    egui_extras::install_image_loaders(&cc.egui_ctx);
                    let mut app = CopaibaApp::default();
                    app.load_prefs();
                    
                    apply_theme(&cc.egui_ctx, app.config.theme);
                    let lang = app.config.language.clone();
                    app.set_language(&lang);
                    
                    app.load_ui_sounds();
                    setup_fonts(&cc.egui_ctx);
                    
                    Ok(Box::new(app))
                }),
            ).await.expect("failed to start eframe");
        });
        Ok(())
    }
}

// ── Arabic text pre-processing ──────────────────────────────────────────────

/// Parse the ar-SA.egl key=value lines and apply Arabic shaping + visual
/// reordering to each value, then return the whole file as a new String.
/// Lines starting with `#` (comments) and blank lines are kept verbatim.
fn reshape_egl_arabic(egl: &str) -> String {
    let mut out = String::with_capacity(egl.len() + 512);
    for line in egl.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            out.push_str(line);
        } else if let Some(eq) = trimmed.find('=') {
            // key = value
            let key = &trimmed[..eq];
            let value = &trimmed[eq + 1..];
            let shaped = bidi::reshape(value);
            out.push_str(key);
            out.push('=');
            out.push_str(&shaped);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

// ── Font setup ────────────────────────────────────────────────────────────────

fn setup_fonts(ctx: &egui::Context) {
    println!("Setting up fonts...");
    #[allow(unused_mut)]
    let mut fonts = egui::FontDefinitions::default();
    #[cfg(not(target_arch = "wasm32"))]
    {
        // CJK Fonts
        let system_fonts = [
            // Linux
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            // Windows
            //Japones
            "C:\\Windows\\Fonts\\msyh.ttc",   // Microsoft YaHei
            "C:\\Windows\\Fonts\\msgothic.ttc", // MS Gothic
            "C:\\Windows\\Fonts\\simsun.ttc",  // SimSun
            "C:\\Windows\\Fonts\\meiryo.ttc",  // Meiryo
            "C:\\Windows\\Fonts\\msmincho.ttc",   // MS Mincho
            "C:\\Windows\\Fonts\\YuGothic.ttf",   // Yu Gothic
            "C:\\Windows\\Fonts\\YuMincho.ttf",   // Yu Mincho
            "C:\\Windows\\Fonts\\meiryo.ttc",     // Meiryo
            "C:\\Windows\\Fonts\\msjhl.ttc",      // MS JHL
            //Coreano
            "C:\\Windows\\Fonts\\malgun.ttf",    // Malgun Gothic
            "C:\\Windows\\Fonts\\gulim.ttc",     // Gulim
            //Chines
            "C:\\Windows\\Fonts\\simsun.ttc",     // SimSun
            "C:\\Windows\\Fonts\\simhei.ttf",    // SimHei
            "C:\\Windows\\Fonts\\mingliu.ttc",    // MingLiu
            "C:\\Windows\\Fonts\\kai.ttf",       // Kai
            "C:\\Windows\\Fonts\\arialuni.ttf",  // Arial Unicode MS
        ];

        for path in system_fonts {
            if Path::new(path).exists() {
                if let Ok(data) = std::fs::read(path) {
                    fonts.font_data.insert("cjk_font".to_owned(), Arc::new(egui::FontData::from_owned(data)));
                    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().push("cjk_font".to_owned());
                    fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("cjk_font".to_owned());
                    break;
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Arabic Fonts — must have the Arabic Presentation Forms block (U+FE70-FEFF)
        // since ar_reshaper outputs those codepoints.
        // "Traditional Arabic" (times.ttf) and "Scheherazade New" are the most
        // complete on Windows; Noto Naskh is best on Linux.
        let arabic_fonts = [
            // Windows — fonts with full Arabic Presentation Forms coverage
            "C:\\Windows\\Fonts\\times.ttf",          // Times New Roman (has Arabic PF)
            "C:\\Windows\\Fonts\\arabtype.ttf",       // Arabic Typesetting
            "C:\\Windows\\Fonts\\tradbdo.ttf",        // Traditional Arabic Bold
            "C:\\Windows\\Fonts\\trad.ttf",           // Traditional Arabic
            "C:\\Windows\\Fonts\\tahoma.ttf",         // Tahoma (good Arabic PF coverage)
            "C:\\Windows\\Fonts\\scheherazadnew.ttf", // Scheherazade New
            "C:\\Windows\\Fonts\\aldhabi.ttf",        // Al Dhabi
            "C:\\Windows\\Fonts\\alfirat.ttf",        // Al Firat
            "C:\\Windows\\Fonts\\segoeui.ttf",        // Segoe UI (fallback)
            "C:\\Windows\\Fonts\\arial.ttf",          // Arial (last resort)
            // Linux
            "/usr/share/fonts/truetype/noto/NotoNaskhArabic-Regular.ttf",
            "/usr/share/fonts/truetype/noto/NotoSansArabic-Regular.ttf",
            "/usr/share/fonts/opentype/noto/NotoNaskhArabic-Regular.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansArabic-Regular.ttf",
            "/usr/share/fonts/truetype/droid/DroidNaskh-Regular.ttf",
        ];

        for path in arabic_fonts {
            if Path::new(path).exists() {
                if let Ok(data) = std::fs::read(path) {
                    fonts.font_data.insert("arabic_font".to_owned(), Arc::new(egui::FontData::from_owned(data)));
                    // Push to END: egui falls back to this font only when the
                    // default font lacks a glyph. Arabic Presentation Forms
                    // (U+FE70-FEFF) are not in the default egui font, so Arabic
                    // text will use arabic_font while all other text keeps the
                    // original clean font.
                    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().push("arabic_font".to_owned());
                    fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("arabic_font".to_owned());
                    break;
                }
            }
        }
    }
    
    ctx.set_fonts(fonts);
}

// ── eframe::App implementation ─────────────────────────────────────────────────
impl eframe::App for CopaibaApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = ctx.input(|i| i.time);
        if self.session_start_time == 0.0 { self.session_start_time = now; }

        if self.ui.show_splash {
            self.ui.splash_progress += ctx.input(|i| i.stable_dt).min(0.1);
            if self.ui.splash_progress > 1.6 {
                self.ui.show_splash = false;
            }
            ctx.request_repaint();
        }

        // Close confirmation
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.cur().dirty {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.ui.show_exit_dialog = true;
            }
        }

        // ── Keyboard shortcuts (before waveform/panels to avoid consuming events) ─
        self.handle_shortcuts(ctx);

        // ── Panels (order matters: top/bottom before central) ──────────────────
        self.show_menu_bar(ctx);
        self.show_status_bar(ctx, now);
        
        if self.ui.show_home {
            self.show_home_screen(ctx);
        } else {
            self.show_tab_bar(ctx);
            self.show_voicebank_header(ctx);
            self.show_alias_table(ctx);
            self.show_tools_panel(ctx);
            self.show_waveform_panel(ctx);
        }

        // ── Modal windows ──────────────────────────────────────────────────────
        self.show_modals(ctx);
        self.show_pmap_editor(ctx);

        // ── Toasts ─────────────────────────────────────────────────────────────
        self.ui.toast_manager.draw(ctx);

        // Repaint rate
        if self.audio.playback_start.is_some() || self.ui.show_splash {
            ctx.request_repaint_after(std::time::Duration::from_millis(32));
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }
    }
}

// ── Themes ───────────────────────────────────────────────────────────────────

pub fn apply_theme(ctx: &egui::Context, theme: AppTheme) {
    match theme {
        AppTheme::Dark => apply_dark_theme(ctx),
        AppTheme::Light => apply_light_theme(ctx),
    }
}

pub fn apply_light_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    
    // Custom light theme adjustments
    visuals.panel_fill = Color32::from_rgb(245, 245, 250);
    visuals.window_fill = Color32::from_rgb(255, 255, 255);
    visuals.extreme_bg_color = Color32::from_rgb(235, 235, 240);
    visuals.faint_bg_color = Color32::from_rgb(240, 240, 245);
    
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(235, 235, 240);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(10);
    
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(220, 220, 225);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(10);
    
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(210, 210, 220);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(10);
    
    visuals.widgets.active.bg_fill = Color32::from_rgb(180, 160, 220);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(10);

    visuals.override_text_color = Some(Color32::from_rgb(30, 30, 46));
    visuals.hyperlink_color = Color32::from_rgb(50, 100, 200);
    visuals.selection.bg_fill = Color32::from_rgb(200, 180, 255);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(100, 120, 240));
    
    ctx.set_visuals(visuals);
    setup_common_style(ctx);
}

pub fn apply_dark_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(18, 18, 28);
    visuals.window_fill = Color32::from_rgb(24, 24, 36);
    visuals.extreme_bg_color = Color32::from_rgb(12, 12, 20);
    visuals.faint_bg_color = Color32::from_rgb(24, 24, 36);
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgba_premultiplied(30, 30, 46, 200);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(10);
    visuals.widgets.inactive.bg_fill = Color32::from_rgba_premultiplied(38, 38, 56, 220);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(10);
    visuals.widgets.hovered.bg_fill = Color32::from_rgba_premultiplied(50, 50, 72, 230);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(10);
    visuals.widgets.active.bg_fill = Color32::from_rgba_premultiplied(70, 50, 110, 255);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(10);
    
    visuals.override_text_color = Some(Color32::from_rgb(205, 214, 244));
    visuals.hyperlink_color = Color32::from_rgb(137, 180, 250);
    visuals.selection.bg_fill = Color32::from_rgb(70, 50, 120);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(137, 180, 250));
    ctx.set_visuals(visuals);
    setup_common_style(ctx);
}

fn setup_common_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Resize Handle (make panels easier to grab)
    style.spacing.interact_size.y = 12.0;

    style.spacing.item_spacing = egui::Vec2::new(8.0, 6.0);
    style.spacing.button_padding = egui::Vec2::new(10.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12);
    style.interaction.selectable_labels = false;
    ctx.set_style(style);
}

