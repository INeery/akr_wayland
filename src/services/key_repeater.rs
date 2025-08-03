use crate::config::Config;
use crate::debug_if_enabled;
use crate::error::Result;
use crate::events::{KeyEvent, WindowEvent, VirtualKeyEvent, KeyState, KeyCode, Modifiers, WindowInfo};
use crate::services::{keycode_map::KeycodeMap, VirtualDevice};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::{info, error};
use parking_lot::RwLock;
use dashmap::DashMap;

pub struct KeyRepeater {
    config: Arc<Config>,
    virtual_device: Arc<VirtualDevice>,  // ✅ Используем Arc для безопасного разделения
    dry_run: bool,
    // Состояние компонента
    active_window: Arc<RwLock<Option<WindowInfo>>>,
    active_repeaters: Arc<DashMap<u64, RepeaterTask>>,
}

/// Задача повторения для конкретной комбинации клавиш
#[derive(Debug)]
struct RepeaterTask {
    handle: JoinHandle<()>,
    key_code: KeyCode,
    modifiers: Modifiers,
}

impl KeyRepeater {
    pub fn new(
        config: Arc<Config>,
        dry_run: bool,
    ) -> Result<Self> {
        info!("Инициализация KeyRepeater (dry_run: {})", dry_run);

        let virtual_device = Arc::new(VirtualDevice::new("AHK-Rust KeyRepeater Virtual Device", dry_run)?);  // ✅ Оборачиваем в Arc

        Ok(Self {
            config,
            virtual_device,
            dry_run,
            active_window: Arc::new(RwLock::new(None)),
            active_repeaters: Arc::new(DashMap::new()),
        })
    }


    /// Обработка события клавиши
    pub async fn handle_key_event(&self, event: &KeyEvent) -> Result<()> {
        debug_if_enabled!("Обработка события клавиши: {}", event);

        // Проверяем, нужно ли повторять эту клавишу
        let should_repeat = if let Some(key_name) = KeycodeMap::get_key_name(event.key_code.value()) {
            let current_window = self.active_window.read();
            let window_title = current_window
                .as_ref()
                .map(|w| w.title.as_str())
                .unwrap_or("");

            debug_if_enabled!("KeyRepeater проверяет повторение для клавиши '{}' с заголовком окна: '{}'", key_name, window_title);
            debug_if_enabled!("Паттерны окон в конфигурации: {:?}", self.config.window.window_title_patterns);
            debug_if_enabled!("Модификаторы события: {:?}", event.modifiers.to_vec());

            // ✅ Используем оптимизированный метод без аллокаций
            let result = self.config.should_repeat_key_optimized(key_name, window_title);
            debug_if_enabled!("Результат проверки повторения для '{}': {}", key_name, result);

            result
        } else {
            false
        };

        if should_repeat {
            // Обрабатываем как повторяемую клавишу
            match event.state {
                KeyState::Pressed => {
                    self.handle_key_press(event).await?
                }
                KeyState::Released => {
                    self.handle_key_release(event).await?
                }
                KeyState::Repeat => {
                    // Игнорируем аппаратные повторы - мы делаем свои
                    debug_if_enabled!("Игнорируем аппаратный повтор для {}", event.key_code);
                }
            }
        } else {
            // Пробрасываем обычное событие
            debug_if_enabled!("Клавиша {} не нуждается в повторении - пробрасываем", 
                   KeycodeMap::get_key_name(event.key_code.value()).unwrap_or("Unknown"));

            let virtual_event = match event.state {
                KeyState::Pressed => VirtualKeyEvent::press(event.key_code, event.modifiers),
                KeyState::Released => VirtualKeyEvent::release(event.key_code, event.modifiers),
                KeyState::Repeat => VirtualKeyEvent::new(event.key_code, KeyState::Repeat, event.modifiers),
            };


            if let Err(e) = self.virtual_device.send_event(virtual_event) {
                error!("Не удалось пробросить обычное событие: {}", e);
            }
        }

        Ok(())
    }

    /// Обработка нажатия клавиши
    async fn handle_key_press(&self, event: &KeyEvent) -> Result<()> {
        let combination_id = event.combination_id();
        let combination_hash = event.combination_hash();
        
        info!("Получено нажатие клавиши для повторения: {}", combination_id);
        
        // СНАЧАЛА отправляем оригинальное событие нажатия
        let original_press_event = VirtualKeyEvent::press(event.key_code, event.modifiers);  // ✅ Без clone - Copy type
        if let Err(e) = self.virtual_device.send_event(original_press_event) {
            error!("Не удалось отправить оригинальное событие нажатия: {}", e);
        }

        // Если уже есть активный повторитель для этой комбинации, не пересоздаем его
        if self.active_repeaters.contains_key(&combination_hash) {
            debug_if_enabled!("Повторитель для {} уже активен, пропускаем создание", combination_id);
            return Ok(());
        }

        // Запускаем новый повторитель только если его еще нет
        info!("Запуск повторения для комбинации: {}", combination_id);
        self.start_repeater(event).await;

        Ok(())
    }

    /// Обработка отпускания клавиши
    async fn handle_key_release(&self, event: &KeyEvent) -> Result<()> {
        let combination_id = event.combination_id();
        let combination_hash = event.combination_hash();
        
        info!("Получено отпускание клавиши: {}", combination_id);
        
        // Останавливаем повторитель если он активен
        if self.active_repeaters.contains_key(&combination_hash) {
            info!("Остановка повторения для комбинации: {}", combination_id);
            self.stop_repeater(combination_hash).await;
        }
        
        // Отправляем оригинальное событие отпускания
        let original_release_event = VirtualKeyEvent::release(event.key_code, event.modifiers);  // ✅ Без clone - Copy type
        if let Err(e) = self.virtual_device.send_event(original_release_event) {
            error!("Не удалось отправить оригинальное событие отпускания: {}", e);
        }

        Ok(())
    }

    /// Обработка события окна
    pub async fn handle_window_event(&self, event: WindowEvent) -> Result<()> {
        debug_if_enabled!("Обработка события окна: {}", event);
        
        // Обновляем активное окно
        {
            let mut active_window = self.active_window.write();
            *active_window = Some(event.window.clone());
        }

        info!("Активное окно изменено на: {}", event.window);

        // Останавливаем все повторители с отправкой финальных release событий
        // для предотвращения "залипания" клавиш при смене окна
        self.stop_all_repeaters_gracefully().await;

        Ok(())
    }


    /// Запустить повторитель для комбинации клавиш
    async fn start_repeater(&self, event: &KeyEvent) {
        let combination_id = event.combination_id();
        let combination_hash = event.combination_hash();
        let virtual_device = Arc::clone(&self.virtual_device);  // ✅ Используем Arc::clone
        let config = Arc::clone(&self.config);  // ✅ Используем Arc::clone
        let key_code = event.key_code;
        let modifiers = event.modifiers;  // ✅ Без clone - Copy type
        let dry_run = self.dry_run;

        // Клонируем данные для использования в задаче и в структуре
        let combination_id_for_task = combination_id.clone();
        let modifiers_for_task = modifiers;  // ✅ Без clone - Copy type

        // Создаем задачу повторения
        let handle = tokio::spawn(async move {
            Self::repeater_task(
                combination_id_for_task,
                key_code,
                modifiers_for_task,
                virtual_device,
                config,
                dry_run,
            ).await;
        });

        // Сохраняем задачу
        let task = RepeaterTask {
            handle,
            key_code,
            modifiers,
        };

        self.active_repeaters.insert(combination_hash, task);
    }

    /// Остановить повторитель для конкретной комбинации
    async fn stop_repeater(&self, combination_hash: u64) {
        if let Some((_, task)) = self.active_repeaters.remove(&combination_hash) {
            task.handle.abort();
            debug_if_enabled!("Повторитель с хешем {} остановлен", combination_hash);
        }
    }

    /// Остановить все активные повторители
    async fn stop_all_repeaters(&self) {
        let count = self.active_repeaters.len();
        if count > 0 {
            info!("Остановка {} активных повторителей", count);

            // Собираем все ключи
            let keys: Vec<u64> = self.active_repeaters
                .iter()
                .map(|entry| *entry.key())
                .collect();

            // Удаляем и останавливаем каждый повторитель
            for key in keys {
                if let Some((_, task)) = self.active_repeaters.remove(&key) {
                    task.handle.abort();
                }
            }
        }
    }

    /// Остановить все активные повторители с отправкой финальных release событий
    /// для предотвращения "залипания" клавиш
    pub async fn stop_all_repeaters_gracefully(&self) {
        let count = self.active_repeaters.len();
        if count > 0 {
            info!("Корректная остановка {} активных повторителей с отправкой release событий", count);

            // Собираем информацию о всех активных повторителях
            let repeater_info: Vec<(u64, KeyCode, Modifiers)> = self.active_repeaters
                .iter()
                .map(|entry| {
                    let combination_hash = *entry.key();
                    let task = entry.value();
                    (combination_hash, task.key_code, task.modifiers)  // ✅ Без clone - Copy type
                })
                .collect();

            // Отправляем release события для всех активных повторителей
            for (_combination_hash, key_code, modifiers) in &repeater_info {
                // Создаем combination_id для логирования
                let combination_id = if modifiers.is_empty() {
                    format!("{}", key_code.value())
                } else {
                    format!("{}+{}", modifiers, key_code.value())
                };
                debug_if_enabled!("Отправка финального release события для {}", combination_id);
                let release_event = VirtualKeyEvent::release(*key_code, *modifiers);  // ✅ Без clone - Copy type
                if let Err(e) = self.virtual_device.send_event(release_event) {
                    error!("Не удалось отправить финальное событие отпускания для {}: {}", combination_id, e);
                }
            }

            // Теперь останавливаем все повторители
            let keys: Vec<u64> = self.active_repeaters
                .iter()
                .map(|entry| *entry.key())
                .collect();

            for key in keys {
                if let Some((_, task)) = self.active_repeaters.remove(&key) {
                    task.handle.abort();
                }
            }
        }
    }

    /// Задача повторения клавиши
    async fn repeater_task(
        combination_id: String,
        key_code: KeyCode,
        modifiers: Modifiers,
        virtual_device: Arc<VirtualDevice>,  // ✅ Используем Arc<VirtualDevice>
        config: Arc<Config>,
        dry_run: bool,
    ) {
        let repeat_delay = Duration::from_millis(config.input.repeat_delay_ms);

        debug_if_enabled!(
            "Запуск задачи повторения: {}, интервал: {}мс",
            combination_id,
            config.input.repeat_delay_ms
        );

        let mut repeat_count = 0;
        loop {
            repeat_count += 1;

            if dry_run {
                info!(
                    "[DRY RUN] Повтор #{} для {}: нажатие + отпускание",
                    repeat_count, combination_id
                );
            } else {
                // Отправляем нажатие
                let press_event = VirtualKeyEvent::press(key_code, modifiers);  // ✅ Без clone - Copy type
                if let Err(e) = virtual_device.send_event(press_event) {
                    error!("Ошибка отправки события нажатия: {}", e);
                    break;
                }

                // Отправляем отпускание сразу (без задержки для лучшей производительности)
                let release_event = VirtualKeyEvent::release(key_code, modifiers);  // ✅ Без clone - Copy type
                if let Err(e) = virtual_device.send_event(release_event) {
                    error!("Ошибка отправки события отпускания: {}", e);
                    break;
                }

                debug_if_enabled!("Повтор #{} для {} отправлен", repeat_count, combination_id);
            }

            // Ждем до следующего повтора
            sleep(repeat_delay).await;
        }

        debug_if_enabled!("Задача повторения для {} завершена после {} повторов", combination_id, repeat_count);
    }
}
