/// MQTT 模拟器管理器
///
/// 统一管理所有 MQTT 模拟器实例
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use super::broker::MqttBroker;
use super::persistence::MqttSimulatorPersistence;
use super::proxy::MqttProxy;
use super::rules::MqttRuleSet;
use super::state::{
    CreateMqttSimulatorRequest, MqttMode, MqttSimulatorInfo, MqttSimulatorState,
    MqttSimulatorStatus,
};

/// MQTT 模拟器实例
enum MqttSimulatorInstance {
    Broker(MqttBroker),
    Proxy(MqttProxy),
}

/// 模拟器包装
struct SimulatorWrapper {
    info: Arc<RwLock<MqttSimulatorInfo>>,
    state: Arc<RwLock<MqttSimulatorState>>,
    rules: Arc<RwLock<MqttRuleSet>>,
    instance: Arc<RwLock<Option<MqttSimulatorInstance>>>,
}

/// MQTT 模拟器管理器
pub struct MqttSimulatorManager {
    simulators: DashMap<String, SimulatorWrapper>,
    config_file: Option<PathBuf>,
}

impl MqttSimulatorManager {
    pub fn new() -> Self {
        Self {
            simulators: DashMap::new(),
            config_file: None,
        }
    }

    /// 创建带持久化的管理器
    pub async fn new_with_persistence(config_file: PathBuf) -> Self {
        let manager = Self {
            simulators: DashMap::new(),
            config_file: Some(config_file.clone()),
        };

        // 尝试加载配置
        if let Err(e) = manager.load_config().await {
            warn!("Failed to load MQTT simulator config: {}", e);
        }

        manager
    }

    /// 加载配置
    async fn load_config(&self) -> Result<(), String> {
        let config_file = self.config_file.as_ref().ok_or("No config file set")?;

        let persistence = MqttSimulatorPersistence::load(config_file).await?;

        info!(
            "Loaded {} MQTT simulators from config",
            persistence.mqtt_simulators.len()
        );

        // 恢复模拟器
        for info in persistence.mqtt_simulators {
            let id = info.id.clone();
            let mode = info.mode.clone();
            let auto_start = info.auto_start;

            let state = Arc::new(RwLock::new(MqttSimulatorState::new(mode)));
            let mut rule_set = MqttRuleSet::new();
            rule_set.rules = info.rules.clone();
            let rules = Arc::new(RwLock::new(rule_set));
            let info_arc = Arc::new(RwLock::new(info));

            let wrapper = SimulatorWrapper {
                info: info_arc,
                state,
                rules,
                instance: Arc::new(RwLock::new(None)),
            };

            self.simulators.insert(id.clone(), wrapper);
            info!("Restored MQTT simulator: {}", id);

            // 如果设置了自动启动，则启动模拟器
            if auto_start {
                info!("Auto-starting MQTT simulator: {}", id);
                if let Err(e) = self.start(&id).await {
                    error!("Failed to auto-start MQTT simulator {}: {}", id, e);
                }
            }
        }

        Ok(())
    }

    /// 保存配置
    async fn save_config(&self) -> Result<(), String> {
        let config_file = match &self.config_file {
            Some(f) => f,
            None => return Ok(()), // 没有配置文件，跳过保存
        };

        let mut mqtt_simulators = Vec::new();
        for entry in self.simulators.iter() {
            let mut info = entry.value().info.read().await.clone();
            let rules = entry.value().rules.read().await;
            info.rules = rules.rules.clone();
            mqtt_simulators.push(info);
        }

        let persistence = MqttSimulatorPersistence { mqtt_simulators };
        persistence.save(config_file).await
    }

    /// 创建模拟器
    pub async fn create(
        &self,
        request: CreateMqttSimulatorRequest,
    ) -> Result<MqttSimulatorInfo, String> {
        info!("Creating MQTT simulator with request: name={}, mode={:?}, port={}, auto_start={:?}",
            request.name, request.mode, request.port, request.auto_start);

        let mut info = MqttSimulatorInfo::new(request.name, request.mode.clone(), request.port);
        info.description = request.description;
        info.bind_addr = request.bind_addr.unwrap_or("0.0.0.0".to_string());
        info.mqtt_versions = request.mqtt_versions.unwrap_or_else(|| vec![super::state::MqttVersion::V4]);
        info.proxy_config = request.proxy_config.clone();
        info.auto_start = request.auto_start.unwrap_or(false);

        let id = info.id.clone();
        info!("Generated simulator ID: {}, bind_addr={}, auto_start={}", id, info.bind_addr, info.auto_start);

        let state = Arc::new(RwLock::new(MqttSimulatorState::new(request.mode.clone())));
        let rules = Arc::new(RwLock::new(MqttRuleSet::new()));
        let info_arc = Arc::new(RwLock::new(info.clone()));

        info!("Created simulator wrapper for ID: {}", id);

        let wrapper = SimulatorWrapper {
            info: info_arc,
            state,
            rules,
            instance: Arc::new(RwLock::new(None)),
        };

        self.simulators.insert(id.clone(), wrapper);
        info!("Inserted simulator {} into manager", id);

        // 自动启动
        if info.auto_start {
            info!("Auto-start enabled for simulator {}, starting now...", id);
            if let Err(e) = self.start(&id).await {
                error!("Auto-start failed for simulator {}: {}", id, e);
                // 继续返回成功，因为模拟器已创建，只是启动失败
            } else {
                info!("Auto-start succeeded for simulator {}", id);
            }
        } else {
            info!("Auto-start disabled for simulator {}, skipping auto-start", id);
        }

        info!("Created MQTT simulator: {}", id);

        // 保存配置
        if let Err(e) = self.save_config().await {
            error!("Failed to save config after creating simulator: {}", e);
        }

        Ok(info)
    }

    /// 获取所有模拟器
    pub async fn list(&self) -> Vec<MqttSimulatorInfo> {
        let mut result = Vec::new();
        for entry in self.simulators.iter() {
            let info = entry.value().info.read().await;
            result.push(info.clone());
        }
        result
    }

    /// 获取单个模拟器
    pub async fn get(&self, id: &str) -> Option<MqttSimulatorInfo> {
        if let Some(wrapper) = self.simulators.get(id) {
            let mut info = wrapper.info.read().await.clone();
            let state = wrapper.state.read().await;
            info.state = state.clone();
            let rules = wrapper.rules.read().await;
            info.rules = rules.rules.clone();
            Some(info)
        } else {
            None
        }
    }

    /// 删除模拟器
    pub async fn delete(&self, id: &str) -> Result<(), String> {
        // 先停止
        self.stop(id).await.ok();

        if self.simulators.remove(id).is_some() {
            info!("Deleted MQTT simulator: {}", id);

            // 保存配置
            if let Err(e) = self.save_config().await {
                error!("Failed to save config after deleting simulator: {}", e);
            }

            Ok(())
        } else {
            Err("Simulator not found".to_string())
        }
    }

    /// 启动模拟器
    pub async fn start(&self, id: &str) -> Result<(), String> {
        info!("Starting MQTT simulator: {}", id);

        let wrapper = self.simulators.get(id).ok_or_else(|| {
            error!("Simulator {} not found in manager", id);
            "Simulator not found".to_string()
        })?;

        info!("Found simulator {} in manager, acquiring instance lock...", id);
        let mut instance_guard = wrapper.instance.write().await;
        if instance_guard.is_some() {
            error!("Simulator {} already running", id);
            return Err("Simulator already running".to_string());
        }

        info!("Reading simulator {} info...", id);
        let info = wrapper.info.read().await;
        let state = wrapper.state.clone();
        let rules = wrapper.rules.clone();

        info!("Simulator {} mode: {:?}, port: {}, bind_addr: {}",
            id, info.mode, info.port, info.bind_addr);

        let instance = match info.mode {
            MqttMode::Broker => {
                info!("Creating MQTT Broker instance for simulator {}", id);
                let mut broker = MqttBroker::new(
                    info.port,
                    info.bind_addr.clone(),
                    info.mqtt_versions.clone(),
                    state,
                    rules,
                );
                info!("Starting MQTT Broker for simulator {}...", id);
                match broker.start().await {
                    Ok(_) => {
                        info!("MQTT Broker started successfully for simulator {}", id);
                        MqttSimulatorInstance::Broker(broker)
                    }
                    Err(e) => {
                        error!("Failed to start MQTT Broker for simulator {}: {}", id, e);
                        return Err(e);
                    }
                }
            }
            MqttMode::Proxy => {
                info!("Creating MQTT Proxy instance for simulator {}", id);
                let proxy_config = info.proxy_config.clone().unwrap_or_default();
                info!("Proxy config: upstream={}:{}", proxy_config.upstream_host, proxy_config.upstream_port);
                let mut proxy = MqttProxy::new(proxy_config, info.port, state, rules);
                info!("Starting MQTT Proxy for simulator {}...", id);
                match proxy.start().await {
                    Ok(_) => {
                        info!("MQTT Proxy started successfully for simulator {}", id);
                        MqttSimulatorInstance::Proxy(proxy)
                    }
                    Err(e) => {
                        error!("Failed to start MQTT Proxy for simulator {}: {}", id, e);
                        return Err(e);
                    }
                }
            }
        };

        info!("Setting instance for simulator {}", id);
        *instance_guard = Some(instance);
        drop(instance_guard);
        drop(info);

        // 更新状态
        info!("Updating status to Running for simulator {}", id);
        let mut info_mut = wrapper.info.write().await;
        info_mut.status = MqttSimulatorStatus::Running;

        info!("Started MQTT simulator: {}", id);
        Ok(())
    }

    /// 停止模拟器
    pub async fn stop(&self, id: &str) -> Result<(), String> {
        let wrapper = self.simulators.get(id).ok_or("Simulator not found")?;

        let mut instance_guard = wrapper.instance.write().await;
        if let Some(instance) = instance_guard.take() {
            match instance {
                MqttSimulatorInstance::Broker(mut broker) => {
                    broker.stop().await?;
                }
                MqttSimulatorInstance::Proxy(mut proxy) => {
                    proxy.stop().await?;
                }
            }
        }
        drop(instance_guard);

        // 更新状态
        let mut info_mut = wrapper.info.write().await;
        info_mut.status = MqttSimulatorStatus::Stopped;

        info!("Stopped MQTT simulator: {}", id);
        Ok(())
    }

    /// 获取报文记录
    pub async fn get_packets(
        &self,
        id: &str,
        limit: usize,
        after_id: Option<u64>,
    ) -> Result<Vec<super::state::MqttPacketRecord>, String> {
        let wrapper = self.simulators.get(id).ok_or("Simulator not found")?;
        let state = wrapper.state.read().await;
        Ok(state
            .get_packets(limit, after_id)
            .into_iter()
            .cloned()
            .collect())
    }

    /// 清空报文记录
    pub async fn clear_packets(&self, id: &str) -> Result<(), String> {
        let wrapper = self.simulators.get(id).ok_or("Simulator not found")?;
        let mut state = wrapper.state.write().await;
        state.clear_packets();
        Ok(())
    }

    /// 添加规则
    pub async fn add_rule(&self, id: &str, rule: super::rules::MqttRule) -> Result<(), String> {
        let wrapper = self.simulators.get(id).ok_or("Simulator not found")?;
        let mut rules = wrapper.rules.write().await;
        rules.add_rule(rule);
        drop(rules);
        // 保存配置
        if let Err(e) = self.save_config().await {
            tracing::error!("Failed to save config after adding rule: {}", e);
        }
        Ok(())
    }

    /// 获取规则列表
    pub async fn get_rules(&self, id: &str) -> Result<Vec<super::rules::MqttRule>, String> {
        let wrapper = self.simulators.get(id).ok_or("Simulator not found")?;
        let rules = wrapper.rules.read().await;
        Ok(rules.rules.clone())
    }

    /// 删除规则
    pub async fn remove_rule(&self, id: &str, rule_id: &str) -> Result<(), String> {
        let wrapper = self.simulators.get(id).ok_or("Simulator not found")?;
        let mut rules = wrapper.rules.write().await;
        rules
            .remove_rule(rule_id)
            .ok_or("Rule not found".to_string())?;
        drop(rules);
        // 保存配置
        if let Err(e) = self.save_config().await {
            tracing::error!("Failed to save config after removing rule: {}", e);
        }
        Ok(())
    }
}

impl Default for MqttSimulatorManager {
    fn default() -> Self {
        Self::new()
    }
}
