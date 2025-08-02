use thiserror::Error;

#[derive(Error, Debug)]
pub enum AhkError {
    #[error("Ошибка конфигурации: {0}")]
    Config(#[from] anyhow::Error),

    #[error("Ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),


    #[error("Ошибка uinput: {0}")]
    Uinput(#[from] uinput::Error),

    #[error("Ошибка D-Bus: {0}")]
    DBus(#[from] zbus::Error),

    #[error("Ошибка канала: {0}")]
    Channel(String),

    #[error("Устройство не найдено: {0}")]
    DeviceNotFound(String),

    #[error("Недостаточно прав доступа: {0}")]
    Permission(String),

    #[error("Сервис недоступен: {0}")]
    ServiceUnavailable(String),

    #[error("Внутренняя ошибка: {0}")]
    Internal(String),
}

impl AhkError {
    pub fn channel<T>(msg: impl Into<String>) -> Result<T> {
        Err(AhkError::Channel(msg.into()))
    }

    pub fn device_not_found<T>(msg: impl Into<String>) -> Result<T> {
        Err(AhkError::DeviceNotFound(msg.into()))
    }

    pub fn permission<T>(msg: impl Into<String>) -> Result<T> {
        Err(AhkError::Permission(msg.into()))
    }

    pub fn service_unavailable<T>(msg: impl Into<String>) -> Result<T> {
        Err(AhkError::ServiceUnavailable(msg.into()))
    }

    pub fn internal<T>(msg: impl Into<String>) -> Result<T> {
        Err(AhkError::Internal(msg.into()))
    }
}

pub type Result<T> = std::result::Result<T, AhkError>;

// Удобные макросы для создания ошибок
#[macro_export]
macro_rules! ahk_error {
    (channel, $($arg:tt)*) => {
        $crate::error::AhkError::Channel(format!($($arg)*))
    };
    (device_not_found, $($arg:tt)*) => {
        $crate::error::AhkError::DeviceNotFound(format!($($arg)*))
    };
    (permission, $($arg:tt)*) => {
        $crate::error::AhkError::Permission(format!($($arg)*))
    };
    (invalid_key, $($arg:tt)*) => {
        $crate::error::AhkError::InvalidKey(format!($($arg)*))
    };
    (service_unavailable, $($arg:tt)*) => {
        $crate::error::AhkError::ServiceUnavailable(format!($($arg)*))
    };
    (timeout, $($arg:tt)*) => {
        $crate::error::AhkError::Timeout(format!($($arg)*))
    };
    (internal, $($arg:tt)*) => {
        $crate::error::AhkError::Internal(format!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = AhkError::channel::<()>("Тестовая ошибка канала");
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Тестовая ошибка канала"));
    }

    #[test]
    fn test_error_macro() {
        let err = ahk_error!(internal, "Код ошибки: {}", 42);
        assert_eq!(err.to_string(), "Внутренняя ошибка: Код ошибки: 42");
    }
}
