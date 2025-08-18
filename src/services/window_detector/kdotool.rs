use crate::events::WindowInfo;
use crate::error::{AhkError, Result};
use std::process::Command;
use tracing::debug;
use std::collections::HashMap;

pub struct KdotoolDetector;

fn build_env_overrides() -> HashMap<String, String> {
    let mut env_vars = HashMap::new();

    if std::env::var("USER").unwrap_or_default() == "root" {
        if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            if let Ok(output) = Command::new("id").args(&["-u", &sudo_user]).output() {
                if let Ok(uid_str) = String::from_utf8(output.stdout) {
                    let uid = uid_str.trim();
                    let user_runtime_dir = format!("/run/user/{}", uid);
                    let dbus_address = format!("unix:path={}/bus", user_runtime_dir);

                    debug!("Подставляем переменные окружения для пользователя {}: uid={}", sudo_user, uid);
                    env_vars.insert("DBUS_SESSION_BUS_ADDRESS".to_string(), dbus_address);
                    env_vars.insert("XDG_RUNTIME_DIR".to_string(), user_runtime_dir);
                    env_vars.insert("USER".to_string(), sudo_user);
                }
            }
        }
    }

    if let Ok(display_var) = std::env::var("DISPLAY") {
        env_vars.insert("DISPLAY".to_string(), display_var);
    }

    env_vars
}

impl KdotoolDetector {
    pub fn new() -> Self {
        Self
    }

    fn create_command(args: &[&str]) -> Command {
        let mut cmd = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
            let mut cmd = Command::new("sudo");
            cmd.args(&["-E", "-u", &sudo_user, "kdotool"]);
            cmd.args(args);
            cmd
        } else {
            let mut cmd = Command::new("kdotool");
            cmd.args(args);
            cmd
        };

        // Применяем подстановки переменных окружения (строим на лету без глобального кэша)
        for (key, value) in build_env_overrides() {
            cmd.env(key, value);
        }

        cmd
    }

    pub async fn test(&self) -> Result<()> {
        debug!("=== Тестируем kdotool ===");

        let id_output = Self::create_command(&["getactivewindow"]).output()?;
        if !id_output.status.success() { 
            debug!("kdotool getactivewindow failed: {}", String::from_utf8_lossy(&id_output.stderr));
            return Err(AhkError::Internal("kdotool getactivewindow failed".to_string())); 
        }

        let window_id = String::from_utf8_lossy(&id_output.stdout).trim().to_string();
        debug!("kdotool получил window_id: '{}'", window_id);

        let name_output = Self::create_command(&["getwindowname", &window_id]).output()?;
        if !name_output.status.success() { 
            debug!("kdotool getwindowname failed: {}", String::from_utf8_lossy(&name_output.stderr));
            return Err(AhkError::Internal("kdotool getwindowname failed".to_string())); 
        }

        debug!("=== kdotool работает ===");
        Ok(())
    }

    pub async fn get_active_window(&self) -> Result<WindowInfo> {
        // Получаем ID окна
        let id_output = Self::create_command(&["getactivewindow"]).output()?;
        if !id_output.status.success() { 
            return Err(AhkError::Internal("kdotool getactivewindow failed".to_string()));
        }

        let window_id = String::from_utf8_lossy(&id_output.stdout).trim().to_string();

        // Получаем название окна по ID
        let name_output = Self::create_command(&["getwindowname", &window_id]).output()?;
        if !name_output.status.success() {
            return Err(AhkError::Internal("kdotool getwindowname failed".to_string()));
        }

        let title = String::from_utf8_lossy(&name_output.stdout).trim().to_string();
        if title.is_empty() {
            return Err(AhkError::Internal("kdotool вернул пустое название".to_string()));
        }

        Ok(WindowInfo::new(title).with_class("KDE".to_string()))
    }

}
