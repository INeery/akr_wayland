/// Преобразование имён клавиш в evdev коды
/// Отвечает за трансляцию строковых имён клавиш в числовые коды evdev
pub struct KeyNameToEvdevCode;

impl KeyNameToEvdevCode {
    /// Получить evdev код клавиши по её имени
    pub fn translate(key_name: &str) -> Result<u16, String> {
        let normalized = key_name.to_lowercase();
        let code = match normalized.as_str() {
            // Буквенные клавиши
            "a" => 30,   // KEY_A
            "b" => 48,   // KEY_B
            "c" => 46,   // KEY_C
            "d" => 32,   // KEY_D
            "e" => 18,   // KEY_E
            "f" => 33,   // KEY_F
            "g" => 34,   // KEY_G
            "h" => 35,   // KEY_H
            "i" => 23,   // KEY_I
            "j" => 36,   // KEY_J
            "k" => 37,   // KEY_K
            "l" => 38,   // KEY_L
            "m" => 50,   // KEY_M
            "n" => 49,   // KEY_N
            "o" => 24,   // KEY_O
            "p" => 25,   // KEY_P
            "q" => 16,   // KEY_Q
            "r" => 19,   // KEY_R
            "s" => 31,   // KEY_S
            "t" => 20,   // KEY_T
            "u" => 22,   // KEY_U
            "v" => 47,   // KEY_V
            "w" => 17,   // KEY_W
            "x" => 45,   // KEY_X
            "y" => 21,   // KEY_Y
            "z" => 44,   // KEY_Z

            // Цифровые клавиши (верхний ряд)
            "1" => 2,    // KEY_1
            "2" => 3,    // KEY_2
            "3" => 4,    // KEY_3
            "4" => 5,    // KEY_4
            "5" => 6,    // KEY_5
            "6" => 7,    // KEY_6
            "7" => 8,    // KEY_7
            "8" => 9,    // KEY_8
            "9" => 10,   // KEY_9
            "0" => 11,   // KEY_0

            // Специальные клавиши
            "space" => 57,       // KEY_SPACE
            "enter" => 28,       // KEY_ENTER
            "escape" => 1,       // KEY_ESC
            "backspace" => 14,   // KEY_BACKSPACE
            "tab" => 15,         // KEY_TAB

            // Модификаторы
            "ctrl" => 29,        // KEY_LEFTCTRL
            "alt" => 56,         // KEY_LEFTALT
            "shift" => 42,       // KEY_LEFTSHIFT
            "super" => 125,      // KEY_LEFTMETA

            // Стрелки
            "up" => 103,         // KEY_UP
            "down" => 108,       // KEY_DOWN
            "left" => 105,       // KEY_LEFT
            "right" => 106,      // KEY_RIGHT

            // Функциональные клавиши
            "f1" => 59,          // KEY_F1
            "f2" => 60,          // KEY_F2
            "f3" => 61,          // KEY_F3
            "f4" => 62,          // KEY_F4
            "f5" => 63,          // KEY_F5
            "f6" => 64,          // KEY_F6
            "f7" => 65,          // KEY_F7
            "f8" => 66,          // KEY_F8
            "f9" => 67,          // KEY_F9
            "f10" => 68,         // KEY_F10
            "f11" => 87,         // KEY_F11
            "f12" => 88,         // KEY_F12

            _ => return Err(format!("Unknown key: {}", key_name)),
        };

        Ok(code)
    }

    /// Получить имя клавиши по evdev коду
    pub fn reverse_translate(keycode: u16) -> Option<&'static str> {
        match keycode {
            // Буквенные клавиши
            30 => Some("a"),   // KEY_A
            48 => Some("b"),   // KEY_B
            46 => Some("c"),   // KEY_C
            32 => Some("d"),   // KEY_D
            18 => Some("e"),   // KEY_E
            33 => Some("f"),   // KEY_F
            34 => Some("g"),   // KEY_G
            35 => Some("h"),   // KEY_H
            23 => Some("i"),   // KEY_I
            36 => Some("j"),   // KEY_J
            37 => Some("k"),   // KEY_K
            38 => Some("l"),   // KEY_L
            50 => Some("m"),   // KEY_M
            49 => Some("n"),   // KEY_N
            24 => Some("o"),   // KEY_O
            25 => Some("p"),   // KEY_P
            16 => Some("q"),   // KEY_Q
            19 => Some("r"),   // KEY_R
            31 => Some("s"),   // KEY_S
            20 => Some("t"),   // KEY_T
            22 => Some("u"),   // KEY_U
            47 => Some("v"),   // KEY_V
            17 => Some("w"),   // KEY_W
            45 => Some("x"),   // KEY_X
            21 => Some("y"),   // KEY_Y
            44 => Some("z"),   // KEY_Z

            // Цифровые клавиши
            2 => Some("1"),    // KEY_1
            3 => Some("2"),    // KEY_2
            4 => Some("3"),    // KEY_3
            5 => Some("4"),    // KEY_4
            6 => Some("5"),    // KEY_5
            7 => Some("6"),    // KEY_6
            8 => Some("7"),    // KEY_7
            9 => Some("8"),    // KEY_8
            10 => Some("9"),   // KEY_9
            11 => Some("0"),   // KEY_0

            // Специальные клавиши
            57 => Some("space"),      // KEY_SPACE
            28 => Some("enter"),      // KEY_ENTER
            1 => Some("escape"),      // KEY_ESC
            14 => Some("backspace"),  // KEY_BACKSPACE
            15 => Some("tab"),        // KEY_TAB

            // Модификаторы
            29 => Some("ctrl"),       // KEY_LEFTCTRL
            56 => Some("alt"),        // KEY_LEFTALT
            42 => Some("shift"),      // KEY_LEFTSHIFT
            125 => Some("super"),     // KEY_LEFTMETA

            // Стрелки
            103 => Some("up"),        // KEY_UP
            108 => Some("down"),      // KEY_DOWN
            105 => Some("left"),      // KEY_LEFT
            106 => Some("right"),     // KEY_RIGHT

            // Функциональные клавиши
            59 => Some("f1"),         // KEY_F1
            60 => Some("f2"),         // KEY_F2
            61 => Some("f3"),         // KEY_F3
            62 => Some("f4"),         // KEY_F4
            63 => Some("f5"),         // KEY_F5
            64 => Some("f6"),         // KEY_F6
            65 => Some("f7"),         // KEY_F7
            66 => Some("f8"),         // KEY_F8
            67 => Some("f9"),         // KEY_F9
            68 => Some("f10"),        // KEY_F10
            87 => Some("f11"),        // KEY_F11
            88 => Some("f12"),        // KEY_F12

            _ => None,
        }
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
        assert_eq!(KeyNameToEvdevCode::translate("a").unwrap(), 30);
        assert_eq!(KeyNameToEvdevCode::translate("space").unwrap(), 57);
        assert_eq!(KeyNameToEvdevCode::translate("ctrl").unwrap(), 29);
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(KeyNameToEvdevCode::translate("A").unwrap(), 30);
        assert_eq!(KeyNameToEvdevCode::translate("SPACE").unwrap(), 57);
    }

    #[test]
    fn test_reverse_mapping() {
        assert_eq!(KeyNameToEvdevCode::reverse_translate(30), Some("a"));
        assert_eq!(KeyNameToEvdevCode::reverse_translate(57), Some("space"));
    }

    #[test]
    fn test_invalid_key() {
        assert!(KeyNameToEvdevCode::translate("invalid_key").is_err());
    }

    #[test]
    fn test_modifier_detection() {
        assert!(KeyNameToEvdevCode::is_modifier("ctrl"));
        assert!(KeyNameToEvdevCode::is_modifier("SHIFT"));
        assert!(!KeyNameToEvdevCode::is_modifier("a"));
        assert!(!KeyNameToEvdevCode::is_modifier("space"));
    }

    #[test]
    fn test_function_keys() {
        assert_eq!(KeyNameToEvdevCode::translate("f1").unwrap(), 59);
        assert_eq!(KeyNameToEvdevCode::translate("F12").unwrap(), 88);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(59), Some("f1"));
        assert_eq!(KeyNameToEvdevCode::reverse_translate(88), Some("f12"));
    }
}