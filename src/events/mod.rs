pub mod keyboard;
pub mod window;

pub use keyboard::{KeyEvent, KeyState, KeyCode, Modifiers};
pub use window::{WindowEvent, WindowInfo};

/// События для виртуальной клавиатуры
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualKeyEvent {
    pub key_code: KeyCode,
    pub state: KeyState,
    pub modifiers: Modifiers,
    pub timestamp: std::time::Instant,
}

impl VirtualKeyEvent {
    pub fn new(key_code: KeyCode, state: KeyState, modifiers: Modifiers) -> Self {
        Self {
            key_code,
            state,
            modifiers,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn press(key_code: KeyCode, modifiers: Modifiers) -> Self {
        Self::new(key_code, KeyState::Pressed, modifiers)
    }

    pub fn release(key_code: KeyCode, modifiers: Modifiers) -> Self {
        Self::new(key_code, KeyState::Released, modifiers)
    }
}
