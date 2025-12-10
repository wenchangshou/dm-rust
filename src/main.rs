use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber;
use clap::Parser;

mod config;
mod device;
mod protocols;
mod utils;
mod web;
mod db;

/// 设备控制系统
#[derive(Parser, Debug)]
#[command(name = "dm-rust")]
#[command(author = "Device Control Team")]
#[command(version = "1.0.0")]
#[command(about = "工业设备统一控制系统", long_about = None)]
struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "config.json")]
    config: String,

    /// 日志级别 (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let args = Args::parse();

    // 解析日志级别
    let log_level = match args.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => {
            eprintln!("警告: 无效的日志级别 '{}', 使用默认值 'info'", args.log_level);
            Level::INFO
        }
    };

    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    info!("设备控制系统启动中...");
    info!("配置文件: {}", args.config);
    info!("日志级别: {:?}", log_level);

    // 加载配置
    let cfg = config::load_config_from_file(&args.config)?;
    info!("配置加载成功");

    // 初始化设备控制器
    let device_controller = device::DeviceController::new(cfg.clone()).await?;
    info!("设备控制器初始化成功");

    // 初始化数据库（可选）
    let database = if let Some(ref db_config) = cfg.database {
        if db_config.enable {
            match db::Database::new(&db_config.url).await {
                Ok(db) => {
                    info!("数据库连接成功");
                    // 如果配置了资源路径，设置到数据库实例
                    let db = if let Some(ref resource_config) = cfg.resource {
                        if resource_config.enable {
                            db.with_resource_path(resource_config.path.clone())
                        } else {
                            db
                        }
                    } else {
                        db
                    };
                    Some(db)
                }
                Err(e) => {
                    tracing::error!("数据库连接失败: {:?}", e);
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    // 启动Web服务器（HTTP控制接口）
    let web_server = if let Some(db) = database {
        web::WebServer::with_database(cfg.clone(), device_controller.clone(), db)
    } else {
        web::WebServer::new(cfg.clone(), device_controller.clone())
    };
    let web_handle = tokio::spawn(async move {
        if let Err(e) = web_server.run().await {
            tracing::error!("Web服务器错误: {:?}", e);
        }
    });

    info!("系统启动完成");

    // 等待Web服务器任务
    if let Err(e) = web_handle.await {
        tracing::error!("Web服务器任务错误: {:?}", e);
    }

    Ok(())
}
