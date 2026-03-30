use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use egui_i18n::tr;

use crate::audio::load_wav;
use crate::oto::{parse_oto, save_oto};
use crate::waveform::WaveformView;
use super::state::{CopaibaApp, TabState};

#[derive(Serialize, Deserialize)]
struct PersistedPrefs {
    pub last_oto_path: Option<PathBuf>,
    pub visual: crate::app::state::VisualSettings,
    pub config: crate::app::state::AppConfig,
}

impl CopaibaApp {
    pub fn get_prefs_path() -> PathBuf {
        // Prefer APPDATA on Windows
        if cfg!(target_os = "windows") {
            if let Ok(appdata) = std::env::var("APPDATA") {
                return PathBuf::from(appdata).join("copaiba_prefs.json");
            }
        }
        
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".copaiba_prefs.json")
        } else {
            PathBuf::from("copaiba_prefs.json")
        }
    }

    pub fn load_prefs(&mut self) {
        let path = Self::get_prefs_path();
        // Try .json first, then fallback to .txt
        let content = std::fs::read_to_string(&path)
            .or_else(|_| std::fs::read_to_string(path.with_extension("txt")));

        if let Ok(content) = content {
            if let Ok(prefs) = serde_json::from_str::<PersistedPrefs>(&content) {
                self.visual = prefs.visual;
                self.config = prefs.config;
                // Consistency: don't auto-load the last project, let user pick from Home Screen
            } else {
                // Fallback for old format
                if let Some(line) = content.lines().next() {
                    let path = PathBuf::from(line.trim());
                    if path.exists() { self.load_oto(path); }
                }
            }
        }
    }

    pub fn save_prefs(&self) {
        let prefs = PersistedPrefs {
            last_oto_path: self.cur().oto_path.clone(),
            visual: self.visual.clone(),
            config: self.config.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&prefs) {
            let _ = std::fs::write(Self::get_prefs_path(), json);
        }
    }

    pub fn open_voicebank_dir(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
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
                            tr!("file_ops.label.default_tab").to_string()
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
                self.ui.show_home = false;
                for i in 0..self.tabs.len() {
                    self.load_character_metadata(i);
                    self.add_to_recent(i);
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
                    new_tab.name = path.file_name().and_then(|s| s.to_str()).unwrap_or(&tr!("file_ops.label.new_set")).to_string();
                    new_tab.oto_dir = Some(path.to_path_buf());
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
                    self.ui.show_home = false;
                    self.rebuild_filter();
                    self.ensure_wav_loaded();
                    self.add_to_recent(self.current_tab);
                }
            }
        }
    }

    pub fn add_to_recent(&mut self, tab_idx: usize) {
        let (path, name, root_path, image_path) = {
            let tab = &self.tabs[tab_idx];
            if tab.oto_path.is_none() { return; }
            
            let name = if tab.character_name.is_empty() { 
                tab.name.clone() 
            } else { 
                tab.character_name.clone() 
            };
            
            (
                tab.oto_path.clone().unwrap(),
                name,
                tab.root_path.clone(),
                tab.character_image_path.clone()
            )
        };
        
        let recent = crate::app::state::RecentVoicebank {
            name,
            path: path.clone(),
            root_path,
            image_path,
        };

        // Remove if already exists (same path)
        self.config.recent_voicebanks.retain(|r| r.path != path);
        
        // Insert at front
        self.config.recent_voicebanks.insert(0, recent);
        
        // Limit to 40
        if self.config.recent_voicebanks.len() > 40 {
            self.config.recent_voicebanks.pop();
        }
        self.save_prefs();
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

    pub fn load_oto_in_new_tab(&mut self, path: PathBuf) {
        // Reuse current tab if empty and untouched
        let reuse = self.tabs.len() == 1 && self.tabs[0].oto_path.is_none() && !self.tabs[0].dirty;
        
        if !reuse {
            self.tabs.push(crate::app::state::TabState::default());
            self.current_tab = self.tabs.len() - 1;
        } else {
            self.current_tab = 0;
        }
        self.load_oto(path);
    }

    pub fn open_oto(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
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

        if let Some(original_dir) = dir_opt {
            let mut root_dir = original_dir.clone();
            let mut curr = original_dir.clone();
            let mut found_txt = false;

            // Step 1: Find the actual root by looking for character.txt up to 4 levels up
            for _ in 0..4 {
                if curr.join("character.txt").exists() {
                    root_dir = curr.clone();
                    found_txt = true;
                    break;
                }
                if let Some(p) = curr.parent() {
                    curr = p.to_path_buf();
                } else {
                    break;
                }
            }

            // If we didn't find character.txt, we just use the original directory as root
            // Step 2: Load metadata from root_dir
            let mut name = String::new();
            let mut image = None;

            if found_txt {
                if let Ok(bytes) = std::fs::read(root_dir.join("character.txt")) {
                    let content = match encoding {
                        crate::oto::OtoEncoding::Utf8 => String::from_utf8_lossy(&bytes).to_string(),
                        crate::oto::OtoEncoding::ShiftJis => encoding_rs::SHIFT_JIS.decode(&bytes).0.to_string(),
                        crate::oto::OtoEncoding::Gbk => encoding_rs::GBK.decode(&bytes).0.to_string(),
                    };

                    for line in content.lines() {
                        let line = line.trim();
                        if line.to_lowercase().starts_with("name=") {
                            name = line[5..].to_string();
                        } else if line.to_lowercase().starts_with("image=") {
                            let img_name = line[6..].trim().to_string();
                            if !img_name.is_empty() {
                                image = Some(root_dir.join(img_name));
                            }
                        }
                    }
                }
            }

            // Fallback image search
            if image.is_none() {
                let possible_images = ["character.png", "character.jpg", "character.bmp", "icon.png", "char.png"];
                for pi in possible_images {
                    let pp = root_dir.join(pi);
                    if pp.exists() {
                        image = Some(pp);
                        break;
                    }
                }
            }

            // Fallback image search in parent if not found in original dir but character.txt is missing
            if image.is_none() && !found_txt && original_dir.parent().is_some() {
                 let possible_images = ["character.png", "character.jpg", "character.bmp", "icon.png", "char.png"];
                 let parent = original_dir.parent().unwrap();
                 for pi in possible_images {
                    let pp = parent.join(pi);
                    if pp.exists() {
                        image = Some(pp);
                        root_dir = parent.to_path_buf(); // Assume parent is the real root
                        break;
                    }
                }
            }

            // Search for readme/license
            let mut readme = String::new();
            let mut readme_path = None;
            let mut license = String::new();
            let readme_files = ["readme.txt", "readme.html", "README.txt", "Readme.txt", "readme", "README"];
            let license_files = ["license.txt", "licence.txt", "LICENSE.txt", "license", "licence", "LICENSE"];
            
            for rf in readme_files {
                let rp = root_dir.join(rf);
                if rp.exists() {
                    if let Ok(rb) = std::fs::read(&rp) {
                        readme = match encoding {
                            crate::oto::OtoEncoding::Utf8 => String::from_utf8_lossy(&rb).to_string(),
                            crate::oto::OtoEncoding::ShiftJis => encoding_rs::SHIFT_JIS.decode(&rb).0.to_string(),
                            crate::oto::OtoEncoding::Gbk => encoding_rs::GBK.decode(&rb).0.to_string(),
                        };
                        readme_path = Some(rp);
                        break;
                    }
                }
            }
            
            for lf in license_files {
                let lp = root_dir.join(lf);
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

            let mut prefix_map_entries = Vec::new();
            let pmap_path = root_dir.join("prefix.map");
            if pmap_path.exists() {
                if let Ok(pb) = std::fs::read(&pmap_path) {
                    let content = match encoding {
                        crate::oto::OtoEncoding::Utf8 => String::from_utf8_lossy(&pb).to_string(),
                        crate::oto::OtoEncoding::ShiftJis => encoding_rs::SHIFT_JIS.decode(&pb).0.to_string(),
                        crate::oto::OtoEncoding::Gbk => encoding_rs::GBK.decode(&pb).0.to_string(),
                    };
                    for line in content.lines() {
                        let parts: Vec<&str> = line.split('\t').collect();
                        if parts.len() >= 2 {
                             let pitch = parts[0].trim().to_string();
                             let p = if parts.len() > 2 { parts[1].trim().to_string() } else { "".to_string() };
                             let s = if parts.len() > 2 { parts[2].trim().to_string() } else { parts[1].trim().to_string() };
                             prefix_map_entries.push(crate::app::state::PrefixMapEntry { pitch, prefix: p, suffix: s, selected: false });
                        }
                    }
                }
            }

            let tab: &mut crate::app::state::TabState = &mut self.tabs[tab_idx];
            tab.character_name = name;
            tab.character_image_path = image;
            tab.character_texture = None;
            tab.root_path = Some(root_dir);
            tab.readme_path = readme_path;
            tab.original_readme_text = readme.clone();
            tab.readme_text = readme;
            tab.license_text = license;
            tab.prefix_map = prefix_map_entries.clone();
            tab.original_prefix_map = prefix_map_entries;
            tab.prefix_map_path = if pmap_path.exists() { Some(pmap_path) } else { None };
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
                    if let Some(dir) = &tab.oto_dir {
                        if let Some(fname) = dir.file_name() {
                            tab.name = fname.to_string_lossy().to_string();
                        }
                    }
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
                self.load_character_metadata(self.current_tab);
                self.add_to_recent(self.current_tab);
                let msg = format!("{} {}", self.cur().entries.len(), tr!("file_ops.status.aliases_loaded"));
                self.ui.toast_manager.success(msg.clone());
                self.ui.status = msg;
                self.save_prefs();
            }
            Err(e) => {
                let msg = format!("{} {e}", tr!("file_ops.status.open_error"));
                self.ui.toast_manager.error(msg.clone());
                self.ui.status = msg;
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
                    let msg = tr!("file_ops.status.saved_success").to_string();
                    self.ui.toast_manager.success(msg.clone());
                    self.ui.status = msg;
                }
                Err(e) => {
                    let msg = format!("{} {e}", tr!("file_ops.status.save_error"));
                    self.ui.toast_manager.error(msg.clone());
                    self.ui.status = msg;
                }
            }
        } else {
            self.save_as();
        }
    }

    pub fn save_as(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
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

        let full_path_key = dir_opt.as_ref().map(|d: &PathBuf| d.join(&fname).to_string_lossy().to_string()).unwrap_or_else(|| fname.clone());
        
        let needs_spec = self.visual.show_spectrogram && !self.spec_data_cache.contains_key(&full_path_key);
        if self.wav_cache.contains_key(&full_path_key) && !needs_spec { return; }

        if let Some(dir) = dir_opt {
            let wav_path = dir.join(&fname);
            let full_path_key = wav_path.to_string_lossy().to_string();

            if let Some(wav) = self.wav_cache.get(&full_path_key) {
                // WAV is loaded but spectrogram is missing (likely settings changed)
                let spec_set = self.visual.spec.clone();
                if let Some(sd) = crate::spectrogram::compute_spectrogram_data(&wav.samples, wav.sample_rate, &spec_set) {
                    self.spec_data_cache.insert(full_path_key, sd);
                }
                return;
            }

            match load_wav(&wav_path) {
                Ok(wav_with_spec) => {
                    if self.wav_cache.len() >= 5 {
                        let to_rem = self.wav_cache.keys().next().cloned();
                        if let Some(k) = to_rem {
                            self.wav_cache.remove(&k);
                            self.spec_data_cache.remove(&k);
                        }
                    }

                    let dur = wav_with_spec.wav.duration_ms;
                    let spec_set = self.visual.spec.clone();
                    let full_path_key = wav_path.to_string_lossy().to_string();
                    if let Some(sd) = crate::spectrogram::compute_spectrogram_data(&wav_with_spec.wav.samples, wav_with_spec.wav.sample_rate, &spec_set) {
                        self.spec_data_cache.insert(full_path_key.clone(), sd);
                    }
                    if let Some(pd) = crate::app::pitch::compute_pitch_data(&wav_with_spec.wav.samples, wav_with_spec.wav.sample_rate) {
                        self.pitch_data_cache.insert(full_path_key.clone(), pd);
                    }
                    self.wav_cache.insert(full_path_key, wav_with_spec.wav);

                    let persistent = self.visual.persistent_zoom;
                    let persistent_y = self.visual.persistent_y_zoom;
                    let tab = self.cur_mut();
                    // New WAV loaded: clear all visual caches unconditionally so
                    // the next frame rebuilds textures even if data_ptr happens to
                    // match (e.g. same Arc address recycled in heap) or if
                    // is_animating is true.
                    tab.wave_view.wave_cache    = crate::waveform::WaveCache::default();
                    tab.wave_view.spec_cache    = crate::waveform::SpecCache::default();
                    tab.wave_view.minimap_cache = crate::waveform::MinimapCache::default();
                    if !persistent {
                        tab.wave_view.reset_to(dur, persistent_y);
                    }
                    
                    // Centering logic (Request 7 fix)
                    let entry_off = {
                        let tab = self.cur();
                        tab.filtered.get(tab.selected).copied().and_then(|idx| tab.entries.get(idx)).map(|e| e.offset).unwrap_or(0.0)
                    };
                    let tab = self.cur_mut();
                    tab.wave_view.target_view_start_ms = (entry_off - tab.wave_view.target_view_range_ms * 0.3)
                        .clamp(0.0, (dur - tab.wave_view.target_view_range_ms).max(0.0));
                    tab.wave_view.view_start_ms = tab.wave_view.target_view_start_ms;
                }
                Err(e) => { self.ui.status = format!("WAV '{fname}': {e}"); }
            }
        }
    }
}
