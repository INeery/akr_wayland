use parking_lot::RwLock;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

/// Abstraction over the current window context used by KeyRepeater.
///
/// Responsibilities:
/// - Provide lowercase window title for fast, allocation-free reads on hot path.
/// - Provide stable hashes for title and pattern set to support caching and invalidation.
/// - Maintain a small decision cache for should_repeat checks, automatically invalidated on changes.
pub trait WindowContext: Send + Sync {
    fn get_title_lower(&self) -> String;
    fn get_title_hash(&self) -> u64;
    fn get_patterns_hash(&self) -> u64;
    fn update_title(&self, title: &str);
    fn update_patterns_hash(&self, patterns: &[String]);
    fn get_cached_decision(&self, key: &str) -> Option<bool>;
    fn put_cached_decision(&self, key: String, value: bool);
}

/// Default implementation of WindowContext backed by an internal cache structure.
pub struct DefaultWindowContext {
    title_hash: AtomicU64,
    title_lower: RwLock<String>,
    patterns_hash: AtomicU64,
    should_repeat_cache: RwLock<HashMap<String, bool>>, // key -> decision
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
            title_lower: RwLock::new(String::new()),
            patterns_hash: AtomicU64::new(0),
            should_repeat_cache: RwLock::new(HashMap::new()),
        }
    }
}

impl WindowContext for DefaultWindowContext {
    fn get_title_lower(&self) -> String {
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
            *self.title_lower.write() = title.to_lowercase();
            // Invalidate decision cache on title change
            self.should_repeat_cache.write().clear();
        }
    }

    fn update_patterns_hash(&self, patterns: &[String]) {
        let mut hasher = DefaultHasher::new();
        patterns.hash(&mut hasher);
        let new_hash = hasher.finish();
        let old_hash = self.patterns_hash.swap(new_hash, Ordering::Relaxed);
        if old_hash != new_hash {
            // Invalidate decision cache on patterns change
            self.should_repeat_cache.write().clear();
        }
    }

    fn get_cached_decision(&self, key: &str) -> Option<bool> {
        self.should_repeat_cache.read().get(key).copied()
    }

    fn put_cached_decision(&self, key: String, value: bool) {
        self.should_repeat_cache.write().insert(key, value);
    }
}
