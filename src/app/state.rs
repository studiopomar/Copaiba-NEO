use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use serde::{Serialize, Deserialize};

use egui_i18n::tr;

use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::audio::WavData;
use crate::oto::{OtoEncoding, OtoEntry};
use crate::spectrogram::{SpectrogramData, SpectrogramSettings};
use crate::waveform::{WaveformSettings, WaveformView};
use crate::plugins;

#[derive(Clone, PartialEq)]
pub struct TabState {
    pub name: String,
    pub entries: Vec<OtoEntry>,
    pub original_entries: Vec<OtoEntry>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub filter: String,
    pub oto_path: Option<PathBuf>,
    pub oto_dir: Option<PathBuf>,
    pub character_name: String,
    pub character_image_path: Option<PathBuf>,
    pub character_texture: Option<egui::TextureHandle>,
    pub readme_text: String,
    pub license_text: String,
    pub dirty: bool,
    pub wave_view: WaveformView,
    pub undo_stack: Vec<Vec<OtoEntry>>,
    pub redo_stack: Vec<Vec<OtoEntry>>,
    pub multi_selection: HashSet<usize>,
    pub focus_col: usize,
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            name: tr!("state.tab.default_name").to_string(),
            entries: Vec::new(),
            original_entries: Vec::new(),
            filtered: Vec::new(),
            selected: 0,
            filter: String::new(),
            oto_path: None,
            oto_dir: None,
            character_name: String::new(),
            character_image_path: None,
            character_texture: None,
            readme_text: String::new(),
            license_text: String::new(),
            dirty: false,
            wave_view: WaveformView::default(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            multi_selection: HashSet::new(),
            focus_col: 0,
        }
    }
}

#[derive(Clone, Default)]
pub struct Preset {
    pub name: String,
    pub offset: f64,
    pub consonant: f64,
    pub cutoff: f64,
    pub preutter: f64,
    pub overlap: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RecentVoicebank {
    pub name: String,
    pub path: PathBuf,
    pub image_path: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ShortcutProfile {
    #[default]
    Copaiba,
    Utau,
    VLabeler,
    SetParam,
    Custom,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CustomShortcuts {
    pub play: String,
    pub stop: String,
    pub save: String,
    pub undo: String,
    pub redo: String,
    pub off: egui::Key,
    pub ove: egui::Key,
    pub pre: egui::Key,
    pub con: egui::Key,
    pub cut: egui::Key,
}

impl Default for CustomShortcuts {
    fn default() -> Self {
        Self {
            play: "P".to_string(),
            stop: "S".to_string(),
            save: "S".to_string(),
            undo: "Z".to_string(),
            redo: "Y".to_string(),
            off: egui::Key::Q,
            ove: egui::Key::W,
            pre: egui::Key::E,
            con: egui::Key::R,
            cut: egui::Key::T,
        }
    }
}

pub struct AudioState {
    pub _stream: Option<OutputStream>,
    pub _stream_handle: Option<OutputStreamHandle>,
    pub sink: Option<Arc<Sink>>,
    pub playback_start: Option<std::time::Instant>,
    pub playback_offset_ms: f64,
    pub playback_limit_ms: Option<f64>,

    // Recorder
    pub is_recording: bool,
    pub recorder_samples: Arc<Mutex<Vec<f32>>>,
    pub recorder_stop_signal: Arc<AtomicBool>,
    pub recorder_stream: Option<cpal::Stream>,
    pub recorded_wav: Option<WavData>,
    pub recorder_sample_rate: u32,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            _stream: None,
            _stream_handle: None,
            sink: None,
            playback_start: None,
            playback_offset_ms: 0.0,
            playback_limit_ms: None,
            is_recording: false,
            recorder_samples: Arc::new(Mutex::new(Vec::new())),
            recorder_stop_signal: Arc::new(AtomicBool::new(false)),
            recorder_stream: None,
            recorded_wav: None,
            recorder_sample_rate: 44100,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VisualSettings {
    pub spec: SpectrogramSettings,
    pub wave: WaveformSettings,
    pub show_spectrogram: bool,
    pub show_minimap: bool,
    pub persistent_zoom: bool,
}

impl Default for VisualSettings {
    fn default() -> Self {
        Self {
            spec: SpectrogramSettings::default(),
            wave: WaveformSettings::default(),
            show_spectrogram: true,
            show_minimap: true,
            persistent_zoom: false,
        }
    }
}

pub struct UiState {
    pub status: String,
    pub show_exit_dialog: bool,
    pub show_preset_editor: bool,
    pub show_settings: bool,
    pub show_help: bool,
    pub renaming_tab: Option<usize>,
    pub is_renaming: bool,
    pub search_string: String,
    pub auto_scroll_to_selected: bool,
    
    // Tools/Plugins windows
    pub show_consistency_checker: bool,
    pub show_batch_rename: bool,
    pub show_batch_edit: bool,
    pub show_alias_sorter: bool,
    pub show_duplicate_detector: bool,
    pub show_pitch_analyzer: bool,
    
    // Results
    pub consistency_issues: Vec<plugins::ValidationIssue>,
    pub duplicate_results: Vec<plugins::Duplicate>,
    
    // Recorder
    pub show_recorder: bool,
    pub show_home: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            status: tr!("state.ui.status.ready").to_string(),
            show_exit_dialog: false,
            show_preset_editor: false,
            show_settings: false,
            show_help: false,
            renaming_tab: None,
            is_renaming: false,
            search_string: String::new(),
            auto_scroll_to_selected: true,
            show_consistency_checker: false,
            show_batch_rename: false,
            show_batch_edit: false,
            show_alias_sorter: false,
            show_duplicate_detector: false,
            show_pitch_analyzer: false,
            consistency_issues: Vec::new(),
            duplicate_results: Vec::new(),
            show_recorder: false,
            show_home: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: String,
    pub shortcut_profile: ShortcutProfile,
    pub custom_shorts: CustomShortcuts,
    pub play_on_select: bool,
    pub auto_save_enabled: bool,
    pub auto_save_interval_mins: u32,
    pub test_duration_ms: f64,
    pub test_pitch: String,
    pub resampler_path: Option<PathBuf>,
    pub recent_voicebanks: Vec<RecentVoicebank>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: "pt-BR".to_string(),
            shortcut_profile: ShortcutProfile::Copaiba,
            custom_shorts: CustomShortcuts {
                play: "P".to_string(),
                stop: "S".to_string(),
                save: "S".to_string(),
                undo: "Z".to_string(),
                redo: "Y".to_string(),
                off: egui::Key::Q,
                ove: egui::Key::W,
                pre: egui::Key::E,
                con: egui::Key::R,
                cut: egui::Key::T,
            },
            play_on_select: false,
            auto_save_enabled: false,
            auto_save_interval_mins: 5,
            test_duration_ms: 500.0,
            test_pitch: "C4".to_string(),
            resampler_path: None,
            recent_voicebanks: Vec::new(),
        }
    }
}

// ── CopaibaApp ────────────────────────────────────────────────────────────────

pub struct CopaibaApp {
    pub tabs: Vec<TabState>,
    pub current_tab: usize,

    pub wav_cache: HashMap<String, WavData>,
    pub spec_data_cache: HashMap<String, SpectrogramData>,
    pub encoding: OtoEncoding,
    
    pub audio: AudioState,
    pub visual: VisualSettings,
    pub ui: UiState,
    pub config: AppConfig,

    pub sort_settings: plugins::SortSettings,
    pub shift_pivot: Option<usize>,

    pub pitch_times: Vec<f64>,
    pub pitch_values: Vec<f64>,
    pub pitch_window_ms: f64,

    pub presets: Vec<Preset>,

    // Batch Rename State
    pub rename_find: String,
    pub rename_replace: String,
    pub rename_prefix: String,
    pub rename_suffix: String,

    // Batch Edit State
    pub batch_edit_enabled: [bool; 5],
    pub batch_edit_values: [f64; 5],

    // Logic
    pub session_start_time: f64,
    pub last_auto_save_time: f64,
    pub project_path: Option<PathBuf>,
}

impl Default for CopaibaApp {
    fn default() -> Self {
        Self {
            tabs: vec![TabState::default()],
            current_tab: 0,

            wav_cache: HashMap::new(),
            spec_data_cache: HashMap::new(),
            encoding: OtoEncoding::ShiftJis,

            audio: AudioState::default(),
            visual: VisualSettings::default(),
            ui: UiState::default(),
            config: AppConfig::default(),

            sort_settings: plugins::SortSettings::default(),
            shift_pivot: None,
            pitch_times: Vec::new(),
            pitch_values: Vec::new(),
            pitch_window_ms: 10.0,

            presets: vec![
                Preset { name: "CV".into(), offset: 0.0, consonant: 100.0, cutoff: -250.0, preutter: 20.0, overlap: 10.0 },
                Preset { name: "VC".into(), offset: 0.0, consonant: 150.0, cutoff: -400.0, preutter: 70.0, overlap: 40.0 },
                Preset { name: "VCV".into(), offset: 0.0, consonant: 100.0, cutoff: -300.0, preutter: 50.0, overlap: 50.0 },
                Preset { name: "VV".into(), offset: 0.0, consonant: 100.0, cutoff: -150.0, preutter: 20.0, overlap: 20.0 },
                Preset { name: "- CV".into(), offset: 0.0, consonant: 100.0, cutoff: -200.0, preutter: 40.0, overlap: 20.0 },
            ],

            rename_find: String::new(),
            rename_replace: String::new(),
            rename_prefix: String::new(),
            rename_suffix: String::new(),

            batch_edit_enabled: [false; 5],
            batch_edit_values: [0.0; 5],

            project_path: None,
            session_start_time: 0.0,
            last_auto_save_time: 0.0,
        }
    }
}

impl CopaibaApp {
    pub fn cur(&self) -> &TabState { &self.tabs[self.current_tab] }
    pub fn cur_mut(&mut self) -> &mut TabState { &mut self.tabs[self.current_tab] }

    pub fn set_language(&mut self, lang: &str) {
        egui_i18n::set_language(lang);
        self.config.language = lang.to_string();
    }
}
