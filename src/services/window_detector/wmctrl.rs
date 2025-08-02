use crate::events::WindowInfo;
use crate::error::{AhkError, Result};
use std::process::Command;

pub struct WmctrlDetector;

impl WmctrlDetector {
    pub fn new() -> Self {
        Self
    }

    pub async fn test(&self) -> Result<()> {
        let output = Command::new("wmctrl").args(&["-l"]).output()?;
        if output.status.success() { 
            Ok(()) 
        } else { 
            Err(AhkError::Internal("wmctrl failed".to_string())) 
        }
    }

    pub async fn get_active_window(&self) -> Result<WindowInfo> {
        let output = Command::new("wmctrl")
            .args(&["-l"])
            .output()
            .map_err(|e| AhkError::Internal(format!("wmctrl не найден: {}", e)))?;

        if !output.status.success() {
            return Err(AhkError::Internal("wmctrl вернул ошибку".to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.contains("*") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 3 {
                    let title = parts[3..].join(" ");
                    return Ok(WindowInfo::new(title));
                }
            }
        }

        Err(AhkError::Internal("Активное окно не найдено".to_string()))
    }
}
