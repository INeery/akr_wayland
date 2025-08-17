use anyhow::Context;
use crate::error::Result;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub logging: LoggingConfig,
    pub input: InputConfig,
    pub repeat: RepeatConfig,
    pub window: WindowConfig,
    #[serde(default)]
    pub mappings: Vec<KeyMapping>,
    // Оптимизационные индексы - не сериализуются, строятся после загрузки
    #[serde(skip)]
    key_set: HashSet<String>, // O(1) lookup для ключей
    #[serde(skip)]
    pattern_set_lower: HashSet<String>, // Предварительно нормализованные паттерны
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub filter: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InputConfig {
    pub device_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepeatConfig {
    pub repeat_delay_ms: u64,
    #[serde(default, rename = "repeat_toggle_key")]
    pub repeat_toggle_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowConfig {
    pub detection_mode: String,
    pub polling_interval_ms: u64,
    pub window_title_patterns: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyMapping {
    pub key: String,
    pub modifiers: Vec<String>,
}

impl Config {
    pub fn mappings(&self) -> &Vec<KeyMapping> {
        &self.mappings
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut config = Self {
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
                filter: "ahk_rust=info".to_string(),
            },
            input: InputConfig {
                device_path: "auto".to_string(),
            },
            repeat: RepeatConfig {
                repeat_delay_ms: 50,
                repeat_toggle_key: None,
            },
            window: WindowConfig {
                detection_mode: "dbus".to_string(),
                polling_interval_ms: 1000,
                window_title_patterns: Vec::new(),
            },
            mappings: Vec::new(),
            key_set: HashSet::new(),
            pattern_set_lower: HashSet::new(),
        };
        config.build_optimization_indexes();
        config
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config_path = config_path.as_ref();

        let figment = Figment::new()
            .merge(Toml::file(config_path))
            .merge(Env::prefixed("AHK_"));

        let mut config: Config = figment
            .extract()
            .with_context(|| format!("Не удалось загрузить конфигурацию из {:?}", config_path))?;

        config.validate()?;
        config.build_optimization_indexes();

        Ok(config)
    }

    /// Строит оптимизационные индексы для быстрого поиска
    pub fn build_optimization_indexes(&mut self) {
        // Строим HashSet для O(1) поиска ключей
        self.key_set = self
            .mappings
            .iter()
            .map(|mapping| mapping.key.clone())
            .collect();

        // Предварительно нормализуем паттерны окон
        self.pattern_set_lower = self
            .window
            .window_title_patterns
            .iter()
            .map(|pattern| pattern.to_lowercase())
            .collect();
    }

    pub fn validate(&self) -> Result<()> {
        // Валидация настроек логирования
        match self.logging.level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => return Err(anyhow::anyhow!("Неверный уровень логирования: {}", self.logging.level).into()),
        }

        match self.logging.format.as_str() {
            "pretty" | "json" => {}
            _ => return Err(anyhow::anyhow!("Неверный формат логирования: {}", self.logging.format).into()),
        }

        // Валидация настроек повторения
        if self.repeat.repeat_delay_ms == 0 {
            return Err(anyhow::anyhow!("repeat_delay_ms должно быть больше 0").into());
        }

        // Валидация настроек окон
        match self.window.detection_mode.as_str() {
            "dbus" | "polling" => {}
            _ => return Err(anyhow::anyhow!(
                "Неверный режим детекции окон: {}",
                self.window.detection_mode
            ).into()),
        }

        if self.window.polling_interval_ms < 100 {
            return Err(anyhow::anyhow!("polling_interval_ms должно быть минимум 100").into());
        }

        // Валидация маппингов
        for (i, mapping) in self.mappings().iter().enumerate() {
            if mapping.key.is_empty() {
                return Err(anyhow::anyhow!("Пустая клавиша в маппинге #{}", i + 1).into());
            }

            for modifier in &mapping.modifiers {
                match modifier.as_str() {
                    "ctrl" | "alt" | "shift" | "super" => {}
                    _ => return Err(anyhow::anyhow!("Неверный модификатор '{}' в маппинге #{}", modifier, i + 1).into()),
                }
            }
        }

        Ok(())
    }

    /// ЕДИНСТВЕННЫЙ метод проверки повторения - объединяет логику модификаторов с оптимизацией производительности
    pub fn should_repeat_key(&self, key: &str, modifiers: &[String], window_title: &str) -> bool {
        // Быстрая проверка: если клавиши нет в маппингах, сразу возвращаем false (O(1))
        if !self.key_set.contains(key) {
            return false;
        }

        // Оптимизация для случаев без модификаторов - используем предварительно нормализованные паттерны
        if modifiers.is_empty() {
            // Если паттерны пустые, повторяем для всех окон
            if self.pattern_set_lower.is_empty() {
                return true;
            }

            // Минимизируем аллокации: понижаем регистр только если есть верхний
            use std::borrow::Cow;
            let window_title_lower: Cow<str> = if window_title.chars().any(|c| c.is_uppercase()) {
                Cow::Owned(window_title.to_lowercase())
            } else {
                Cow::Borrowed(window_title)
            };
            return self
                .pattern_set_lower
                .iter()
                .any(|pattern| window_title_lower.contains(pattern));
        }

        // Для случаев с модификаторами используем полную логику (O(n) но неизбежно)
        for mapping in self.mappings() {
            if mapping.key == key {
                // Логика: клавиша работает БЕЗ модификаторов + с любыми указанными модификаторами
                // mapping.modifiers = ["ctrl", "alt"] означает работает:
                // - просто клавиша
                // - ctrl + клавиша
                // - alt + клавиша
                // - ctrl + alt + клавиша
                // Но НЕ работает shift + клавиша (shift не в списке)

                let modifiers_match = if mapping.modifiers.is_empty() {
                    // Если модификаторы не указаны, работает только без модификаторов
                    modifiers.is_empty()
                } else {
                    // Если модификаторы указаны, работает ВСЕГДА без модификаторов + с любой комбинацией указанных
                    // 1. Без модификаторов - всегда работает
                    // 2. С модификаторами - только если все нажатые модификаторы есть в разрешённом списке
                    modifiers.is_empty() || modifiers.iter().all(|m| mapping.modifiers.contains(m))
                };

                if modifiers_match {
                    // Если паттерны окон пустые, работаем для всех окон
                    if self.window.window_title_patterns.is_empty() {
                        return true;
                    }

                    // Используем предварительно нормализованные паттерны и понижаем регистр только при необходимости
                    use std::borrow::Cow;
                    let window_title_lower: Cow<str> = if window_title.chars().any(|c| c.is_uppercase()) {
                        Cow::Owned(window_title.to_lowercase())
                    } else {
                        Cow::Borrowed(window_title)
                    };
                    return self
                        .pattern_set_lower
                        .iter()
                        .any(|pattern| window_title_lower.contains(pattern));
                }
            }
        }

        false
    }

    /// Получить все клавиши из маппингов
    #[allow(dead_code)]
    pub fn get_all_keys(&self) -> HashSet<String> {
        let mut keys = HashSet::new();

        for mapping in &self.mappings {
            keys.insert(mapping.key.clone());
            for modifier in &mapping.modifiers {
                keys.insert(modifier.clone());
            }
        }

        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_default_config_validation() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_should_repeat_key() {
        let mut config = Config::default();
        config.mappings = vec![
            KeyMapping { key: "j".to_string(), modifiers: vec![] },
            KeyMapping { key: "space".to_string(), modifiers: vec!["ctrl".to_string()] },
        ];
        config.window.window_title_patterns = vec!["nvim".to_string()];
        config.build_optimization_indexes();
        assert!(config.should_repeat_key("j", &[], "nvim - file.txt"));
        assert!(!config.should_repeat_key("j", &[], "browser"));
    }

    #[test]
    fn test_mapping_logic_examples_and_anti_examples() {
        // 1) mappings = [{ key = "1", modifiers = [] }] -> only bare key
        let mut config1 = Config::default();
        config1.mappings = vec![ KeyMapping { key: "1".into(), modifiers: vec![] } ];
        config1.window.window_title_patterns = vec![]; // any window
        config1.build_optimization_indexes();
        assert!(config1.should_repeat_key("1", &[], "any"));
        assert!(!config1.should_repeat_key("1", &["ctrl".into()], "any"));
        assert!(!config1.should_repeat_key("1", &["shift".into()], "any"));

        // 2) mappings = [{ key = "1", modifiers = ["ctrl"] }]
        let mut config2 = Config::default();
        config2.mappings = vec![ KeyMapping { key: "1".into(), modifiers: vec!["ctrl".into()] } ];
        config2.window.window_title_patterns = vec![];
        config2.build_optimization_indexes();
        assert!(config2.should_repeat_key("1", &[], "any"));
        assert!(config2.should_repeat_key("1", &["ctrl".into()], "any"));
        assert!(!config2.should_repeat_key("1", &["ctrl".into(), "alt".into()], "any"));
        assert!(!config2.should_repeat_key("1", &["shift".into()], "any"));

        // 3) mappings = [{ key = "space", modifiers = ["ctrl","alt"] }]
        let mut config3 = Config::default();
        config3.mappings = vec![ KeyMapping { key: "space".into(), modifiers: vec!["ctrl".into(), "alt".into()] } ];
        config3.window.window_title_patterns = vec![];
        config3.build_optimization_indexes();
        assert!(config3.should_repeat_key("space", &[], "any"));
        assert!(config3.should_repeat_key("space", &["ctrl".into()], "any"));
        assert!(config3.should_repeat_key("space", &["alt".into()], "any"));
        assert!(config3.should_repeat_key("space", &["ctrl".into(), "alt".into()], "any"));
        assert!(!config3.should_repeat_key("space", &["shift".into()], "any"));

        // Window patterns behavior
        let mut config4 = Config::default();
        config4.mappings = vec![ KeyMapping { key: "j".into(), modifiers: vec![] } ];
        config4.window.window_title_patterns = vec!["nvim".into()];
        config4.build_optimization_indexes();
        assert!(config4.should_repeat_key("j", &[], "NVIM — file.txt"));
        assert!(!config4.should_repeat_key("j", &[], "browser"));

        // Key not in mappings -> never repeat
        let mut config5 = Config::default();
        config5.mappings = vec![];
        config5.window.window_title_patterns.clear();
        config5.build_optimization_indexes();
        assert!(!config5.should_repeat_key("k", &[], "any"));

        // Наличие key достаточно для голой клавиши даже если есть modifiers в маппинге
        let mut config6 = Config::default();
        config6.mappings = vec![ KeyMapping { key: "space".into(), modifiers: vec!["ctrl".into()] } ];
        config6.window.window_title_patterns.clear();
        config6.build_optimization_indexes();
        assert!(config6.should_repeat_key("space", &[], "any"));
    }

    #[test]
    fn test_get_all_keys() {
        let mut config = Config::default();
        config.mappings = vec![
            KeyMapping {
                key: "j".to_string(),
                modifiers: vec!["ctrl".to_string()],
            },
            KeyMapping {
                key: "k".to_string(),
                modifiers: vec!["shift".to_string()],
            },
        ];
        config.window.window_title_patterns = vec!["nvim".to_string()];

        let keys = config.get_all_keys();
        let expected: HashSet<String> = ["j", "k", "ctrl", "shift"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(keys, expected);
    }
}
