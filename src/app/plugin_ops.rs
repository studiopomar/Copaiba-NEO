use std::path::{Path, PathBuf};
use crate::app::state::CopaibaApp;

#[derive(Debug, Default)]
pub struct UstPluginData {
    pub voice_dir: Option<PathBuf>,
    pub lyrics: Vec<String>,
}

impl CopaibaApp {
    pub fn handle_cli_args(&mut self, args: Vec<String>) {
        if args.len() < 2 {
            return;
        }

        let first_arg = &args[1];
        let path = PathBuf::from(first_arg);

        if !path.exists() {
            return;
        }

        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        
        if ext == "tmp" || ext == "ust" {
            self.ui.show_splash = false;
            self.load_from_ust_plugin(&path);
        } else if ext == "ini" && path.file_name().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) == Some("oto.ini".to_string()) {
            self.ui.show_splash = false;
            self.load_oto(path);
            self.ui.show_home = false;
        }
    }

    fn load_from_ust_plugin(&mut self, path: &Path) {
        match parse_ust_plugin(path) {
            Ok(data) => {
                if let Some(voice_dir) = data.voice_dir {
                    let mut oto_ini = voice_dir.join("oto.ini");
                    
                    // Fallback: search for any oto.ini if not in root (common in multipitch)
                    if !oto_ini.exists() {
                        let mut found_otos = Vec::new();
                        self.scan_for_oto(&voice_dir, &mut found_otos);
                        if !found_otos.is_empty() {
                            // For now, take the first one or one that might contain the lyrics
                            // Future improvement: load all into tabs
                            oto_ini = found_otos[0].clone();
                        }
                    }

                    if oto_ini.exists() {
                        // Load the voicebank
                        self.load_oto(oto_ini);
                        self.ui.show_home = false;

                        // Select/Filter entries matching the lyrics from the plugin
                        if !data.lyrics.is_empty() {
                            let mut target_indices = Vec::new();
                            {
                                let tab = self.cur_mut();
                                // Automatically set filter to show these lyrics
                                tab.filter = data.lyrics.join(" ");
                                
                                for lyric in &data.lyrics {
                                    let lyric_lower = lyric.trim().to_lowercase();
                                    // Find match (case-insensitive and trimmed)
                                    if let Some(idx) = tab.entries.iter().position(|e| e.alias.trim().to_lowercase() == lyric_lower) {
                                        target_indices.push(idx);
                                    }
                                }
                            }
                            
                            self.rebuild_filter();

                            if !target_indices.is_empty() {
                                let tab = self.cur_mut();
                                // If the first target index is now in a different position in the filtered list, find it
                                let first_raw_idx = target_indices[0];
                                if let Some(fi) = tab.filtered.iter().position(|&i| i == first_raw_idx) {
                                    tab.selected = fi;
                                }
                                
                                tab.multi_selection.clear();
                                for &idx in &target_indices {
                                    if let Some(fi) = tab.filtered.iter().position(|&i| i == idx) {
                                        tab.multi_selection.insert(fi);
                                    }
                                }
                                self.ensure_wav_loaded();
                            } else {
                                self.ui.toast_manager.info(format!("Aviso: {} aliases não encontrados no oto.ini", data.lyrics.len()));
                            }
                        }
                    } else {
                        self.ui.toast_manager.error(format!("Erro: oto.ini não encontrado em {}", voice_dir.display()));
                    }
                } else {
                    self.ui.toast_manager.error("Erro: VoiceDir não definido no arquivo .tmp");
                }
            }
            Err(e) => {
                self.ui.toast_manager.error(format!("Erro ao ler plugin: {}", e));
            }
        }
    }
}

fn parse_ust_plugin(path: &Path) -> Result<UstPluginData, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Falha ao ler arquivo: {}", e))?;
    
    // Try UTF-8 first, fallback to Shift-JIS
    let content = if let Ok(s) = String::from_utf8(bytes.clone()) {
        s
    } else {
        let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&bytes);
        decoded.into_owned()
    };
    
    let mut data = UstPluginData::default();
    let mut current_section = String::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        
        if line.starts_with("[#") && line.ends_with(']') {
            current_section = line.to_string();
            continue;
        }

        if let Some(eq_idx) = line.find('=') {
            let key = line[..eq_idx].trim().to_lowercase();
            let val = line[eq_idx + 1..].trim().trim_matches('"');

            match current_section.as_str() {
                "[#SETTING]" => {
                    if key == "voicedir" {
                        let mut vdir = PathBuf::from(val);
                        if vdir.is_relative() {
                            if let Some(parent) = path.parent() {
                                vdir = parent.join(vdir);
                            }
                        }
                        data.voice_dir = Some(vdir);
                    }
                }
                _ if current_section.starts_with("[#") => {
                    if key == "lyric" {
                        if val.to_lowercase() != "r" {
                            data.lyrics.push(val.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    Ok(data)
}
