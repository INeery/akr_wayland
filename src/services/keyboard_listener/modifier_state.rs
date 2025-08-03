use crate::events::Modifiers;
use evdev::KeyCode;

#[derive(Debug, Default)]
pub struct ModifierState {
    ctrl: bool,
    alt: bool,
    shift: bool,
    super_key: bool,
}

impl ModifierState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_modifiers(&self) -> Modifiers {
        Modifiers::new()
            .with_ctrl(self.ctrl)
            .with_alt(self.alt)
            .with_shift(self.shift)
            .with_super(self.super_key)
    }

    pub fn update_key(&mut self, key_code: u16, pressed: bool) {
        let keycode = KeyCode::new(key_code);
        match keycode {
            KeyCode::KEY_LEFTCTRL | KeyCode::KEY_RIGHTCTRL => self.ctrl = pressed,
            KeyCode::KEY_LEFTALT | KeyCode::KEY_RIGHTALT => self.alt = pressed,
            KeyCode::KEY_LEFTSHIFT | KeyCode::KEY_RIGHTSHIFT => self.shift = pressed,
            KeyCode::KEY_LEFTMETA | KeyCode::KEY_RIGHTMETA => self.super_key = pressed,
            _ => {}
        }
    }
}
