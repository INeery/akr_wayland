use evdev::KeyCode;

/// Маппинг клавиш с использованием современных KeyCode констант из evdev 0.13.1
pub struct KeyMapper;

impl KeyMapper {
    pub fn get_key_name(key_code: u16) -> Option<String> {
        let keycode = KeyCode::new(key_code);
        let name = match keycode {
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
            _ => None,
        };

        name.map(|s| s.to_string())
    }
}
