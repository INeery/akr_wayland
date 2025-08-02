use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Маппинг между именами клавиш и кодами evdev
pub struct KeycodeMap;

// Статическая карта основных клавиш
static KEY_NAME_TO_CODE: Lazy<HashMap<&'static str, u16>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Буквенные клавиши
    map.insert("a", 30);  // KEY_A
    map.insert("b", 48);  // KEY_B
    map.insert("c", 46);  // KEY_C
    map.insert("d", 32);  // KEY_D
    map.insert("e", 18);  // KEY_E
    map.insert("f", 33);  // KEY_F
    map.insert("g", 34);  // KEY_G
    map.insert("h", 35);  // KEY_H
    map.insert("i", 23);  // KEY_I
    map.insert("j", 36);  // KEY_J
    map.insert("k", 37);  // KEY_K
    map.insert("l", 38);  // KEY_L
    map.insert("m", 50);  // KEY_M
    map.insert("n", 49);  // KEY_N
    map.insert("o", 24);  // KEY_O
    map.insert("p", 25);  // KEY_P
    map.insert("q", 16);  // KEY_Q
    map.insert("r", 19);  // KEY_R
    map.insert("s", 31);  // KEY_S
    map.insert("t", 20);  // KEY_T
    map.insert("u", 22);  // KEY_U
    map.insert("v", 47);  // KEY_V
    map.insert("w", 17);  // KEY_W
    map.insert("x", 45);  // KEY_X
    map.insert("y", 21);  // KEY_Y
    map.insert("z", 44);  // KEY_Z

    // Цифровые клавиши (верхний ряд)
    map.insert("1", 2);   // KEY_1
    map.insert("2", 3);   // KEY_2
    map.insert("3", 4);   // KEY_3
    map.insert("4", 5);   // KEY_4
    map.insert("5", 6);   // KEY_5
    map.insert("6", 7);   // KEY_6
    map.insert("7", 8);   // KEY_7
    map.insert("8", 9);   // KEY_8
    map.insert("9", 10);  // KEY_9
    map.insert("0", 11);  // KEY_0

    // Специальные клавиши
    map.insert("space", 57);      // KEY_SPACE
    map.insert("enter", 28);      // KEY_ENTER
    map.insert("escape", 1);      // KEY_ESC
    map.insert("backspace", 14);  // KEY_BACKSPACE
    map.insert("tab", 15);        // KEY_TAB

    // Модификаторы
    map.insert("ctrl", 29);       // KEY_LEFTCTRL
    map.insert("alt", 56);        // KEY_LEFTALT
    map.insert("shift", 42);      // KEY_LEFTSHIFT
    map.insert("super", 125);     // KEY_LEFTMETA

    // Стрелки
    map.insert("up", 103);        // KEY_UP
    map.insert("down", 108);      // KEY_DOWN
    map.insert("left", 105);      // KEY_LEFT
    map.insert("right", 106);     // KEY_RIGHT

    map
});

static CODE_TO_KEY_NAME: Lazy<HashMap<u16, &'static str>> = Lazy::new(|| {
    KEY_NAME_TO_CODE.iter().map(|(&name, &code)| (code, name)).collect()
});

impl KeycodeMap {
    /// Получить код клавиши по её имени
    pub fn get_keycode(key_name: &str) -> Result<u16, String> {
        let normalized = key_name.to_lowercase();
        KEY_NAME_TO_CODE.get(normalized.as_str())
            .copied()
            .ok_or_else(|| format!("Unknown key: {}", key_name))
    }

    /// Получить имя клавиши по её коду
    pub fn get_key_name(keycode: u16) -> Option<&'static str> {
        CODE_TO_KEY_NAME.get(&keycode).copied()
    }

    /// Проверить, является ли клавиша модификатором
    pub fn is_modifier(key_name: &str) -> bool {
        let normalized = key_name.to_lowercase();
        matches!(normalized.as_str(), "ctrl" | "alt" | "shift" | "super")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_key_mapping() {
        assert_eq!(KeycodeMap::get_keycode("a").unwrap(), 30);
        assert_eq!(KeycodeMap::get_keycode("space").unwrap(), 57);
        assert_eq!(KeycodeMap::get_keycode("ctrl").unwrap(), 29);
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(KeycodeMap::get_keycode("A").unwrap(), 30);
        assert_eq!(KeycodeMap::get_keycode("SPACE").unwrap(), 57);
    }

    #[test]
    fn test_reverse_mapping() {
        assert_eq!(KeycodeMap::get_key_name(30), Some("a"));
        assert_eq!(KeycodeMap::get_key_name(57), Some("space"));
    }

    #[test]
    fn test_invalid_key() {
        assert!(KeycodeMap::get_keycode("invalid_key").is_err());
    }

    #[test]
    fn test_modifier_detection() {
        assert!(KeycodeMap::is_modifier("ctrl"));
        assert!(KeycodeMap::is_modifier("SHIFT"));
        assert!(!KeycodeMap::is_modifier("a"));
        assert!(!KeycodeMap::is_modifier("space"));
    }
}
