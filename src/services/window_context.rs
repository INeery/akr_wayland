use parking_lot::RwLock;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// WindowContext provides read-only window state for KeyRepeater and other services.
///
/// Responsibilities (strict):
/// - Detect and cache the current window state (e.g., title) with cheap reads on the hot path.
/// - Provide stable hashes for title and pattern set to support invalidation by consumers.
/// - Do NOT make any decisions related to key repetition or mappings.
/// - Do NOT cache repetition decisions; this belongs to KeyRepeater.
pub trait WindowContext: Send + Sync {
    fn get_title_lower(&self) -> Arc<str>;
    fn get_title_hash(&self) -> u64;
    fn get_patterns_hash(&self) -> u64;
    fn update_title(&self, title: &str);
    fn update_patterns_hash(&self, patterns: &[String]);
}

/// Default implementation of WindowContext backed by a small internal cache of the window title.
pub struct DefaultWindowContext {
    title_hash: AtomicU64,
    title_lower: RwLock<Arc<str>>, // Arc<str> to avoid cloning on reads
    patterns_hash: AtomicU64,
}

impl Default for DefaultWindowContext {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultWindowContext {
    pub fn new() -> Self {
        Self {
            title_hash: AtomicU64::new(0),
            title_lower: RwLock::new(String::new().into_boxed_str().into()),
            patterns_hash: AtomicU64::new(0),
        }
    }
}

impl WindowContext for DefaultWindowContext {
    fn get_title_lower(&self) -> Arc<str> {
        self.title_lower.read().clone()
    }

    fn get_title_hash(&self) -> u64 {
        self.title_hash.load(Ordering::Relaxed)
    }

    fn get_patterns_hash(&self) -> u64 {
        self.patterns_hash.load(Ordering::Relaxed)
    }

    fn update_title(&self, title: &str) {
        let mut hasher = DefaultHasher::new();
        title.hash(&mut hasher);
        let new_hash = hasher.finish();
        let old_hash = self.title_hash.swap(new_hash, Ordering::Relaxed);
        if old_hash != new_hash {
            *self.title_lower.write() = title.to_lowercase().into_boxed_str().into();
        }
    }

    fn update_patterns_hash(&self, patterns: &[String]) {
        let mut hasher = DefaultHasher::new();
        patterns.hash(&mut hasher);
        let new_hash = hasher.finish();
        self.patterns_hash.store(new_hash, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_title_and_hashes_work() {
        let ctx = DefaultWindowContext::new();
        assert_eq!(&*ctx.get_title_lower(), "");
        assert_eq!(ctx.get_title_hash(), 0);

        ctx.update_title("NVIM - FILE");
        assert_eq!(&*ctx.get_title_lower(), "nvim - file");
        assert_ne!(ctx.get_title_hash(), 0);
    }

    #[test]
    fn update_patterns_hash_changes_hash() {
        let ctx = DefaultWindowContext::new();
        let h1 = ctx.get_patterns_hash();
        ctx.update_patterns_hash(&vec!["nvim".into(), "term".into()]);
        let h2 = ctx.get_patterns_hash();
        assert_ne!(h1, h2);
    }
}
