use crate::error::Result;
use crate::events::window::WindowEventType;
use crate::events::{WindowEvent, WindowInfo};
use crate::services::KeyRepeater;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::info;

pub struct DryRunDetector {
    key_repeater: Arc<KeyRepeater>,
}

impl DryRunDetector {
    pub fn new(key_repeater: Arc<KeyRepeater>) -> Self {
        Self { key_repeater }
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("Dry-run режим - WindowDetector работает в режиме эмуляции");

        let fake_windows = vec![
            "Terminal - dry_run",
            "Browser - dry_run",
            "Editor - dry_run",
            "Game - dry_run",
        ];

        let mut window_index = 0;
        let mut interval = interval(Duration::from_secs(10));

        loop {
            interval.tick().await;

            let fake_window = WindowInfo::new(fake_windows[window_index].to_string())
                .with_class("DryRun".to_string());

            info!("Dry-run: эмулируем смену окна на: {}", fake_window.title);
            self.send_window_event(fake_window, WindowEventType::FocusChanged)
                .await?;

            window_index = (window_index + 1) % fake_windows.len();
        }
    }

    async fn send_window_event(
        &mut self,
        window: WindowInfo,
        event_type: WindowEventType,
    ) -> Result<()> {
        let event = WindowEvent::new(window, event_type);
        self.key_repeater
            .handle_window_event(event)
            .await
            .map_err(|e| {
                crate::error::AhkError::Internal(format!(
                    "Ошибка обработки события окна в KeyRepeater: {}",
                    e
                ))
            })?;
        Ok(())
    }
}
