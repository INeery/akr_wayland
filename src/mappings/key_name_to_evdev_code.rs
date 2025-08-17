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

            // Знаки пунктуации и прочие клавиши раскладки
            "minus" => 12,           // KEY_MINUS (-)
            "equal" => 13,           // KEY_EQUAL (=)
            "leftbrace" => 26,       // KEY_LEFTBRACE ([)
            "rightbrace" => 27,      // KEY_RIGHTBRACE (])
            "backslash" => 43,       // KEY_BACKSLASH (\\)
            "semicolon" => 39,       // KEY_SEMICOLON (;)
            "apostrophe" => 40,      // KEY_APOSTROPHE (')
            "grave" => 41,           // KEY_GRAVE (`)
            "comma" => 51,           // KEY_COMMA (,)
            "dot" => 52,             // KEY_DOT (.)
            "slash" => 53,           // KEY_SLASH (/)

            // Навигация/редакция
            "insert" => 110,         // KEY_INSERT
            "delete" => 111,         // KEY_DELETE
            "home" => 102,           // KEY_HOME
            "end" => 107,            // KEY_END
            "pageup" => 104,         // KEY_PAGEUP
            "pagedown" => 109,       // KEY_PAGEDOWN

            // Системные
            "printscreen" => 99,     // KEY_SYSRQ (PrintScreen)
            "scrolllock" => 70,      // KEY_SCROLLLOCK
            "pause" => 119,          // KEY_PAUSE

            // Numpad
            "kp0" => 82,             // KEY_KP0
            "kp1" => 79,             // KEY_KP1
            "kp2" => 80,             // KEY_KP2
            "kp3" => 81,             // KEY_KP3
            "kp4" => 75,             // KEY_KP4
            "kp5" => 76,             // KEY_KP5
            "kp6" => 77,             // KEY_KP6
            "kp7" => 71,             // KEY_KP7
            "kp8" => 72,             // KEY_KP8
            "kp9" => 73,             // KEY_KP9
            "kpdecimal" => 83,       // KEY_KPDOT
            "kpdivide" => 98,        // KEY_KPSLASH
            "kpmultiply" => 55,      // KEY_KPASTERISK
            "kpadd" => 78,           // KEY_KPPLUS
            "kpsubtract" => 74,      // KEY_KPMINUS
            "kpenter" => 96,         // KEY_KPENTER

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

            // Пунктуация
            12 => Some("minus"),      // KEY_MINUS
            13 => Some("equal"),      // KEY_EQUAL
            26 => Some("leftbrace"),  // KEY_LEFTBRACE
            27 => Some("rightbrace"), // KEY_RIGHTBRACE
            43 => Some("backslash"),  // KEY_BACKSLASH
            39 => Some("semicolon"),  // KEY_SEMICOLON
            40 => Some("apostrophe"), // KEY_APOSTROPHE
            41 => Some("grave"),      // KEY_GRAVE
            51 => Some("comma"),      // KEY_COMMA
            52 => Some("dot"),        // KEY_DOT
            53 => Some("slash"),      // KEY_SLASH

            // Навигация/редакция
            110 => Some("insert"),    // KEY_INSERT
            111 => Some("delete"),    // KEY_DELETE
            102 => Some("home"),      // KEY_HOME
            107 => Some("end"),       // KEY_END
            104 => Some("pageup"),    // KEY_PAGEUP
            109 => Some("pagedown"),  // KEY_PAGEDOWN

            // Системные
            99 => Some("printscreen"),// KEY_SYSRQ
            70 => Some("scrolllock"), // KEY_SCROLLLOCK
            119 => Some("pause"),     // KEY_PAUSE

            // Numpad
            82 => Some("kp0"),
            79 => Some("kp1"),
            80 => Some("kp2"),
            81 => Some("kp3"),
            75 => Some("kp4"),
            76 => Some("kp5"),
            77 => Some("kp6"),
            71 => Some("kp7"),
            72 => Some("kp8"),
            73 => Some("kp9"),
            83 => Some("kpdecimal"),
            98 => Some("kpdivide"),
            55 => Some("kpmultiply"),
            78 => Some("kpadd"),
            74 => Some("kpsubtract"),
            96 => Some("kpenter"),

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

    #[test]
    fn test_punctuation_keys_translate_and_reverse() {
        assert_eq!(KeyNameToEvdevCode::translate("minus").unwrap(), 12);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(12), Some("minus"));
        assert_eq!(KeyNameToEvdevCode::translate("equal").unwrap(), 13);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(13), Some("equal"));
        assert_eq!(KeyNameToEvdevCode::translate("leftbrace").unwrap(), 26);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(26), Some("leftbrace"));
        assert_eq!(KeyNameToEvdevCode::translate("semicolon").unwrap(), 39);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(39), Some("semicolon"));
        assert_eq!(KeyNameToEvdevCode::translate("apostrophe").unwrap(), 40);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(40), Some("apostrophe"));
        assert_eq!(KeyNameToEvdevCode::translate("backslash").unwrap(), 43);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(43), Some("backslash"));
        assert_eq!(KeyNameToEvdevCode::translate("grave").unwrap(), 41);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(41), Some("grave"));
        assert_eq!(KeyNameToEvdevCode::translate("comma").unwrap(), 51);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(51), Some("comma"));
        assert_eq!(KeyNameToEvdevCode::translate("dot").unwrap(), 52);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(52), Some("dot"));
        assert_eq!(KeyNameToEvdevCode::translate("slash").unwrap(), 53);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(53), Some("slash"));
    }

    #[test]
    fn test_navigation_system_numpad_keys() {
        // Navigation
        assert_eq!(KeyNameToEvdevCode::translate("insert").unwrap(), 110);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(110), Some("insert"));
        assert_eq!(KeyNameToEvdevCode::translate("delete").unwrap(), 111);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(111), Some("delete"));
        assert_eq!(KeyNameToEvdevCode::translate("home").unwrap(), 102);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(102), Some("home"));
        assert_eq!(KeyNameToEvdevCode::translate("end").unwrap(), 107);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(107), Some("end"));
        assert_eq!(KeyNameToEvdevCode::translate("pageup").unwrap(), 104);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(104), Some("pageup"));
        assert_eq!(KeyNameToEvdevCode::translate("pagedown").unwrap(), 109);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(109), Some("pagedown"));

        // System
        assert_eq!(KeyNameToEvdevCode::translate("printscreen").unwrap(), 99);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(99), Some("printscreen"));
        assert_eq!(KeyNameToEvdevCode::translate("scrolllock").unwrap(), 70);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(70), Some("scrolllock"));
        assert_eq!(KeyNameToEvdevCode::translate("pause").unwrap(), 119);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(119), Some("pause"));

        // Numpad
        assert_eq!(KeyNameToEvdevCode::translate("kp0").unwrap(), 82);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(82), Some("kp0"));
        assert_eq!(KeyNameToEvdevCode::translate("kpadd").unwrap(), 78);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(78), Some("kpadd"));
        assert_eq!(KeyNameToEvdevCode::translate("kpsubtract").unwrap(), 74);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(74), Some("kpsubtract"));
        assert_eq!(KeyNameToEvdevCode::translate("kpmultiply").unwrap(), 55);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(55), Some("kpmultiply"));
        assert_eq!(KeyNameToEvdevCode::translate("kpdivide").unwrap(), 98);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(98), Some("kpdivide"));
        assert_eq!(KeyNameToEvdevCode::translate("kpdecimal").unwrap(), 83);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(83), Some("kpdecimal"));
        assert_eq!(KeyNameToEvdevCode::translate("kpenter").unwrap(), 96);
        assert_eq!(KeyNameToEvdevCode::reverse_translate(96), Some("kpenter"));
    }
}