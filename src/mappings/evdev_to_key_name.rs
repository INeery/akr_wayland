use evdev::Key;

/// Преобразование evdev::Key в имена клавиш для конфигурации
/// Отвечает за трансляцию evdev типов в строковые имена клавиш
pub struct EvdevToKeyName;

impl EvdevToKeyName {
    /// Получить имя клавиши из evdev::Key
    pub fn translate(key: Key) -> Option<&'static str> {
        match key {
            // Буквенные клавиши
            Key::KEY_A => Some("a"),
            Key::KEY_B => Some("b"),
            Key::KEY_C => Some("c"),
            Key::KEY_D => Some("d"),
            Key::KEY_E => Some("e"),
            Key::KEY_F => Some("f"),
            Key::KEY_G => Some("g"),
            Key::KEY_H => Some("h"),
            Key::KEY_I => Some("i"),
            Key::KEY_J => Some("j"),
            Key::KEY_K => Some("k"),
            Key::KEY_L => Some("l"),
            Key::KEY_M => Some("m"),
            Key::KEY_N => Some("n"),
            Key::KEY_O => Some("o"),
            Key::KEY_P => Some("p"),
            Key::KEY_Q => Some("q"),
            Key::KEY_R => Some("r"),
            Key::KEY_S => Some("s"),
            Key::KEY_T => Some("t"),
            Key::KEY_U => Some("u"),
            Key::KEY_V => Some("v"),
            Key::KEY_W => Some("w"),
            Key::KEY_X => Some("x"),
            Key::KEY_Y => Some("y"),
            Key::KEY_Z => Some("z"),
            
            // Цифровые клавиши
            Key::KEY_1 => Some("1"),
            Key::KEY_2 => Some("2"),
            Key::KEY_3 => Some("3"),
            Key::KEY_4 => Some("4"),
            Key::KEY_5 => Some("5"),
            Key::KEY_6 => Some("6"),
            Key::KEY_7 => Some("7"),
            Key::KEY_8 => Some("8"),
            Key::KEY_9 => Some("9"),
            Key::KEY_0 => Some("0"),
            
            // Специальные клавиши
            Key::KEY_SPACE => Some("space"),
            Key::KEY_ENTER => Some("enter"),
            Key::KEY_UP => Some("up"),
            Key::KEY_DOWN => Some("down"),
            Key::KEY_LEFT => Some("left"),
            Key::KEY_RIGHT => Some("right"),
            Key::KEY_ESC => Some("escape"),
            Key::KEY_BACKSPACE => Some("backspace"),
            Key::KEY_TAB => Some("tab"),

            // Модификаторы
            Key::KEY_LEFTCTRL => Some("ctrl"),
            Key::KEY_LEFTALT => Some("alt"),
            Key::KEY_LEFTSHIFT => Some("shift"),
            Key::KEY_LEFTMETA => Some("super"),
            
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_letter_keys() {
        assert_eq!(EvdevToKeyName::translate(Key::KEY_A), Some("a"));
        assert_eq!(EvdevToKeyName::translate(Key::KEY_Z), Some("z"));
    }

    #[test]
    fn test_number_keys() {
        assert_eq!(EvdevToKeyName::translate(Key::KEY_1), Some("1"));
        assert_eq!(EvdevToKeyName::translate(Key::KEY_0), Some("0"));
    }

    #[test]
    fn test_special_keys() {
        assert_eq!(EvdevToKeyName::translate(Key::KEY_SPACE), Some("space"));
        assert_eq!(EvdevToKeyName::translate(Key::KEY_ENTER), Some("enter"));
    }

    #[test]
    fn test_unknown_key() {
        assert_eq!(EvdevToKeyName::translate(Key::KEY_F1), None);
    }
}