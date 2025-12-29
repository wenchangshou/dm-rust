/// TCP 模拟器管理器
///
/// 管理多个 TCP 模拟服务器实例，提供创建、删除、启动、停止等操作。
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::persistence::{PersistedSimulator, PersistenceManager};
use super::protocols::{ModbusValues, ProtocolRegistry};
use super::server::{ServerConfig, TcpSimulatorServer};
use super::state::PacketRecord;
use super::state::{
    ProtocolInfo, SimulatorInfo, SimulatorState, SimulatorStatus, TcpSimulatorConfig,
};
use super::template::{CreateFromTemplateRequest, SimulatorTemplate, TemplateManager};

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
    /// 持久化管理器
    persistence: PersistenceManager,
    /// 模板管理器
    pub template_manager: TemplateManager,
}

impl TcpSimulatorManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            simulators: DashMap::new(),
            registry: ProtocolRegistry::new(),
            persistence: PersistenceManager::with_default_path(),
            template_manager: TemplateManager::new(),
        }
    }

    /// 使用自定义持久化路径创建
    pub fn with_persistence_path(path: &str) -> Self {
        Self {
            simulators: DashMap::new(),
            registry: ProtocolRegistry::new(),
            persistence: PersistenceManager::new(path),
            template_manager: TemplateManager::new(),
        }
    }

    /// 从持久化存储加载所有模拟器
    pub async fn load_from_persistence(&self) -> Result<usize, String> {
        // 加载模板
        if let Err(e) = self.template_manager.load_from_file().await {
            warn!("加载模板失败: {}", e);
        }

        let data = self.persistence.load().await?;
        let mut loaded = 0;

        for persisted in data.simulators {
            // 创建模拟器
            match self
                .create_internal(persisted.config.clone(), persisted.values.clone())
                .await
            {
                Ok(info) => {
                    // 如果需要自动启动
                    if persisted.auto_start {
                        if let Err(e) = self.start(&info.id).await {
                            warn!("自动启动模拟器 {} 失败: {}", info.id, e);
                        }
                    }
                    loaded += 1;
                }
                Err(e) => {
                    error!("加载模拟器 {} 失败: {}", persisted.config.id, e);
                }
            }
        }

        info!("从持久化存储加载了 {} 个模拟器", loaded);
        Ok(loaded)
    }

    // ... 省略原有方法 ...

    // ============ 模板管理 ============

    /// 从模板创建模拟器
    pub async fn create_from_template(
        &self,
        req: CreateFromTemplateRequest,
    ) -> Result<SimulatorInfo, String> {
        let template = self
            .template_manager
            .get(&req.template_id)
            .await
            .ok_or_else(|| format!("模板 '{}' 不存在", req.template_id))?;

        let mut initial_state = serde_json::from_value::<
            std::collections::HashMap<String, serde_json::Value>,
        >(template.values.clone())
        .unwrap_or_default();

        let config = TcpSimulatorConfig {
            id: String::new(), // new_id 会在 create_internal 中生成
            name: req.name,
            protocol: template.protocol,
            bind_addr: req.bind_addr,
            port: req.port,
            initial_state: template.config,
        };

        let simulator_info = self.create_internal(config, initial_state).await?;
        self.persist_simulator(&simulator_info.id, true).await;

        Ok(simulator_info)
    }

    /// 导出为模板
    pub async fn save_as_template(
        &self,
        simulator_id: &str,
        template_name: String,
        description: String,
    ) -> Result<SimulatorTemplate, String> {
        let instance = self
            .simulators
            .get(simulator_id)
            .ok_or_else(|| format!("Simulator '{}' not found", simulator_id))?;

        let server = instance.server.read().await;
        let state = server.get_state().await;

        let request = super::template::CreateTemplateRequest {
            name: template_name,
            description,
            protocol: instance.config.protocol.clone(),
            config: instance.config.initial_state.clone(), // 这里的配置应该用当前的配置？不，initial_state 包含了协议配置
            values: serde_json::to_value(state.values).unwrap_or_default(),
        };

        self.template_manager.create(request).await
    }

    /// 保存模拟器到持久化存储
    async fn persist_simulator(&self, id: &str, auto_start: bool) {
        if let Some(instance) = self.simulators.get(id) {
            let server = instance.server.read().await;
            let state = server.get_state().await;

            let persisted = PersistedSimulator {
                config: instance.config.clone(),
                values: state.values.clone(),
                auto_start,
            };

            if let Err(e) = self.persistence.upsert_simulator(persisted).await {
                error!("持久化模拟器 {} 失败: {}", id, e);
            }
        }
    }

    /// 从持久化存储删除模拟器
    async fn unpersist_simulator(&self, id: &str) {
        if let Err(e) = self.persistence.remove_simulator(id).await {
            error!("从持久化存储删除模拟器 {} 失败: {}", id, e);
        }
    }

    /// 获取支持的协议列表
    pub fn get_protocols(&self) -> Vec<ProtocolInfo> {
        self.registry.get_protocol_infos()
    }

    /// 创建模拟器（内部方法，支持预设的 values）
    async fn create_internal(
        &self,
        mut config: TcpSimulatorConfig,
        initial_values: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<SimulatorInfo, String> {
        // 生成 ID
        if config.id.is_empty() {
            config.id = format!(
                "sim_{}",
                Uuid::new_v4().to_string().split('-').next().unwrap()
            );
        }

        // 检查 ID 是否重复
        if self.simulators.contains_key(&config.id) {
            return Err(format!("Simulator with ID '{}' already exists", config.id));
        }

        // 检查端口是否被占用
        for entry in self.simulators.iter() {
            let instance = entry.value();
            if instance.config.port == config.port && instance.config.bind_addr == config.bind_addr
            {
                return Err(format!(
                    "Port {} is already in use by simulator '{}'",
                    config.port, instance.config.id
                ));
            }
        }

        // 检查协议是否存在
        let handler = self.registry.create(&config.protocol).ok_or_else(|| {
            format!(
                "Unknown protocol: '{}'. Available: {:?}",
                config.protocol,
                self.registry.list_protocols()
            )
        })?;

        // 创建服务器配置
        let server_config = ServerConfig {
            bind_addr: config.bind_addr.clone(),
            port: config.port,
            ..Default::default()
        };

        // 创建初始状态
        let mut initial_state = SimulatorState::from_initial(&config.initial_state);
        // 合并预设的 values（从持久化加载）
        for (key, value) in initial_values {
            initial_state.values.insert(key, value);
        }

        // 创建服务器
        let server = TcpSimulatorServer::new(server_config, handler, initial_state.clone());

        // 保存实例
        let instance = Arc::new(SimulatorInstance {
            config: config.clone(),
            server: RwLock::new(server),
        });

        self.simulators.insert(config.id.clone(), instance);

        info!(
            "模拟器已创建: {} ({}:{})",
            config.id, config.bind_addr, config.port
        );

        Ok(SimulatorInfo::new(
            &config,
            SimulatorStatus::Stopped,
            initial_state,
        ))
    }

    /// 创建模拟器
    pub async fn create(&self, config: TcpSimulatorConfig) -> Result<SimulatorInfo, String> {
        let info = self
            .create_internal(config, std::collections::HashMap::new())
            .await?;

        // 持久化保存
        self.persist_simulator(&info.id, true).await;

        Ok(info)
    }

    /// 删除模拟器
    pub async fn delete(&self, id: &str) -> Result<(), String> {
        // 获取实例
        let instance = self
            .simulators
            .remove(id)
            .map(|(_, v)| v)
            .ok_or_else(|| format!("Simulator '{}' not found", id))?;

        // 停止服务器
        {
            let mut server = instance.server.write().await;
            let _ = server.stop().await;
        }

        // 从持久化存储删除
        self.unpersist_simulator(id).await;

        info!("模拟器已删除: {}", id);
        Ok(())
    }

    /// 启动模拟器
    pub async fn start(&self, id: &str) -> Result<(), String> {
        let instance = self
            .simulators
            .get(id)
            .ok_or_else(|| format!("Simulator '{}' not found", id))?;

        let mut server = instance.server.write().await;
        server.start().await?;

        info!("模拟器已启动: {}", id);
        Ok(())
    }

    /// 停止模拟器
    pub async fn stop(&self, id: &str) -> Result<(), String> {
        let instance = self
            .simulators
            .get(id)
            .ok_or_else(|| format!("Simulator '{}' not found", id))?;

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
    pub async fn update_state(
        &self,
        id: &str,
        online: Option<bool>,
        fault: Option<String>,
    ) -> Result<SimulatorInfo, String> {
        let instance = self
            .simulators
            .get(id)
            .ok_or_else(|| format!("Simulator '{}' not found", id))?;

        let server = instance.server.read().await;

        server
            .update_state(|state| {
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
            })
            .await;

        let status = server.get_status().await;
        let state = server.get_state().await;
        Ok(SimulatorInfo::new(&instance.config, status, state))
    }

    /// 触发故障
    pub async fn trigger_fault(&self, id: &str, fault_type: &str) -> Result<SimulatorInfo, String> {
        self.update_state(id, None, Some(fault_type.to_string()))
            .await
    }

    /// 清除故障
    pub async fn clear_fault(&self, id: &str) -> Result<SimulatorInfo, String> {
        self.update_state(id, None, Some(String::new())).await
    }

    /// 设置在线状态
    pub async fn set_online(&self, id: &str, online: bool) -> Result<SimulatorInfo, String> {
        self.update_state(id, Some(online), None).await
    }

    /// 更新 Modbus 状态
    ///
    /// 通用方法，允许传入闭包来修改 ModbusValues
    pub async fn update_modbus_state<F>(&self, id: &str, f: F) -> Result<SimulatorInfo, String>
    where
        F: FnOnce(&mut ModbusValues) -> Result<(), String>,
    {
        // 先执行更新
        {
            let instance = self
                .simulators
                .get(id)
                .ok_or_else(|| format!("Simulator '{}' not found", id))?;

            let server = instance.server.read().await;

            // 获取当前状态并更新
            let mut update_result = Ok(());
            server
                .update_state(|state| {
                    let mut values = ModbusValues::from_state(state);
                    update_result = f(&mut values);
                    if update_result.is_ok() {
                        values.save_to_state(state);
                    }
                })
                .await;

            update_result?;
        }

        // 持久化保存状态变更
        self.persist_simulator(id, true).await;

        // 返回最新状态
        self.get(id)
            .await
            .ok_or_else(|| format!("Simulator '{}' not found", id))
    }

    // ============ 报文监控方法 ============

    /// 获取报文列表
    pub async fn get_packets(
        &self,
        id: &str,
        after_id: Option<u64>,
        limit: Option<usize>,
    ) -> Result<Vec<PacketRecord>, String> {
        let instance = self
            .simulators
            .get(id)
            .ok_or_else(|| format!("Simulator '{}' not found", id))?;

        let server = instance.server.read().await;
        let state = server.get_state().await;

        let packets = if let Some(after) = after_id {
            state.packet_monitor.get_after(after)
        } else if let Some(n) = limit {
            state.packet_monitor.get_recent(n)
        } else {
            state.packet_monitor.get_packets()
        };

        Ok(packets)
    }

    /// 清空报文记录
    pub async fn clear_packets(&self, id: &str) -> Result<(), String> {
        let instance = self
            .simulators
            .get(id)
            .ok_or_else(|| format!("Simulator '{}' not found", id))?;

        let server = instance.server.read().await;
        server
            .update_state(|state| {
                state.packet_monitor.clear();
            })
            .await;

        Ok(())
    }

    /// 设置报文监控开关
    pub async fn set_packet_monitor_enabled(
        &self,
        id: &str,
        enabled: bool,
    ) -> Result<SimulatorInfo, String> {
        let instance = self
            .simulators
            .get(id)
            .ok_or_else(|| format!("Simulator '{}' not found", id))?;

        let server = instance.server.read().await;
        server
            .update_state(|state| {
                state.packet_monitor.set_enabled(enabled);
            })
            .await;

        let status = server.get_status().await;
        let state = server.get_state().await;
        Ok(SimulatorInfo::new(&instance.config, status, state))
    }

    /// 设置最大报文记录数
    pub async fn set_packet_monitor_max(
        &self,
        id: &str,
        max: usize,
    ) -> Result<SimulatorInfo, String> {
        let instance = self
            .simulators
            .get(id)
            .ok_or_else(|| format!("Simulator '{}' not found", id))?;

        let server = instance.server.read().await;
        server
            .update_state(|state| {
                state.packet_monitor.set_max_packets(max);
            })
            .await;

        let status = server.get_status().await;
        let state = server.get_state().await;
        Ok(SimulatorInfo::new(&instance.config, status, state))
    }

    // ============ 客户端连接管理 ============

    /// 断开指定客户端连接
    ///
    /// 注意：当前实现仅从追踪列表中移除客户端。
    /// 真正的 TCP 连接断开需要扩展服务器架构来支持向特定连接发送关闭信号。
    pub async fn disconnect_client(&self, id: &str, client_id: &str) -> Result<(), String> {
        let instance = self
            .simulators
            .get(id)
            .ok_or_else(|| format!("Simulator '{}' not found", id))?;

        let server = instance.server.read().await;

        // 检查客户端是否存在并移除
        let mut found = false;
        server
            .update_state(|state| {
                if state.clients.contains_key(client_id) {
                    state.clients.remove(client_id);
                    state.stats.record_disconnection();
                    found = true;
                }
            })
            .await;

        if found {
            info!("客户端 {} 已从模拟器 {} 断开", client_id, id);
            Ok(())
        } else {
            Err(format!("客户端 '{}' 不存在", client_id))
        }
    }

    // ============ Debug 模式 ============

    /// 设置 Debug 模式
    pub async fn set_debug_mode(&self, id: &str, enabled: bool) -> Result<SimulatorInfo, String> {
        let instance = self
            .simulators
            .get(id)
            .ok_or_else(|| format!("Simulator '{}' not found", id))?;

        let server = instance.server.read().await;
        let simulator_id = id.to_string();
        server
            .update_state(|state| {
                state.packet_monitor.set_debug_mode(enabled, &simulator_id);
            })
            .await;

        let status = server.get_status().await;
        let state = server.get_state().await;

        if enabled {
            if let Some(path) = state.packet_monitor.get_debug_log_path() {
                info!("模拟器 {} Debug 模式已启用，日志: {}", id, path);
            }
        } else {
            info!("模拟器 {} Debug 模式已关闭", id);
        }

        Ok(SimulatorInfo::new(&instance.config, status, state))
    }

    /// 获取 Debug 日志内容
    pub async fn get_debug_log(&self, id: &str) -> Result<String, String> {
        let instance = self
            .simulators
            .get(id)
            .ok_or_else(|| format!("Simulator '{}' not found", id))?;

        let server = instance.server.read().await;
        let state = server.get_state().await;

        if let Some(path) = state.packet_monitor.get_debug_log_path() {
            std::fs::read_to_string(path).map_err(|e| format!("读取日志失败: {}", e))
        } else {
            Err("Debug 模式未启用或日志文件不存在".to_string())
        }
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
