//! WindowDetector service: responsibility and boundaries
//!
//! This module and its submodules are responsible ONLY for detecting the active window
//! (title/class/pid/geometry depending on platform) and emitting WindowEvent(s).
//! It MUST NOT contain any business logic related to key repetition, mappings, or
//! decision caches. All repetition decisions are made exclusively by KeyRepeater,
//! using Config::should_repeat_key().

mod dry_window_detector;
mod kdotool;
mod xdotool;
mod wmctrl;
mod sway;
mod window_detector;
mod r#trait;

pub use self::r#trait::{create_window_detector};
