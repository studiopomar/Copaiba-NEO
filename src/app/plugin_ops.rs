use std::path::{Path, PathBuf};
use crate::app::state::{CopaibaApp, TabState};
use crate::oto::parse_oto;

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
        if let Ok(data) = parse_ust_plugin(path) {
            if let Some(voice_dir) = data.voice_dir {
                let oto_ini = voice_dir.join("oto.ini");
                if oto_ini.exists() {
                    // Load the voicebank
                    self.load_oto(oto_ini);
                    self.ui.show_home = false;

                    // Select/Filter entries matching the lyrics from the plugin
                    if !data.lyrics.is_empty() {
                        let mut target_indices = Vec::new();
                        {
                            let tab = self.cur();
                            for lyric in &data.lyrics {
                                // Find first match for each lyric
                                if let Some(idx) = tab.entries.iter().position(|e| &e.alias == lyric) {
                                    target_indices.push(idx);
                                }
                            }
                        }

                        if !target_indices.is_empty() {
                            let tab = self.cur_mut();
                            tab.selected = target_indices[0];
                            tab.multi_selection.clear();
                            for &idx in &target_indices {
                                tab.multi_selection.insert(idx);
                            }
                            self.ensure_wav_loaded();
                        }
                    }
                }
            }
        }
    }
}

fn parse_ust_plugin(path: &Path) -> Result<UstPluginData, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    
    // OpenUtau usually exports in Shift-JIS for classic plugins
    let (content, _, _) = encoding_rs::SHIFT_JIS.decode(&bytes);
    
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
            let key = &line[..eq_idx].trim();
            let val = &line[eq_idx + 1..].trim();

            match current_section.as_str() {
                "[#SETTING]" => {
                    if key.to_lowercase() == "voicedir" {
                        data.voice_dir = Some(PathBuf::from(val));
                    }
                }
                _ if current_section.starts_with("[#") => {
                    if key.to_lowercase() == "lyric" {
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
