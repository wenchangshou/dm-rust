/// 节点管理器 - 负责逻辑设备状态管理
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::broadcast;
use tracing::debug;

use crate::config::NodeConfig;
use super::DeviceEvent;

/// 节点状态
#[derive(Debug, Clone)]
pub struct NodeState {
    pub global_id: u32,
    pub channel_id: u32,
    pub device_id: u32,
    pub category: String,
    pub alias: String,
    pub current_value: Option<i32>,
    pub online: bool,
    pub last_update: Option<std::time::Instant>,
}

/// 节点管理器
pub struct NodeManager {
    nodes: DashMap<u32, NodeConfig>,
    states: DashMap<u32, NodeState>,
    event_tx: broadcast::Sender<DeviceEvent>,
}

impl NodeManager {
    /// 创建节点管理器
    pub fn new(
        node_configs: &[NodeConfig],
        event_tx: broadcast::Sender<DeviceEvent>,
    ) -> Self {
        let nodes = DashMap::new();
        let states = DashMap::new();
        
        for config in node_configs {
            // 保存配置
            nodes.insert(config.global_id, config.clone());
            
            // 初始化状态
            let state = NodeState {
                global_id: config.global_id,
                channel_id: config.channel_id,
                device_id: config.id,
                category: config.category.clone(),
                alias: config.alias.clone(),
                current_value: None,
                online: false,
                last_update: None,
            };
            states.insert(config.global_id, state);
        }
        
        Self { nodes, states, event_tx }
    }
    
    /// 获取节点配置
    pub fn get_node(&self, global_id: u32) -> Option<NodeConfig> {
        self.nodes.get(&global_id).map(|n| n.clone())
    }
    
    /// 获取节点状态
    pub fn get_state(&self, global_id: u32) -> Option<NodeState> {
        self.states.get(&global_id).map(|s| s.clone())
    }
    
    /// 获取所有节点状态
    pub fn get_all_states(&self) -> Vec<(u32, NodeState)> {
        self.states
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }
    
    /// 更新节点值
    pub fn update_value(&self, global_id: u32, new_value: i32) {
        if let Some(mut state) = self.states.get_mut(&global_id) {
            let old_value = state.current_value.unwrap_or(0);
            state.current_value = Some(new_value);
            state.last_update = Some(std::time::Instant::now());
            state.online = true;
            
            // 发送状态变化事件
            if old_value != new_value {
                let _ = self.event_tx.send(DeviceEvent::NodeStateChanged {
                    global_id,
                    old_value,
                    new_value,
                });
                
                debug!("节点 {} 状态更新: {} -> {}", global_id, old_value, new_value);
            }
        }
    }
    
    /// 设置节点在线状态
    pub fn set_online(&self, global_id: u32, online: bool) {
        if let Some(mut state) = self.states.get_mut(&global_id) {
            state.online = online;
        }
    }
    
    /// 通过通道ID和设备ID查找全局ID
    pub fn find_global_id(&self, channel_id: u32, device_id: u32) -> Option<u32> {
        self.nodes
            .iter()
            .find(|entry| {
                let node = entry.value();
                node.channel_id == channel_id && node.id == device_id
            })
            .map(|entry| *entry.key())
    }
    
    /// 获取节点数量
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}
