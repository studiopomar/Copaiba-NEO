use std::path::{Path, PathBuf};

use crate::audio::load_wav;
use crate::oto::{parse_oto, save_oto};
use crate::waveform::WaveformView;
use super::state::{CopaibaApp, TabState};

impl CopaibaApp {
    pub fn get_prefs_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".copaiba_prefs.txt")
        } else if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata).join(".copaiba_prefs.txt")
        } else {
            PathBuf::from(".copaiba_prefs.txt")
        }
    }

    pub fn load_prefs(&mut self) {
        if let Ok(content) = std::fs::read_to_string(Self::get_prefs_path()) {
            if let Some(line) = content.lines().next() {
                if !line.trim().is_empty() {
                    let path = PathBuf::from(line.trim());
                    if path.exists() {
                        self.load_oto(path);
                    }
                }
            }
        }
    }

    pub fn save_prefs(&self) {
        if let Some(ref p) = self.cur().oto_path {
            let _ = std::fs::write(Self::get_prefs_path(), p.display().to_string());
        }
    }

    pub fn open_voicebank_dir(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            let mut otos = Vec::new();
            self.scan_for_oto(&path, &mut otos);

            if !otos.is_empty() {
                let was_empty = self.tabs.len() == 1 && self.tabs[0].entries.is_empty();
                let mut first_new_idx = None;

                for oto_path in otos {
                    if let Ok(parsed) = parse_oto(&oto_path) {
                        let mut new_tab = TabState::default();
                        let name = if let Some(parent) = oto_path.parent() {
                            let p_name = parent.file_name().and_then(|s| s.to_str()).unwrap_or("oto");
                            if let Some(gp) = parent.parent() {
                                let gp_name = gp.file_name().and_then(|s| s.to_str()).unwrap_or("");
                                if !gp_name.is_empty() && gp_name != "voicebank" {
                                    format!("{}/{}", gp_name, p_name)
                                } else {
                                    p_name.to_string()
                                }
                            } else {
                                p_name.to_string()
                            }
                        } else {
                            "oto".to_string()
                        };
                        new_tab.name = name;
                        new_tab.entries = parsed.entries.clone();
                        new_tab.original_entries = parsed.entries;
                        new_tab.oto_path = Some(oto_path.clone());
                        new_tab.oto_dir = Some(oto_path.parent().unwrap().to_path_buf());
                        new_tab.dirty = false;

                        if was_empty && first_new_idx.is_none() {
                            self.tabs[0] = new_tab;
                            first_new_idx = Some(0);
                        } else {
                            self.tabs.push(new_tab);
                            if first_new_idx.is_none() {
                                first_new_idx = Some(self.tabs.len() - 1);
                            }
                        }
                    }
                }

                if let Some(idx) = first_new_idx {
                    self.current_tab = idx;
                }
                for tab in &mut self.tabs {
                    tab.filtered = (0..tab.entries.len()).collect();
                }
                self.rebuild_filter();
                self.ensure_wav_loaded();
                for i in 0..self.tabs.len() {
                    self.load_character_metadata(i);
                }
            } else {
                // FALLBACK: no oto.ini → create from .wav files
                let mut wavs = Vec::new();
                if let Ok(entries) = std::fs::read_dir(&path) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) == Some("wav".to_string()) {
                            wavs.push(p);
                        }
                    }
                }

                if !wavs.is_empty() {
                    let mut new_tab = TabState::default();
                    new_tab.name = path.file_name().and_then(|s| s.to_str()).unwrap_or("Novo Set").to_string();
                    new_tab.oto_dir = Some(path.clone());
                    new_tab.oto_path = Some(path.join("oto.ini"));

                    for w in wavs {
                        let fname = w.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                        let alias = w.file_stem().and_then(|s| s.to_str()).unwrap_or(&fname).to_string();
                        new_tab.entries.push(crate::oto::OtoEntry {
                            filename: fname,
                            alias,
                            offset: 0.0,
                            consonant: 100.0,
                            cutoff: -250.0,
                            preutter: 25.0,
                            overlap: 10.0,
                            line_index: new_tab.entries.len(),
                            done: false,
                            notes: String::new(),
                        });
                    }
                    new_tab.original_entries = new_tab.entries.clone();
                    new_tab.dirty = true;

                    if self.tabs.len() == 1 && self.tabs[0].entries.is_empty() {
                        self.tabs[0] = new_tab;
                        self.current_tab = 0;
                    } else {
                        self.tabs.push(new_tab);
                        self.current_tab = self.tabs.len() - 1;
                    }
                    self.rebuild_filter();
                    self.ensure_wav_loaded();
                }
            }
        }
    }

    pub fn scan_for_oto(&self, dir: &Path, acc: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    self.scan_for_oto(&p, acc);
                } else if p.file_name().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) == Some("oto.ini".to_string()) {
                    acc.push(p);
                }
            }
        }
    }

    pub fn open_oto(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("oto.ini", &["ini"])
            .pick_file()
        {
            self.load_oto(path);
        }
    }

    pub fn load_character_metadata(&mut self, tab_idx: usize) {
        let dir_opt = self.tabs[tab_idx].oto_dir.clone();
        let encoding = self.encoding;

        if let Some(mut current_dir) = dir_opt {
            // Search up to 4 levels for character.txt (recursive search for voicebank root)
            for _ in 0..4 {
                let char_path = current_dir.join("character.txt");
                if char_path.exists() {
                    if let Ok(bytes) = std::fs::read(&char_path) {
                        // Decode using the current encoding (same as oto.ini)
                        let content = match encoding {
                            crate::oto::OtoEncoding::Utf8 => String::from_utf8_lossy(&bytes).to_string(),
                            crate::oto::OtoEncoding::ShiftJis => {
                                let (res, _, _) = encoding_rs::SHIFT_JIS.decode(&bytes);
                                res.to_string()
                            }
                            crate::oto::OtoEncoding::Gbk => {
                                let (res, _, _) = encoding_rs::GBK.decode(&bytes);
                                res.to_string()
                            }
                        };

                        let mut name = String::new();
                        let mut image = None;

                        for line in content.lines() {
                            let line = line.trim();
                            if line.to_lowercase().starts_with("name=") {
                                name = line[5..].to_string();
                            } else if line.to_lowercase().starts_with("image=") {
                                let img_name = line[6..].trim().to_string();
                                if !img_name.is_empty() {
                                    image = Some(current_dir.join(img_name));
                                }
                            }
                        }
                        
                        // Search for readme/license in the same directory
                        let mut readme = String::new();
                        let mut license = String::new();
                        
                        let readme_files = ["readme.txt", "readme.html", "README.txt", "Readme.txt", "readme", "README"];
                        let license_files = ["license.txt", "licence.txt", "LICENSE.txt", "license", "licence", "LICENSE"];
                        
                        for rf in readme_files {
                            let rp = current_dir.join(rf);
                            if rp.exists() {
                                if let Ok(rb) = std::fs::read(&rp) {
                                    readme = match encoding {
                                        crate::oto::OtoEncoding::Utf8 => String::from_utf8_lossy(&rb).to_string(),
                                        crate::oto::OtoEncoding::ShiftJis => encoding_rs::SHIFT_JIS.decode(&rb).0.to_string(),
                                        crate::oto::OtoEncoding::Gbk => encoding_rs::GBK.decode(&rb).0.to_string(),
                                    };
                                    break;
                                }
                            }
                        }
                        
                        for lf in license_files {
                            let lp = current_dir.join(lf);
                            if lp.exists() {
                                if let Ok(lb) = std::fs::read(&lp) {
                                    license = match encoding {
                                        crate::oto::OtoEncoding::Utf8 => String::from_utf8_lossy(&lb).to_string(),
                                        crate::oto::OtoEncoding::ShiftJis => encoding_rs::SHIFT_JIS.decode(&lb).0.to_string(),
                                        crate::oto::OtoEncoding::Gbk => encoding_rs::GBK.decode(&lb).0.to_string(),
                                    };
                                    break;
                                }
                            }
                        }

                        let tab = &mut self.tabs[tab_idx];
                        tab.character_name = name;
                        tab.character_image_path = image;
                        tab.character_texture = None;
                        tab.readme_text = readme;
                        tab.license_text = license;
                        return;
                    }
                }
                
                // Go up one level
                if let Some(parent) = current_dir.parent() {
                    current_dir = parent.to_path_buf();
                } else {
                    break;
                }
            }
        }
    }

    pub fn load_oto(&mut self, path: PathBuf) {
        match parse_oto(&path) {
            Ok(parsed) => {
                self.encoding = parsed.encoding;
                self.wav_cache.clear();
                self.spec_data_cache.clear();
                {
                    let tab = self.cur_mut();
                    tab.entries = parsed.entries.clone();
                    tab.original_entries = parsed.entries;
                    tab.oto_dir = path.parent().map(|p| p.to_path_buf());
                    tab.oto_path = Some(path);
                    tab.selected = 0;
                    tab.dirty = false;
                    tab.undo_stack.clear();
                    tab.redo_stack.clear();
                }
                self.save_undo_state();
                self.rebuild_filter();
                {
                    let tab = self.cur_mut();
                    tab.wave_view = WaveformView::default();
                    tab.selected = usize::MAX;
                }
                if !self.cur().filtered.is_empty() {
                    self.select_multi(0, false, false);
                }
                self.load_character_metadata(self.current_tab);
                self.status = format!("{} aliases carregados.", self.cur().entries.len());
                self.save_prefs();
            }
            Err(e) => {
                self.status = format!("Erro ao abrir: {e}");
            }
        }
    }

    pub fn save_oto(&mut self) {
        let (path_opt, encoding) = {
            let tab = self.cur();
            (tab.oto_path.clone(), self.encoding)
        };
        if let Some(path) = path_opt {
            let path: PathBuf = path; // Force PathBuf
            let res = {
                let tab = self.cur();
                save_oto(&tab.entries, &path, encoding)
            };
            match res {
                Ok(_) => {
                    self.save_prefs();
                    let tab = self.cur_mut();
                    tab.original_entries = tab.entries.clone();
                    tab.dirty = false;
                    self.status = "Salvo com sucesso.".to_string();
                }
                Err(e) => {
                    self.status = format!("Erro ao salvar: {e}");
                }
            }
        } else {
            self.save_as();
        }
    }

    pub fn save_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("oto.ini")
            .add_filter("oto.ini", &["ini"])
            .save_file()
        {
            let tab = self.cur_mut();
            tab.oto_path = Some(path.clone());
            tab.oto_dir = path.parent().map(|p| p.to_path_buf());
            self.save_oto();
        }
    }

    pub fn ensure_wav_loaded(&mut self) {
        let (fname, dir_opt) = {
            let tab = self.cur();
            if tab.filtered.is_empty() { return; }
            let idx = tab.filtered.get(tab.selected).copied().unwrap_or(0);
            if idx >= tab.entries.len() { return; }
            (tab.entries[idx].filename.clone(), tab.oto_dir.clone())
        };

        let full_path_key = dir_opt.as_ref().map(|d| d.join(&fname).to_string_lossy().to_string()).unwrap_or_else(|| fname.clone());
        if self.wav_cache.contains_key(&full_path_key) { return; }

        let dir_opt: Option<PathBuf> = dir_opt;
        if let Some(dir) = dir_opt {
            let wav_path = dir.join(&fname);
            match load_wav(&wav_path) {
                Ok(wav_with_spec) => {
                    if self.wav_cache.len() >= 5 {
                        let mut to_rem = None;
                        if let Some(k) = self.wav_cache.keys().next() {
                            to_rem = Some(k.clone());
                        }
                        if let Some(k) = to_rem {
                            self.wav_cache.remove(&k);
                            self.spec_data_cache.remove(&k);
                        }
                    }

                    let dur = wav_with_spec.wav.duration_ms;
                    let spec_set = self.spec_settings.clone();
                    let full_path_key = wav_path.to_string_lossy().to_string();
                    if let Some(sd) = crate::spectrogram::compute_spectrogram_data(&wav_with_spec.wav.samples, wav_with_spec.wav.sample_rate, &spec_set) {
                        self.spec_data_cache.insert(full_path_key.clone(), sd);
                    }
                    self.wav_cache.insert(full_path_key, wav_with_spec.wav);

                    let persistent = self.persistent_zoom;
                    if !persistent {
                        self.cur_mut().wave_view.reset_to(dur);
                    }
                }
                Err(e) => { self.status = format!("WAV '{fname}': {e}"); }
            }
        }
    }
}
