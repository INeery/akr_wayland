use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::fmt;
use std::hash::{Hash, Hasher};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Modifiers {
    bits: u8,
}

impl Modifiers {
    const CTRL: u8 = 1 << 0;
    const ALT: u8 = 1 << 1;
    const SHIFT: u8 = 1 << 2;
    const SUPER: u8 = 1 << 3;

    pub fn new() -> Self {
        Self { bits: 0 }
    }

    #[allow(dead_code)]
    pub fn with_ctrl(mut self, ctrl: bool) -> Self {
        if ctrl {
            self.bits |= Self::CTRL;
        } else {
            self.bits &= !Self::CTRL;
        }
        self
    }

    #[allow(dead_code)]
    pub fn with_alt(mut self, alt: bool) -> Self {
        if alt {
            self.bits |= Self::ALT;
        } else {
            self.bits &= !Self::ALT;
        }
        self
    }

    #[allow(dead_code)]
    pub fn with_shift(mut self, shift: bool) -> Self {
        if shift {
            self.bits |= Self::SHIFT;
        } else {
            self.bits &= !Self::SHIFT;
        }
        self
    }

    #[allow(dead_code)]
    pub fn with_super(mut self, super_key: bool) -> Self {
        if super_key {
            self.bits |= Self::SUPER;
        } else {
            self.bits &= !Self::SUPER;
        }
        self
    }

    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    #[allow(dead_code)]
    pub fn has_any(&self) -> bool {
        !self.is_empty()
    }

    // Геттеры для совместимости
    #[allow(dead_code)]
    pub fn ctrl(&self) -> bool {
        self.bits & Self::CTRL != 0
    }

    #[allow(dead_code)]
    pub fn alt(&self) -> bool {
        self.bits & Self::ALT != 0
    }

    #[allow(dead_code)]
    pub fn shift(&self) -> bool {
        self.bits & Self::SHIFT != 0
    }

    #[allow(dead_code)]
    pub fn super_key(&self) -> bool {
        self.bits & Self::SUPER != 0
    }

    pub fn to_vec(&self) -> Vec<String> {
        let mut result = Vec::new();
        if self.bits & Self::CTRL != 0 {
            result.push("ctrl".to_string());
        }
        if self.bits & Self::ALT != 0 {
            result.push("alt".to_string());
        }
        if self.bits & Self::SHIFT != 0 {
            result.push("shift".to_string());
        }
        if self.bits & Self::SUPER != 0 {
            result.push("super".to_string());
        }
        result
    }

    // Оптимизированная версия без аллокаций
    #[allow(dead_code)]
    pub fn to_string_vec(&self) -> SmallVec<[&'static str; 4]> {
        let mut result = SmallVec::new();
        if self.bits & Self::CTRL != 0 {
            result.push("ctrl");
        }
        if self.bits & Self::ALT != 0 {
            result.push("alt");
        }
        if self.bits & Self::SHIFT != 0 {
            result.push("shift");
        }
        if self.bits & Self::SUPER != 0 {
            result.push("super");
        }
        result
    }

    #[allow(dead_code)]
    pub fn from_vec(modifiers: &[String]) -> Self {
        let mut result = Self::new();
        for modifier in modifiers {
            match modifier.as_str() {
                "ctrl" => result.bits |= Self::CTRL,
                "alt" => result.bits |= Self::ALT,
                "shift" => result.bits |= Self::SHIFT,
                "super" => result.bits |= Self::SUPER,
                _ => {}
            }
        }
        result
    }

    #[allow(dead_code)]
    pub fn from_bits(bits: u8) -> Self {
        Self { bits }
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

// Именованные константы для device_id (нулевой overhead)
pub mod device_ids {
    pub const LISTENER_VIRTUAL_KEYBOARD: u8 = 0;
    pub const REPEATER_VIRTUAL_KEYBOARD: u8 = 1;
    
    pub fn name(device_id: u8) -> &'static str {
        match device_id {
            LISTENER_VIRTUAL_KEYBOARD => "Listener Virtual Keyboard",
            REPEATER_VIRTUAL_KEYBOARD => "Repeater Virtual Keyboard",
            _ => "Unknown Device",
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
    pub device_id: u8,
}

impl KeyEvent {
    #[allow(dead_code)]
    pub fn new(key_code: KeyCode, state: KeyState, modifiers: Modifiers, device_id: u8) -> Self {
        Self {
            key_code,
            state,
            modifiers,
            timestamp: std::time::Instant::now(),
            device_id,
        }
    }

    /// Получить имя устройства без аллокации
    pub fn device_name(&self) -> &'static str {
        device_ids::name(self.device_id)
    }

    /// Получить уникальный идентификатор комбинации клавиш
    pub fn combination_id(&self) -> String {
        if self.modifiers.is_empty() {
            format!("{}", self.key_code.value())
        } else {
            format!("{}+{}", self.modifiers, self.key_code.value())
        }
    }

    /// Кэшированный combination_id через hash (без аллокаций)
    #[allow(dead_code)]
    pub fn combination_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.key_code.value().hash(&mut hasher);
        self.modifiers.hash(&mut hasher);
        hasher.finish()
    }

    /// Хеш только по основной клавише (для устойчивости к race conditions с модификаторами)
    pub fn key_only_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.key_code.value().hash(&mut hasher);
        hasher.finish()
    }
}

impl fmt::Display for KeyEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}[{}] {:?} ({})",
            self.combination_id(),
            self.device_name(),
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
        let modifiers = Modifiers::new().with_ctrl(true).with_shift(true);

        assert!(modifiers.ctrl());
        assert!(modifiers.shift());
        assert!(!modifiers.alt());
        assert!(!modifiers.super_key());
        assert!(modifiers.has_any());
    }

    #[test]
    fn test_modifiers_to_from_vec() {
        let original = Modifiers::new().with_ctrl(true).with_alt(true);

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
            0, // device_id
        );

        let event2 = KeyEvent::new(
            KeyCode::new(42),
            KeyState::Pressed,
            Modifiers::new().with_ctrl(true),
            0, // device_id
        );

        assert_eq!(event1.combination_id(), "42");
        assert_eq!(event2.combination_id(), "ctrl+42");
    }

    #[test]
    fn test_combination_hash_same_for_press_release() {
        let press_event = KeyEvent::new(
            KeyCode::new(42),
            KeyState::Pressed,
            Modifiers::new().with_ctrl(true),
            0,
        );

        let release_event = KeyEvent::new(
            KeyCode::new(42),
            KeyState::Released,
            Modifiers::new().with_ctrl(true),
            0,
        );

        // Press и Release события должны иметь одинаковый hash
        assert_eq!(press_event.combination_hash(), release_event.combination_hash());
        
        // Но разные combination_id из-за разного состояния? Нет, combination_id не включает state
        assert_eq!(press_event.combination_id(), release_event.combination_id());
    }
}
