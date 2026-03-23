// spectrogram.rs — Pixel-perfect spectrogram rendering

/// All user-configurable rendering parameters
#[derive(Clone, Debug)]
pub struct SpectrogramSettings {
    pub fft_size: usize,       // 512, 1024, 2048, 4096, 8192
    pub hop_size: usize,       // 64, 128, 256, 512
    pub min_freq: f64,         // Hz floor
    pub max_freq: f64,         // Hz ceiling (0 = Nyquist)
    pub min_db: f32,           // noise floor in dB (e.g. -90)
    pub gamma: f32,            // 0.1 (sharp) to 1.0 (linear)
    pub colormap: ColormapKind,
    pub adaptive_norm: bool,   // per-column normalization vs global
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColormapKind {
    Fire,
    Inferno,
    Grayscale,
    Viridis,
}

impl Default for SpectrogramSettings {
    fn default() -> Self {
        Self {
            fft_size: 1024,
            hop_size: 128,
            min_freq: 80.0,
            max_freq: 24000.0,
            min_db: -65.0,
            gamma: 1.5,
            colormap: ColormapKind::Fire,
            adaptive_norm: false,
        }
    }
}

/// Pre-computed FFT data that can be sliced for any view range.
#[derive(Clone)]
pub struct SpectrogramData {
    pub frames_mag: std::sync::Arc<Vec<Vec<f32>>>,
    pub global_peak: f32,
    pub num_bins: usize,
    pub hop_size: usize,
    pub sample_rate: f64,
}

/// Compute FFT magnitudes for the entire audio.
pub fn compute_spectrogram_data(samples: &[f32], sample_rate: u32, settings: &SpectrogramSettings) -> Option<SpectrogramData> {
    if samples.is_empty() { return None; }

    let fft_size = settings.fft_size;
    let hop_size = settings.hop_size;

    if samples.len() < fft_size { return None; }

    let window: Vec<f32> = (0..fft_size)
        .map(|n| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * n as f32 / fft_size as f32).cos()))
        .collect();

    let mut planner = realfft::RealFftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);
    let mut input = fft.make_input_vec();
    let mut output = fft.make_output_vec();

    let half_fft = fft_size / 2;
    let mut padded = Vec::with_capacity(samples.len() + fft_size);
    padded.resize(half_fft, 0.0);
    padded.extend_from_slice(samples);
    padded.resize(padded.len() + half_fft, 0.0);

    let num_frames = (padded.len() - fft_size) / hop_size + 1;
    let num_bins = fft_size / 2;

    let mut frames_mag = Vec::with_capacity(num_frames);
    let mut global_peak: f32 = 1e-10;

    for i in 0..num_frames {
        let start = i * hop_size;
        for j in 0..fft_size {
            input[j] = padded[start + j] * window[j];
        }
        let _ = fft.process(&mut input, &mut output);

        let mut frame = Vec::with_capacity(num_bins);
        for b in 0..num_bins {
            let mag = output[b].norm();
            if mag > global_peak { global_peak = mag; }
            frame.push(mag);
        }
        frames_mag.push(frame);
    }

    Some(SpectrogramData {
        frames_mag: std::sync::Arc::new(frames_mag),
        global_peak,
        num_bins,
        hop_size,
        sample_rate: sample_rate as f64,
    })
}

/// Render the spectrogram for the visible time range at exact screen resolution.
pub fn render_spectrogram_view(
    data: &SpectrogramData,
    view_start_ms: f64,
    view_range_ms: f64,
    pixel_width: usize,
    pixel_height: usize,
    settings: &SpectrogramSettings,
) -> egui::ColorImage {
    if pixel_width == 0 || pixel_height == 0 || data.frames_mag.is_empty() {
        return egui::ColorImage::new([1, 1], egui::Color32::BLACK);
    }

    let global_ref_db = data.global_peak.max(1e-10).log10() * 20.0;
    let min_db = settings.min_db;
    let max_db = 0.0_f32;
    let range = max_db - min_db;
    let gamma = settings.gamma;

    let sr = data.sample_rate as f64;
    let hop = data.hop_size as f64;
    let total_frames = data.frames_mag.len();
    let num_bins = data.num_bins;
    let nyquist = sr / 2.0;

    let min_freq = settings.min_freq.clamp(1.0, nyquist - 1.0);
    let max_freq = if settings.max_freq <= 0.0 { nyquist } else { settings.max_freq.clamp(min_freq + 1.0, nyquist) };
    let log_min = min_freq.log2();
    let log_max = max_freq.log2();

    let mut pixels = vec![egui::Color32::BLACK; pixel_width * pixel_height];

    for px in 0..pixel_width {
        // Sample at the center of the pixel for better alignment
        let t_ms = view_start_ms + ((px as f64 + 0.5) / pixel_width as f64) * view_range_ms;
        
        let center_sample = t_ms * sr / 1000.0;
        let frame_f = center_sample / hop;
        
        // Time interpolation bounds (Bicubic requires 4 points)
        let f1 = (frame_f as usize).min(total_frames.saturating_sub(1));
        let f0 = f1.saturating_sub(1);
        let f2 = (f1 + 1).min(total_frames.saturating_sub(1));
        let f3 = (f1 + 2).min(total_frames.saturating_sub(1));
        let t_alpha = (frame_f - f1 as f64) as f32;
        
        let frame0 = &data.frames_mag[f0];
        let frame1 = &data.frames_mag[f1];
        let frame2 = &data.frames_mag[f2];
        let frame3 = &data.frames_mag[f3];

        let ref_db = if settings.adaptive_norm {
            let col_peak1 = frame1.iter().copied().fold(1e-10_f32, f32::max);
            let col_peak2 = frame2.iter().copied().fold(1e-10_f32, f32::max);
            let p = col_peak1 * (1.0 - t_alpha) + col_peak2 * t_alpha;
            p.log10() * 20.0
        } else {
            global_ref_db
        };

        for py in 0..pixel_height {
            let frac = (pixel_height - 1 - py) as f64 / pixel_height as f64;
            let freq = 2.0_f64.powf(log_min + frac * (log_max - log_min));
            let bin_f = freq / nyquist * num_bins as f64;

            let bin_idx = (bin_f as usize).min(num_bins.saturating_sub(1));
            let f_alpha = (bin_f - bin_idx as f64) as f32;
            
            // Bicubic interpolation
            let m0 = interpolate_frame(frame0, bin_idx, num_bins, f_alpha);
            let m1 = interpolate_frame(frame1, bin_idx, num_bins, f_alpha);
            let m2 = interpolate_frame(frame2, bin_idx, num_bins, f_alpha);
            let m3 = interpolate_frame(frame3, bin_idx, num_bins, f_alpha);
            let mag = catmull_rom(m0, m1, m2, m3, t_alpha);

            let db = (mag.max(1e-10).log10() * 20.0) - ref_db;
            let t = ((db.clamp(min_db, max_db) - min_db) / range).powf(gamma);

            pixels[py * pixel_width + px] = apply_colormap(t, &settings.colormap);
        }
    }

    egui::ColorImage { size: [pixel_width, pixel_height], pixels }
}

fn apply_colormap(t: f32, kind: &ColormapKind) -> egui::Color32 {
    match kind {
        ColormapKind::Fire     => fire_colormap(t),
        ColormapKind::Inferno  => inferno_colormap(t),
        ColormapKind::Grayscale => {
            let v = (t.clamp(0.0, 1.0) * 255.0) as u8;
            egui::Color32::from_rgb(v, v, v)
        },
        ColormapKind::Viridis  => viridis_colormap(t),
    }
}

fn fire_colormap(t: f32) -> egui::Color32 {
    const STOPS: [(f32, f32, f32, f32); 8] = [
        (0.00,   0.0,   0.0,   0.0),
        (0.10,  15.0,   0.0,  40.0),
        (0.25,  80.0,   0.0,  90.0),
        (0.42, 185.0,   0.0,  35.0),
        (0.58, 235.0,  65.0,   0.0),
        (0.74, 255.0, 155.0,   0.0),
        (0.88, 255.0, 225.0,  55.0),
        (1.00, 255.0, 255.0, 230.0),
    ];
    gradient(t, &STOPS)
}

fn inferno_colormap(t: f32) -> egui::Color32 {
    const STOPS: [(f32, f32, f32, f32); 7] = [
        (0.00,   0.0,   0.0,   4.0),
        (0.14,  12.0,   7.0,  55.0),
        (0.30,  72.0,  12.0, 108.0),
        (0.48, 158.0,  42.0,  72.0),
        (0.65, 220.0, 100.0,  20.0),
        (0.82, 252.0, 195.0,  56.0),
        (1.00, 252.0, 254.0, 164.0),
    ];
    gradient(t, &STOPS)
}

fn viridis_colormap(t: f32) -> egui::Color32 {
    const STOPS: [(f32, f32, f32, f32); 6] = [
        (0.00,  68.0,   1.0,  84.0),
        (0.20,  59.0,  82.0, 139.0),
        (0.40,  33.0, 145.0, 140.0),
        (0.60,  94.0, 201.0,  98.0),
        (0.80, 178.0, 220.0,  53.0),
        (1.00, 253.0, 231.0,  37.0),
    ];
    gradient(t, &STOPS)
}

fn gradient(t: f32, stops: &[(f32, f32, f32, f32)]) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    let mut i = 0;
    while i < stops.len() - 2 && t > stops[i + 1].0 { i += 1; }
    let (t0, r0, g0, b0) = stops[i];
    let (t1, r1, g1, b1) = stops[i + 1];
    let f = if (t1 - t0).abs() < 1e-6 { 0.0 } else { ((t - t0) / (t1 - t0)).clamp(0.0, 1.0) };
    egui::Color32::from_rgb(
        (r0 + f * (r1 - r0)).clamp(0.0, 255.0) as u8,
        (g0 + f * (g1 - g0)).clamp(0.0, 255.0) as u8,
        (b0 + f * (b1 - b0)).clamp(0.0, 255.0) as u8,
    )
}

#[inline(always)]
fn catmull_rom(p0: f32, p1: f32, p2: f32, p3: f32, x: f32) -> f32 {
    let x2 = x * x;
    let x3 = x2 * x;
    let v = 0.5 * (
        (2.0 * p1) +
        (-p0 + p2) * x +
        (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * x2 +
        (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * x3
    );
    v.max(0.0) // magnitude cannot be negative
}

#[inline(always)]
fn interpolate_frame(frame: &[f32], b0: usize, num_bins: usize, alpha: f32) -> f32 {
    let p0 = frame[b0.saturating_sub(1)];
    let p1 = frame[b0];
    let p2 = frame[(b0 + 1).min(num_bins - 1)];
    let p3 = frame[(b0 + 2).min(num_bins - 1)];
    catmull_rom(p0, p1, p2, p3, alpha)
}
