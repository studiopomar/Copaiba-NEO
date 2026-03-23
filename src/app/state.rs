use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::audio::WavData;
use crate::oto::{OtoEncoding, OtoEntry};
use crate::spectrogram::{SpectrogramData, SpectrogramSettings};
use crate::waveform::{WaveformSettings, WaveformView};
use crate::plugins;

// ── Simple types ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Preset {
    pub name: String,
    pub offset: f64,
    pub consonant: f64,
    pub cutoff: f64,
    pub preutter: f64,
    pub overlap: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShortcutProfile {
    Copaiba,
    SetParam,
    Custom,
}

#[derive(Clone, Debug)]
pub struct CustomShortcuts {
    pub off: egui::Key,
    pub ove: egui::Key,
    pub pre: egui::Key,
    pub con: egui::Key,
    pub cut: egui::Key,
}

impl Default for CustomShortcuts {
    fn default() -> Self {
        Self {
            off: egui::Key::Q,
            ove: egui::Key::W,
            pre: egui::Key::E,
            con: egui::Key::R,
            cut: egui::Key::T,
        }
    }
}

// ── TabState ─────────────────────────────────────────────────────────────────

pub struct TabState {
    pub name: String,
    pub entries: Vec<OtoEntry>,
    pub selected: usize,
    pub multi_selection: HashSet<usize>,
    pub oto_path: Option<PathBuf>,
    pub oto_dir: Option<PathBuf>,
    pub dirty: bool,
    pub undo_stack: Vec<Vec<OtoEntry>>,
    pub redo_stack: Vec<Vec<OtoEntry>>,
    pub wave_view: WaveformView,
    pub filter: String,
    pub filtered: Vec<usize>,
    pub original_entries: Vec<OtoEntry>,
    pub focus_col: usize,
    pub character_name: String,
    pub character_image_path: Option<PathBuf>,
    pub character_texture: Option<egui::TextureHandle>,
    pub readme_text: String,
    pub license_text: String,
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            name: "Novo Set".to_string(),
            entries: Vec::new(),
            selected: 0,
            multi_selection: HashSet::new(),
            oto_path: None,
            oto_dir: None,
            dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            wave_view: WaveformView::default(),
            filter: String::new(),
            filtered: Vec::new(),
            original_entries: Vec::new(),
            focus_col: 2,
            character_name: String::new(),
            character_image_path: None,
            character_texture: None,
            readme_text: String::new(),
            license_text: String::new(),
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
    pub status: String,

    pub _stream: Option<OutputStream>,
    pub _stream_handle: Option<OutputStreamHandle>,
    pub sink: Option<Arc<Sink>>,
    pub playback_start: Option<std::time::Instant>,
    pub playback_offset_ms: f64,
    pub playback_limit_ms: Option<f64>,
    pub resampler_path: Option<PathBuf>,
    pub test_duration_ms: f64,
    pub test_pitch: String,
    pub persistent_zoom: bool,

    // Window visibility
    pub show_exit_dialog: bool,
    pub show_preset_editor: bool,
    pub show_settings: bool,
    pub show_help: bool,
    pub play_on_select: bool,
    pub renaming_tab: Option<usize>,

    pub spec_settings: SpectrogramSettings,
    pub wave_settings: WaveformSettings,

    pub show_consistency_checker: bool,
    pub consistency_issues: Vec<plugins::ValidationIssue>,

    pub show_batch_rename: bool,
    pub show_batch_edit: bool,
    pub show_spectrogram: bool,
    pub show_alias_sorter: bool,
    pub sort_settings: plugins::SortSettings,

    pub shift_pivot: Option<usize>,

    pub show_duplicate_detector: bool,
    pub duplicate_results: Vec<plugins::Duplicate>,

    pub show_pitch_analyzer: bool,
    pub shortcut_profile: ShortcutProfile,
    pub custom_shorts: CustomShortcuts,
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
    pub auto_save_enabled: bool,
    pub auto_save_interval_mins: u32,
    pub last_auto_save_time: f64,

    pub search_string: String,
    pub project_path: Option<PathBuf>,
    pub is_renaming: bool,
    pub show_minimap: bool,
    pub auto_scroll_to_selected: bool,

    // Recorder
    pub show_recorder: bool,
    pub is_recording: bool,
    pub recorder_samples: Arc<Mutex<Vec<f32>>>,
    pub recorder_stop_signal: Arc<AtomicBool>,
    pub recorder_stream: Option<cpal::Stream>,
    pub recorded_wav: Option<WavData>,
    pub recorder_sample_rate: u32,
}

impl Default for CopaibaApp {
    fn default() -> Self {
        Self {
            tabs: vec![TabState::default()],
            current_tab: 0,

            wav_cache: HashMap::new(),
            spec_data_cache: HashMap::new(),
            encoding: OtoEncoding::ShiftJis,
            status: String::from("Abrir um arquivo oto.ini para começar."),

            _stream: None,
            _stream_handle: None,
            sink: None,
            playback_start: None,
            playback_offset_ms: 0.0,
            playback_limit_ms: None,
            resampler_path: None,
            test_duration_ms: 500.0,
            test_pitch: "C4".to_string(),
            persistent_zoom: false,

            show_exit_dialog: false,
            show_preset_editor: false,
            show_settings: false,
            show_help: false,
            play_on_select: false,
            renaming_tab: None,

            spec_settings: SpectrogramSettings::default(),
            wave_settings: WaveformSettings::default(),

            show_consistency_checker: false,
            consistency_issues: Vec::new(),
            show_batch_rename: false,
            show_batch_edit: false,
            show_spectrogram: true,
            show_alias_sorter: false,
            sort_settings: plugins::SortSettings::default(),

            shift_pivot: None,
            show_duplicate_detector: false,
            duplicate_results: Vec::new(),
            show_pitch_analyzer: false,
            shortcut_profile: ShortcutProfile::Copaiba,
            custom_shorts: CustomShortcuts::default(),
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

            search_string: String::new(),
            project_path: None,
            session_start_time: 0.0,
            auto_save_enabled: false,
            auto_save_interval_mins: 5,
            last_auto_save_time: 0.0,
            is_renaming: false,
            show_minimap: true,
            auto_scroll_to_selected: true,

            show_recorder: false,
            is_recording: false,
            recorder_samples: Arc::new(Mutex::new(Vec::new())),
            recorder_stop_signal: Arc::new(AtomicBool::new(false)),
            recorder_stream: None,
            recorded_wav: None,
            recorder_sample_rate: 44100,
        }
    }
}

impl CopaibaApp {
    pub fn cur(&self) -> &TabState { &self.tabs[self.current_tab] }
    pub fn cur_mut(&mut self) -> &mut TabState { &mut self.tabs[self.current_tab] }
}
