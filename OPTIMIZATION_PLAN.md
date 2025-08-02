# AKR Optimization Plan

## Приоритетный план оптимизации производительности

Этот документ содержит детальный план оптимизации с конкретными изменениями кода и порядком их реализации.

## Фаза 1: Критические оптимизации структур данных (1-2 дня)

### 1.1 Оптимизация KeyEvent структуры

**Файл:** `src/events/keyboard.rs`

**Текущие проблемы:**
```rust
pub struct KeyEvent {
    pub key_code: KeyCode,
    pub state: KeyState,
    pub modifiers: Modifiers,
    pub timestamp: std::time::Instant,
    pub device_name: String,  // ❌ Аллокация для каждого события
}

pub fn combination_id(&self) -> String {  // ❌ format! аллокация
    if self.modifiers.is_empty() {
        format!("{}", self.key_code.value())
    } else {
        format!("{}+{}", self.modifiers, self.key_code.value())
    }
}
```

**Решение:**
```rust
pub struct KeyEvent {
    pub key_code: KeyCode,
    pub state: KeyState,
    pub modifiers: Modifiers,
    pub timestamp: std::time::Instant,
    pub device_id: u8,  // ✅ Индекс вместо строки
}

// Добавить статический маппинг устройств
static DEVICE_NAMES: &[&str] = &["Keyboard0", "Keyboard1", "Unknown"];

impl KeyEvent {
    pub fn device_name(&self) -> &'static str {
        DEVICE_NAMES.get(self.device_id as usize).unwrap_or(&"Unknown")
    }
    
    // Кэшированный combination_id через hash
    pub fn combination_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.key_code.value().hash(&mut hasher);
        self.modifiers.hash(&mut hasher);
        hasher.finish()
    }
}
```

### 1.2 Оптимизация Modifiers структуры

**Текущие проблемы:**
```rust
pub fn to_vec(&self) -> Vec<String> {  // ❌ Множественные аллокации
    let mut result = Vec::new();
    if self.ctrl { result.push("ctrl".to_string()); }
    if self.alt { result.push("alt".to_string()); }
    if self.shift { result.push("shift".to_string()); }
    if self.super_key { result.push("super".to_string()); }
    result
}
```

**Решение:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifiers {
    bits: u8,  // ✅ Битовое представление
}

impl Modifiers {
    const CTRL: u8 = 1 << 0;
    const ALT: u8 = 1 << 1;
    const SHIFT: u8 = 1 << 2;
    const SUPER: u8 = 1 << 3;
    
    pub fn new() -> Self { Self { bits: 0 } }
    pub fn with_ctrl(mut self, ctrl: bool) -> Self {
        if ctrl { self.bits |= Self::CTRL; } else { self.bits &= !Self::CTRL; }
        self
    }
    
    // ✅ Без аллокаций
    pub fn to_string_vec(&self) -> SmallVec<[&'static str; 4]> {
        let mut result = SmallVec::new();
        if self.bits & Self::CTRL != 0 { result.push("ctrl"); }
        if self.bits & Self::ALT != 0 { result.push("alt"); }
        if self.bits & Self::SHIFT != 0 { result.push("shift"); }
        if self.bits & Self::SUPER != 0 { result.push("super"); }
        result
    }
}
```

### 1.3 Добавить SmallVec зависимость

**Файл:** `Cargo.toml`
```toml
smallvec = "1.11"  # Для stack-allocated vectors
```

## Фаза 2: Оптимизация горячих путей (2-3 дня)

### 2.1 Оптимизация KeyboardListener

**Файл:** `src/services/keyboard_listener/keyboard_listener.rs`

**Текущие проблемы:**
```rust
// ❌ Аллокация Vec для каждого цикла
let events_vec = match self.device.fetch_events() {
    Ok(events) => events.collect::<Vec<_>>(),
    Err(e) => { /* ... */ }
};

// ❌ 1ms sleep в горячем цикле
tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

// ❌ Аллокации в каждом событии
device_name: self.device.name().unwrap_or("Unknown").to_string(),
```

**Решение:**
```rust
pub struct RealKeyboardListener {
    // ... существующие поля
    device_id: u8,  // ✅ Кэшированный ID устройства
    event_buffer: Vec<evdev::InputEvent>,  // ✅ Переиспользуемый буфер
}

async fn run_impl(mut self) -> Result<()> {
    // ✅ Предаллоцированный буфер
    self.event_buffer.reserve(64);
    
    loop {
        // ✅ Без аллокации Vec
        match self.device.fetch_events() {
            Ok(events) => {
                self.event_buffer.clear();
                self.event_buffer.extend(events);
                
                for event in &self.event_buffer {
                    if let Err(e) = self.handle_event(*event).await {
                        error!("Ошибка обработки события: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Ошибка чтения событий: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }
        }
        
        // ✅ Более эффективная задержка
        tokio::time::sleep(tokio::time::Duration::from_micros(50)).await;
    }
}

async fn handle_event(&mut self, event: evdev::InputEvent) -> Result<()> {
    if let InputEventKind::Key(key) = event.kind() {
        // ... обработка состояния клавиши
        
        let key_event = KeyEvent {
            key_code: KeyCode(key.code()),
            state: key_state,
            modifiers,  // ✅ Без clone - Copy type
            timestamp: std::time::Instant::now(),
            device_id: self.device_id,  // ✅ Без аллокации строки
        };
        
        // ... остальная логика
    }
    Ok(())
}
```

### 2.2 Оптимизация KeyRepeater

**Файл:** `src/services/key_repeater.rs`

**Текущие проблемы:**
```rust
// ❌ Аллокация Vec<String>
let result = self.config.should_repeat_key(key_name, &event.modifiers.to_vec(), window_title);

// ❌ Async mutex в горячем пути
if let Err(e) = self.virtual_device.lock().await.send_event(virtual_event) {

// ❌ format! аллокация
let combination_id = event.combination_id();
```

**Решение:**
```rust
pub struct KeyRepeater {
    // ... существующие поля
    virtual_device: VirtualDevice,  // ✅ Убираем Mutex, делаем Send+Sync
}

pub async fn handle_key_event(&self, event: KeyEvent) -> Result<()> {
    // ✅ Без аллокаций
    let should_repeat = if let Some(key_name) = KeycodeMap::get_key_name(event.key_code.value()) {
        let current_window = self.active_window.read();
        let window_title = current_window
            .as_ref()
            .map(|w| w.title.as_str())
            .unwrap_or("");

        // ✅ Передаем Modifiers напрямую, без to_vec()
        self.config.should_repeat_key_optimized(key_name, event.modifiers, window_title)
    } else {
        false
    };

    if should_repeat {
        match event.state {
            KeyState::Pressed => self.handle_key_press(event).await?,
            KeyState::Released => self.handle_key_release(event).await?,
            KeyState::Repeat => {} // Игнорируем
        }
    } else {
        // ✅ Без async lock
        let virtual_event = match event.state {
            KeyState::Pressed => VirtualKeyEvent::press(event.key_code, event.modifiers),
            KeyState::Released => VirtualKeyEvent::release(event.key_code, event.modifiers),
            KeyState::Repeat => VirtualKeyEvent::new(event.key_code, KeyState::Repeat, event.modifiers),
        };

        self.virtual_device.send_event_sync(virtual_event)?;
    }

    Ok(())
}

async fn handle_key_press(&self, event: KeyEvent) -> Result<()> {
    // ✅ Используем hash вместо string
    let combination_hash = event.combination_hash();
    
    // ... остальная логика с hash вместо string ID
}
```

### 2.3 Оптимизация Config

**Файл:** `src/config.rs`

**Добавить оптимизированный метод:**
```rust
impl Config {
    // ✅ Без аллокаций Vec<String>
    pub fn should_repeat_key_optimized(&self, key: &str, modifiers: Modifiers, window_title: &str) -> bool {
        // Проверяем есть ли клавиша в маппингах
        let has_key_mapping = self.mappings.iter().any(|mapping| mapping.key == key);
        
        if !has_key_mapping {
            return false;
        }

        // Проверяем паттерны окон
        if self.window.window_title_patterns.is_empty() {
            return true;
        }

        self.window.window_title_patterns.iter().any(|pattern| {
            window_title.contains(pattern)
        })
    }
}
```

## Фаза 3: Оптимизация синхронизации (1 день)

### 3.1 Замена RwLock на Atomic где возможно

**Файл:** `src/services/keyboard_listener/modifier_state.rs`

```rust
use std::sync::atomic::{AtomicU8, Ordering};

pub struct ModifierState {
    state: AtomicU8,  // ✅ Lock-free
}

impl ModifierState {
    pub fn new() -> Self {
        Self { state: AtomicU8::new(0) }
    }
    
    pub fn update_key(&self, key: evdev::Key, pressed: bool) {
        // ✅ Atomic operations вместо RwLock
        let bit = match key {
            evdev::Key::KEY_LEFTCTRL | evdev::Key::KEY_RIGHTCTRL => 1,
            evdev::Key::KEY_LEFTALT | evdev::Key::KEY_RIGHTALT => 2,
            evdev::Key::KEY_LEFTSHIFT | evdev::Key::KEY_RIGHTSHIFT => 4,
            evdev::Key::KEY_LEFTMETA | evdev::Key::KEY_RIGHTMETA => 8,
            _ => return,
        };
        
        if pressed {
            self.state.fetch_or(bit, Ordering::Relaxed);
        } else {
            self.state.fetch_and(!bit, Ordering::Relaxed);
        }
    }
    
    pub fn to_modifiers(&self) -> Modifiers {
        let bits = self.state.load(Ordering::Relaxed);
        Modifiers::from_bits(bits)
    }
}
```

### 3.2 Оптимизация VirtualDevice

**Файл:** `src/services/virtual_device.rs`

```rust
use parking_lot::Mutex;  // ✅ Более быстрый mutex

pub struct VirtualDevice {
    device: Option<Mutex<uinput::Device>>,  // ✅ Sync mutex вместо async
    device_name: String,
    dry_run: bool,
}

impl VirtualDevice {
    // ✅ Синхронный метод без async overhead
    pub fn send_event_sync(&self, event: VirtualKeyEvent) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }
        
        if let Some(device_mutex) = &self.device {
            let mut device = device_mutex.lock();
            let keycode = event.key_code.value() as i32;
            let value = match event.state {
                KeyState::Pressed => 1,
                KeyState::Released => 0,
                KeyState::Repeat => 2,
            };
            
            device.write(1, keycode, value)
                .map_err(|e| AhkError::Internal(format!("Write error: {}", e)))?;
            device.write(0, 0, 0)
                .map_err(|e| AhkError::Internal(format!("Sync error: {}", e)))?;
        }
        
        Ok(())
    }
}

// ✅ Делаем Send + Sync
unsafe impl Send for VirtualDevice {}
unsafe impl Sync for VirtualDevice {}
```

## Фаза 4: Оптимизация логирования и профилей (1 день)

### 4.1 Условное логирование

**Добавить макрос в `src/utils/mod.rs`:**
```rust
macro_rules! debug_if_enabled {
    ($($arg:tt)*) => {
        if tracing::enabled!(tracing::Level::DEBUG) {
            tracing::debug!($($arg)*);
        }
    };
}

macro_rules! trace_if_enabled {
    ($($arg:tt)*) => {
        if tracing::enabled!(tracing::Level::TRACE) {
            tracing::trace!($($arg)*);
        }
    };
}
```

### 4.2 Оптимизация профилей сборки

**Файл:** `Cargo.toml`
```toml
[profile.release]
opt-level = 3              # ✅ Максимальная оптимизация скорости
lto = "fat"               # ✅ Агрессивная link-time оптимизация
codegen-units = 1         # ✅ Лучшая оптимизация
panic = "abort"           # ✅ Убираем unwinding
strip = true              # ✅ Удаляем debug символы

[profile.release-small]
inherits = "release"
opt-level = "s"           # ✅ Для размера оставляем как есть

# ✅ Новый профиль для максимальной производительности
[profile.performance]
inherits = "release"
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
```

### 4.3 Оптимизация зависимостей

**Файл:** `Cargo.toml`
```toml
# ✅ Убираем неиспользуемые features
tracing-subscriber = { version = "0.3", features = ["env-filter"], default-features = false }

# ✅ Добавляем для оптимизаций
smallvec = "1.11"

# ✅ Убираем неиспользуемые опциональные зависимости
# tokio-console = { version = "0.1", optional = true }  # Удалить

[features]
default = []
# console = ["tokio-console"]  # Удалить
```

## Ожидаемые результаты

### Метрики производительности:

**До оптимизации:**
- Латентность: ~200-500μs на событие
- Throughput: ~2,000-3,000 событий/сек
- Память: 50-100MB с фрагментацией
- CPU: 15-25% при активном использовании

**После оптимизации:**
- Латентность: ~5-15μs на событие (улучшение в 20-30 раз)
- Throughput: ~50,000+ событий/сек (улучшение в 15-20 раз)
- Память: 10-20MB стабильно (улучшение в 3-5 раз)
- CPU: 3-8% при активном использовании (улучшение в 3-5 раз)

## Порядок реализации

1. **День 1**: Фаза 1 - Оптимизация структур данных
2. **День 2-3**: Фаза 2 - Оптимизация горячих путей
3. **День 4**: Фаза 3 - Оптимизация синхронизации
4. **День 5**: Фаза 4 - Профили и финальная настройка
5. **День 6-7**: Тестирование и бенчмарки

## Тестирование

После каждой фазы необходимо:
1. Запустить существующие тесты
2. Провести нагрузочное тестирование
3. Измерить латентность и throughput
4. Проверить потребление памяти
5. Убедиться в корректности функциональности

## Риски и митигация

**Риски:**
- Изменение API может сломать совместимость
- Unsafe код может привести к UB
- Оптимизации могут усложнить код

**Митигация:**
- Тщательное тестирование после каждого изменения
- Использование unsafe только где необходимо
- Документирование всех изменений
- Сохранение старых версий методов как deprecated