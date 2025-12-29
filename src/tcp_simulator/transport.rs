use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::state::{SimulatorState, SimulatorStatus};

/// 模拟器服务器接口
///
/// 定义了不同传输协议（TCP/UDP）服务器的通用行为。
#[async_trait]
pub trait SimulatorServer: Send + Sync {
    /// 启动服务器
    async fn start(&mut self) -> Result<(), String>;

    /// 停止服务器
    async fn stop(&mut self) -> Result<(), String>;

    /// 获取状态引用
    fn get_state_ref(&self) -> Arc<RwLock<SimulatorState>>;

    /// 获取运行状态引用
    fn get_status_ref(&self) -> Arc<RwLock<SimulatorStatus>>;

    /// 获取当前状态快照
    async fn get_state(&self) -> SimulatorState {
        self.get_state_ref().read().await.clone()
    }

    /// 获取当前运行状态
    async fn get_status(&self) -> SimulatorStatus {
        *self.get_status_ref().read().await
    }
}
