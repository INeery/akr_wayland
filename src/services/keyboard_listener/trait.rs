use crate::config::Config;
use crate::error::Result;
use crate::services::KeyRepeater;
use std::sync::Arc;

/// Trait for keyboard listeners that can run in different modes.
///
/// Architectural contract (see guidelines):
/// - KeyboardListener ONLY reads raw events from devices.
/// - It MUST NOT decide about key repetition and MUST NOT filter events by mappings/patterns.
/// - All events are forwarded to KeyRepeater which is the single source of truth for repetition decisions.
#[async_trait::async_trait]
pub trait KeyboardListenerTrait {
    /// Run the keyboard listener
    async fn run(self: Box<Self>) -> Result<()>;
}

/// Factory function to create an appropriate keyboard listener based on the dry_run flag.
///
/// Contract: the created listener will not perform any decisions about key repetition. All
/// events are forwarded to KeyRepeater. This function does not read Config::should_repeat_key.
pub fn create_keyboard_listener(
    config: Arc<Config>,
    key_repeater: Arc<KeyRepeater>,
    virtual_device: Arc<crate::services::VirtualDevice>,
    dry_run: bool,
) -> Result<Box<dyn KeyboardListenerTrait + Send>> {
    if dry_run {
        Ok(Box::new(super::dry_keyboard_listener::DryRunKeyboardListener::new(
            config,
        )?))
    } else {
        Ok(Box::new(super::keyboard_listener::RealKeyboardListener::new(
            config,
            key_repeater,
            virtual_device,
        )?))
    }
}