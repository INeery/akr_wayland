use crate::error::{AhkError, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tracing::{info, warn};

/// Проверить права доступа к необходимым ресурсам
pub fn check_permissions() -> Result<()> {
    info!("Проверка прав доступа...");

    // Проверка доступа к /dev/input/
    check_input_devices_access()?;

    // Проверка доступа к /dev/uinput
    check_uinput_access()?;

    // Проверка, что не запущен от root (рекомендация безопасности)
    check_not_root();

    info!("Проверка прав доступа завершена успешно");
    Ok(())
}

fn check_input_devices_access() -> Result<()> {
    let input_dir = "/dev/input";

    if !std::path::Path::new(input_dir).exists() {
        return Err(AhkError::Permission(
            format!("Директория {} не существует", input_dir)
        ));
    }

    // Проверяем возможность чтения директории
    match fs::read_dir(input_dir) {
        Ok(_) => {
            info!("Доступ к {} подтвержден", input_dir);
            Ok(())
        }
        Err(e) => {
            Err(AhkError::Permission(
                format!("Нет доступа к {}: {}. Добавьте пользователя в группу 'input'", input_dir, e)
            ))
        }
    }
}

fn check_uinput_access() -> Result<()> {
    let uinput_device = "/dev/uinput";

    if !std::path::Path::new(uinput_device).exists() {
        warn!("{} не существует, возможно модуль uinput не загружен", uinput_device);
        return Ok(()); // Не критичная ошибка, модуль может быть загружен позже
    }

    match fs::metadata(uinput_device) {
        Ok(metadata) => {
            let permissions = metadata.permissions();
            let mode = permissions.mode();

            // Проверяем права доступа (обычно 660 или 666)
            if mode & 0o006 == 0 && mode & 0o060 == 0 {
                return Err(AhkError::Permission(
                    format!("Нет прав доступа к {}. Добавьте пользователя в группу 'uinput' или 'input'", uinput_device)
                ));
            }

            info!("Доступ к {} подтвержден", uinput_device);
            Ok(())
        }
        Err(e) => {
            Err(AhkError::Permission(
                format!("Не удалось проверить права доступа к {}: {}", uinput_device, e)
            ))
        }
    }
}

fn check_not_root() {
    // Проверяем переменную окружения USER
    match std::env::var("USER") {
        Ok(user) if user == "root" => {
            warn!("⚠️  Приложение запущено от имени root!");
            warn!("   Рекомендуется добавить пользователя в группы 'input' и 'uinput'");
            warn!("   и запускать приложение от имени обычного пользователя");
            warn!("   Команды:");
            warn!("   sudo usermod -a -G input,uinput $USER");
            warn!("   sudo modprobe uinput");
            warn!("   (затем перезайдите в систему)");
        }
        Ok(user) => {
            info!("Приложение запущено от имени пользователя: {}", user);
        }
        Err(_) => {
            warn!("Не удалось определить пользователя");
        }
    }
}

/// Получить рекомендуемые команды для настройки прав доступа
#[allow(dead_code)]
pub fn get_setup_commands() -> Vec<String> {
    vec![
        "# Добавить пользователя в необходимые группы:".to_string(),
        "sudo usermod -a -G input,uinput $USER".to_string(),
        "".to_string(),
        "# Загрузить модуль uinput:".to_string(),
        "sudo modprobe uinput".to_string(),
        "".to_string(),
        "# Автоматическая загрузка модуля при загрузке системы:".to_string(),
        "echo 'uinput' | sudo tee /etc/modules-load.d/uinput.conf".to_string(),
        "".to_string(),
        "# После выполнения команд перезайдите в систему".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_commands() {
        let commands = get_setup_commands();
        assert!(!commands.is_empty());
        assert!(commands.iter().any(|cmd| cmd.contains("usermod")));
        assert!(commands.iter().any(|cmd| cmd.contains("modprobe")));
    }
}
