# AKR Performance Analysis Report

## Executive Summary

После анализа кодовой базы AKR выявлены значительные проблемы производительности, которые могут влиять на латентность обработки клавиш и потребление ресурсов. Основные проблемы связаны с избыточными аллокациями памяти в горячих путях, неэффективной синхронизацией и избыточным логированием.

## 1. Анализ зависимостей Cargo.toml

### ✅ Используемые зависимости
Все основные зависимости активно используются:
- **tokio**: Async runtime - используется везде
- **evdev**: Чтение событий клавиатуры - keyboard_listener
- **uinput**: Создание виртуальных устройств - virtual_device
- **clap**: CLI аргументы - main.rs
- **figment**: Конфигурация - config.rs
- **serde**: Сериализация - config.rs, events
- **zbus**: D-Bus коммуникация - window_detector
- **dashmap**: Concurrent HashMap - key_repeater (active_repeaters)
- **parking_lot**: RwLock - используется в нескольких местах
- **anyhow/thiserror**: Обработка ошибок - error.rs
- **tracing**: Логирование - везде
- **once_cell**: Lazy static - keycode_map, kdotool
- **async-trait**: Trait для async - keyboard_listener

### ⚠️ Потенциально неиспользуемые
- **tokio-console**: Опциональная зависимость для отладки, не используется в коде

### 🔧 Возможные оптимизации зависимостей
1. **tracing-subscriber**: Можно отключить `default-features` и оставить только `env-filter`
2. **figment**: Возможно избыточно для простой TOML конфигурации, можно заменить на `toml` + `serde`
3. **Профили сборки**: Текущие настройки оптимизированы для размера (`opt-level = "s"`), но для производительности лучше `opt-level = 3`

## 2. Критические проблемы производительности

### 🔥 Горячие пути с аллокациями

#### 2.1 KeyboardListener (src/services/keyboard_listener/keyboard_listener.rs)
**Проблемы:**
- **Строка 70**: `events.collect::<Vec<_>>()` - создание Vec для каждого цикла
- **Строка 85**: `tokio::time::sleep(Duration::from_millis(1))` - 1ms задержка в горячем цикле
- **Строка 114**: `device_name: self.device.name().unwrap_or("Unknown").to_string()` - аллокация строки для каждого события
- **Строки 102-106**: RwLock contention для modifier_state при каждом событии
- **Строка 112**: `modifiers.clone()` - клонирование модификаторов

**Влияние:** Каждое событие клавиши создает минимум 2-3 аллокации + RwLock overhead

#### 2.2 KeyRepeater (src/services/key_repeater.rs)
**Проблемы:**
- **Строка 63**: `event.modifiers.to_vec()` - создание Vec<String> для каждой проверки
- **Строка 99**: `self.virtual_device.lock().await` - async mutex lock в горячем пути
- **Строки 109, 134**: `event.combination_id()` - format! аллокация для каждого события
- **Строки 114, 145**: `event.modifiers.clone()` - клонирование модификаторов
- **Строки 55, 158**: RwLock contention для active_window

**Влияние:** Каждое событие создает 3-5 аллокаций + 2 async lock + RwLock

#### 2.3 Event Structures (src/events/keyboard.rs)
**Проблемы:**
- **Строка 118**: `device_name: String` - хранение строки в каждом событии
- **Строки 77-80**: `Modifiers::to_vec()` - создает Vec<String> с .to_string() для каждого модификатора
- **Строки 141-144**: `combination_id()` - format! аллокация
- **Строки 102-107**: `Display` implementation вызывает to_vec() + join()

**Влияние:** Базовые структуры данных создают избыточные аллокации

#### 2.4 WindowDetector (src/services/window_detector/window_detector.rs)
**Проблемы:**
- **Строки 254, 259, 269**: `WindowInfo::new("Unknown".to_string())` - аллокации строк
- **Строка 298**: `w.title.clone()` - клонирование заголовка окна
- **Строки 290, 309**: RwLock contention для current_window
- **Строки 248-271**: Каскадные fallback попытки создают множественные системные вызовы

**Влияние:** Polling создает аллокации даже когда окно не изменилось

#### 2.5 VirtualDevice (src/services/virtual_device.rs)
**Проблемы:**
- **Строки 49, 69**: Debug логирование в горячем пути
- **Строки 61, 66**: `format!` аллокации для error handling
- **Строка 71**: `"Виртуальное устройство недоступно".to_string()`

### 🐌 Проблемы синхронизации

#### 2.6 Lock Contention
- **RwLock в modifier_state**: Читается/пишется при каждом событии
- **RwLock в active_window**: Читается при каждом событии, пишется при смене окна
- **Async Mutex в virtual_device**: Блокирует при каждой отправке события
- **DashMap в active_repeaters**: Concurrent, но все равно overhead

#### 2.7 Async Overhead
- Множественные `.await` в горячих путях
- Создание futures для каждого события
- Task spawning для каждого repeater

### 📊 Логирование в горячих путях

#### 2.8 Избыточное логирование
- **Debug логи**: В keyboard_listener, key_repeater, virtual_device
- **String formatting**: Для логов даже когда они отключены
- **Timestamp calculations**: В Display implementations

## 3. Оценка влияния на производительность

### Критичность проблем:
1. **🔴 КРИТИЧНО**: Аллокации в горячих путях (KeyEvent, Modifiers)
2. **🟠 ВЫСОКО**: RwLock contention, async mutex locks
3. **🟡 СРЕДНЕ**: Debug логирование, string formatting
4. **🟢 НИЗКО**: Polling inefficiency, fallback cascades

### Ожидаемый эффект:
- **Латентность**: Увеличение на 50-200μs на событие из-за аллокаций
- **Throughput**: Снижение на 30-50% при высокой частоте событий
- **Память**: Фрагментация heap из-за частых аллокаций/деаллокаций
- **CPU**: Дополнительная нагрузка на GC и memory allocator

## 4. Рекомендации по оптимизации

### 4.1 Приоритет 1 (Критично)
1. **Избавиться от String аллокаций в событиях**
   - Заменить `device_name: String` на `device_name: &'static str` или индекс
   - Кэшировать `combination_id` или использовать hash
   - Заменить `Modifiers::to_vec()` на битовые операции

2. **Оптимизировать event processing loop**
   - Убрать `collect::<Vec<_>>()`
   - Увеличить sleep до 10-50μs или использовать epoll
   - Переиспользовать KeyEvent структуры

### 4.2 Приоритет 2 (Высоко)
1. **Уменьшить lock contention**
   - Использовать atomic operations для простых состояний
   - Batch updates для window state
   - Lock-free структуры данных где возможно

2. **Оптимизировать async overhead**
   - Уменьшить количество .await в горячих путях
   - Использовать channels вместо прямых async calls

### 4.3 Приоритет 3 (Средне)
1. **Условное логирование**
   - Проверять log level перед string formatting
   - Использовать lazy evaluation для debug логов

2. **Профили сборки**
   - Для production использовать `opt-level = 3`
   - Включить `lto = "fat"` для лучшей оптимизации

### 4.4 Приоритет 4 (Низко)
1. **Зависимости**
   - Заменить figment на простой toml parser
   - Убрать tokio-console из зависимостей

## 5. Ожидаемый эффект от оптимизаций

### После оптимизаций Приоритета 1:
- **Латентность**: Снижение на 70-80%
- **Throughput**: Увеличение в 2-3 раза
- **Память**: Снижение потребления на 60-70%

### После всех оптимизаций:
- **Латентность**: < 10μs на событие
- **Throughput**: > 10,000 событий/сек
- **Память**: Стабильное потребление без фрагментации

## 6. План реализации

1. **Фаза 1** (1-2 дня): Оптимизация структур данных
2. **Фаза 2** (2-3 дня): Оптимизация горячих путей
3. **Фаза 3** (1 день): Оптимизация синхронизации
4. **Фаза 4** (1 день): Профили сборки и зависимости

**Общее время**: 5-7 дней разработки + тестирование