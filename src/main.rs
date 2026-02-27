use anyhow::Result;
use clap::Parser;
use dm_rust::{service, Args, run_app};

#[tokio::main]
async fn main() -> Result<()> {
    // 检查是否作为Windows服务运行（没有命令行参数时）
    #[cfg(windows)]
    {
        let args: Vec<String> = std::env::args().collect();
        if args.len() == 1 {
            if let Ok(_) = service::run_as_service() {
                return Ok(());
            }
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
                eprintln!(
                    "错误: 无效的服务命令 '{}', 支持的命令: start, stop, restart",
                    service_cmd
                );
                std::process::exit(1);
            }
        }
    }

    // 解析日志级别
    let log_level = match args.log_level.to_lowercase().as_str() {
        "trace" | "debug" | "info" | "warn" | "error" => args.log_level.clone(),
        _ => {
            eprintln!(
                "警告: 无效的日志级别 '{}', 使用默认值 'info'",
                args.log_level
            );
            "info".to_string()
        }
    };

    // 运行核心应用
    run_app(&args.config, &log_level).await
}
