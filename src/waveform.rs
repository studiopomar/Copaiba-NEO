use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, pos2};

use crate::audio::WavData;
use crate::oto::OtoEntry;
use crate::spectrogram::{SpectrogramData, SpectrogramSettings, render_spectrogram_view};

#[derive(Clone, Debug, PartialEq)]
pub enum WaveformRenderMode {
    Auto,           // Spline if zoomed in, blocks if out
    AlwaysSpline,
    AlwaysBlocks,
}

#[derive(Clone, Copy, Debug)]
pub struct InteractionResult {
    pub drag_started: bool,
    pub drag_released: bool,
    pub clicked: bool,
    pub modified: bool,
    pub nav_delta: i32,
}

#[derive(Clone, Debug)]
pub struct WaveformSettings {
    pub top_color: egui::Color32,
    pub bot_color: egui::Color32,
    pub line_color: egui::Color32,
    pub thickness: f32,
    pub render_mode: WaveformRenderMode,
    pub spline_threshold: f64,
}

impl Default for WaveformSettings {
    fn default() -> Self {
        Self {
            top_color: Color32::from_rgb(46, 204, 113),
            bot_color: Color32::from_rgb(39, 174, 96),
            line_color: Color32::from_rgb(70, 220, 150),
            thickness: 1.5,
            render_mode: WaveformRenderMode::Auto,
            spline_threshold: 4.0, // samples per pixel
        }
    }
}

/// Which parameter marker is currently being dragged
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DragTarget {
    None,
    Offset,
    Preutter,
    Overlap,
    Consonant,
    Cutoff,
}

/// Cached spectrogram texture to avoid re-rendering every frame.
pub struct SpecCache {
    pub texture: Option<egui::TextureHandle>,
    pub view_start: f64,
    pub view_range: f64,
    pub width: usize,
    pub height: usize,
    pub data_ptr: usize,
}

impl Default for SpecCache {
    fn default() -> Self {
        Self { texture: None, view_start: -1.0, view_range: -1.0, width: 0, height: 0, data_ptr: 0 }
    }
}

/// Cached minimap texture (only rebuilt when WAV width changes)
pub struct MinimapCache {
    pub texture: Option<egui::TextureHandle>,
    pub width: usize,  // canvas pixel width at which it was built
    pub data_ptr: usize,
}

impl Default for MinimapCache {
    fn default() -> Self { Self { texture: None, width: 0, data_ptr: 0 } }
}

/// Cached envelope (waveform blocks) texture
pub struct WaveCache {
    pub texture: Option<egui::TextureHandle>,
    pub view_start: f64,
    pub view_range: f64,
    pub width: usize,
    pub height: usize,
    pub scale_y: f32,
    pub data_ptr: usize,
}

impl Default for WaveCache {
    fn default() -> Self {
        Self { texture: None, view_start: -1.0, view_range: -1.0, width: 0, height: 0, scale_y: 1.0, data_ptr: 0 }
    }
}

/// View state: persistent zoom & pan
pub struct WaveformView {
    pub view_start_ms: f64,
    pub view_range_ms: f64,
    pub target_view_start_ms: f64,
    pub target_view_range_ms: f64,
    pub scale_y: f32,
    pub drag_target: DragTarget,
    /// Current mouse position in milliseconds (None if mouse is outside waveform)
    pub mouse_ms: Option<f64>,
    pub scroll_accum: f32,
    pub spec_cache: SpecCache,
    pub minimap_cache: MinimapCache,
    pub wave_cache: WaveCache,
    pub srp: bool,
    pub srna: bool,
    pub snap_to_peaks: bool,
    pub spec_height_ratio: f32, // 0.0 to 1.0 (portion of available height for spec)
    pub show_minimap: bool,
}

impl Default for WaveformView {
    fn default() -> Self {
        Self {
            view_start_ms: 0.0,
            view_range_ms: 500.0,
            target_view_start_ms: 0.0,
            target_view_range_ms: 500.0,
            scale_y: 1.0,
            drag_target: DragTarget::None,
            mouse_ms: None,
            scroll_accum: 0.0,
            spec_cache: SpecCache::default(),
            minimap_cache: MinimapCache::default(),
            wave_cache: WaveCache::default(),
            srp: false,
            srna: false,
            snap_to_peaks: false,
            spec_height_ratio: 0.45,
            show_minimap: true,
        }
    }
}

impl WaveformView {
    pub fn reset_to(&mut self, duration_ms: f64) {
        self.view_start_ms = 0.0;
        self.view_range_ms = duration_ms.min(1000.0);
        self.target_view_start_ms = 0.0;
        self.target_view_range_ms = self.view_range_ms;
        self.scale_y = 1.0;
        self.drag_target = DragTarget::None;
        self.scroll_accum = 0.0;
    }
}

fn color_for(t: DragTarget) -> Color32 {
    match t {
        DragTarget::Offset    => Color32::from_rgb(86,  156, 214),  // Azul
        DragTarget::Preutter  => Color32::from_rgb(235, 87,  87),   // Vermelho
        DragTarget::Overlap   => Color32::from_rgb(80,  200, 120),  // Verde
        DragTarget::Consonant => Color32::from_rgb(200, 110, 160),  // Rosa mais suave
        DragTarget::Cutoff    => Color32::from_rgb(179, 122, 235),  // Roxo
        DragTarget::None      => Color32::WHITE,
    }
}

fn label_for(t: DragTarget) -> &'static str {
    match t {
        DragTarget::Offset    => "Offset",
        DragTarget::Preutter  => "Preutt",
        DragTarget::Overlap   => "Overlap",
        DragTarget::Consonant => "Consonant",
        DragTarget::Cutoff    => "Cutoff",
        DragTarget::None      => "",
    }
}

/// Convert ms → pixel x, using plain f64 arguments to avoid borrow issues.
#[inline]
fn ms_to_x(ms: f64, view_start: f64, view_range: f64, rect: &egui::Rect) -> f32 {
    let t = (ms - view_start) / view_range;
    rect.left() + t as f32 * rect.width()
}

/// Draw the waveform + parameter markers. Returns true if a parameter changed.
pub fn draw_waveform(
    ui: &mut Ui,
    wav: &WavData,
    spec_data: Option<&SpectrogramData>,
    view: &mut WaveformView,
    entry: &mut OtoEntry,
    playback_cursor: Option<f64>,
    spec_settings: &SpectrogramSettings,
    wave_settings: &WaveformSettings,
) -> InteractionResult {
    let mut nav_delta = 0;
    let available = ui.available_size();
    let total_h = available.y;
    let mini_reserved_h = if view.show_minimap { 45.0 } else { 0.0 };
    let main_h = (total_h - mini_reserved_h).max(100.0);
    
    let (response, painter) = ui.allocate_painter(egui::vec2(available.x, main_h), Sense::click_and_drag());
    let rect = response.rect;

    let (mini_resp, mini_painter) = if view.show_minimap {
        ui.add_space(5.0);
        let resp = ui.allocate_response(egui::vec2(available.x, 40.0), Sense::click_and_drag());
        let p = ui.painter().with_clip_rect(resp.rect);
        (Some(resp), Some(p))
    } else {
        (None, None)
    };

    // ── Pre-calculate Layout ──────────────────────────────────────────────
    let dur = wav.duration_ms;
    let has_spec = spec_data.is_some();
    let mini_h = if view.show_minimap { 28.0 } else { 0.0 };
    let gap = if has_spec { 26.0 } else { 0.0 };
    let avail_h = rect.height() - mini_h - gap;
    let wave_h = if has_spec { avail_h * (1.0 - view.spec_height_ratio) } else { avail_h };
    
    let wave_outer_rect = Rect::from_min_max(rect.min, Pos2::new(rect.max.x, rect.min.y + wave_h));
    let wave_rect = wave_outer_rect.shrink(2.0);
    
    let axis_rect = Rect::from_min_max(Pos2::new(rect.min.x, wave_outer_rect.max.y), Pos2::new(rect.max.x, wave_outer_rect.max.y + gap));
    let spec_outer_rect = if has_spec { Rect::from_min_max(Pos2::new(rect.min.x, axis_rect.max.y), rect.max) } else { Rect::NOTHING };
    
    let (mini_outer_rect, mini_rect) = if let Some(ref resp) = mini_resp {
        let r = resp.rect;
        (r, r.shrink2(egui::vec2(2.0, 4.0)))
    } else {
        (Rect::NOTHING, Rect::NOTHING)
    };

    // ── Resizer Interaction ───────────────────────────────────────────────
    if has_spec {
        let resizer_id = ui.id().with("spec_resizer");
        let resizer_resp = ui.interact(axis_rect, resizer_id, egui::Sense::drag());
        if resizer_resp.dragged() {
            let dy = resizer_resp.drag_delta().y;
            let ratio_delta = dy / avail_h;
            view.spec_height_ratio = (view.spec_height_ratio - ratio_delta).clamp(0.1, 0.9);
        }
        if resizer_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
    }

    // Ensure we don't paint the whole background single color if not needed
    // painter.rect_filled(rect, 0.0, Color32::from_rgb(18, 18, 28));

    // ── Interaction: Zoom, Pan, Minimap ──────────────────────────────────
    if response.hovered() {
        // Track mouse position in ms (on screen)
        if let Some(pos) = response.hover_pos() {
            let t = (pos.x - wave_rect.left()) as f64 / wave_rect.width() as f64;
            view.mouse_ms = Some(view.view_start_ms + t * view.view_range_ms);
        }

        let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
        let scroll_x = ui.input(|i| i.smooth_scroll_delta.x);
        let zoom = ui.input(|i| i.zoom_delta());
        let mods = ui.input(|i| i.modifiers);
        
        let mut zoom_factor = 1.0;
        if mods.ctrl {
            if zoom != 1.0 { 
                zoom_factor = (1.0 / zoom) as f64; 
            } else if scroll_y.abs() > 0.1 { 
                zoom_factor = (-scroll_y / 300.0).exp() as f64; 
            }
        }

        if zoom_factor != 1.0 {
            let mut center = view.target_view_start_ms + view.target_view_range_ms * 0.5;
            if let Some(pos) = response.hover_pos() {
                let t = (pos.x - wave_rect.left()) as f64 / wave_rect.width() as f64;
                center = view.view_start_ms + t * view.view_range_ms;
            }
            view.target_view_range_ms = (view.target_view_range_ms * zoom_factor).clamp(10.0, wav.duration_ms.max(10.0));
            if let Some(pos) = response.hover_pos() {
                let t = (pos.x - wave_rect.left()) as f64 / wave_rect.width() as f64;
                view.target_view_start_ms = center - t * view.target_view_range_ms;
            }
        } else if mods.shift && (scroll_x.abs() > 0.1 || scroll_y.abs() > 0.1) {
            let actual_scroll = if scroll_x.abs() > 0.1 { scroll_x } else { scroll_y };
            let pan = -actual_scroll as f64 * (view.target_view_range_ms / 1000.0);
            view.target_view_start_ms += pan;
        } else if scroll_y.abs() > 0.1 {
            if mods.alt {
                let factor = (scroll_y / 150.0).exp() as f32;
                view.scale_y = (view.scale_y * factor).clamp(0.1, 10.0);
            } else {
                view.scroll_accum += scroll_y;
                if view.scroll_accum > 40.0 {
                    nav_delta = -1;
                    view.scroll_accum = 0.0;
                } else if view.scroll_accum < -40.0 {
                    nav_delta = 1;
                    view.scroll_accum = 0.0;
                }
            }
        } else {
            view.scroll_accum = 0.0;
        }
        
        if zoom_factor != 1.0 || scroll_x.abs() > 0.1 || (mods.shift && scroll_y.abs() > 0.1) {
            view.target_view_start_ms = view.target_view_start_ms.clamp(0.0, (wav.duration_ms - view.target_view_range_ms).max(0.0));
        }
    } else {
        view.mouse_ms = None;
    }

    if response.dragged() && view.drag_target == DragTarget::None {
        let vr = view.target_view_range_ms;
        let dms = -(response.drag_delta().x as f64 / wave_rect.width() as f64) * vr;
        view.target_view_start_ms = (view.target_view_start_ms + dms).clamp(0.0, (dur - vr).max(0.0));
        view.view_start_ms = view.target_view_start_ms;
    }

    if view.show_minimap {
        let mini_resp = ui.interact(mini_rect, ui.id().with("minimap"), egui::Sense::click_and_drag());
        if mini_resp.dragged() || mini_resp.clicked() {
            if let Some(pos) = mini_resp.interact_pointer_pos() {
                let t = ((pos.x - mini_rect.left()) / mini_rect.width()).clamp(0.0, 1.0) as f64;
                view.view_start_ms = (t * dur - view.view_range_ms * 0.5).clamp(0.0, (dur - view.view_range_ms).max(0.0));
                view.target_view_start_ms = view.view_start_ms;
            }
        }
    }

    // ── Smooth Zoom Interpolation ─────────────────────────────────────────
    let dt = (ui.input(|i| i.stable_dt) as f64).min(0.032);
    let lerp_factor = 1.0 - (-18.0 * dt).exp();
    
    let start_diff = (view.target_view_start_ms - view.view_start_ms).abs();
    let range_diff = (view.target_view_range_ms - view.view_range_ms).abs();
    let is_animating = start_diff > 0.05 || range_diff > 0.05;

    if is_animating {
        view.view_start_ms += (view.target_view_start_ms - view.view_start_ms) * lerp_factor;
        view.view_range_ms += (view.target_view_range_ms - view.view_range_ms) * lerp_factor;
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));
    } else {
        view.view_start_ms = view.target_view_start_ms;
        view.view_range_ms = view.target_view_range_ms;
    }

    // ── Final Snapshots for Rendering ─────────────────────────────────────
    let vs = view.view_start_ms;
    let vr = view.view_range_ms;
    let ve = vs + vr;

    // ── Minimap Interaction & Pre-calculate Minimap Texture ──────────────
    if view.show_minimap {
        let mini_w = mini_rect.width().max(1.0) as usize;
        let mini_h = mini_rect.height().max(1.0) as usize;
        let cache = &mut view.minimap_cache;
        let cur_wav_ptr = std::sync::Arc::as_ptr(&wav.samples) as usize;
        
        if cache.texture.is_none() || cache.width != mini_w || cache.data_ptr != cur_wav_ptr {
            let mut img = egui::ColorImage::new([mini_w, mini_h], Color32::TRANSPARENT);
            if mini_w > 0 && !wav.samples.is_empty() {
                let step = (wav.samples.len() / mini_w).max(1);
                let mid_y = mini_h as f32 * 0.5;
                let half_h = mini_h as f32 * 0.45;
                let wc = wave_settings.top_color;
                
                for px in 0..mini_w {
                    let s0 = px * step;
                    let s1 = (s0 + step).min(wav.samples.len());
                    if s0 >= wav.samples.len() { break; }
                    let chunk = &wav.samples[s0..s1];
                    let mx = chunk.iter().fold(0f32, |a, &s| a.max(s.abs()));
                    
                    let y0 = (mid_y - mx.min(1.0) * half_h).round() as i32;
                    let y1 = (mid_y + mx.min(1.0) * half_h).round() as i32;
                    for py in y0.max(0)..(y1.max(0).min(mini_h as i32)) {
                        img[(px, py as usize)] = wc;
                    }
                }
            }
            cache.texture = Some(ui.ctx().load_texture("minimap_cache", img, egui::TextureOptions::LINEAR));
            cache.width = mini_w;
            cache.data_ptr = cur_wav_ptr;
        }
        
        // Draw cached minimap
        if let Some(ref mp) = mini_painter {
            mp.rect_filled(mini_rect, 0.0, Color32::from_rgb(10, 10, 16));
            if let Some(ref tex) = cache.texture {
                mp.image(tex.into(), mini_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
            }
            
            // Minimap Highlight (Visible range)
            let t_start = (vs / dur).clamp(0.0, 1.0) as f32;
            let t_end = ((vs + vr) / dur).clamp(0.0, 1.0) as f32;
            let hl_rect = Rect::from_min_max(
                Pos2::new(mini_rect.left() + t_start * mini_rect.width(), mini_rect.top()),
                Pos2::new(mini_rect.left() + t_end * mini_rect.width(), mini_rect.bottom()),
            );
            mp.rect_filled(hl_rect, 1.0, Color32::from_white_alpha(40));
            mp.rect_stroke(hl_rect, 1.0, Stroke::new(1.0, Color32::from_white_alpha(80)), egui::StrokeKind::Inside);
        }
    }

    let spec_rect = if has_spec { spec_outer_rect.shrink(2.0) } else { Rect::NOTHING };

    // Draw visual windows
    painter.rect_filled(wave_outer_rect, 6.0, Color32::from_rgb(18, 18, 28));
    painter.rect_stroke(wave_outer_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(50, 50, 70)), egui::StrokeKind::Inside);
    
    if has_spec {
        painter.rect_filled(spec_outer_rect, 6.0, Color32::from_rgb(18, 18, 28));
        painter.rect_stroke(spec_outer_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(50, 50, 70)), egui::StrokeKind::Inside);
    }



    // ── Time Axis ─────────────────────────────────────────────────────────
    if has_spec {
        // painter.rect_filled(axis_rect, 0.0, Color32::from_rgb(18, 18, 28)); // leave transparent
        let step = 500.0f64;
        let mut t = (vs / step).ceil() * step;
        while t <= ve {
            let x = ms_to_x(t, vs, vr, &wave_rect);
            painter.line_segment(
                [Pos2::new(x, axis_rect.top()), Pos2::new(x, axis_rect.top() + 5.0)],
                Stroke::new(1.0, Color32::GRAY)
            );
            painter.text(
                Pos2::new(x + 2.0, axis_rect.center().y),
                egui::Align2::LEFT_CENTER,
                format!("{:.1}", t / 1000.0),
                egui::FontId::monospace(10.0),
                Color32::GRAY,
            );
            t += step;
        }
    }

    // ── Spectrogram (pixel-perfect, cached at screen resolution) ──────────
    if let Some(sd) = spec_data {
        let pw = spec_rect.width().max(1.0) as usize;
        let ph = spec_rect.height().max(1.0) as usize;

        let cache = &mut view.spec_cache;
        let diff_start = (cache.view_start - vs).abs();
        let diff_range = (cache.view_range - vr).abs();
        let cur_sd_ptr = std::sync::Arc::as_ptr(&sd.frames_mag) as usize;
        
        // Use the global is_animating to decide if we wait
        let needs_update = cache.texture.is_none()
            || (!is_animating && (diff_start > 0.05 || diff_range > 0.05))
            || cache.width != pw
            || cache.height != ph
            || cache.data_ptr != cur_sd_ptr;

        if needs_update {
            let img = render_spectrogram_view(sd, vs, vr, pw, ph, spec_settings);
            let tex = ui.ctx().load_texture("_spec_live", img, egui::TextureOptions::NEAREST);
            cache.texture = Some(tex);
            cache.view_start = vs;
            cache.view_range = vr;
            cache.width = pw;
            cache.height = ph;
            cache.data_ptr = cur_sd_ptr;
        }

        if let Some(ref tex) = cache.texture {
            // During animation, stretch the old texture to avoid recalculated stutter
            let mut uv = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0));
            if is_animating {
                let s_rel = (vs - cache.view_start) / cache.view_range;
                let e_rel = (ve - cache.view_start) / cache.view_range;
                uv = Rect::from_min_max(Pos2::new(s_rel as f32, 0.0), Pos2::new(e_rel as f32, 1.0));
            }
            painter.image(tex.into(), spec_rect, uv, Color32::WHITE);
        }
    }

    // ── Draw waveform pixels ──────────────────────────────────────────────
    let s_start = ((view.view_start_ms / 1000.0) * wav.sample_rate as f64) as usize;
    let s_end   = (((view.view_start_ms + view.view_range_ms) / 1000.0) * wav.sample_rate as f64)
        .min(wav.samples.len() as f64) as usize;
    let px_w = wave_rect.width() as usize;

    if px_w > 0 && s_end > s_start {
        let step_f64 = (s_end - s_start) as f64 / px_w as f64;
        let mid_y  = wave_rect.center().y;
        let half_h = wave_rect.height() * 0.45 * view.scale_y;
        
        let c_top = wave_settings.top_color;
        let c_bot = wave_settings.bot_color;
        let c_line = wave_settings.line_color;

        let use_spline = match wave_settings.render_mode {
            WaveformRenderMode::Auto => step_f64 < wave_settings.spline_threshold,
            WaveformRenderMode::AlwaysSpline => true,
            WaveformRenderMode::AlwaysBlocks => false,
        };

        if use_spline {
            // Zoomed in: draw exact continuous line
            let mut points = Vec::with_capacity((s_end.saturating_sub(s_start)).min(10000));
            for i in s_start..s_end.min(wav.samples.len()) {
                let x = wave_rect.left() + (i - s_start) as f32 / step_f64 as f32;
                let s = wav.samples[i];
                let y = (mid_y - s * half_h).clamp(wave_rect.top(), wave_rect.bottom());
                points.push(Pos2::new(x, y));
            }
            if points.len() > 1 {
                painter.add(egui::Shape::line(points, Stroke::new(wave_settings.thickness, c_line)));
            }
        } else {
            // Zoomed out: Use WaveCache texture
            let cache = &mut view.wave_cache;
            let cur_wav_ptr = std::sync::Arc::as_ptr(&wav.samples) as usize;
            
            let needs_update = cache.texture.is_none()
                || (!is_animating && (
                    (cache.view_start - vs).abs() > 0.05 
                    || (cache.view_range - vr).abs() > 0.05
                    || (cache.scale_y - view.scale_y).abs() > 0.01
                ))
                || cache.width != px_w
                || cache.height != wave_rect.height() as usize
                || cache.data_ptr != cur_wav_ptr;

            if needs_update {
                let wh = wave_rect.height() as usize;
                let mut img = egui::ColorImage::new([px_w, wh], Color32::TRANSPARENT);
                let mid_px = wh as f32 * 0.5;
                let h_px = wh as f32 * 0.45 * view.scale_y;
                
                for px in 0..px_w {
                    let s0 = s_start + (px as f64 * step_f64) as usize;
                    let mut s1 = s_start + ((px + 1) as f64 * step_f64) as usize;
                    if s1 <= s0 { s1 = s0 + 1; }
                    if s0 >= s_end { break; }
                    let chunk = &wav.samples[s0..s1.min(wav.samples.len())];
                    let (mn, mx) = chunk.iter().fold((0f32, 0f32), |(a, b), &s| (a.min(s), b.max(s)));
                    
                    let y_top = (mid_px - mx * h_px).clamp(0.0, wh as f32 - 1.0) as usize;
                    let y_bot = (mid_px - mn * h_px).clamp(0.0, wh as f32 - 1.0) as usize;
                    
                    let y_mid = mid_px as usize;
                    for py in y_top..=y_mid { img[(px, py)] = c_top; }
                    for py in y_mid..=y_bot { img[(px, py)] = c_bot; }
                }
                cache.texture = Some(ui.ctx().load_texture("wave_cache", img, egui::TextureOptions::LINEAR));
                cache.view_start = vs;
                cache.view_range = vr;
                cache.scale_y = view.scale_y;
                cache.width = px_w;
                cache.height = wh;
                cache.data_ptr = cur_wav_ptr;
            }

            if let Some(ref tex) = cache.texture {
                let mut uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
                if is_animating {
                    let s_rel = (vs - cache.view_start) / cache.view_range;
                    let e_rel = (ve - cache.view_start) / cache.view_range;
                    uv = Rect::from_min_max(pos2(s_rel as f32, 0.0), pos2(e_rel as f32, 1.0));
                }
                painter.image(tex.into(), wave_rect, uv, Color32::WHITE);
            }
        }
    }

    // ── Grid lines ────────────────────────────────────────────────────────
    {
        let step = 50.0f64;
        let mut t = (vs / step).ceil() * step;
        while t <= ve {
            let x = ms_to_x(t, vs, vr, &wave_rect);
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(0.5, Color32::from_rgba_premultiplied(80, 80, 100, 120)),
            );
            painter.text(
                Pos2::new(x + 2.0, rect.top() + 2.0),
                egui::Align2::LEFT_TOP,
                format!("{:.0}ms", t),
                egui::FontId::monospace(9.0),
                Color32::from_rgb(100, 100, 120),
            );
            t += step;
        }
    }

    // ── Helper: cutoff in abs ms ──────────────────────────────────────────
    let abs_cutoff = |cutoff: f64, offset: f64| -> f64 {
        if cutoff < 0.0 { offset - cutoff } else { dur - cutoff }
    };

    // Snapshot param positions (no closures holding borrows of `view`)
    let cutoff_ms = abs_cutoff(entry.cutoff, entry.offset);
    let lines = [
        (DragTarget::Offset,    entry.offset),
        (DragTarget::Preutter,  entry.offset + entry.preutter),
        (DragTarget::Overlap,   entry.offset + entry.overlap),
        (DragTarget::Consonant, entry.offset + entry.consonant),
        (DragTarget::Cutoff,    cutoff_ms),
    ];

    let grab_r = 6.0f32;
    let mut modified = false;

    // ── Drag end ──────────────────────────────────────────────────────────
    if !response.dragged() && view.drag_target != DragTarget::None {
        view.drag_target = DragTarget::None;
    }

    // ── Drag start: pick nearest line ─────────────────────────────────────
    if response.drag_started() {
        if let Some(pos) = response.interact_pointer_pos() {
            let mut best = grab_r * 2.0;
            let mut pick = DragTarget::None;
            for (t, ms) in &lines {
                let d = (pos.x - ms_to_x(*ms, vs, vr, &wave_rect)).abs();
                if d < best { best = d; pick = *t; }
            }
            view.drag_target = pick;
        }
    }

    // ── Apply drag to params ──────────────────────────────────────────────
    if response.dragged() && view.drag_target != DragTarget::None {
        if let Some(pos) = response.interact_pointer_pos() {
            let mut ms = (view.view_start_ms + ((pos.x - wave_rect.left()) as f64 / wave_rect.width() as f64) * view.view_range_ms).round();
            
            if view.snap_to_peaks {
                let center_idx = (ms * wav.sample_rate as f64 / 1000.0) as usize;
                let window = (wav.sample_rate as f64 * 0.01) as usize; // 10ms window
                let start = center_idx.saturating_sub(window);
                let end = (center_idx + window).min(wav.samples.len());
                let mut max_abs = 0.0;
                let mut peak_idx = center_idx;
                for i in start..end {
                    let v = wav.samples[i].abs();
                    if v > max_abs {
                        max_abs = v;
                        peak_idx = i;
                    }
                }
                ms = peak_idx as f64 * 1000.0 / wav.sample_rate as f64;
            }

            let c_ms = abs_cutoff(entry.cutoff, entry.offset);
            match view.drag_target {
                DragTarget::Offset    => { 
                    let old_offset = entry.offset;
                    let new_offset = ms.max(0.0);
                    let delta = new_offset - old_offset;

                    if view.srna {
                        // SRnA ON (Independent): Other lines stay fixed in absolute time
                        // This means relative values must change by -delta
                        entry.offset = new_offset;
                        entry.overlap = (entry.overlap - delta).max(0.0);
                        entry.preutter = (entry.preutter - delta).max(0.0);
                        entry.consonant = (entry.consonant - delta).max(0.0);
                        if entry.cutoff < 0.0 {
                            entry.cutoff += delta;
                        }
                    } else {
                        // SRnA OFF (Standard): Everything moves together
                        entry.offset = new_offset;
                        // For Cutoff > 0 (relative to end), we must adjust it by -delta to shift it.
                        if entry.cutoff >= 0.0 {
                            entry.cutoff = (entry.cutoff - delta).max(0.0);
                        }
                    }
                }
                DragTarget::Preutter  => { 
                    if view.srp {
                        // SRP ON (Grouped): Moving preutterance moves everything else too
                        let old_abs = entry.offset + entry.preutter;
                        let delta = ms - old_abs;
                        
                        let old_off = entry.offset;
                        entry.offset = (entry.offset + delta).max(0.0);
                        let off_real_delta = entry.offset - old_off;
                        
                        // In SRP mode (everything moves together):
                        // Relative values for preutter, overlap, consonant stay same => they shift with offset.
                        // For Cutoff > 0 (relative to end), we must adjust it by -delta to shift it.
                        if entry.cutoff >= 0.0 {
                            entry.cutoff = (entry.cutoff - off_real_delta).max(0.0);
                        }
                    } else {
                        // SRP OFF: Independent preutter move
                        let p_ms = ms.max(entry.offset).min(c_ms);
                        entry.preutter = p_ms - entry.offset;
                    }
                }
                DragTarget::Overlap   => { 
                    let o_ms = ms.min(c_ms);
                    entry.overlap = o_ms - entry.offset;
                }
                DragTarget::Consonant => { 
                    let cs_ms = ms.max(entry.offset).min(c_ms);
                    entry.consonant = cs_ms - entry.offset; 
                }
                DragTarget::Cutoff => {
                    let max_rel = entry.consonant.max(entry.preutter).max(entry.overlap);
                    let min_ms = entry.offset + max_rel;
                    
                    let mods = ui.input(|i| i.modifiers);
                    if mods.alt {
                        // Toggle mode while dragging
                        if entry.cutoff < 0.0 {
                            entry.cutoff = (dur - ms).max(0.0);
                        } else {
                            entry.cutoff = -(ms - entry.offset);
                        }
                    } else {
                        if entry.cutoff < 0.0 {
                            let target_ms = ms.max(min_ms + 1.0);
                            entry.cutoff = -(target_ms - entry.offset);
                        } else {
                            let target_ms = ms.max(min_ms);
                            entry.cutoff = (dur - target_ms).max(0.0);
                        }
                    }
                }
                DragTarget::None => {}
            }
            modified = true;
        }
    }


    // Snapshot after pan/edit (unused, we use unified vs/vr)
    let cutoff_ms2 = abs_cutoff(entry.cutoff, entry.offset);
    let lines2 = [
        (DragTarget::Offset,    entry.offset),
        (DragTarget::Preutter,  entry.offset + entry.preutter),
        (DragTarget::Overlap,   entry.offset + entry.overlap),
        (DragTarget::Consonant, entry.offset + entry.consonant),
        (DragTarget::Cutoff,    cutoff_ms2),
    ];

    // ── Consonant shaded region (light pink from Offset to Consonant) ─────
    {
        let t = mini_outer_rect.top();
        let cons_x_left  = ms_to_x(entry.offset, vs, vr, &wave_rect).clamp(wave_rect.left(), wave_rect.right());
        let cons_x_right = ms_to_x(entry.offset + entry.consonant, vs, vr, &wave_rect).clamp(wave_rect.left(), wave_rect.right());
        if cons_x_right > cons_x_left {
            painter.rect_filled(
                Rect::from_min_max(Pos2::new(cons_x_left, rect.top()), Pos2::new(cons_x_right, t)),
                0.0,
                Color32::from_rgba_unmultiplied(200, 110, 160, 38), // 15% real (usando unmultiplied)
            );
        }
    }

    // ── Shaded inactive regions ───────────────────────────────────────────
    let ox = ms_to_x(entry.offset, vs, vr, &wave_rect).clamp(wave_rect.left(), wave_rect.right());
    let cx = ms_to_x(cutoff_ms2,   vs, vr, &wave_rect).clamp(wave_rect.left(), wave_rect.right());
    let st = mini_outer_rect.top();
    let shade = Color32::from_rgba_premultiplied(0, 0, 0, 100);
    painter.rect_filled(
        Rect::from_min_max(Pos2::new(rect.left(), rect.top()), Pos2::new(ox, st)),
        0.0, shade,
    );
    painter.rect_filled(
        Rect::from_min_max(Pos2::new(cx, rect.top()), Pos2::new(rect.right(), st)),
        0.0, shade,
    );

    // ── Interaction: play cursor & Loading Bar ──────────────────────────
    if is_animating {
        let bar_y = wave_outer_rect.top() + 1.0;
        let bar_h = 2.5;
        let color = Color32::from_rgb(137, 180, 250); // Catppuccin Blue
        
        // Moving glow effect
        let t = (ui.input(|i| i.time) * 2.0).fract() as f32;
        let w = wave_outer_rect.width();
        let x0 = wave_outer_rect.left() + (t - 0.2).max(0.0) * w;
        let x1 = wave_outer_rect.left() + (t + 0.2).min(1.0) * w;
        
        painter.line_segment(
            [Pos2::new(wave_outer_rect.left(), bar_y), Pos2::new(wave_outer_rect.right(), bar_y)],
            Stroke::new(1.0, color.gamma_multiply(0.2))
        );
        painter.line_segment(
            [Pos2::new(x0, bar_y), Pos2::new(x1, bar_y)],
            Stroke::new(bar_h, color)
        );
    }

    // ── Playback Cursor (Dotted Yellow Line) ─────────────────────────────
    if let Some(cursor_ms) = playback_cursor {
        let px = ms_to_x(cursor_ms, vs, vr, &wave_rect);
        if px >= wave_rect.left() && px <= wave_rect.right() {
            let mut y = rect.top();
            while y < rect.bottom() {
                painter.line_segment(
                    [Pos2::new(px, y), Pos2::new(px, (y + 6.0).min(rect.bottom()))],
                    Stroke::new(2.0, Color32::YELLOW),
                );
                y += 12.0;
            }
        }
    }

    // ── Draw parameter lines (TOP LAYER) ──────────────────────────────────
    for (target, ms) in &lines2 {
        let x = ms_to_x(*ms, vs, vr, &wave_rect);
        if x < wave_rect.left() - 2.0 || x > wave_rect.right() + 2.0 { continue; }
        
        let col   = color_for(*target);
        let width = if view.drag_target == *target { 2.5 } else { 1.5 };
        
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, mini_outer_rect.top())],
            Stroke::new(width, col),
        );

        let is_bottom = *target == DragTarget::Overlap || *target == DragTarget::Consonant;
        let (circle_y, text_pos, align) = if is_bottom {
            let b = if view.show_minimap { mini_outer_rect.top() } else { rect.bottom() };
            (b - 12.0, Pos2::new(x + 4.0, b - 4.0), egui::Align2::LEFT_BOTTOM)
        } else {
            (rect.top() + 12.0, Pos2::new(x + 4.0, rect.top() + 4.0), egui::Align2::LEFT_TOP)
        };

        painter.circle_filled(Pos2::new(x, circle_y), grab_r, col);
        painter.text(
            text_pos,
            align,
            label_for(*target),
            egui::FontId::monospace(10.0),
            col,
        );
    }

    InteractionResult {
        drag_started: response.drag_started(),
        drag_released: response.drag_stopped(),
        clicked: response.clicked(),
        modified,
        nav_delta,
    }
}
