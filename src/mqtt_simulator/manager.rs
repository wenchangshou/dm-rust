/// MQTT 模拟器管理器
///
/// 统一管理所有 MQTT 模拟器实例
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use super::broker::MqttBroker;
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
}

impl MqttSimulatorManager {
    pub fn new() -> Self {
        Self {
            simulators: DashMap::new(),
        }
    }

    /// 创建模拟器
    pub async fn create(
        &self,
        request: CreateMqttSimulatorRequest,
    ) -> Result<MqttSimulatorInfo, String> {
        let mut info = MqttSimulatorInfo::new(request.name, request.mode.clone(), request.port);
        info.description = request.description;
        info.bind_addr = request.bind_addr.unwrap_or("0.0.0.0".to_string());
        info.proxy_config = request.proxy_config;
        info.auto_start = request.auto_start.unwrap_or(false);

        let id = info.id.clone();
        let state = Arc::new(RwLock::new(MqttSimulatorState::new(request.mode)));
        let rules = Arc::new(RwLock::new(MqttRuleSet::new()));
        let info_arc = Arc::new(RwLock::new(info.clone()));

        let wrapper = SimulatorWrapper {
            info: info_arc,
            state,
            rules,
            instance: Arc::new(RwLock::new(None)),
        };

        self.simulators.insert(id.clone(), wrapper);

        // 自动启动
        if info.auto_start {
            if let Err(e) = self.start(&id).await {
                error!("Auto-start failed: {}", e);
            }
        }

        info!("Created MQTT simulator: {}", id);
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
            Ok(())
        } else {
            Err("Simulator not found".to_string())
        }
    }

    /// 启动模拟器
    pub async fn start(&self, id: &str) -> Result<(), String> {
        let wrapper = self.simulators.get(id).ok_or("Simulator not found")?;

        let mut instance_guard = wrapper.instance.write().await;
        if instance_guard.is_some() {
            return Err("Simulator already running".to_string());
        }

        let info = wrapper.info.read().await;
        let state = wrapper.state.clone();
        let rules = wrapper.rules.clone();

        let instance = match info.mode {
            MqttMode::Broker => {
                let mut broker = MqttBroker::new(info.port, info.bind_addr.clone(), state, rules);
                broker.start().await?;
                MqttSimulatorInstance::Broker(broker)
            }
            MqttMode::Proxy => {
                let proxy_config = info.proxy_config.clone().unwrap_or_default();
                let mut proxy = MqttProxy::new(proxy_config, info.port, state, rules);
                proxy.start().await?;
                MqttSimulatorInstance::Proxy(proxy)
            }
        };

        *instance_guard = Some(instance);
        drop(instance_guard);
        drop(info);

        // 更新状态
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
        Ok(())
    }
}

impl Default for MqttSimulatorManager {
    fn default() -> Self {
        Self::new()
    }
}
