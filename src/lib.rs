// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod app;
pub mod audio;
pub mod oto;
pub mod waveform;
pub mod spectrogram;
pub mod plugins;
pub mod wsola;
#[cfg(target_arch = "wasm32")]
pub mod web_files;
#[cfg(target_arch = "wasm32")]
pub mod web_audio;
#[cfg(target_arch = "wasm32")]
pub mod web_download;

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
use egui::{Color32, Stroke};
use app::CopaibaApp;
use app::state::AppTheme;
use app::bidi;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

pub fn run() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    // Load translations
    let _ = egui_i18n::load_translations_from_text("en-US", include_str!("assets/en-US.egl"));
    let _ = egui_i18n::load_translations_from_text("pt-BR", include_str!("assets/pt-BR.egl"));
    {
        let raw_ar = include_str!("assets/ar-SA.egl");
        let reshaped = reshape_egl_arabic(raw_ar);
        let _ = egui_i18n::load_translations_from_text("ar-SA", &reshaped);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let icon_data = {
            let icon_bytes = include_bytes!("../favicon_mori.png");
            if let Ok(img) = image::load_from_memory(icon_bytes) {
                // Resize to max 256x256 to avoid X11 MaximumRequestLengthExceeded
                let img = img.thumbnail(256, 256);
                let (width, height) = (img.width(), img.height());
                let rgba = img.to_rgba8().into_raw();
                Some(egui::IconData { rgba, width, height })
            } else {
                None
            }
        };

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Copaiba NEO v246 - Damasco")
                .with_inner_size([1280.0, 720.0])
                .with_min_inner_size([360.0, 300.0])
                .with_icon(icon_data.unwrap_or_default()),
            multisampling: 0,
            ..Default::default()
        };

        eframe::run_native(
            "Copaiba NEO v246 - Damasco",
            options,
            Box::new(|cc| {
                Ok(setup_app_box(cc, args))
            }),
        )
    }

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        let web_options = eframe::WebOptions::default();
        wasm_bindgen_futures::spawn_local(async {
            let document = web_sys::window().unwrap().document().unwrap();
            let canvas = document
                .get_element_by_id("the_canvas_id")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();

            let runner = eframe::WebRunner::new();
            runner.start(
                canvas,
                web_options,
                Box::new(|cc| Ok(setup_app_box(cc, Vec::new()))),
            ).await.expect("failed to start eframe");
        });
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start_web() {
    run().expect("failed to start Copaiba NEO in the browser");
}

fn setup_app_box(cc: &eframe::CreationContext<'_>, args: Vec<String>) -> Box<dyn eframe::App> {
    egui_extras::install_image_loaders(&cc.egui_ctx);
    let mut app = CopaibaApp::default();
    app.load_prefs();
    app.handle_cli_args(args);
    apply_theme(&cc.egui_ctx, app.config.theme);
    let lang = app.config.language.clone();
    app.set_language(&lang);

    // Trigger update check
    #[cfg(not(target_arch = "wasm32"))]
    {
        let ver = env!("CARGO_PKG_VERSION").to_string();
        let result_arc = app.ui.update_available.clone();
        app::updater::spawn_update_check(ver, result_arc);
    }

    if app.tabs.len() == 1 && (app.tabs[0].name.is_empty() || app.tabs[0].name == "Novo Set") {
        app.tabs[0].name = egui_i18n::tr!("state.tab.default_name").to_string();
    }
    app.load_ui_sounds();
    setup_fonts(&cc.egui_ctx);
    Box::new(app)
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };

    eframe::run_native(
        "Copaiba NEO v246 - Damasco",
        options,
        Box::new(|cc| {
            Ok(setup_app_box(cc))
        }),
    ).unwrap();
}

fn reshape_egl_arabic(egl: &str) -> String {
    let mut out = String::with_capacity(egl.len() + 512);
    for line in egl.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            out.push_str(line);
        } else if let Some(eq) = trimmed.find('=') {
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

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    // Bundle a CJK fallback on every target. System font paths are not
    // portable (notably on macOS), and missing glyphs render as squares.
    let data = include_bytes!("assets/fonts/NotoSansJP-Regular.ttf").to_vec();
    fonts.font_data.insert("bundled_cjk_font".to_owned(), Arc::new(egui::FontData::from_owned(data)));
    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "bundled_cjk_font".to_owned());
    fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().insert(0, "bundled_cjk_font".to_owned());
    // Arabic fallback must be bundled too: system font paths are unavailable
    // in the browser and vary across desktop platforms (especially macOS).
    let arabic_data = include_bytes!("assets/fonts/Amiri-Regular.ttf").to_vec();
    fonts.font_data.insert("bundled_arabic_font".to_owned(), Arc::new(egui::FontData::from_owned(arabic_data)));
    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().push("bundled_arabic_font".to_owned());
    fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("bundled_arabic_font".to_owned());
    #[cfg(not(target_arch = "wasm32"))]
    {
        let system_fonts = [
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\msgothic.ttc",
            "C:\\Windows\\Fonts\\simsun.ttc",
            "C:\\Windows\\Fonts\\meiryo.ttc",
            "C:\\Windows\\Fonts\\malgun.ttf",
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

        let arabic_fonts = [
            "C:\\Windows\\Fonts\\times.ttf",
            "C:\\Windows\\Fonts\\tahoma.ttf",
            "/usr/share/fonts/truetype/noto/NotoNaskhArabic-Regular.ttf",
        ];

        for path in arabic_fonts {
            if Path::new(path).exists() {
                if let Ok(data) = std::fs::read(path) {
                    fonts.font_data.insert("arabic_font".to_owned(), Arc::new(egui::FontData::from_owned(data)));
                    fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().push("arabic_font".to_owned());
                    fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("arabic_font".to_owned());
                    break;
                }
            }
        }
    }
    ctx.set_fonts(fonts);
}

impl eframe::App for CopaibaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(target_arch = "wasm32")]
        self.poll_web_voicebank();
        let now = ctx.input(|i| i.time);
        if self.session_start_time == 0.0 { self.session_start_time = now; }
        // Splash logic removed
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.cur().dirty {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.ui.show_exit_dialog = true;
            }
        }
        self.handle_shortcuts(ctx);
        self.show_menu_bar(ctx);
        self.show_status_bar(ctx, now);

        if self.ui.show_home {
            self.show_home_screen(ctx);
        } else {
            // First, panels that take precedence or side space
            self.show_alias_table(ctx); // SidePanel::right
            self.show_tools_panel(ctx); // TopBottomPanel::bottom

            // Then, top-level navigation and headers
            self.show_tab_bar(ctx);
            self.show_voicebank_header(ctx);

            // Finally, the central workspace
            self.show_waveform_panel(ctx);
        }
        self.show_modals(ctx);
        self.show_pmap_editor(ctx);
        self.ui.toast_manager.draw(ctx);
        if self.audio.playback_start.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(32));
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }
    }
}

pub fn apply_theme(ctx: &egui::Context, theme: AppTheme) {
    match theme {
        AppTheme::Dark => apply_dark_theme(ctx),
        AppTheme::Light => apply_light_theme(ctx),
    }
}

pub fn apply_light_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = Color32::from_rgb(246, 244, 250);
    visuals.window_fill = Color32::from_rgb(255, 254, 255);
    visuals.extreme_bg_color = Color32::from_rgb(250, 248, 252);
    visuals.faint_bg_color = Color32::from_rgb(241, 237, 247);
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(241, 237, 247);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(224, 216, 235));
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(9);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(247, 244, 250);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(220, 211, 232));
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(9);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(237, 227, 248);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(184, 157, 218));
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(9);
    visuals.widgets.active.bg_fill = Color32::from_rgb(220, 201, 241);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(151, 115, 192));
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(9);
    visuals.override_text_color = Some(Color32::from_rgb(43, 35, 55));
    visuals.hyperlink_color = Color32::from_rgb(112, 72, 158);
    visuals.selection.bg_fill = Color32::from_rgb(226, 211, 244);
    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::from_rgb(132, 91, 177));
    ctx.set_visuals(visuals);
    setup_common_style(ctx);
}

pub fn apply_dark_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(23, 20, 30);
    visuals.window_fill = Color32::from_rgb(31, 27, 40);
    visuals.extreme_bg_color = Color32::from_rgb(17, 15, 23);
    visuals.faint_bg_color = Color32::from_rgb(31, 27, 40);
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(37, 32, 47);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(60, 51, 74));
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(9);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(44, 38, 55);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(72, 60, 87));
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(9);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(58, 47, 72);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(131, 99, 164));
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(9);
    visuals.widgets.active.bg_fill = Color32::from_rgb(77, 55, 102);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(171, 132, 207));
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(9);
    visuals.override_text_color = Some(Color32::from_rgb(232, 224, 241));
    visuals.hyperlink_color = Color32::from_rgb(195, 157, 226);
    visuals.selection.bg_fill = Color32::from_rgb(76, 55, 99);
    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::from_rgb(194, 158, 226));
    ctx.set_visuals(visuals);
    setup_common_style(ctx);
}

fn setup_common_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.interact_size = egui::vec2(24.0, 28.0);
    style.spacing.item_spacing = egui::Vec2::new(10.0, 8.0);
    style.spacing.button_padding = egui::Vec2::new(12.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(18);
    style.spacing.menu_margin = egui::Margin::same(10);
    style.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
    style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
    style.text_styles.insert(egui::TextStyle::Small, egui::FontId::proportional(12.0));
    style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(21.0));
    style.interaction.selectable_labels = false;
    ctx.set_style(style);
}
