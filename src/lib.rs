use anyhow::Result;
use tracing::info;

pub mod config;
pub mod db;
pub mod device;
pub mod protocols;
pub mod service;
pub mod utils;
pub mod web;

pub use clap::Parser; // Re-export Parser if needed, or just let users dep on clap

/// 设备控制系统参数
#[derive(Parser, Debug, Clone)]
#[command(name = "dm-rust")]
#[command(author = "Device Control Team")]
#[command(version = "1.0.0")]
#[command(about = "工业设备统一控制系统", long_about = None)]
pub struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "config.json")]
    pub config: String,

    /// 日志级别 (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    pub log_level: String,

    /// 安装为Windows服务
    #[arg(long)]
    pub install: bool,

    /// 卸载Windows服务
    #[arg(long)]
    pub uninstall: bool,

    /// 服务控制命令 (start, stop, restart)
    #[arg(short = 's', long)]
    pub service: Option<String>,
}

/// 启动核心应用 (加载配置, DB, WebServer, DeviceController)
pub async fn run_app(config_path: &str, log_level: &str) -> Result<()> {
    info!("设备控制系统启动中...");
    info!("配置文件: {}", config_path);

    // 加载配置
    let cfg = config::load_config_from_file(config_path)?;
    info!("配置加载成功");

    // 初始化日志系统（使用配置文件中的日志配置，命令行参数作为默认值）
    utils::logger::init_logger(cfg.log.as_ref(), log_level)?;

    info!("日志系统初始化完成");

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
        web::WebServer::with_database(
            cfg.clone(),
            config_path.to_string(),
            device_controller.clone(),
            db,
        )
    } else {
        web::WebServer::new(cfg.clone(), config_path.to_string(), device_controller.clone())
    };
    
    // 运行服务器 (这里会阻塞，直到 server 结束)
    if let Err(e) = web_server.run().await {
        tracing::error!("Web服务器错误: {:?}", e);
        return Err(anyhow::anyhow!("Web服务器错误: {:?}", e));
    }

    info!("系统停止");
    Ok(())
}
