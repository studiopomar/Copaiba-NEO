use std::path::PathBuf;
use std::sync::Arc;

use egui_i18n::tr;
use rodio::{OutputStream, Sink};

use crate::audio::{load_wav, WavData};
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn stop_playback(&mut self) {
        if let Some(sink) = &self.audio.sink {
            sink.stop();
        }
        self.audio.playback_start = None;
        self.audio.playback_limit_ms = None;
    }

    pub fn init_audio(&mut self) {
        if self.audio.sink.is_some() { return; }
        if let Ok((stream, handle)) = OutputStream::try_default() {
            if let Ok(sink) = Sink::try_new(&handle) {
                self.audio._stream = Some(stream);
                self.audio._stream_handle = Some(handle);
                self.audio.sink = Some(Arc::new(sink));
            }
        }
    }

    pub fn play_current_segment(&mut self, full: bool) {
        self.init_audio();
        let sink = match &self.audio.sink {
            Some(s) => s,
            None => { self.ui.status = tr!("audio.error.no_device").to_string(); return; }
        };

        let tab = self.cur();
        if let Some(&idx) = tab.filtered.get(tab.selected) {
            if let Some(entry) = tab.entries.get(idx) {
                let full_path = tab.oto_dir.as_ref().map(|d| d.join(&entry.filename).to_string_lossy().to_string()).unwrap_or_else(|| entry.filename.clone());
                if let Some(wav) = self.wav_cache.get(&full_path) {
                    sink.stop();
                    let start_idx = if full { 0 } else {
                        ((entry.offset / 1000.0) * wav.sample_rate as f64) as usize
                    };
                    let dur = wav.duration_ms;
                    let abs_cutoff = if entry.cutoff < 0.0 { entry.offset - entry.cutoff } else { dur - entry.cutoff };
                    let end_idx = if full { wav.samples.len() } else {
                        ((abs_cutoff / 1000.0) * wav.sample_rate as f64).min(wav.samples.len() as f64) as usize
                    };

                    if end_idx > start_idx {
                        let samples = wav.samples[start_idx..end_idx].to_vec();
                        let source = rodio::buffer::SamplesBuffer::new(1, wav.sample_rate, samples);
                        sink.append(source);
                        sink.play();
                        self.audio.playback_offset_ms = (start_idx as f64 / wav.sample_rate as f64) * 1000.0;
                        self.audio.playback_limit_ms = if full { None } else { Some((end_idx as f64 / wav.sample_rate as f64) * 1000.0) };
                        self.audio.playback_start = Some(std::time::Instant::now());
                    }
                }
            }
        }
    }

    pub fn play_wav_data(&mut self, wav: WavData) {
        self.init_audio();
        if let Some(sink) = &self.audio.sink {
            sink.stop();
            let samples = (*wav.samples).clone();
            let source = rodio::buffer::SamplesBuffer::new(1, wav.sample_rate, samples);
            sink.append(source);
            self.audio.playback_start = Some(std::time::Instant::now());
            self.audio.playback_offset_ms = 0.0;
        }
    }

    pub fn resample_current(&mut self) {
        if self.config.resampler_path.is_none() {
            if let Some(path) = rfd::FileDialog::new()
                .set_title(tr!("audio.resampler.select_file"))
                .add_filter("Executáveis", &["exe", "bin", "sh"])
                .pick_file() {
                self.config.resampler_path = Some(path);
            } else {
                return;
            }
        }

        let tab = self.cur();
        if let Some(&idx) = tab.filtered.get(tab.selected) {
            let entry = tab.entries[idx].clone();

            let resampler_exe = self.config.resampler_path.as_ref().unwrap().clone();
            let input_wav = if let Some(dir) = &tab.oto_dir { dir.join(&entry.filename) } else { PathBuf::from(&entry.filename) };
            let output_wav = std::env::temp_dir().join("copaiba_resample.wav");

            let mut cmd = std::process::Command::new("wine");
            if std::process::Command::new("wine").arg("--version").output().is_err() {
                cmd = std::process::Command::new(&resampler_exe);
            } else {
                let _ = cmd.arg(&resampler_exe);
            }

            cmd.arg(&input_wav);
            cmd.arg(&output_wav);
            cmd.arg(&self.config.test_pitch);
            cmd.arg("100");
            cmd.arg("");
            cmd.arg(format!("{:.0}", entry.offset));
            cmd.arg(format!("{:.0}", self.config.test_duration_ms));
            cmd.arg(format!("{:.0}", entry.consonant));
            cmd.arg(format!("{:.0}", entry.cutoff));
            cmd.arg("100");
            cmd.arg("0");
            cmd.arg("120");
            cmd.arg("AA");

            self.ui.status = tr!("audio.resampler.status.resampling").to_string();
            match cmd.output() {
                Ok(output) => {
                    if output.status.success() {
                        match load_wav(&output_wav) {
                            Ok(ws) => {
                                self.play_wav_data(ws.wav);
                                self.ui.status = format!("{} {}", tr!("audio.resampler.status.success"), entry.alias);
                            }
                            Err(e) => { self.ui.status = format!("{} {}", tr!("audio.resampler.status.load_error"), e); }
                        }
                    } else {
                        self.ui.status = format!("{} {}", tr!("audio.resampler.status.error"), String::from_utf8_lossy(&output.stderr));
                    }
                }
                Err(e) => { self.ui.status = format!("{} {}", tr!("audio.resampler.status.exec_error"), e); }
            }
        }
    }
}
