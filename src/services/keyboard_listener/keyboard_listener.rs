use crate::config::Config;
use crate::debug_if_enabled;
use crate::error::{AhkError, Result};
use crate::events::{KeyCode, KeyEvent, KeyState, VirtualKeyEvent};
use crate::events::keyboard::device_ids;
use crate::services::{KeyRepeater, VirtualDevice};
use crate::utils::DeviceFinder;
use evdev::{Device, EventType};
use parking_lot::RwLock;
use std::io::Error;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};

use super::key_mapping::KeyMapper;
use super::modifier_state::ModifierState;
use super::r#trait::KeyboardListenerTrait;

pub struct RealKeyboardListener {
    config: Arc<Config>,
    key_repeater: Arc<KeyRepeater>,
    device: Device,
    virtual_device: VirtualDevice,
    modifier_state: Arc<RwLock<ModifierState>>,
    device_id: u8,  // ✅ Кэшированный ID устройства
    event_buffer: Vec<evdev::InputEvent>,  // ✅ Переиспользуемый буфер
}

impl RealKeyboardListener {
    pub fn new(config: Arc<Config>, key_repeater: Arc<KeyRepeater>) -> Result<Self> {
        info!("Инициализация RealKeyboardListener");

        let virtual_device = VirtualDevice::new("AHK-Rust KeyboardListener Virtual Device", false)?;

        let device_path = DeviceFinder::find_keyboard_device(&config.input.device_path)?;

        let mut device = Device::open(&device_path).map_err(|e| {
            AhkError::DeviceNotFound(format!(
                "Не удалось открыть устройство {:?}: {}",
                device_path, e
            ))
        })?;

        match device.grab() {
            Ok(_) => Self::log_grabbed_device(&mut device),
            Err(e) => {
                Self::log_grab_error(device_path, &e);
                return Err(AhkError::Permission(
                    format!("Не удалось захватить устройство эксклюзивно: {}. Device busy - скорее всего используется X11/Wayland", e)
                ));
            }
        }

        Ok(Self {
            config,
            key_repeater,
            device,
            virtual_device,
            modifier_state: Arc::new(RwLock::new(ModifierState::new())),
            device_id: device_ids::LISTENER_VIRTUAL_KEYBOARD,  // ✅ Кэшированный ID устройства
            event_buffer: Vec::with_capacity(64),  // ✅ Предаллоцированный буфер
        })
    }

    async fn run_impl(mut self) -> Result<()> {
        info!("RealKeyboardListener запущен, начинаем чтение событий");

        info!(
            "Настроено {} маппингов для повторения",
            self.config.mappings.len()
        );

        loop {
            // ✅ Без аллокации Vec - используем переиспользуемый буфер
            // Собираем события в отдельном scope для освобождения borrow
            let has_events = {
                match self.device.fetch_events() {
                    Ok(events) => {
                        self.event_buffer.clear();
                        self.event_buffer.extend(events);
                        !self.event_buffer.is_empty()
                    }
                    Err(e) => {
                        error!("Ошибка чтения событий: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        continue;
                    }
                }
            }; // borrow от fetch_events() освобожден здесь

            // Теперь обрабатываем события из буфера
            if has_events {
                let event_count = self.event_buffer.len();
                for i in 0..event_count {
                    let event = self.event_buffer[i];
                    if let Err(e) = self.handle_event(event).await {
                        error!("Ошибка обработки события: {}", e);
                    }
                }
            }

            // ✅ Более эффективная задержка - 50μs вместо 1ms
            tokio::time::sleep(tokio::time::Duration::from_micros(50)).await;
        }
    }

    async fn handle_event(&mut self, event: evdev::InputEvent) -> Result<()> {
        if event.event_type() == EventType::KEY {
            let key_code = event.code();
            let key_state = match event.value() {
                0 => KeyState::Released,
                1 => KeyState::Pressed,
                2 => KeyState::Repeat,
                _ => {
                    debug_if_enabled!("Неизвестное значение события: {}", event.value());
                    return Ok(());
                }
            };

            {
                let mut modifier_state = self.modifier_state.write();
                modifier_state.update_key(key_code, key_state == KeyState::Pressed);
            }

            let modifiers = self.modifier_state.read().to_modifiers();
            let key_name = KeyMapper::get_key_name(key_code);

            let key_event = KeyEvent {
                key_code: KeyCode(key_code),
                state: key_state,
                modifiers,  // ✅ Без clone - Copy type
                timestamp: std::time::Instant::now(),
                device_id: self.device_id,  // ✅ Используем кэшированный device_id
            };

            debug_if_enabled!("Событие клавиши: {}", key_event);

            // Вызываем KeyRepeater напрямую для принятия решения
            debug_if_enabled!(
                "Вызываем KeyRepeater напрямую для обработки клавиши {}",
                key_name.as_deref().unwrap_or("Unknown")
            );

            if let Err(e) = self.key_repeater.handle_key_event(&key_event).await {
                error!("Ошибка при обработке события в KeyRepeater: {}", e);
                // Если произошла ошибка в KeyRepeater, пробрасываем как обычное событие
                self.passthrough_event(&key_event).await?;
            }
        } else {
            debug_if_enabled!("Проброс не-клавиатурного события: {:?}", event);
        }

        Ok(())
    }

    async fn passthrough_event(&mut self, key_event: &KeyEvent) -> Result<()> {
        let virtual_event = match key_event.state {
            KeyState::Pressed => {
                VirtualKeyEvent::press(key_event.key_code, key_event.modifiers)  // ✅ Без clone - Copy type
            }
            KeyState::Released => {
                VirtualKeyEvent::release(key_event.key_code, key_event.modifiers)  // ✅ Без clone - Copy type
            }
            KeyState::Repeat => VirtualKeyEvent::new(
                key_event.key_code,
                KeyState::Repeat,
                key_event.modifiers,  // ✅ Без clone - Copy type
            ),
        };

        if let Err(e) = self.virtual_device.send_event(virtual_event) {
            debug_if_enabled!(
                "Не удалось пробросить событие для клавиши {}: {}",
                key_event.key_code.value(),
                e
            );
        }

        Ok(())
    }

    fn log_grabbed_device(device: &mut Device) {
        info!("Устройство: {}", device.name().unwrap_or("Unknown"));
        info!("Физический путь: {:?}", device.physical_path());
        info!("Уникальный ID: {:?}", device.unique_name());
        info!("Устройство захвачено эксклюзивно");
    }

    fn log_grab_error(device_path: PathBuf, e: &Error) {
        warn!(
            "Не удалось захватить устройство {}: {}",
            device_path.display(),
            e
        );
        warn!("Попробуйте:");
        warn!("1. Закрыть X11/Wayland сессию и запустить из консоли");
        warn!("2. Добавить пользователя в группу input: sudo usermod -a -G input $USER");
        warn!("3. Перезайти в систему после добавления в группу");
    }
}

#[async_trait::async_trait]
impl KeyboardListenerTrait for RealKeyboardListener {
    async fn run(self: Box<Self>) -> Result<()> {
        (*self).run_impl().await
    }
}

impl Drop for RealKeyboardListener {
    fn drop(&mut self) {
        info!("Освобождение захваченного устройства");
        if let Err(e) = self.device.ungrab() {
            error!("Не удалось освободить устройство: {}", e);
        }
    }
}
