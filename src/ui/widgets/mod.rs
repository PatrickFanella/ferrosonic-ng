//! Custom UI widgets

pub mod cava;
pub mod hotkeys;
pub mod now_playing;
pub mod progress_bar;

pub use cava::CavaWidget;
pub use hotkeys::render_hotkeys_modal;
pub use now_playing::NowPlayingWidget;
