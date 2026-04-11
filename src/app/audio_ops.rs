use std::path::PathBuf;
use std::sync::Arc;

use egui_i18n::tr;
use rodio::{OutputStream, Sink};

use crate::audio::{load_wav, WavData};
use super::state::CopaibaApp;

impl CopaibaApp {
    pub fn log(&mut self, text: impl Into<String>, color: egui::Color32) {
        let msg = text.into();
        self.ui.status = msg.clone();
        self.ui.log_history.push((msg, color));
        if self.ui.log_history.len() > 20 { self.ui.log_history.remove(0); }
    }

    pub fn stop_playback(&mut self) {
        if let Some(sink) = &self.audio.sink {
            sink.stop();
        }
        self.audio.playback_start = None;
        self.audio.playback_limit_ms = None;
    }

    pub fn init_audio(&mut self) {
        if self.audio.sink.is_some() && self.audio.ui_sink.is_some() { return; }
        if let Ok((stream, handle)) = OutputStream::try_default() {
            if let Ok(sink) = Sink::try_new(&handle) {
                if let Ok(ui_sink) = Sink::try_new(&handle) {
                    self.audio._stream = Some(stream);
                    self.audio._stream_handle = Some(handle);
                    self.audio.sink = Some(Arc::new(sink));
                    self.audio.ui_sink = Some(Arc::new(ui_sink));
                }
            }
        }
    }

    pub fn load_ui_sounds(&mut self) {
        let sounds = [
            ("key01", include_bytes!("../../sounds/key01.wav").as_slice()),
            ("key02", include_bytes!("../../sounds/key02.wav").as_slice()),
            ("key03", include_bytes!("../../sounds/key03.wav").as_slice()),
            ("key04", include_bytes!("../../sounds/key04.wav").as_slice()),
            ("key05", include_bytes!("../../sounds/key05.wav").as_slice()),
            ("key06", include_bytes!("../../sounds/key06.wav").as_slice()),
            ("enter", include_bytes!("../../sounds/enter.wav").as_slice()),
            ("space", include_bytes!("../../sounds/space.wav").as_slice()),
        ];

        for (name, bytes) in sounds {
            if let Ok(ws) = crate::audio::load_wav_from_bytes(bytes) {
                self.audio.ui_sounds.insert(name.to_string(), ws.wav);
            }
        }
    }

    pub fn play_ui_sound(&mut self, name: &str) {
        if !self.config.play_ui_sounds { return; }
        self.init_audio();
        if let (Some(sink), Some(wav)) = (&self.audio.ui_sink, self.audio.ui_sounds.get(name)) {
            let samples = (*wav.samples).clone();
            let source = rodio::buffer::SamplesBuffer::new(1, wav.sample_rate, samples);
            sink.append(source);
        }
    }

    pub fn play_key_sound(&mut self) {
        let idx = self.ui.key_sound_idx;
        let name = format!("key{:02}", (idx % 6) + 1);
        self.play_ui_sound(&name);
        self.ui.key_sound_idx = (idx + 1) % 6;
    }

    pub fn play_current_segment(&mut self, full: bool) {
        self.init_audio();
        let sink = match &self.audio.sink {
            Some(s) => s,
            None => { self.log(tr!("audio.error.no_device"), egui::Color32::RED); return; }
        };
        sink.set_volume(self.config.test_volume);

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
                        let mut samples = wav.samples[start_idx..end_idx].to_vec();

                        // Apply Time-Stretching if speed != 1.0 using manual WSOLA
                        let playback_speed = self.audio.playback_speed;
                        if (playback_speed - 1.0).abs() > 0.01 {
                            samples = crate::wsola::wsola_stretch(&samples, playback_speed);
                        }

                        let source = rodio::buffer::SamplesBuffer::new(1, wav.sample_rate, samples);
                        sink.append(source);
                        sink.set_speed(1.0); // Pitch is already preserved by timestretch logic
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
            sink.set_volume(self.config.test_volume);
            let mut samples = (*wav.samples).clone();

            // Apply Time-Stretching if speed != 1.0 using manual WSOLA
            let playback_speed = self.audio.playback_speed;
            if (playback_speed - 1.0).abs() > 0.01 {
                samples = crate::wsola::wsola_stretch(&samples, playback_speed);
            }

            let source = rodio::buffer::SamplesBuffer::new(1, wav.sample_rate, samples);
            sink.append(source);
            sink.set_speed(1.0);
            self.audio.playback_start = Some(std::time::Instant::now());
            self.audio.playback_offset_ms = 0.0;
        }
    }

    pub fn resample_current(&mut self) {
        if self.config.resampler_path.is_none() {
            #[cfg(any(windows, target_os = "linux", target_os = "macos"))]
            if let Some(path) = rfd::FileDialog::new()
                .set_title(tr!("audio.resampler.select_file"))
                .add_filter("Executáveis", &["exe", "bin", "sh"])
                .pick_file() {
                self.config.resampler_path = Some(path);
            } else {
                return;
            }
            #[cfg(target_arch = "wasm32")]
            return;
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
            cmd.arg(&self.config.test_flags); // Flag field
            cmd.arg(format!("{:.0}", entry.offset));
            cmd.arg(format!("{:.0}", self.config.test_duration_ms));
            cmd.arg(format!("{:.0}", entry.consonant));
            cmd.arg(format!("{:.0}", entry.cutoff));
            cmd.arg("100");
            cmd.arg("0");
            cmd.arg("120");
            cmd.arg("AA");

            self.log(tr!("audio.resampler.status.resampling"), egui::Color32::from_rgb(137, 180, 250));
            match cmd.output() {
                Ok(output) => {
                    if output.status.success() {
                        match load_wav(&output_wav) {
                            Ok(ws) => {
                                let dur_ms = ws.wav.duration_ms;
                                self.play_wav_data(ws.wav);
                                // Sincroniza o cursor com a posição real no waveform original
                                self.audio.playback_offset_ms = entry.offset;
                                self.audio.playback_limit_ms = Some(entry.offset + dur_ms);
                                self.log(format!("{} {}", tr!("audio.resampler.status.success"), entry.alias), egui::Color32::GREEN);
                            }
                            Err(e) => { self.log(format!("{} {}", tr!("audio.resampler.status.load_error"), e), egui::Color32::RED); }
                        }
                    } else {
                        let msg = format!("{} {}", tr!("audio.resampler.status.error"), String::from_utf8_lossy(&output.stderr));
                        self.ui.toast_manager.error(msg.clone());
                        self.log(msg, egui::Color32::RED);
                    }
                }
                Err(e) => { self.log(format!("{} {}", tr!("audio.resampler.status.exec_error"), e), egui::Color32::RED); }
            }
        }
    }

    pub fn refresh_audio_devices(&mut self) {
        use cpal::traits::{HostTrait, DeviceTrait};
        let host = cpal::default_host();
        if let Ok(devices) = host.output_devices() {
            self.ui.available_devices = devices
                .filter_map(|d| d.name().ok())
                .collect();
        }
    }

    pub fn set_audio_device(&mut self, name: Option<String>) {
        use cpal::traits::{HostTrait, DeviceTrait};
        self.config.audio_device = name.clone();
        
        // Stop current sinks
        self.stop_playback();
        self.audio.sink = None;
        self.audio.ui_sink = None;
        self.audio._stream = None;
        self.audio._stream_handle = None;

        let host = cpal::default_host();
        let device = if let Some(n) = &name {
            host.output_devices().ok().and_then(|mut ds| ds.find(|d| d.name().as_ref().ok() == Some(n)))
        } else {
            host.default_output_device()
        };

        if let Some(dev) = device {
            match OutputStream::try_from_device(&dev) {
                Ok((stream, handle)) => {
                    if let Ok(sink) = Sink::try_new(&handle) {
                        if let Ok(ui_sink) = Sink::try_new(&handle) {
                            self.audio._stream = Some(stream);
                            self.audio._stream_handle = Some(handle);
                            self.audio.sink = Some(Arc::new(sink));
                            self.audio.ui_sink = Some(Arc::new(ui_sink));
                            self.log(format!("Audio device: {}", dev.name().unwrap_or_default()), egui::Color32::from_rgb(150, 200, 150));
                        }
                    }
                }
                Err(e) => {
                    self.log(format!("Error setting device: {}", e), egui::Color32::RED);
                }
            }
        }
    }
}
