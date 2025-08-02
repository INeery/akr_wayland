use crate::events::WindowInfo;
use crate::error::{AhkError, Result};
use std::process::Command;
use tracing::debug;

pub struct XdotoolDetector;

impl XdotoolDetector {
    pub fn new() -> Self {
        Self
    }

    pub async fn test(&self) -> Result<()> {
        let output = Command::new("xdotool").args(&["getactivewindow", "getwindowname"]).output()?;
        if output.status.success() { 
            Ok(()) 
        } else { 
            Err(AhkError::Internal("xdotool failed".to_string())) 
        }
    }

    pub async fn get_active_window(&self) -> Result<WindowInfo> {
        debug!("Попытка получить активное окно через xdotool");
        let output = Command::new("xdotool")
            .args(&["getactivewindow", "getwindowname"])
            .output()
            .map_err(|e| {
                debug!("xdotool не найден или не работает: {}", e);
                AhkError::Internal(format!("xdotool не найден: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug!("xdotool вернул ошибку: {}", stderr);
            return Err(AhkError::Internal(format!("xdotool вернул ошибку: {}", stderr)));
        }

        let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
        debug!("xdotool получил заголовок окна: '{}'", title);

        let class_output = Command::new("xdotool")
            .args(&["getactivewindow", "getwindowclassname"])
            .output();

        let class = if let Ok(output) = class_output {
            let class_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            debug!("xdotool получил класс окна: '{}'", class_name);
            class_name
        } else {
            debug!("Не удалось получить класс окна");
            "Unknown".to_string()
        };

        Ok(WindowInfo::new(title).with_class(class))
    }
}
