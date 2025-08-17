use serde::{Deserialize, Serialize};
use std::fmt;

/// Информация об окне
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowInfo {
    pub title: String,
    pub class: String,
    pub pid: Option<u32>,
    pub geometry: Option<WindowGeometry>,
}

impl WindowInfo {
    pub fn new(title: String) -> Self {
        Self {
            title,
            class: String::new(),
            pid: None,
            geometry: None,
        }
    }

    pub fn with_class(mut self, class: String) -> Self {
        self.class = class;
        self
    }

    #[allow(dead_code)]
    pub fn with_pid(mut self, pid: u32) -> Self {
        self.pid = Some(pid);
        self
    }

    /// Проверить, соответствует ли окно паттерну (регистронезависимо)
    #[allow(dead_code)]
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        if pattern.is_empty() {
            return true;
        }
        let pattern_lower = pattern.to_lowercase();
        let title_lower = self.title.to_lowercase();
        let class_lower = self.class.to_lowercase();
        title_lower.contains(&pattern_lower) || class_lower.contains(&pattern_lower)
    }

    /// Проверить, соответствует ли окно любому из паттернов
    #[allow(dead_code)]
    pub fn matches_any_pattern(&self, patterns: &[String]) -> bool {
        if patterns.is_empty() {
            return true; // Пустой список паттернов означает "любое окно"
        }

        patterns.iter().any(|pattern| self.matches_pattern(pattern))
    }
}

impl fmt::Display for WindowInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.class.is_empty() {
            write!(f, "\"{}\"", self.title)
        } else {
            write!(f, "\"{}\" ({})", self.title, self.class)
        }
    }
}

/// Геометрия окна
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}


/// Событие смены активного окна
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowEvent {
    pub window: WindowInfo,
    pub timestamp: std::time::Instant,
    pub event_type: WindowEventType,
}

impl WindowEvent {
    pub fn new(window: WindowInfo, event_type: WindowEventType) -> Self {
        Self {
            window,
            timestamp: std::time::Instant::now(),
            event_type,
        }
    }

    #[allow(dead_code)]
    pub fn focus_changed(window: WindowInfo) -> Self {
        Self::new(window, WindowEventType::FocusChanged)
    }
}

impl fmt::Display for WindowEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}: {} ({}ms ago)",
            self.event_type,
            self.window,
            self.timestamp.elapsed().as_millis()
        )
    }
}

/// Тип события окна
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WindowEventType {
    FocusChanged,
    Created,
    Destroyed,
    TitleChanged,
    GeometryChanged,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_info_creation() {
        let window = WindowInfo::new("Test Window".to_string())
            .with_class("TestApp".to_string())
            .with_pid(1234);

        assert_eq!(window.title, "Test Window");
        assert_eq!(window.class, "TestApp");
        assert_eq!(window.pid, Some(1234));
    }

    #[test]
    fn test_window_pattern_matching() {
        let window = WindowInfo::new("Vim - file.txt".to_string())
            .with_class("vim".to_string());

        assert!(window.matches_pattern("Vim"));
        assert!(window.matches_pattern("vim"));
        assert!(window.matches_pattern("file.txt"));
        assert!(!window.matches_pattern("emacs"));

        let patterns = vec!["vim".to_string(), "emacs".to_string()];
        assert!(window.matches_any_pattern(&patterns));

        let no_patterns: Vec<String> = vec![];
        assert!(window.matches_any_pattern(&no_patterns));
    }

    #[test]
    fn test_window_event_creation() {
        let window = WindowInfo::new("Test".to_string());
        let event = WindowEvent::focus_changed(window.clone());

        assert_eq!(event.window, window);
        assert_eq!(event.event_type, WindowEventType::FocusChanged);
    }
}
