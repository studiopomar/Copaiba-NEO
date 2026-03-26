use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AutoOtoSettings {
    pub noise_floor_db: f32,
    pub min_silence_ms: f64, // minimal length of silence to trim
    pub margin_ms: f64,      // padding to leave before offset / after cutoff
}

impl Default for AutoOtoSettings {
    fn default() -> Self {
        Self {
            noise_floor_db: -40.0,
            min_silence_ms: 30.0,
            margin_ms: 10.0,
        }
    }
}

pub struct AutoOtoResult {
    pub offset: f64,
    pub consonant: f64,
    pub cutoff: f64, // typically negative (distance from end)
    pub preutterance: f64,
    pub overlap: f64,
}

pub fn compute_auto_oto(samples: &[f32], sample_rate: u32, settings: &AutoOtoSettings) -> Option<AutoOtoResult> {
    if samples.is_empty() { return None; }
    
    // Window size (e.g. 5ms)
    let window_ms = 5.0;
    let window_size = ((window_ms / 1000.0) * sample_rate as f64) as usize;
    if window_size == 0 { return None; }
    
    // Calculate RMS profile
    let mut rms_profile = Vec::with_capacity(samples.len() / window_size + 1);
    let mut max_rms = 0.0_f32;
    
    let mut i = 0;
    while i + window_size <= samples.len() {
        let chunk = &samples[i..i+window_size];
        let sum_sq: f32 = chunk.iter().map(|&s| s * s).sum();
        let rms = (sum_sq / window_size as f32).sqrt();
        rms_profile.push((i, rms));
        max_rms = max_rms.max(rms);
        i += window_size;
    }
    
    if rms_profile.is_empty() { return None; }
    
    let total_dur_ms = (samples.len() as f64 / sample_rate as f64) * 1000.0;
    
    // Convert DB to amplitude threshold
    // Let's assume max_rms is 0 dB reference or 1.0 is 0 dB?
    // Standard approach: 20 * log10(amplitude)
    let threshold_amp = 10.0_f32.powf(settings.noise_floor_db / 20.0);
    
    // 1. Find Offset (Search forward)
    let mut offset_ms = 0.0;
    for &(idx, rms) in &rms_profile {
        if rms > threshold_amp {
            let found_ms = (idx as f64 / sample_rate as f64) * 1000.0;
            offset_ms = (found_ms - settings.margin_ms).max(0.0);
            break;
        }
    }
    
    // 2. Find Cutoff (Search backward)
    let mut cutoff_abs_ms = total_dur_ms;
    for &(idx, rms) in rms_profile.iter().rev() {
        if rms > threshold_amp {
            let found_ms = ((idx + window_size) as f64 / sample_rate as f64) * 1000.0;
            cutoff_abs_ms = (found_ms + settings.margin_ms).min(total_dur_ms);
            break;
        }
    }
    
    let mut cutoff = -(total_dur_ms - cutoff_abs_ms);
    if cutoff.abs() < settings.min_silence_ms {
        cutoff = 0.0; // Don't cutoff if trailing silence is too small
    }
    
    // 3. Find Vowel start (Preutterance & Consonant)
    // Heuristic: The vowel starts around the highest derivative of energy or when energy reaches 50% of max.
    let vowel_threshold = max_rms * 0.4;
    let mut vowel_start_ms = offset_ms;
    for &(idx, rms) in &rms_profile {
        let current_ms = (idx as f64 / sample_rate as f64) * 1000.0;
        if current_ms > offset_ms && rms > vowel_threshold {
            vowel_start_ms = current_ms;
            break;
        }
    }
    
    let preutterance = (vowel_start_ms - offset_ms).max(10.0); // at least 10ms
    let consonant = preutterance + 20.0; // Safe default for CV: 20ms past preutterance
    
    // 4. Overlap
    let overlap = (preutterance / 2.0).clamp(5.0, 30.0);

    Some(AutoOtoResult {
        offset: offset_ms,
        consonant,
        cutoff,
        preutterance,
        overlap,
    })
}

impl crate::app::CopaibaApp {
    pub fn apply_auto_oto_to_selection(&mut self) {
        let settings = self.auto_oto_settings.clone();
        
        // Find indices to process
        let indices_to_process = {
            let tab = self.cur();
            if tab.multi_selection.is_empty() {
                if let Some(mut sel) = tab.filtered.get(tab.selected).copied() {
                    vec![sel]
                } else {
                    Vec::new()
                }
            } else {
                tab.multi_selection.iter().copied().collect()
            }
        };

        if indices_to_process.is_empty() { return; }

        let mut processed_any = false;
        
        for idx in indices_to_process {
            let (full_path, fname) = {
                let tab = self.cur();
                if let Some(entry) = tab.entries.get(idx) {
                    let fname = entry.filename.clone();
                    let full_path = tab.oto_dir.as_ref()
                        .map(|d| d.join(&fname).to_string_lossy().to_string())
                        .unwrap_or(fname.clone());
                    (full_path, fname)
                } else {
                    continue;
                }
            };

            // Needs WAV data
            let mut wav_opt = self.wav_cache.get(&full_path).cloned();
            
            // If not cached, try to load blocking (may be slow but ok for MVP)
            if wav_opt.is_none() {
                if let Some(tab_dir) = self.cur().oto_dir.as_ref() {
                    let path = tab_dir.join(&fname);
                    if let Ok(wav_with_spec) = crate::audio::load_wav(&path) {
                        wav_opt = Some(wav_with_spec.wav);
                    }
                }
            }

            if let Some(wav) = wav_opt {
                if let Some(oto_res) = compute_auto_oto(&wav.samples, wav.sample_rate, &settings) {
                    let tab = self.cur_mut();
                    if let Some(entry) = tab.entries.get_mut(idx) {
                        entry.offset = oto_res.offset;
                        entry.consonant = oto_res.consonant;
                        entry.cutoff = oto_res.cutoff;
                        entry.preutter = oto_res.preutterance;
                        entry.overlap = oto_res.overlap;
                        processed_any = true;
                    }
                }
            }
        }
        
        if processed_any {
            self.ui.status = format!("{} (Auto-Oto)", egui_i18n::tr!("modal.batch.edit.status.success"));
        }
    }
}
