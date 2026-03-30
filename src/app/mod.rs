pub mod state;
pub mod file_ops;
pub mod audio_ops;
pub mod selection;
pub mod ui_menu;
pub mod ui_tabs;
pub mod ui_table;
pub mod ui_tools;
pub mod ui_waveform;
pub mod ui_status;
mod ui_header;
pub mod ui_modals;
pub mod ui_pmap_editor;
pub mod recorder;
pub mod ui_recorder;
pub mod ui_home;

pub mod phonetic;
pub mod pitch;
pub mod layout;
pub mod auto_oto;
pub mod toast;
pub mod bidi;

pub use state::CopaibaApp;
