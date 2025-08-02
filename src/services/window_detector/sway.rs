use crate::events::WindowInfo;
use crate::error::{AhkError, Result};
use std::process::Command;

pub struct SwayDetector;

impl SwayDetector {
    pub fn new() -> Self {
        Self
    }

    pub async fn test(&self) -> Result<()> {
        let output = Command::new("swaymsg").args(&["-t", "get_tree"]).output()?;
        if output.status.success() { 
            Ok(()) 
        } else { 
            Err(AhkError::Internal("sway failed".to_string())) 
        }
    }

    pub async fn get_active_window(&self) -> Result<WindowInfo> {
        let output = Command::new("swaymsg")
            .args(&["-t", "get_tree"])
            .output()
            .map_err(|e| AhkError::Internal(format!("swaymsg не найден: {}", e)))?;

        if !output.status.success() {
            return Err(AhkError::Internal("swaymsg вернул ошибку".to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        if let Some(start) = stdout.find("\"focused\":true") {
            let before = &stdout[..start];
            if let Some(name_start) = before.rfind("\"name\":\"") {
                let name_part = &before[name_start + 8..];
                if let Some(name_end) = name_part.find('"') {
                    let title = name_part[..name_end].to_string();
                    return Ok(WindowInfo::new(title));
                }
            }
        }

        Err(AhkError::Internal("Активное окно в Sway не найдено".to_string()))
    }
}
