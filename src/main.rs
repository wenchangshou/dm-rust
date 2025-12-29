use anyhow::Result;
use tracing::info;
use clap::Parser;

mod config;
mod device;
mod protocols;
mod utils;
mod web;
mod db;
mod service;
mod tcp_simulator;

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

    /// 安装为Windows服务
    #[arg(long)]
    install: bool,

    /// 卸载Windows服务
    #[arg(long)]
    uninstall: bool,

    /// 服务控制命令 (start, stop, restart)
    #[arg(short = 's', long)]
    service: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 检查是否作为Windows服务运行（没有命令行参数时）
    #[cfg(windows)]
    {
        let args: Vec<String> = std::env::args().collect();
        // 如果没有参数或只有可执行文件名，尝试作为服务运行
        if args.len() == 1 {
            // 尝试作为服务运行
            if let Ok(_) = service::run_as_service() {
                return Ok(());
            }
            // 如果不是作为服务运行，继续正常流程
        }
    }

    // 解析命令行参数
    let args = Args::parse();

    // 处理服务管理命令
    if args.install {
        return service::install_service();
    }

    if args.uninstall {
        return service::uninstall_service();
    }

    if let Some(service_cmd) = args.service {
        match service_cmd.to_lowercase().as_str() {
            "start" => return service::start_service(),
            "stop" => return service::stop_service(),
            "restart" => return service::restart_service(),
            _ => {
                eprintln!("错误: 无效的服务命令 '{}', 支持的命令: start, stop, restart", service_cmd);
                std::process::exit(1);
            }
        }
    }

    // 解析日志级别
    let log_level = match args.log_level.to_lowercase().as_str() {
        "trace" | "debug" | "info" | "warn" | "error" => args.log_level.clone(),
        _ => {
            eprintln!("警告: 无效的日志级别 '{}', 使用默认值 'info'", args.log_level);
            "info".to_string()
        }
    };

    info!("设备控制系统启动中...");
    info!("配置文件: {}", args.config);

    // 加载配置
    let cfg = config::load_config_from_file(&args.config)?;
    info!("配置加载成功");

    // 初始化日志系统（使用配置文件中的日志配置，命令行参数作为默认值）
    utils::logger::init_logger(cfg.log.as_ref(), &log_level)?;
    
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
        web::WebServer::with_database(cfg.clone(), args.config.clone(), device_controller.clone(), db)
    } else {
        web::WebServer::new(cfg.clone(), args.config.clone(), device_controller.clone())
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
