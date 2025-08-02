use evdev::Key;

pub struct KeyMapper;

impl KeyMapper {
    pub fn get_key_name(key: Key) -> Option<String> {
        let name = match key {
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
            
            Key::KEY_SPACE => Some("space"),
            Key::KEY_ENTER => Some("enter"),
            Key::KEY_UP => Some("up"),
            Key::KEY_DOWN => Some("down"),
            Key::KEY_LEFT => Some("left"),
            Key::KEY_RIGHT => Some("right"),
            _ => None,
        };

        name.map(|s| s.to_string())
    }
}
