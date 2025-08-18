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
    DBus(String),

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
    pub fn device_not_found<T>(msg: impl Into<String>) -> Result<T> {
        Err(AhkError::DeviceNotFound(msg.into()))
    }
}

pub type Result<T> = std::result::Result<T, AhkError>;