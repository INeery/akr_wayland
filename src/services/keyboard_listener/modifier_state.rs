use crate::events::Modifiers;
use evdev::Key;

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
        Modifiers {
            ctrl: self.ctrl,
            alt: self.alt,
            shift: self.shift,
            super_key: self.super_key,
        }
    }

    pub fn update_key(&mut self, key: Key, pressed: bool) {
        match key {
            Key::KEY_LEFTCTRL | Key::KEY_RIGHTCTRL => self.ctrl = pressed,
            Key::KEY_LEFTALT | Key::KEY_RIGHTALT => self.alt = pressed,
            Key::KEY_LEFTSHIFT | Key::KEY_RIGHTSHIFT => self.shift = pressed,
            Key::KEY_LEFTMETA | Key::KEY_RIGHTMETA => self.super_key = pressed,
            _ => {}
        }
    }
}
