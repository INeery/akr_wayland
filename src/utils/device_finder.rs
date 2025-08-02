use crate::error::{AhkError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

pub struct DeviceFinder;

impl DeviceFinder {
    /// Найти подходящее клавиатурное устройство
    pub fn find_keyboard_device(device_path: &str) -> Result<PathBuf> {
        if device_path != "auto" {
            let path = PathBuf::from(device_path);
            return if path.exists() {
                info!("Используется указанное устройство: {:?}", path);
                Ok(path)
            } else {
                AhkError::device_not_found(
                    format!("Указанное устройство не найдено: {:?}", path)
                )
            }
        }

        // Автопоиск клавиатурного устройства
        Self::auto_find_keyboard()
    }

    fn auto_find_keyboard() -> Result<PathBuf> {
        info!("Начинаем автопоиск клавиатурного устройства...");

        // Попробуем найти устройство по ID
        if let Ok(device) = Self::find_by_id() {
            info!("Найдено устройство по ID: {:?}", device);
            return Ok(device);
        }

        // Попробуем найти устройство в /dev/input/event*
        if let Ok(device) = Self::find_by_event_devices() {
            info!("Найдено устройство среди event устройств: {:?}", device);
            return Ok(device);
        }

        AhkError::device_not_found(
            "Не удалось найти подходящее клавиатурное устройство. \
             Убедитесь, что пользователь добавлен в группу 'input'"
        )
    }

    fn find_by_id() -> Result<PathBuf> {
        let by_id_dir = Path::new("/dev/input/by-id");

        if !by_id_dir.exists() {
            debug!("Директория /dev/input/by-id не существует");
            return AhkError::device_not_found("Директория by-id не найдена");
        }

        let entries = fs::read_dir(by_id_dir)
            .map_err(|e| AhkError::Permission(
                format!("Нет доступа к /dev/input/by-id: {}", e)
            ))?;

        let mut potential_keyboards = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| AhkError::Io(e))?;
            let path = entry.path();
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Сначала ищем все устройства с kbd или keyboard в названии
            if (name.contains("kbd") || name.contains("keyboard")) && name.contains("event") {
                debug!("Найдено потенциальное клавиатурное устройство: {:?}", path);

                if Self::is_device_accessible(&path) {
                    potential_keyboards.push((path.clone(), name.to_string()));
                } else {
                    warn!("Устройство {:?} недоступно", path);
                }
            }
        }

        // Теперь фильтруем и приоритизируем
        let mut filtered_keyboards = Vec::new();

        for (path, name) in potential_keyboards {
            // Исключаем известные модели мышей
            if name.contains("DeathAdder") || 
               name.contains("mouse") || 
               name.contains("Mouse") {
                debug!("Исключаем как мышь: {} -> {}", name, path.display());
                continue;
            }

            // Проверяем, что это действительно клавиатура через evdev
            if Self::is_keyboard_device(&path)? {
                let priority = if name.ends_with("event-kbd") {
                    100 // Высший приоритет для -event-kbd устройств
                } else if name.contains("Keyboard") || name.contains("keyboard") {
                    50  // Высокий приоритет для устройств с "keyboard" в названии
                } else {
                    10  // Обычный приоритет
                };

                filtered_keyboards.push((path, priority));
                info!("Добавлена клавиатура: {} (приоритет: {})", name, priority);
            } else {
                debug!("Устройство не прошло проверку как клавиатура: {}", name);
            }
        }

        // Сортируем по приоритету и возвращаем лучшее
        filtered_keyboards.sort_by(|a, b| b.1.cmp(&a.1));

        if let Some((keyboard, _)) = filtered_keyboards.into_iter().next() {
            Ok(keyboard)
        } else {
            AhkError::device_not_found("Клавиатурное устройство не найдено в by-id")
        }
    }

    fn find_by_event_devices() -> Result<PathBuf> {
        let input_dir = Path::new("/dev/input");

        let entries = fs::read_dir(input_dir)
            .map_err(|e| AhkError::Permission(
                format!("Нет доступа к /dev/input: {}", e)
            ))?;

        let mut event_devices = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| AhkError::Io(e))?;
            let path = entry.path();
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if name.starts_with("event") {
                event_devices.push(path);
            }
        }

        // Сортируем устройства по номеру
        event_devices.sort();

        // Проверяем каждое устройство на предмет того, является ли оно клавиатурой
        for device_path in event_devices {
            debug!("Проверяем устройство: {:?}", device_path);

            if Self::is_keyboard_device(&device_path)? && Self::is_device_accessible(&device_path) {
                return Ok(device_path);
            }
        }

        AhkError::device_not_found("Не найдено доступное клавиатурное устройство среди event устройств")
    }

    fn is_keyboard_device(device_path: &Path) -> Result<bool> {
        // Используем evdev для проверки возможностей устройства
        match evdev::Device::open(device_path) {
            Ok(device) => {
                let device_name = device.name().unwrap_or("Unknown").to_lowercase();

                // Исключаем мыши по имени устройства
                if device_name.contains("mouse") || 
                   device_name.contains("deathadder") || 
                   device_name.contains("touchpad") ||
                   device_name.contains("trackpoint") {
                    debug!("Исключаем устройство как мышь/тачпад: {:?} ({})", device_path, device_name);
                    return Ok(false);
                }

                // Проверяем, поддерживает ли устройство клавиатурные события
                let has_keys = device.supported_keys().map_or(false, |keys| {
                    // Проверяем наличие основных клавиш для клавиатуры
                    let basic_keys = keys.contains(evdev::KeyCode::KEY_A) &&
                                   keys.contains(evdev::KeyCode::KEY_SPACE) &&
                                   keys.contains(evdev::KeyCode::KEY_ENTER);

                    // Проверяем наличие достаточного количества клавиш (у клавиатуры их много)
                    let key_count = keys.iter().count();

                    basic_keys && key_count > 20 // У настоящей клавиатуры много клавиш
                });

                if has_keys {
                    info!("Устройство {:?} подходит как клавиатура", device_path);
                    debug!("Имя устройства: {:?}", device.name());
                } else {
                    debug!("Устройство {:?} не подходит как клавиатура (имя: {})", device_path, device_name);
                }

                Ok(has_keys)
            }
            Err(e) => {
                debug!("Не удалось открыть устройство {:?}: {}", device_path, e);
                Ok(false)
            }
        }
    }

    fn is_device_accessible(device_path: &Path) -> bool {
        match fs::File::open(device_path) {
            Ok(_) => true,
            Err(e) => {
                debug!("Устройство {:?} недоступно: {}", device_path, e);
                false
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_finder_creation() {
        // Просто проверяем, что структура создается без ошибок
        let _finder = DeviceFinder;
    }

    #[test]
    fn test_find_keyboard_device_with_specific_path() {
        // Тест с несуществующим путем
        let result = DeviceFinder::find_keyboard_device("/non/existent/path");
        assert!(result.is_err());
    }
}
