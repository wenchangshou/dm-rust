/// TCP 模拟器管理器
///
/// 管理多个 TCP 模拟服务器实例，提供创建、删除、启动、停止等操作。

use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use super::protocols::ProtocolRegistry;
use super::server::{ServerConfig, TcpSimulatorServer};
use super::state::{ProtocolInfo, SimulatorInfo, SimulatorState, SimulatorStatus, TcpSimulatorConfig};

/// 模拟器实例
struct SimulatorInstance {
    config: TcpSimulatorConfig,
    server: RwLock<TcpSimulatorServer>,
}

/// TCP 模拟器管理器
pub struct TcpSimulatorManager {
    /// 模拟器实例映射
    simulators: DashMap<String, Arc<SimulatorInstance>>,
    /// 协议注册表
    registry: ProtocolRegistry,
}

impl TcpSimulatorManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            simulators: DashMap::new(),
            registry: ProtocolRegistry::new(),
        }
    }

    /// 获取支持的协议列表
    pub fn get_protocols(&self) -> Vec<ProtocolInfo> {
        self.registry.get_protocol_infos()
    }

    /// 创建模拟器
    pub async fn create(&self, mut config: TcpSimulatorConfig) -> Result<SimulatorInfo, String> {
        // 生成 ID
        if config.id.is_empty() {
            config.id = format!("sim_{}", Uuid::new_v4().to_string().split('-').next().unwrap());
        }

        // 检查 ID 是否重复
        if self.simulators.contains_key(&config.id) {
            return Err(format!("Simulator with ID '{}' already exists", config.id));
        }

        // 检查端口是否被占用
        for entry in self.simulators.iter() {
            let instance = entry.value();
            if instance.config.port == config.port && instance.config.bind_addr == config.bind_addr {
                return Err(format!(
                    "Port {} is already in use by simulator '{}'",
                    config.port, instance.config.id
                ));
            }
        }

        // 检查协议是否存在
        let handler = self.registry.create(&config.protocol).ok_or_else(|| {
            format!("Unknown protocol: '{}'. Available: {:?}", config.protocol, self.registry.list_protocols())
        })?;

        // 创建服务器配置
        let server_config = ServerConfig {
            bind_addr: config.bind_addr.clone(),
            port: config.port,
            ..Default::default()
        };

        // 创建初始状态
        let initial_state = SimulatorState::from_initial(&config.initial_state);

        // 创建服务器
        let server = TcpSimulatorServer::new(server_config, handler, initial_state.clone());

        // 保存实例
        let instance = Arc::new(SimulatorInstance {
            config: config.clone(),
            server: RwLock::new(server),
        });

        self.simulators.insert(config.id.clone(), instance);

        info!("模拟器已创建: {} ({}:{})", config.id, config.bind_addr, config.port);

        Ok(SimulatorInfo::new(&config, SimulatorStatus::Stopped, initial_state))
    }

    /// 删除模拟器
    pub async fn delete(&self, id: &str) -> Result<(), String> {
        // 获取实例
        let instance = self.simulators.remove(id).map(|(_, v)| v).ok_or_else(|| {
            format!("Simulator '{}' not found", id)
        })?;

        // 停止服务器
        {
            let mut server = instance.server.write().await;
            let _ = server.stop().await;
        }

        info!("模拟器已删除: {}", id);
        Ok(())
    }

    /// 启动模拟器
    pub async fn start(&self, id: &str) -> Result<(), String> {
        let instance = self.simulators.get(id).ok_or_else(|| {
            format!("Simulator '{}' not found", id)
        })?;

        let mut server = instance.server.write().await;
        server.start().await?;

        info!("模拟器已启动: {}", id);
        Ok(())
    }

    /// 停止模拟器
    pub async fn stop(&self, id: &str) -> Result<(), String> {
        let instance = self.simulators.get(id).ok_or_else(|| {
            format!("Simulator '{}' not found", id)
        })?;

        let mut server = instance.server.write().await;
        server.stop().await?;

        info!("模拟器已停止: {}", id);
        Ok(())
    }

    /// 获取模拟器信息
    pub async fn get(&self, id: &str) -> Option<SimulatorInfo> {
        let instance = self.simulators.get(id)?;
        let server = instance.server.read().await;
        let status = server.get_status().await;
        let state = server.get_state().await;
        Some(SimulatorInfo::new(&instance.config, status, state))
    }

    /// 列出所有模拟器
    pub async fn list(&self) -> Vec<SimulatorInfo> {
        let mut result = Vec::new();

        for entry in self.simulators.iter() {
            let instance = entry.value();
            let server = instance.server.read().await;
            let status = server.get_status().await;
            let state = server.get_state().await;
            result.push(SimulatorInfo::new(&instance.config, status, state));
        }

        result
    }

    /// 更新模拟器状态
    pub async fn update_state(&self, id: &str, online: Option<bool>, fault: Option<String>) -> Result<SimulatorInfo, String> {
        let instance = self.simulators.get(id).ok_or_else(|| {
            format!("Simulator '{}' not found", id)
        })?;

        let server = instance.server.read().await;

        server.update_state(|state| {
            if let Some(online) = online {
                state.online = online;
            }
            if let Some(fault) = fault {
                if fault.is_empty() {
                    state.clear_fault();
                } else {
                    state.set_fault(&fault);
                }
            }
        }).await;

        let status = server.get_status().await;
        let state = server.get_state().await;
        Ok(SimulatorInfo::new(&instance.config, status, state))
    }

    /// 触发故障
    pub async fn trigger_fault(&self, id: &str, fault_type: &str) -> Result<SimulatorInfo, String> {
        self.update_state(id, None, Some(fault_type.to_string())).await
    }

    /// 清除故障
    pub async fn clear_fault(&self, id: &str) -> Result<SimulatorInfo, String> {
        self.update_state(id, None, Some(String::new())).await
    }

    /// 设置在线状态
    pub async fn set_online(&self, id: &str, online: bool) -> Result<SimulatorInfo, String> {
        self.update_state(id, Some(online), None).await
    }

    /// 停止所有模拟器
    pub async fn stop_all(&self) {
        for entry in self.simulators.iter() {
            let mut server = entry.value().server.write().await;
            if let Err(e) = server.stop().await {
                warn!("停止模拟器 {} 失败: {}", entry.key(), e);
            }
        }
        info!("所有模拟器已停止");
    }

    /// 获取模拟器数量
    pub fn count(&self) -> usize {
        self.simulators.len()
    }
}

impl Default for TcpSimulatorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TcpSimulatorManager {
    fn drop(&mut self) {
        // 注意：这里不能使用 async，所以只能发送停止信号
        // 实际清理由各个服务器的 Drop 处理
        for entry in self.simulators.iter() {
            // Drop 会自动发送停止信号
            drop(entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_simulator() {
        let manager = TcpSimulatorManager::new();

        let config = TcpSimulatorConfig {
            id: String::new(),
            name: "Test".to_string(),
            protocol: "scene_loader".to_string(),
            bind_addr: "127.0.0.1".to_string(),
            port: 15000,
            initial_state: serde_json::json!({}),
        };

        let result = manager.create(config).await;
        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(!info.id.is_empty());
        assert_eq!(info.status, SimulatorStatus::Stopped);
    }

    #[tokio::test]
    async fn test_unknown_protocol() {
        let manager = TcpSimulatorManager::new();

        let config = TcpSimulatorConfig {
            id: String::new(),
            name: "Test".to_string(),
            protocol: "unknown".to_string(),
            bind_addr: "127.0.0.1".to_string(),
            port: 15001,
            initial_state: serde_json::json!({}),
        };

        let result = manager.create(config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown protocol"));
    }

    #[tokio::test]
    async fn test_list_protocols() {
        let manager = TcpSimulatorManager::new();
        let protocols = manager.get_protocols();

        assert!(!protocols.is_empty());
        assert!(protocols.iter().any(|p| p.name == "scene_loader"));
    }
}
