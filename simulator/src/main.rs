//! 协议模拟器服务
//!
//! 独立运行的 TCP/MQTT 协议模拟器 Web 服务

use anyhow::Result;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod mqtt;
mod tcp;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .init();

    info!("协议模拟器服务启动中...");

    // 创建管理器 - 使用同一个配置文件
    let config_file = std::path::PathBuf::from("simulators.json");

    let tcp_manager = Arc::new(tcp::TcpSimulatorManager::new());
    let mqtt_manager = Arc::new(
        mqtt::MqttSimulatorManager::new_with_persistence(config_file.clone())
        .await
    );

    // 加载持久化数据
    if let Ok(count) = tcp_manager.load_from_persistence().await {
        if count > 0 {
            info!("已加载 {} 个 TCP 模拟器", count);
        }
    }

    // 构建路由
    let app = Router::new()
        .route("/", get(|| async { "Protocol Simulator Service" }))
        .nest("/api/tcp-simulator", web::tcp_routes(tcp_manager))
        .nest("/api/mqtt-simulator", web::mqtt_routes(mqtt_manager))
        .layer(CorsLayer::permissive());

    // 启动服务器
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3030);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("服务监听于 http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
