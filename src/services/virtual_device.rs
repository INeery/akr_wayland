use crate::debug_if_enabled;
use crate::error::{AhkError, Result};
use crate::events::{KeyState, VirtualKeyEvent};
use parking_lot::Mutex;
use tracing::info;

pub struct VirtualDevice {
    device: Option<Mutex<uinput::Device>>, // Используем sync mutex
    dry_run: bool,
}

impl VirtualDevice {
    pub fn new(device_name: &str, dry_run: bool) -> Result<Self> {
        info!(
            "Инициализация VirtualDevice '{}' (dry_run: {})",
            device_name, dry_run
        );

        let device = if dry_run {
            None
        } else {
            Some(Mutex::new(Self::create_virtual_device(device_name)?)) // ✅ Оборачиваем в Mutex
        };

        Ok(Self {
            device,
            dry_run,
        })
    }

    fn create_virtual_device(device_name: &str) -> Result<uinput::Device> {
        info!(
            "Создание виртуального устройства uinput '{}' для инъекции клавиш",
            device_name
        );

        let virtual_device = uinput::default()?
            .name(device_name)
            .unwrap()
            .event(uinput::event::Keyboard::All)
            .unwrap()
            .create()
            .map_err(|e| {
                AhkError::Internal(format!(
                    "Не удалось создать виртуальное устройство '{}': {}",
                    device_name, e
                ))
            })?;

        info!("Виртуальное устройство '{}' создано успешно", device_name);
        Ok(virtual_device)
    }

    pub fn send_event(&self, event: VirtualKeyEvent) -> Result<()> {
        if self.dry_run {
            info!("[DRY RUN] Виртуальное событие: {:?}", event);
            return Ok(());
        }

        debug_if_enabled!("Обработка виртуального события: {:?}", event);

        if let Some(device_mutex) = &self.device {
            let keycode = event.key_code.value() as i32;
            let value = match event.state {
                KeyState::Pressed => 1,
                KeyState::Released => 0,
                KeyState::Repeat => 2,
            };

            // Блокируем mutex для доступа к устройству
            let mut device = device_mutex.lock();

            // Отправляем событие клавиши
            if let Err(e) = device.write(1, keycode, value) {
                return Err(AhkError::Internal(format!(
                    "Не удалось отправить событие клавиши {}: {}",
                    keycode, e
                )));
            }

            // Синхронизируем события
            if let Err(e) = device.write(0, 0, 0) {
                return Err(AhkError::Internal(format!(
                    "Не удалось синхронизировать события: {}",
                    e
                )));
            }

            debug_if_enabled!("Виртуальное событие {} отправлено", event.key_code);
        } else {
            return Err(AhkError::Internal(
                "Виртуальное устройство недоступно".to_string(),
            ));
        }

        Ok(())
    }
}

impl Drop for VirtualDevice {
    fn drop(&mut self) {
        if !self.dry_run {
            info!("Закрытие виртуального устройства");
        }
    }
}
