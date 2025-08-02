use serde::{Deserialize, Serialize};
use std::fmt;

/// Состояние клавиши
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyState {
    Pressed,
    Released,
    Repeat,
}

/// Код клавиши (evdev коды)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCode(pub u16);

impl KeyCode {
    #[allow(dead_code)]
    pub fn new(code: u16) -> Self {
        Self(code)
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KEY_{}", self.0)
    }
}

/// Модификаторы клавиш
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
}

impl Modifiers {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn with_ctrl(mut self, ctrl: bool) -> Self {
        self.ctrl = ctrl;
        self
    }

    #[allow(dead_code)]
    pub fn with_alt(mut self, alt: bool) -> Self {
        self.alt = alt;
        self
    }

    #[allow(dead_code)]
    pub fn with_shift(mut self, shift: bool) -> Self {
        self.shift = shift;
        self
    }

    pub fn is_empty(&self) -> bool {
        !self.ctrl && !self.alt && !self.shift && !self.super_key
    }

    #[allow(dead_code)]
    pub fn has_any(&self) -> bool {
        !self.is_empty()
    }

    pub fn to_vec(&self) -> Vec<String> {
        let mut result = Vec::new();
        if self.ctrl { result.push("ctrl".to_string()); }
        if self.alt { result.push("alt".to_string()); }
        if self.shift { result.push("shift".to_string()); }
        if self.super_key { result.push("super".to_string()); }
        result
    }

    #[allow(dead_code)]
    pub fn from_vec(modifiers: &[String]) -> Self {
        let mut result = Self::new();
        for modifier in modifiers {
            match modifier.as_str() {
                "ctrl" => result.ctrl = true,
                "alt" => result.alt = true,
                "shift" => result.shift = true,
                "super" => result.super_key = true,
                _ => {}
            }
        }
        result
    }
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let modifiers = self.to_vec();
        if modifiers.is_empty() {
            write!(f, "none")
        } else {
            write!(f, "{}", modifiers.join("+"))
        }
    }
}

/// Событие клавиатуры
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    pub key_code: KeyCode,
    pub state: KeyState,
    pub modifiers: Modifiers,
    pub timestamp: std::time::Instant,
    pub device_name: String,
}

impl KeyEvent {
    #[allow(dead_code)]
    pub fn new(
        key_code: KeyCode,
        state: KeyState,
        modifiers: Modifiers,
        device_name: String,
    ) -> Self {
        Self {
            key_code,
            state,
            modifiers,
            timestamp: std::time::Instant::now(),
            device_name,
        }
    }

    /// Получить уникальный идентификатор комбинации клавиш
    pub fn combination_id(&self) -> String {
        if self.modifiers.is_empty() {
            format!("{}", self.key_code.value())
        } else {
            format!("{}+{}", self.modifiers, self.key_code.value())
        }
    }
}

impl fmt::Display for KeyEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}[{}] {:?} ({})",
            self.combination_id(),
            self.device_name,
            self.state,
            self.timestamp.elapsed().as_millis()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifiers_creation() {
        let modifiers = Modifiers::new()
            .with_ctrl(true)
            .with_shift(true);

        assert!(modifiers.ctrl);
        assert!(modifiers.shift);
        assert!(!modifiers.alt);
        assert!(!modifiers.super_key);
        assert!(modifiers.has_any());
    }

    #[test]
    fn test_modifiers_to_from_vec() {
        let original = Modifiers::new()
            .with_ctrl(true)
            .with_alt(true);

        let vec = original.to_vec();
        let restored = Modifiers::from_vec(&vec);

        assert_eq!(original, restored);
    }

    #[test]
    fn test_key_event_combination_id() {
        let event1 = KeyEvent::new(
            KeyCode::new(42),
            KeyState::Pressed,
            Modifiers::new(),
            "test".to_string(),
        );

        let event2 = KeyEvent::new(
            KeyCode::new(42),
            KeyState::Pressed,
            Modifiers::new().with_ctrl(true),
            "test".to_string(),
        );

        assert_eq!(event1.combination_id(), "42");
        assert_eq!(event2.combination_id(), "ctrl+42");
    }
}
