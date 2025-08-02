use crate::config::Config;
use crate::error::Result;
use crate::services::KeyRepeater;
use std::sync::Arc;

/// Trait for window detectors that can run in different modes
#[async_trait::async_trait]
pub trait WindowDetectorTrait {
    /// Run the window detector
    async fn run(self: Box<Self>) -> Result<()>;
}

/// Factory function to create an appropriate window detector based on the dry_run flag
pub fn create_window_detector(
    config: Arc<Config>,
    key_repeater: Arc<KeyRepeater>,
    dry_run: bool,
) -> Result<Box<dyn WindowDetectorTrait + Send>> {
    if dry_run {
        Ok(Box::new(super::dry_window_detector::DryRunDetector::new(
            key_repeater,
        )))
    } else {
        Ok(Box::new(super::window_detector::RealWindowDetector::new(
            config,
            key_repeater,
        )?))
    }
}