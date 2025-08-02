use crate::config::Config;
use crate::error::Result;
use crate::services::KeyRepeater;
use std::sync::Arc;
use tracing::{debug, info};

use super::r#trait::KeyboardListenerTrait;

pub struct DryRunKeyboardListener {
    config: Arc<Config>,
    key_repeater: Arc<KeyRepeater>,
}

impl DryRunKeyboardListener {
    pub fn new(config: Arc<Config>, key_repeater: Arc<KeyRepeater>) -> Result<Self> {
        info!("Инициализация DryRunKeyboardListener");
        Ok(Self {
            config,
            key_repeater,
        })
    }

    async fn run_impl(self) -> Result<()> {
        info!("Dry-run режим - KeyboardListener работает в режиме эмуляции");
        info!(
            "Настроено {} маппингов для повторения (dry-run)",
            self.config.mappings.len()
        );

        loop {
            // Эмулируем периодические события для тестирования
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            debug!("KeyboardListener работает в dry-run режиме");
        }
    }
}

#[async_trait::async_trait]
impl KeyboardListenerTrait for DryRunKeyboardListener {
    async fn run(self: Box<Self>) -> Result<()> {
        (*self).run_impl().await
    }
}
