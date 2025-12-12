//! 日志系统初始化模块

use anyhow::Result;
use tracing::{info, warn, error, debug, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use std::path::Path;

use crate::config::LogConfig;

/// 日志辅助函数
pub struct Logger;

impl Logger {
    pub fn info(msg: &str) {
        info!("{}", msg);
    }

    pub fn warn(msg: &str) {
        warn!("{}", msg);
    }

    pub fn error(msg: &str) {
        error!("{}", msg);
    }

    pub fn debug(msg: &str) {
        debug!("{}", msg);
    }
}

/// 初始化日志系统
pub fn init_logger(log_config: Option<&LogConfig>, default_level: &str) -> Result<()> {
    let config = log_config.cloned().unwrap_or_default();
    
    // 解析日志级别
    let level = parse_log_level(&config.level, default_level);
    
    // 根据目标类型初始化日志
    match config.target.to_lowercase().as_str() {
        "file" => init_file_logger(&config.file, level, config.append)?,
        "both" => init_both_logger(&config.file, level, config.append)?,
        _ => init_console_logger(level)?,
    }
    
    Ok(())
}

/// 初始化控制台日志
fn init_console_logger(level: Level) -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(level.into()))
        .init();
    Ok(())
}

/// 初始化文件日志
fn init_file_logger(file_path: &str, level: Level, append: bool) -> Result<()> {
    // 确保日志目录存在
    if let Some(parent) = Path::new(file_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let file = if append {
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(file_path)?
    } else {
        std::fs::File::create(file_path)?
    };
    
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::sync::Arc::new(file)))
        .with(EnvFilter::from_default_env().add_directive(level.into()))
        .init();
    
    Ok(())
}

/// 初始化控制台和文件双输出日志
fn init_both_logger(file_path: &str, level: Level, append: bool) -> Result<()> {
    // 确保日志目录存在
    if let Some(parent) = Path::new(file_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let file = if append {
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(file_path)?
    } else {
        std::fs::File::create(file_path)?
    };
    
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            fmt::layer()
                .with_writer(std::sync::Arc::new(file))
                .with_ansi(false)
        )
        .with(EnvFilter::from_default_env().add_directive(level.into()))
        .init();
    
    Ok(())
}

/// 解析日志级别
fn parse_log_level(level_str: &str, default: &str) -> Level {
    let level_str = if level_str.is_empty() { default } else { level_str };
    
    match level_str.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => {
            eprintln!("警告: 无效的日志级别 '{}', 使用默认值 '{}'", level_str, default);
            match default.to_lowercase().as_str() {
                "trace" => Level::TRACE,
                "debug" => Level::DEBUG,
                "warn" => Level::WARN,
                "error" => Level::ERROR,
                _ => Level::INFO,
            }
        }
    }
}
