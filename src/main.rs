use anyhow::Result;
use clap::Parser;
use tokio::signal;
use tracing::{info, error, warn};
use std::sync::Arc;
mod config;
mod error;
mod events;
pub mod mappings;
mod services;
mod utils;

use config::Config;
use services::{
    create_keyboard_listener,
    create_window_detector,
    KeyRepeater,
};

#[derive(Parser, Debug)]
#[command(name = "ahk-rust")]
#[command(about = "Утилита для повторения нажатых клавиш при их удерживании")]
struct Args {
    /// Путь к файлу конфигурации
    #[arg(short, long, default_value = "ahk.toml")]
    config: String,

    /// Режим сухого запуска (без реальных действий)
    #[arg(long)]
    dry_run: bool,

    /// Уровень логирования
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Включить метрики Prometheus
    #[arg(long)]
    enable_metrics: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Инициализация системы логирования
    init_tracing(&args.log_level)?;

    info!("Запуск AHK Rust v{}", env!("CARGO_PKG_VERSION"));

    // Загрузка конфигурации
    let config = Arc::new(Config::load(&args.config)?);
    info!("Конфигурация загружена из: {}", args.config);

    if args.dry_run {
        warn!("Режим сухого запуска - реальные действия отключены");
    }

    // Проверка прав доступа
    utils::permissions::check_permissions()?;

    // Инициализация компонентов (KeyRepeater создается первым для передачи в другие сервисы)
    let key_repeater = Arc::new(KeyRepeater::new(config.clone(), args.dry_run)?);
    let keyboard_listener = create_keyboard_listener(config.clone(), key_repeater.clone(), args.dry_run)?;
    let window_detector = create_window_detector(config.clone(), key_repeater.clone(), args.dry_run)?;

    info!("Все компоненты инициализированы");

    // Запуск всех сервисов параллельно
    let handles = vec![
        tokio::spawn(async move {
            if let Err(e) = keyboard_listener.run().await {
                error!("Ошибка в KeyboardListener: {}", e);
            }
        }),
        tokio::spawn(async move {
            if let Err(e) = window_detector.run().await {
                error!("Ошибка в WindowDetector: {}", e);
            }
        }),
    ];

    info!("Все сервисы запущены");

    // Ожидание сигнала завершения
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Получен сигнал завершения (Ctrl+C)");
        }
        Err(err) => {
            error!("Ошибка при ожидании сигнала завершения: {}", err);
        }
    }

    info!("Завершение работы...");

    // Корректная остановка KeyRepeater с отправкой финальных release событий
    key_repeater.stop_all_repeaters_gracefully().await;

    // Ожидание завершения всех задач (с таймаутом)
    let shutdown_timeout = tokio::time::Duration::from_secs(5);
    let shutdown_result = tokio::time::timeout(shutdown_timeout, async {
        for handle in handles {
            let _ = handle.await;
        }
    }).await;

    match shutdown_result {
        Ok(_) => info!("Все сервисы завершили работу корректно"),
        Err(_) => warn!("Таймаут при завершении сервисов"),
    }

    info!("AHK Rust завершил работу");
    Ok(())
}

fn init_tracing(level: &str) -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))?;

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().compact())
        .init();

    Ok(())
}
