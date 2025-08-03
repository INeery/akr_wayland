pub mod device_finder;
pub mod permissions;

pub use device_finder::DeviceFinder;

// ✅ Макросы условного логирования для оптимизации производительности
#[macro_export]
macro_rules! debug_if_enabled {
    ($($arg:tt)*) => {
        if tracing::enabled!(tracing::Level::DEBUG) {
            tracing::debug!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! trace_if_enabled {
    ($($arg:tt)*) => {
        if tracing::enabled!(tracing::Level::TRACE) {
            tracing::trace!($($arg)*);
        }
    };
}
