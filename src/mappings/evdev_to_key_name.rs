use evdev::KeyCode;

/// Преобразование evdev KeyCode в имена клавиш для конфигурации
/// Использует современные KeyCode константы из evdev 0.13.1 вместо сырых числовых кодов
pub struct EvdevToKeyName;

impl EvdevToKeyName {
    /// Получить имя клавиши из key code
    pub fn translate(key_code: u16) -> Option<&'static str> {
        let keycode = KeyCode::new(key_code);
        match keycode {
            // Буквенные клавиши
            KeyCode::KEY_A => Some("a"),
            KeyCode::KEY_B => Some("b"),
            KeyCode::KEY_C => Some("c"),
            KeyCode::KEY_D => Some("d"),
            KeyCode::KEY_E => Some("e"),
            KeyCode::KEY_F => Some("f"),
            KeyCode::KEY_G => Some("g"),
            KeyCode::KEY_H => Some("h"),
            KeyCode::KEY_I => Some("i"),
            KeyCode::KEY_J => Some("j"),
            KeyCode::KEY_K => Some("k"),
            KeyCode::KEY_L => Some("l"),
            KeyCode::KEY_M => Some("m"),
            KeyCode::KEY_N => Some("n"),
            KeyCode::KEY_O => Some("o"),
            KeyCode::KEY_P => Some("p"),
            KeyCode::KEY_Q => Some("q"),
            KeyCode::KEY_R => Some("r"),
            KeyCode::KEY_S => Some("s"),
            KeyCode::KEY_T => Some("t"),
            KeyCode::KEY_U => Some("u"),
            KeyCode::KEY_V => Some("v"),
            KeyCode::KEY_W => Some("w"),
            KeyCode::KEY_X => Some("x"),
            KeyCode::KEY_Y => Some("y"),
            KeyCode::KEY_Z => Some("z"),
            
            // Цифровые клавиши
            KeyCode::KEY_1 => Some("1"),
            KeyCode::KEY_2 => Some("2"),
            KeyCode::KEY_3 => Some("3"),
            KeyCode::KEY_4 => Some("4"),
            KeyCode::KEY_5 => Some("5"),
            KeyCode::KEY_6 => Some("6"),
            KeyCode::KEY_7 => Some("7"),
            KeyCode::KEY_8 => Some("8"),
            KeyCode::KEY_9 => Some("9"),
            KeyCode::KEY_0 => Some("0"),
            
            // Специальные клавиши
            KeyCode::KEY_SPACE => Some("space"),
            KeyCode::KEY_ENTER => Some("enter"),
            KeyCode::KEY_UP => Some("up"),
            KeyCode::KEY_DOWN => Some("down"),
            KeyCode::KEY_LEFT => Some("left"),
            KeyCode::KEY_RIGHT => Some("right"),
            KeyCode::KEY_ESC => Some("escape"),
            KeyCode::KEY_BACKSPACE => Some("backspace"),
            KeyCode::KEY_TAB => Some("tab"),

            // Модификаторы
            KeyCode::KEY_LEFTCTRL => Some("ctrl"),
            KeyCode::KEY_LEFTALT => Some("alt"),
            KeyCode::KEY_LEFTSHIFT => Some("shift"),
            KeyCode::KEY_LEFTMETA => Some("super"),
            
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use evdev::KeyCode;

    #[test]
    fn test_keycode_constants() {
        // Test that KeyCode constants have the expected values
        assert_eq!(KeyCode::KEY_A.code(), 30);
        assert_eq!(KeyCode::KEY_SPACE.code(), 57);
        assert_eq!(KeyCode::KEY_ENTER.code(), 28);
        assert_eq!(KeyCode::KEY_1.code(), 2);
        assert_eq!(KeyCode::KEY_LEFTCTRL.code(), 29);
    }

    #[test]
    fn test_letter_keys() {
        assert_eq!(EvdevToKeyName::translate(30), Some("a")); // KEY_A
        assert_eq!(EvdevToKeyName::translate(44), Some("z")); // KEY_Z
    }

    #[test]
    fn test_number_keys() {
        assert_eq!(EvdevToKeyName::translate(2), Some("1"));  // KEY_1
        assert_eq!(EvdevToKeyName::translate(11), Some("0")); // KEY_0
    }

    #[test]
    fn test_special_keys() {
        assert_eq!(EvdevToKeyName::translate(57), Some("space")); // KEY_SPACE
        assert_eq!(EvdevToKeyName::translate(28), Some("enter")); // KEY_ENTER
    }

    #[test]
    fn test_unknown_key() {
        assert_eq!(EvdevToKeyName::translate(59), None); // KEY_F1
    }
}