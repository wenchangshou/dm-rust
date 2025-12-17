//! Windows 服务管理模块
//!
//! 提供服务安装、卸载、启动、停止、重启功能

#[cfg(windows)]
use anyhow::{Result, Context};
#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::time::Duration;
#[cfg(windows)]
use windows_service::{
    define_windows_service,
    service::{
        ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, 
        ServiceExitCode, ServiceInfo, ServiceStartType, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
    service_manager::{ServiceManager, ServiceManagerAccess},
};

/// 服务名称
pub const SERVICE_NAME: &str = "device-manage";
/// 服务显示名称
pub const SERVICE_DISPLAY_NAME: &str = "Device Manage Service";
/// 服务描述
pub const SERVICE_DESCRIPTION: &str = "工业设备统一控制系统服务";

// 定义Windows服务入口点
#[cfg(windows)]
define_windows_service!(ffi_service_main, service_main);

/// Windows服务主函数
#[cfg(windows)]
fn service_main(_arguments: Vec<OsString>) {
    if let Err(e) = run_service() {
        // 记录错误到Windows事件日志
        eprintln!("服务运行错误: {:?}", e);
    }
}

/// 运行服务的核心逻辑
#[cfg(windows)]
fn run_service() -> Result<()> {
    // 创建事件处理通道
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();

    // 定义服务控制处理函数
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                shutdown_tx.send(()).ok();
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // 注册服务控制处理程序
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // 告诉Windows服务正在启动
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(3),
        process_id: None,
    })?;

    // 启动实际的应用程序
    let runtime = tokio::runtime::Runtime::new()?;
    let app_handle = std::thread::spawn(move || {
        runtime.block_on(async {
            // 加载配置并运行应用
            if let Err(e) = run_application().await {
                eprintln!("应用程序错误: {:?}", e);
            }
        })
    });

    // 告诉Windows服务已经启动
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    // 等待停止信号
    shutdown_rx.recv().ok();

    // 告诉Windows服务正在停止
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::StopPending,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(5),
        process_id: None,
    })?;

    // 等待应用程序线程结束（最多等待5秒）
    let _ = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(5));
        std::process::exit(0);
    });
    
    app_handle.join().ok();

    // 告诉Windows服务已停止
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

/// 实际的应用程序逻辑
#[cfg(windows)]
async fn run_application() -> Result<()> {
    use crate::config;
    use crate::device;
    use crate::db;
    use crate::web;
    use crate::utils;
    use std::fs;

    // 先初始化一个基本的文件日志，确保错误能被记录
    let log_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("logs")))
        .unwrap_or_else(|| std::path::PathBuf::from("logs"));
    
    let _ = fs::create_dir_all(&log_dir);
    let initial_log_file = log_dir.join("service-startup.log");
    
    // 创建初始日志文件
    if let Ok(file) = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&initial_log_file)
    {
        use std::io::Write;
        let mut file = file;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = writeln!(file, "\n=== 服务启动 timestamp: {} ===", timestamp);
    }

    // 使用基本的日志配置初始化
    let basic_log_config = config::LogConfig {
        level: "info".to_string(),
        target: "file".to_string(),
        file: initial_log_file.to_string_lossy().to_string(),
        append: true,
    };
    
    if let Err(e) = utils::logger::init_logger(Some(&basic_log_config), "info") {
        eprintln!("初始日志系统失败: {:?}", e);
    }

    tracing::info!("服务启动，开始加载配置");

    // 尝试多个配置文件路径
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    
    let config_paths = vec![
        "config.json".to_string(),
        exe_dir.as_ref().map(|d| d.join("config.json").to_string_lossy().to_string()).unwrap_or_default(),
    ];

    let mut cfg = None;
    let mut used_path = String::new();
    
    for config_path in &config_paths {
        if config_path.is_empty() {
            continue;
        }
        tracing::info!("尝试加载配置文件: {}", config_path);
        match config::load_config_from_file(config_path) {
            Ok(config) => {
                tracing::info!("配置文件加载成功: {}", config_path);
                cfg = Some(config);
                used_path = config_path.clone();
                break;
            }
            Err(e) => {
                tracing::warn!("配置文件加载失败 {}: {:?}", config_path, e);
            }
        }
    }

    let cfg = match cfg {
        Some(c) => c,
        None => {
            tracing::error!("无法加载配置文件，已尝试路径: {:?}", config_paths);
            return Err(anyhow::anyhow!("无法加载配置文件"));
        }
    };

    // 如果配置文件中有日志配置，重新初始化日志系统
    if let Some(ref log_config) = cfg.log {
        tracing::info!("使用配置文件中的日志设置");
        tracing::info!("日志级别: {}, 目标: {}, 文件: {}", 
            log_config.level, log_config.target, log_config.file);
        
        // 注意：这里不能重新初始化tracing，只记录配置信息
        // 实际的日志配置应该在服务启动前就确定
    }
    
    tracing::info!("设备控制系统服务启动");
    tracing::info!("使用配置文件: {}", used_path);

    // 初始化设备控制器
    let device_controller = device::DeviceController::new(cfg.clone()).await?;
    tracing::info!("设备控制器初始化成功");

    // 初始化数据库（可选）
    let database = if let Some(ref db_config) = cfg.database {
        if db_config.enable {
            match db::Database::new(&db_config.url).await {
                Ok(db) => {
                    tracing::info!("数据库连接成功");
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

    // 启动Web服务器
    tracing::info!("正在启动Web服务器...");
    let web_server = if let Some(db) = database {
        web::WebServer::with_database(cfg.clone(), used_path.clone(), device_controller.clone(), db)
    } else {
        web::WebServer::new(cfg.clone(), used_path.clone(), device_controller.clone())
    };

    web_server.run().await?;

    Ok(())
}

/// 作为Windows服务运行（从服务控制管理器调用）
#[cfg(windows)]
pub fn run_as_service() -> Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
        .context("启动服务调度器失败")?;
    Ok(())
}

/// 安装Windows服务
#[cfg(windows)]
pub fn install_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)
        .context("无法连接到服务管理器")?;

    // 获取当前执行文件路径
    let exe_path = std::env::current_exe()
        .context("无法获取当前执行文件路径")?;

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: exe_path.clone(),
        launch_arguments: vec![],
        dependencies: vec![],
        account_name: None, // 使用 LocalSystem 账户
        account_password: None,
    };

    let service = service_manager
        .create_service(&service_info, ServiceAccess::CHANGE_CONFIG)
        .context("创建服务失败")?;

    // 设置服务描述
    service
        .set_description(SERVICE_DESCRIPTION)
        .context("设置服务描述失败")?;

    println!("✓ 服务安装成功: {}", SERVICE_NAME);
    println!("  显示名称: {}", SERVICE_DISPLAY_NAME);
    println!("  描述: {}", SERVICE_DESCRIPTION);
    println!("  执行文件: {}", exe_path.display());
    println!("\n提示: 使用 'sc start {}' 或 '{} -s start' 启动服务", SERVICE_NAME, std::env::current_exe().unwrap().display());

    Ok(())
}

/// 卸载Windows服务
#[cfg(windows)]
pub fn uninstall_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)
        .context("无法连接到服务管理器")?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager
        .open_service(SERVICE_NAME, service_access)
        .context(format!("无法打开服务 '{}'", SERVICE_NAME))?;

    // 检查服务状态
    let service_status = service
        .query_status()
        .context("查询服务状态失败")?;

    // 如果服务正在运行，先停止它
    if service_status.current_state != ServiceState::Stopped {
        println!("正在停止服务...");
        service
            .stop()
            .context("停止服务失败")?;
        
        // 等待服务停止
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    // 删除服务
    service.delete().context("删除服务失败")?;

    println!("✓ 服务卸载成功: {}", SERVICE_NAME);

    Ok(())
}

/// 启动Windows服务
#[cfg(windows)]
pub fn start_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)
        .context("无法连接到服务管理器")?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::START;
    let service = service_manager
        .open_service(SERVICE_NAME, service_access)
        .context(format!("无法打开服务 '{}', 请先安装服务", SERVICE_NAME))?;

    // 检查服务状态
    let service_status = service.query_status().context("查询服务状态失败")?;

    if service_status.current_state == ServiceState::Running {
        println!("服务已在运行中");
        return Ok(());
    }

    // 启动服务
    service
        .start(&[] as &[&OsString])
        .context("启动服务失败")?;

    println!("✓ 服务启动成功: {}", SERVICE_NAME);
    println!("提示: 使用 'sc query {}' 查看服务状态", SERVICE_NAME);

    Ok(())
}

/// 停止Windows服务
#[cfg(windows)]
pub fn stop_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)
        .context("无法连接到服务管理器")?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP;
    let service = service_manager
        .open_service(SERVICE_NAME, service_access)
        .context(format!("无法打开服务 '{}'", SERVICE_NAME))?;

    // 检查服务状态
    let service_status = service.query_status().context("查询服务状态失败")?;

    if service_status.current_state == ServiceState::Stopped {
        println!("服务已停止");
        return Ok(());
    }

    // 停止服务
    service.stop().context("停止服务失败")?;

    println!("✓ 服务停止成功: {}", SERVICE_NAME);

    Ok(())
}

/// 重启Windows服务
#[cfg(windows)]
pub fn restart_service() -> Result<()> {
    println!("正在重启服务...");
    
    // 先停止服务
    if let Err(e) = stop_service() {
        // 如果服务已经停止，忽略错误
        if !e.to_string().contains("已停止") {
            return Err(e);
        }
    }

    // 等待服务完全停止
    std::thread::sleep(std::time::Duration::from_secs(2));

    // 启动服务
    start_service()?;

    println!("✓ 服务重启成功");

    Ok(())
}

// 非Windows平台的占位实现
#[cfg(not(windows))]
pub fn install_service() -> anyhow::Result<()> {
    anyhow::bail!("服务管理功能仅在 Windows 平台可用")
}

#[cfg(not(windows))]
pub fn uninstall_service() -> anyhow::Result<()> {
    anyhow::bail!("服务管理功能仅在 Windows 平台可用")
}

#[cfg(not(windows))]
pub fn start_service() -> anyhow::Result<()> {
    anyhow::bail!("服务管理功能仅在 Windows 平台可用")
}

#[cfg(not(windows))]
pub fn stop_service() -> anyhow::Result<()> {
    anyhow::bail!("服务管理功能仅在 Windows 平台可用")
}

#[cfg(not(windows))]
pub fn restart_service() -> anyhow::Result<()> {
    anyhow::bail!("服务管理功能仅在 Windows 平台可用")
}

#[cfg(not(windows))]
pub fn run_as_service() -> anyhow::Result<()> {
    anyhow::bail!("服务管理功能仅在 Windows 平台可用")
}
