use crate::config::Config;
use crate::error::{AhkError, Result};
use crate::events::{WindowEvent, WindowInfo};
use crate::events::window::WindowEventType;
use crate::services::KeyRepeater;
use std::sync::Arc;
use tracing::{debug, info, warn, error};
#[cfg(feature = "dbus")]
use zbus::Connection;
use tokio::time::{interval, Duration};
use parking_lot::RwLock;
use std::process::Command;

use super::kdotool::KdotoolDetector;
use super::xdotool::XdotoolDetector;
use super::wmctrl::WmctrlDetector;
use super::sway::SwayDetector;
use super::r#trait::WindowDetectorTrait;

#[derive(Debug, Clone)]
enum DesktopEnvironment {
    KDE,
    GNOME,
    X11Generic,
    WaylandGeneric,
    Unknown,
}

#[derive(Debug, Clone)]
enum WorkingMethod {
    Kdotool,
    Xdotool,
    Wmctrl,
    Sway,
}

pub struct RealWindowDetector {
    config: Arc<Config>,
    key_repeater: Arc<KeyRepeater>,
    desktop_env: DesktopEnvironment,
    current_window: Arc<RwLock<Option<WindowInfo>>>,
    #[cfg(feature = "dbus")]
    dbus_connection: Option<Connection>,
    #[cfg(not(feature = "dbus"))]
    dbus_connection: Option<()>,
    working_method: Option<WorkingMethod>,

    // Детекторы утилит
    kdotool: KdotoolDetector,
    xdotool: XdotoolDetector,
    wmctrl: WmctrlDetector,
    sway: SwayDetector,
}

impl RealWindowDetector {
    pub fn new(config: Arc<Config>, key_repeater: Arc<KeyRepeater>) -> Result<Self> {
        info!("Инициализация RealWindowDetector");

        let desktop_env = Self::detect_desktop_environment();
        info!("Обнаружена среда рабочего стола: {:?}", desktop_env);

        Ok(Self {
            config,
            key_repeater,
            desktop_env,
            current_window: Arc::new(RwLock::new(None)),
            dbus_connection: None,
            working_method: None,
            kdotool: KdotoolDetector::new(),
            xdotool: XdotoolDetector::new(),
            wmctrl: WmctrlDetector::new(),
            sway: SwayDetector::new(),
        })
    }

    fn detect_desktop_environment() -> DesktopEnvironment {
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            match desktop.to_lowercase().as_str() {
                d if d.contains("kde") => return DesktopEnvironment::KDE,
                d if d.contains("gnome") => return DesktopEnvironment::GNOME,
                _ => {}
            }
        }

        if let Ok(session) = std::env::var("XDG_SESSION_TYPE") {
            match session.as_str() {
                "wayland" => return DesktopEnvironment::WaylandGeneric,
                "x11" => return DesktopEnvironment::X11Generic,
                _ => {}
            }
        }

        if let Ok(output) = Command::new("pgrep").arg("-f").arg("kwin").output() {
            if !output.stdout.is_empty() {
                return DesktopEnvironment::KDE;
            }
        }

        if let Ok(output) = Command::new("pgrep").arg("-f").arg("gnome-shell").output() {
            if !output.stdout.is_empty() {
                return DesktopEnvironment::GNOME;
            }
        }

        DesktopEnvironment::Unknown
    }

    pub async fn run(mut self) -> Result<()> {
        info!("RealWindowDetector запущен для среды: {:?}", self.desktop_env);

        match self.config.window.detection_mode.as_str() {
            "dbus" => {
                if let Err(e) = self.run_dbus_detection().await {
                    warn!("D-Bus отслеживание не удалось: {}, переключаемся на polling", e);
                    self.run_polling_detection().await?;
                }
            }
            "polling" => {
                self.run_polling_detection().await?;
            }
            _ => {
                return Err(AhkError::Internal(
                    format!("Неизвестный режим детекции: {}", self.config.window.detection_mode)
                ));
            }
        }
        Ok(())
    }

    #[cfg(feature = "dbus")]
    async fn run_dbus_detection(&mut self) -> Result<()> {
        info!("Запуск D-Bus отслеживания окон");

        match self.desktop_env {
            DesktopEnvironment::KDE => self.run_kde_dbus().await,
            DesktopEnvironment::GNOME => self.run_gnome_dbus().await,
            _ => {
                warn!("D-Bus отслеживание не поддерживается для данной среды");
                Err(AhkError::ServiceUnavailable("D-Bus не поддерживается".to_string()))
            }
        }
    }

    #[cfg(not(feature = "dbus"))]
    async fn run_dbus_detection(&mut self) -> Result<()> {
        warn!("Бинарь собран без feature=\"dbus\", переключаемся на polling");
        Err(AhkError::ServiceUnavailable("D-Bus отключен при сборке".to_string()))
    }

    fn probe_order(&self) -> Vec<WorkingMethod> {
        match self.desktop_env {
            DesktopEnvironment::KDE => vec![
                WorkingMethod::Kdotool,
                WorkingMethod::Xdotool,
                WorkingMethod::Wmctrl,
                WorkingMethod::Sway,
            ],
            DesktopEnvironment::WaylandGeneric => vec![
                WorkingMethod::Sway,
                WorkingMethod::Kdotool,
                WorkingMethod::Xdotool,
                WorkingMethod::Wmctrl,
            ],
            DesktopEnvironment::X11Generic => vec![
                WorkingMethod::Xdotool,
                WorkingMethod::Wmctrl,
                WorkingMethod::Kdotool,
                WorkingMethod::Sway,
            ],
            DesktopEnvironment::GNOME => vec![
                WorkingMethod::Xdotool,
                WorkingMethod::Wmctrl,
                WorkingMethod::Sway,
                WorkingMethod::Kdotool,
            ],
            DesktopEnvironment::Unknown => vec![
                WorkingMethod::Sway,
                WorkingMethod::Xdotool,
                WorkingMethod::Wmctrl,
                WorkingMethod::Kdotool,
            ],
        }
    }

    async fn detect_working_method(&mut self) -> Result<WorkingMethod> {
        info!("Определяем рабочий метод детекции окон...");

        // Формируем приоритеты в зависимости от среды рабочего стола
        let order: Vec<WorkingMethod> = self.probe_order();

        // Логируем порядок пробинга
        info!("Порядок пробинга: {:?}", order);

        for method in order {
            let ok = match method {
                WorkingMethod::Kdotool => self.kdotool.test().await.is_ok(),
                WorkingMethod::Xdotool => self.xdotool.test().await.is_ok(),
                WorkingMethod::Wmctrl => self.wmctrl.test().await.is_ok(),
                WorkingMethod::Sway => self.sway.test().await.is_ok(),
            };
            if ok {
                info!("Выбран рабочий метод: {:?}", method);
                return Ok(method);
            }
        }

        Err(AhkError::Internal("Ни один метод детекции окон не работает".to_string()))
    }

    #[cfg(feature = "dbus")]
    async fn run_kde_dbus(&mut self) -> Result<()> {
        info!("Подключение к KDE KWin через D-Bus");

        let connection = Connection::session().await
            .map_err(|e| AhkError::DBus(e.to_string()))?;

        self.dbus_connection = Some(connection.clone());

        let working_method = if self.working_method.is_none() {
            let method = self.detect_working_method().await?;
            self.working_method = Some(method.clone());
            method
        } else {
            self.working_method.clone().unwrap()
        };

        let mut interval = interval(Duration::from_millis(self.config.window.polling_interval_ms));
        info!("KDE polling активен с методом: {:?}", working_method);

        loop {
            interval.tick().await;

            match self.get_window_by_method(&working_method).await {
                Ok(window) => {
                    if self.is_window_changed(&window) {
                        info!("Смена активного окна на: {}", window.title);
                        self.send_window_event(window, WindowEventType::FocusChanged).await?;
                    }
                }
                Err(e) => {
                    warn!("Рабочий метод {:?} перестал работать: {}. Переопределяем...", working_method, e);
                    match self.detect_working_method().await {
                        Ok(new_method) => {
                            info!("Переключились на новый метод: {:?}", new_method);
                            self.working_method = Some(new_method);
                        }
                        Err(_) => {
                            error!("Ни один метод не работает. Приостанавливаем детекцию на 10 секунд");
                            tokio::time::sleep(Duration::from_secs(10)).await;
                        }
                    }
                }
            }
        }
    }

    #[cfg(feature = "dbus")]
    async fn run_gnome_dbus(&mut self) -> Result<()> {
        info!("Подключение к GNOME Shell через D-Bus");

        let connection = Connection::session().await
            .map_err(|e| AhkError::DBus(e.to_string()))?;

        self.dbus_connection = Some(connection.clone());

        let mut interval = interval(Duration::from_millis(self.config.window.polling_interval_ms));

        loop {
            interval.tick().await;

            if let Ok(window) = self.xdotool.get_active_window().await {
                if self.is_window_changed(&window) {
                    self.send_window_event(window, WindowEventType::FocusChanged).await?;
                }
            }
        }
    }

    async fn run_polling_detection(&mut self) -> Result<()> {
        info!("Запуск polling отслеживания окон (с обязательным пробингом метода)");

        // Обязательный пробинг рабочего метода при старте
        let mut working_method = match self.detect_working_method().await {
            Ok(m) => {
                self.working_method = Some(m.clone());
                m
            }
            Err(e) => {
                warn!("Не удалось определить рабочий метод: {}. Будет выполнена повторная попытка позже", e);
                // Попробуем через небольшой интервал
                loop {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    if let Ok(m) = self.detect_working_method().await {
                        self.working_method = Some(m.clone());
                        break m;
                    } else {
                        warn!("Повторная попытка определить рабочий метод не удалась, продолжаем ожидание...");
                    }
                }
            }
        };

        info!("Polling активен с методом: {:?}", working_method);
        let mut interval = interval(Duration::from_millis(self.config.window.polling_interval_ms));

        loop {
            interval.tick().await;

            match self.get_window_by_method(&working_method).await {
                Ok(window) => {
                    if self.is_window_changed(&window) {
                        self.send_window_event(window, WindowEventType::FocusChanged).await?;
                    }
                }
                Err(e) => {
                    warn!("Рабочий метод {:?} перестал работать: {}. Переопределяем...", working_method, e);
                    match self.detect_working_method().await {
                        Ok(new_method) => {
                            info!("Переключились на новый метод: {:?}", new_method);
                            self.working_method = Some(new_method.clone());
                            working_method = new_method;
                        }
                        Err(_) => {
                            error!("Ни один метод не работает. Приостанавливаем детекцию на 10 секунд");
                            tokio::time::sleep(Duration::from_secs(10)).await;
                        }
                    }
                }
            }
        }
    }

    async fn get_window_by_method(&self, method: &WorkingMethod) -> Result<WindowInfo> {
        match method {
            WorkingMethod::Kdotool => self.kdotool.get_active_window().await,
            WorkingMethod::Xdotool => self.xdotool.get_active_window().await,
            WorkingMethod::Wmctrl => self.wmctrl.get_active_window().await,
            WorkingMethod::Sway => self.sway.get_active_window().await,
        }
    }

    fn is_window_changed(&self, new_window: &WindowInfo) -> bool {
        let current_window = self.current_window.read();
        match current_window.as_ref() {
            Some(current) => current.title != new_window.title || current.class != new_window.class,
            None => true,
        }
    }

    async fn send_window_event(&mut self, window: WindowInfo, event_type: WindowEventType) -> Result<()> {
        let old_title = self.current_window.read().as_ref().map(|w| w.title.clone()).unwrap_or_else(|| "None".to_string());

        debug!("Смена активного окна: {} -> {}", old_title, window.title);

        let event = WindowEvent::new(window.clone(), event_type);

        if let Err(e) = self.key_repeater.handle_window_event(event).await {
            error!("Не удалось обработать событие окна в KeyRepeater: {}", e);
            return Err(AhkError::Internal(format!("Ошибка обработки события окна: {}", e)));
        }

        *self.current_window.write() = Some(window);
        Ok(())
    }
}

impl Drop for RealWindowDetector {
    fn drop(&mut self) {
        info!("RealWindowDetector завершает работу");
    }
}

#[async_trait::async_trait]
impl WindowDetectorTrait for RealWindowDetector {
    async fn run(self: Box<Self>) -> Result<()> {
        (*self).run().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::VirtualDevice;

    fn make_detector_for_env(env: DesktopEnvironment) -> RealWindowDetector {
        let cfg = Arc::new(Config::default());
        let vd = Arc::new(VirtualDevice::new("TestVD", true).unwrap());
        let kr = Arc::new(crate::services::KeyRepeater::new(cfg.clone(), vd, true).unwrap());
        RealWindowDetector {
            config: cfg,
            key_repeater: kr,
            desktop_env: env,
            current_window: Arc::new(RwLock::new(None)),
            #[cfg(feature = "dbus")]
            dbus_connection: None,
            #[cfg(not(feature = "dbus"))]
            dbus_connection: None,
            working_method: None,
            kdotool: KdotoolDetector::new(),
            xdotool: XdotoolDetector::new(),
            wmctrl: WmctrlDetector::new(),
            sway: SwayDetector::new(),
        }
    }

    #[test]
    fn test_probe_order_priorities_per_env() {
        let det_kde = make_detector_for_env(DesktopEnvironment::KDE);
        let det_wl = make_detector_for_env(DesktopEnvironment::WaylandGeneric);
        let det_x11 = make_detector_for_env(DesktopEnvironment::X11Generic);
        let det_gnome = make_detector_for_env(DesktopEnvironment::GNOME);
        let det_unknown = make_detector_for_env(DesktopEnvironment::Unknown);

        let order_kde = det_kde.probe_order();
        let order_wl = det_wl.probe_order();
        let order_x11 = det_x11.probe_order();
        let order_gnome = det_gnome.probe_order();
        let order_unknown = det_unknown.probe_order();

        assert!(matches!(order_kde.first(), Some(WorkingMethod::Kdotool))); // KDE starts with kdotool
        assert!(matches!(order_wl.first(), Some(WorkingMethod::Sway)));    // WaylandGeneric starts with sway
        assert!(matches!(order_x11.first(), Some(WorkingMethod::Xdotool))); // X11 starts with xdotool
        assert!(matches!(order_gnome.first(), Some(WorkingMethod::Xdotool)));
        assert!(matches!(order_unknown.first(), Some(WorkingMethod::Sway)));
    }

    #[cfg(not(feature = "dbus"))]
    #[tokio::test]
    async fn test_dbus_disabled_falls_back_to_polling() {
        let mut det = make_detector_for_env(DesktopEnvironment::KDE);
        let res = det.run_dbus_detection().await;
        assert!(res.is_err(), "dbus disabled build should return error to trigger polling fallback");
    }
}
