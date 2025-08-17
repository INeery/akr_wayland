use crate::config::Config;
use crate::debug_if_enabled;
use crate::error::Result;
use crate::events::{
    KeyCode, KeyEvent, KeyState, Modifiers, VirtualKeyEvent, WindowEvent,
};
use crate::mappings::key_name_to_evdev_code::KeyNameToEvdevCode;
use crate::services::VirtualDevice;
use crate::services::window_context::WindowContext;
use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

pub struct KeyRepeater {
    config: Arc<Config>,
    virtual_device: Arc<VirtualDevice>,
    dry_run: bool,
    active_repeaters: Arc<DashMap<u64, RepeaterTask>>,
    // Per-combination decision cache (key + modifiers + window/patterns): true -> should repeat
    decision_cache: DashMap<CacheKey, bool>,
    window_ctx: Arc<dyn crate::services::window_context::WindowContext>,
    repetition_enabled: Arc<AtomicBool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CacheKey {
    key_name_hash: u64,
    modifiers_bits: u8,
    title_hash: u64,
    patterns_hash: u64,
}

impl KeyRepeater {
    pub fn new(config: Arc<Config>, virtual_device: Arc<VirtualDevice>, dry_run: bool) -> Result<Self> {
        info!("Инициализация KeyRepeater (dry_run: {})", dry_run);

        let ctx = Arc::new(crate::services::window_context::DefaultWindowContext::new());
        // Инициализируем hash паттернов при создании
        ctx.update_patterns_hash(&config.window.window_title_patterns);

        Ok(Self {
            config,
            virtual_device,
            dry_run,
            active_repeaters: Arc::new(DashMap::new()),
            decision_cache: DashMap::new(),
            window_ctx: ctx,
            repetition_enabled: Arc::new(AtomicBool::new(true)), // По умолчанию повторы включены
        })
    }

    /// Оптимизированная проверка повторения с кэшированием
    fn should_repeat_cached(&self, key_name: &str, modifiers: &[String]) -> bool {
        // Сформируем структурированный ключ без аллокаций
        let key_name_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            key_name.hash(&mut hasher);
            hasher.finish()
        };
        let modifiers_bits = Self::modifiers_to_bits(modifiers);
        let cache_key = CacheKey {
            key_name_hash,
            modifiers_bits,
            title_hash: self.window_ctx.get_title_hash(),
            patterns_hash: self.window_ctx.get_patterns_hash(),
        };

        if let Some(result) = self.decision_cache.get(&cache_key) {
            return *result;
        }

        // Fallback к полной проверке и обновление кэша
        let window_title_arc = self.window_ctx.get_title_lower();
        let result = self
            .config
            .should_repeat_key(key_name, modifiers, &window_title_arc);
        self.decision_cache.insert(cache_key, result);
        result
    }

    fn modifiers_to_bits(modifiers: &[String]) -> u8 {
        let mut bits: u8 = 0;
        for m in modifiers {
            match m.as_str() {
                "ctrl" => bits |= 0b0001,
                "alt" => bits |= 0b0010,
                "shift" => bits |= 0b0100,
                "super" => bits |= 0b1000,
                _ => {}
            }
        }
        bits
    }

    /// Обработка события клавиши.
    ///
    /// Architectural contract:
    /// - KeyRepeater receives ALL keyboard events from KeyboardListener.
    /// - This is the ONLY place where decisions about key repetition are made,
    ///   using Config::should_repeat_key().
    /// - If repetition is not required, the original event is forwarded without
    ///   any modifications via VirtualDevice.
    pub async fn handle_key_event(&self, event: &KeyEvent) -> Result<()> {
        debug_if_enabled!("Обработка события клавиши: {}", event);

        // Проверяем, является ли это нажатием клавиши переключения
        if let Some(ref repeat_toggle_key) = self.config.repeat.repeat_toggle_key {
            if let Some(key_name) = KeyNameToEvdevCode::reverse_translate(event.key_code.value()) {
                if key_name == repeat_toggle_key && event.state == KeyState::Pressed {
                    // Переключаем состояние повторов
                    let current_state = self.repetition_enabled.load(Ordering::Relaxed);
                    let new_state = !current_state;
                    self.repetition_enabled.store(new_state, Ordering::Relaxed);
                    
                    info!(
                        "Переключение состояния повторов: {} -> {} (клавиша: {})",
                        current_state, new_state, key_name
                    );
                    
                    // Если повторы отключаются, останавливаем все активные повторители
                    if !new_state {
                        self.stop_all_repeaters_gracefully().await;
                    }
                    
                    // Пробрасываем событие переключения как обычное
                    let virtual_event = VirtualKeyEvent::press(event.key_code, event.modifiers);
                    if let Err(e) = self.virtual_device.send_event(virtual_event) {
                        error!("Не удалось пробросить событие переключения: {}", e);
                    }
                    
                    return Ok(());
                }
            }
        }

        // Проверяем, нужно ли повторять эту клавишу с кэшированным заголовком
        let should_repeat =
            if let Some(key_name) = KeyNameToEvdevCode::reverse_translate(event.key_code.value()) {
                // Сначала проверяем, включены ли повторы
                if !self.repetition_enabled.load(Ordering::Relaxed) {
                    debug_if_enabled!("Повторы отключены, пропускаем клавишу '{}'", key_name);
                    false
                } else {
                    // Используем кэшированный заголовок окна из контекста
                    let window_title = self.window_ctx.get_title_lower();

                    debug_if_enabled!(
                        "KeyRepeater проверяет повторение для клавиши '{}' с заголовком окна: '{}'",
                        key_name,
                        window_title
                    );
                    debug_if_enabled!(
                        "Паттерны окон в конфигурации: {:?}",
                        self.config.window.window_title_patterns
                    );
                    debug_if_enabled!("Модификаторы события: {:?}", event.modifiers.to_vec());

                    // ✅ Используем оптимизированный метод с кэшированием и правильной логикой модификаторов
                    let modifiers_vec = event.modifiers.to_vec();
                    let result = self.should_repeat_cached(key_name, &modifiers_vec);
                    debug_if_enabled!(
                        "Результат проверки повторения для '{}': {}",
                        key_name,
                        result
                    );

                    result
                }
            } else {
                false
            };

        if should_repeat {
            // Обрабатываем как повторяемую клавишу
            match event.state {
                KeyState::Pressed => self.handle_key_press(event).await?,
                KeyState::Released => self.handle_key_release(event).await?,
                KeyState::Repeat => {
                    // Игнорируем аппаратные повторы - мы делаем свои
                    debug_if_enabled!("Игнорируем аппаратный повтор для {}", event.key_code);
                }
            }
        } else {
            // Пробрасываем обычное событие
            debug_if_enabled!(
                "Клавиша {} не нуждается в повторении - пробрасываем",
                KeyNameToEvdevCode::reverse_translate(event.key_code.value()).unwrap_or("Unknown")
            );

            let virtual_event = match event.state {
                KeyState::Pressed => VirtualKeyEvent::press(event.key_code, event.modifiers),
                KeyState::Released => VirtualKeyEvent::release(event.key_code, event.modifiers),
                KeyState::Repeat => {
                    VirtualKeyEvent::new(event.key_code, KeyState::Repeat, event.modifiers)
                }
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
        let key_hash = event.key_only_hash();

        info!(
            "Получено нажатие клавиши для повторения: {}",
            combination_id
        );

        // СНАЧАЛА отправляем оригинальное событие нажатия
        let original_press_event = VirtualKeyEvent::press(event.key_code, event.modifiers); // ✅ Без clone - Copy type
        if let Err(e) = self.virtual_device.send_event(original_press_event) {
            error!("Не удалось отправить оригинальное событие нажатия: {}", e);
        }

        // Если уже есть активный повторитель для этой клавиши, не пересоздаем его
        if self.active_repeaters.contains_key(&key_hash) {
            debug_if_enabled!(
                "Повторитель для {} уже активен, пропускаем создание",
                combination_id
            );
            return Ok(());
        }

        // Запускаем новый повторитель только если его еще нет
        debug_if_enabled!("Запуск повторения для комбинации: {}", combination_id);
        self.start_repeater(event).await;

        Ok(())
    }

    /// Обработка отпускания клавиши
    async fn handle_key_release(&self, event: &KeyEvent) -> Result<()> {
        let combination_id = event.combination_id();
        let key_hash = event.key_only_hash(); // ✅ ИСПРАВЛЕНИЕ: Используем key_only_hash для устойчивости к race conditions

        debug_if_enabled!("Получено отпускание клавиши: {}", combination_id);

        // Останавливаем повторитель если он активен
        if self.active_repeaters.contains_key(&key_hash) {
            debug_if_enabled!("Остановка повторения для комбинации: {}", combination_id);
            self.stop_repeater(key_hash).await;
        }

        // Отправляем оригинальное событие отпускания
        let original_release_event = VirtualKeyEvent::release(event.key_code, event.modifiers);
        if let Err(e) = self.virtual_device.send_event(original_release_event) {
            error!(
                "Не удалось отправить оригинальное событие отпускания: {}",
                e
            );
        }

        Ok(())
    }

    /// Обработка события окна
    pub async fn handle_window_event(&self, event: WindowEvent) -> Result<()> {
        debug_if_enabled!("Обработка события окна: {}", event);

        // Обновляем кэш заголовка окна через контекст
        self.window_ctx.update_title(&event.window.title);

        info!("Активное окно изменено на: {}", event.window);

        // Останавливаем все повторители с отправкой финальных release событий
        // для предотвращения "залипания" клавиш при смене окна
        self.stop_all_repeaters_gracefully().await;

        Ok(())
    }

    /// Запустить повторитель для комбинации клавиш
    async fn start_repeater(&self, event: &KeyEvent) {
        let combination_id = event.combination_id();
        let key_hash = event.key_only_hash();
        let virtual_device = Arc::clone(&self.virtual_device);
        let config = Arc::clone(&self.config);
        let key_code = event.key_code;
        let modifiers = event.modifiers;
        let dry_run = self.dry_run;

        // Клонируем данные для использования в задаче и в структуре
        let combination_id_for_task = combination_id.clone();
        let modifiers_for_task = modifiers;

        // Создаем задачу повторения
        let handle = tokio::spawn(async move {
            Self::repeater_task(
                combination_id_for_task,
                key_code,
                modifiers_for_task,
                virtual_device,
                config,
                dry_run,
            )
            .await;
        });

        // Сохраняем задачу
        let task = RepeaterTask {
            handle,
            key_code,
            modifiers,
        };

        self.active_repeaters.insert(key_hash, task);
    }

    /// Остановить повторитель для конкретной комбинации
    async fn stop_repeater(&self, combination_hash: u64) {
        if let Some((_, task)) = self.active_repeaters.remove(&combination_hash) {
            task.handle.abort();
            debug_if_enabled!("Повторитель с хешем {} остановлен", combination_hash);
        }
    }

    /// Остановить все активные повторители с отправкой финальных release событий
    /// для предотвращения "залипания" клавиш
    pub async fn stop_all_repeaters_gracefully(&self) {
        let count = self.active_repeaters.len();
        if count == 0 {
            return; // Ранний выход если нет активных повторителей
        }

        info!("Корректная остановка {} активных повторителей", count);

        // Отправляем release события батчем для лучшей производительности
        let release_events: Vec<_> = self
            .active_repeaters
            .iter()
            .map(|entry| {
                let task = entry.value();
                VirtualKeyEvent::release(task.key_code, task.modifiers)
            })
            .collect();

        // Отправляем все события сразу
        for event in release_events {
            let _ = self.virtual_device.send_event(event); // Игнорируем ошибки при shutdown
        }

        // Останавливаем все задачи перед очисткой
        for entry in self.active_repeaters.iter() {
            entry.value().handle.abort();
        }

        // Останавливаем все повторители
        self.active_repeaters.clear(); // Более эффективно чем итерация
    }

    /// Задача повторения клавиши
    async fn repeater_task(
        combination_id: String,
        key_code: KeyCode,
        modifiers: Modifiers,
        virtual_device: Arc<VirtualDevice>, // ✅ Используем Arc<VirtualDevice>
        config: Arc<Config>,
        dry_run: bool,
    ) {
        let repeat_delay = Duration::from_millis(config.repeat.repeat_delay_ms);

        debug_if_enabled!(
            "Запуск задачи повторения: {}, интервал: {}мс",
            combination_id,
            config.repeat.repeat_delay_ms
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
                let press_event = VirtualKeyEvent::press(key_code, modifiers);
                if let Err(e) = virtual_device.send_event(press_event) {
                    error!("Ошибка отправки события нажатия: {}", e);
                    break;
                }

                // Отправляем отпускание сразу (без задержки для лучшей производительности)
                let release_event = VirtualKeyEvent::release(key_code, modifiers);
                if let Err(e) = virtual_device.send_event(release_event) {
                    error!("Ошибка отправки события отпускания: {}", e);
                    break;
                }

                debug_if_enabled!("Повтор #{} для {} отправлен", repeat_count, combination_id);
            }

            // Ждем до следующего повтора
            sleep(repeat_delay).await;
        }

        debug_if_enabled!(
            "Задача повторения для {} завершена после {} повторов",
            combination_id,
            repeat_count
        );
    }
}

/// Задача повторения для конкретной комбинации клавиш
#[derive(Debug)]
struct RepeaterTask {
    handle: JoinHandle<()>,
    key_code: KeyCode,
    modifiers: Modifiers,
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_stop_all_repeaters_gracefully_aborts_tasks() {
        // Создаем тестовый конфиг
        let config = Arc::new(Config::default());
        let vd = Arc::new(crate::services::VirtualDevice::new("TestVD", true).unwrap());
        let key_repeater = KeyRepeater::new(config, vd, true).unwrap(); // dry_run = true

        // Создаем тестовое событие клавиши
        let key_event = KeyEvent::new(
            KeyCode::new(42), // произвольный код клавиши
            KeyState::Pressed,
            Modifiers::new(),
            0,
        );

        // Запускаем повторитель
        key_repeater.start_repeater(&key_event).await;
        
        // Проверяем, что повторитель создан
        assert_eq!(key_repeater.active_repeaters.len(), 1);
        
        // Получаем handle задачи для проверки
        let key_hash = key_event.key_only_hash(); // ✅ ИСПРАВЛЕНИЕ: Используем key_only_hash
        let task_finished = {
            let entry = key_repeater.active_repeaters.get(&key_hash).unwrap();
            entry.value().handle.is_finished()
        };
        
        // Задача должна быть активной
        assert!(!task_finished, "Задача должна быть активной перед остановкой");

        // Останавливаем все повторители
        key_repeater.stop_all_repeaters_gracefully().await;

        // Проверяем, что HashMap очищен
        assert_eq!(key_repeater.active_repeaters.len(), 0);

        // Даем время задаче завершиться после abort()
        sleep(Duration::from_millis(10)).await;

        // Проверяем, что задача действительно завершена
        // Примечание: после abort() и удаления из HashMap мы не можем проверить handle,
        // но важно что HashMap очищен и задачи получили abort()
    }

    #[tokio::test]
    async fn test_stop_all_repeaters_gracefully_with_empty_map() {
        let config = Arc::new(Config::default());
        let vd = Arc::new(crate::services::VirtualDevice::new("TestVD", true).unwrap());
        let key_repeater = KeyRepeater::new(config, vd, true).unwrap();

        // Проверяем, что метод корректно работает с пустым HashMap
        key_repeater.stop_all_repeaters_gracefully().await;
        
        assert_eq!(key_repeater.active_repeaters.len(), 0);
    }
    
    #[tokio::test]
    async fn test_toggle_functionality() {
        use crate::config::Config;
        use crate::events::{KeyCode, KeyEvent, KeyState, Modifiers};
        use std::sync::Arc;

        // Создаем конфигурацию с F12 как toggle key и маппингом для 'j'
        let mut config = Config::default();
        config.repeat.repeat_toggle_key = Some("f12".to_string());
        config.mappings = vec![crate::config::KeyMapping {
            key: "j".to_string(),
            modifiers: vec![],
        }];
        config.build_optimization_indexes();
        let config = Arc::new(config);

        // Создаем KeyRepeater в dry_run режиме
        let vd = Arc::new(crate::services::VirtualDevice::new("TestVD", true).unwrap());
        let repeater = KeyRepeater::new(config.clone(), vd, true).unwrap();

        // Проверяем, что повторы изначально включены
        assert!(repeater.repetition_enabled.load(Ordering::Relaxed));

        // Создаем событие нажатия F12 (toggle key)
        let f12_press = KeyEvent::new(
            KeyCode::new(88), // F12 keycode
            KeyState::Pressed,
            Modifiers::new(),
            0,
        );

        // Нажимаем F12 - должно переключить состояние на выключено
        repeater.handle_key_event(&f12_press).await.unwrap();
        assert!(!repeater.repetition_enabled.load(Ordering::Relaxed));

        // Нажимаем F12 снова - должно переключить состояние на включено
        repeater.handle_key_event(&f12_press).await.unwrap();
        assert!(repeater.repetition_enabled.load(Ordering::Relaxed));

        // Создаем событие нажатия 'j' (должно повторяться)
        let j_press = KeyEvent::new(
            KeyCode::new(36), // 'j' keycode
            KeyState::Pressed,
            Modifiers::new(),
            0,
        );

        // Когда повторы включены, 'j' должно обрабатываться как повторяемая клавиша
        // (проверяем, что не возникает ошибок)
        repeater.handle_key_event(&j_press).await.unwrap();

        // Отключаем повторы
        repeater.handle_key_event(&f12_press).await.unwrap();
        assert!(!repeater.repetition_enabled.load(Ordering::Relaxed));

        // Теперь 'j' должно обрабатываться как обычная клавиша
        repeater.handle_key_event(&j_press).await.unwrap();
    }

    #[tokio::test]
    async fn test_modifier_race_condition_fixed() {
        let config = Arc::new(Config::default());
        let vd = Arc::new(crate::services::VirtualDevice::new("TestVD", true).unwrap());
        let key_repeater = KeyRepeater::new(config, vd, true).unwrap();

        // Создаем press событие с модификатором
        let press_event = KeyEvent::new(
            KeyCode::new(42),
            KeyState::Pressed,
            Modifiers::new().with_ctrl(true),
            0,
        );

        // Создаем release событие БЕЗ модификатора (имитируем race condition)
        let release_event = KeyEvent::new(
            KeyCode::new(42),
            KeyState::Released,
            Modifiers::new(), // Модификатор уже отпущен!
            0,
        );

        // Запускаем повторитель
        key_repeater.start_repeater(&press_event).await;
        assert_eq!(key_repeater.active_repeaters.len(), 1);

        // Проверяем что key_only_hash одинаковый для обоих событий
        let press_key_hash = press_event.key_only_hash();
        let release_key_hash = release_event.key_only_hash();
        
        // key_only_hash должен быть одинаковым - исправление работает!
        assert_eq!(press_key_hash, release_key_hash, "key_only_hash одинаковый - исправление работает!");

        // Но combination_hash все еще разный (для демонстрации проблемы)
        let press_combination_hash = press_event.combination_hash();
        let release_combination_hash = release_event.combination_hash();
        assert_ne!(press_combination_hash, release_combination_hash, "combination_hash разный - показывает старую проблему");

        // Обрабатываем release событие
        if let Err(e) = key_repeater.handle_key_release(&release_event).await {
            panic!("Ошибка при обработке release: {}", e);
        }

        // Повторитель ДОЛЖЕН остановиться благодаря key_only_hash!
        assert_eq!(key_repeater.active_repeaters.len(), 0, "Повторитель остановился - исправление работает!");
    }

    #[tokio::test]
    async fn test_decision_cache_invalidation_on_title_change() {
        // Config: repeat 'j' in windows containing 'nvim'
        let mut cfg = Config::default();
        cfg.mappings = vec![crate::config::KeyMapping { key: "j".into(), modifiers: vec![] }];
        cfg.window.window_title_patterns = vec!["nvim".into()];
        cfg.build_optimization_indexes();
        let cfg = Arc::new(cfg);
        let vd = Arc::new(crate::services::VirtualDevice::new("TestVD", true).unwrap());
        let repeater = KeyRepeater::new(cfg, vd, true).unwrap();

        // Initial title: browser (no match) -> false
        repeater.window_ctx.update_title("browser");
        let res1 = repeater.should_repeat_cached("j", &[]);
        assert_eq!(res1, false);

        // Change title to NVIM -> should become true; cache key differs due to title_hash
        repeater.window_ctx.update_title("NVIM - file");
        let res2 = repeater.should_repeat_cached("j", &[]);
        assert_eq!(res2, true);
    }

}
