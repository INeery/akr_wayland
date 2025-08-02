use crate::config::Config;
use crate::error::Result;
use crate::services::KeyRepeater;
use std::sync::Arc;

/// Trait for keyboard listeners that can run in different modes
#[async_trait::async_trait]
pub trait KeyboardListenerTrait {
    /// Run the keyboard listener
    async fn run(self: Box<Self>) -> Result<()>;
}

/// Factory function to create an appropriate keyboard listener based on the dry_run flag
pub fn create_keyboard_listener(
    config: Arc<Config>,
    key_repeater: Arc<KeyRepeater>,
    dry_run: bool,
) -> Result<Box<dyn KeyboardListenerTrait + Send>> {
    if dry_run {
        Ok(Box::new(super::dry_keyboard_listener::DryRunKeyboardListener::new(
            config,
            key_repeater,
        )?))
    } else {
        Ok(Box::new(super::keyboard_listener::RealKeyboardListener::new(
            config,
            key_repeater,
        )?))
    }
}