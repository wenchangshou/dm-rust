use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, debug};

use crate::config::Config;
use crate::utils::{Result, DeviceError};

mod channel_manager;
mod node_manager;
mod task_scheduler;
mod scene_executor;
mod dependency_resolver;

pub use channel_manager::ChannelManager;
pub use node_manager::{NodeManager, NodeState};
pub use task_scheduler::{TaskScheduler, Task, TaskStatus};
pub use scene_executor::SceneExecutor;
pub use dependency_resolver::DependencyResolver;

/// 设备事件
#[derive(Debug, Clone)]
pub enum DeviceEvent {
    /// 节点状态变化
    NodeStateChanged {
        global_id: u32,
        old_value: i32,
        new_value: i32,
    },
    
    /// 通道连接状态变化
    ChannelConnected {
        channel_id: u32,
    },
    
    ChannelDisconnected {
        channel_id: u32,
        reason: String,
    },
    
    /// 任务状态变化
    TaskCompleted {
        task_id: String,
        success: bool,
    },
    
    /// 场景执行状态
    SceneStarted {
        scene_name: String,
    },
    
    SceneCompleted {
        scene_name: String,
        success: bool,
    },
}

/// 设备控制器 - 系统核心协调器
#[derive(Clone)]
pub struct DeviceController {
    /// 通道管理器 - 负责物理设备通信
    channel_manager: Arc<ChannelManager>,
    
    /// 节点管理器 - 负责逻辑设备状态
    node_manager: Arc<NodeManager>,
    
    /// 任务调度器 - 负责依赖任务调度
    task_scheduler: Arc<TaskScheduler>,
    
    /// 场景执行器 - 负责场景编排
    scene_executor: Arc<SceneExecutor>,
    
    /// 依赖解析器 - 负责依赖检查
    dependency_resolver: Arc<DependencyResolver>,
    
    /// 事件广播器
    event_tx: broadcast::Sender<DeviceEvent>,
}

impl DeviceController {
    /// 创建新的设备控制器
    pub async fn new(config: Config) -> Result<Self> {
        info!("初始化设备控制器...");
        
        // 创建事件广播器
        let (event_tx, _) = broadcast::channel(1000);
        
        // 创建通道管理器
        let channel_manager = Arc::new(
            ChannelManager::new(&config.channels, event_tx.clone()).await?
        );
        
        // 创建节点管理器
        let node_manager = Arc::new(
            NodeManager::new(&config.nodes, event_tx.clone())
        );
        
        // 创建依赖解析器
        let dependency_resolver = Arc::new(
            DependencyResolver::new(node_manager.clone())
        );
        
        // 创建任务调度器
        let task_scheduler = Arc::new(
            TaskScheduler::new(
                config.task_settings.clone(),
                channel_manager.clone(),
                node_manager.clone(),
                dependency_resolver.clone(),
                event_tx.clone(),
            ).await
        );
        
        // 创建场景执行器
        let scene_executor = Arc::new(
            SceneExecutor::new(
                config.scenes.clone(),
                channel_manager.clone(),
                node_manager.clone(),
                event_tx.clone(),
            )
        );
        
        info!("设备控制器初始化完成");
        
        Ok(Self {
            channel_manager,
            node_manager,
            task_scheduler,
            scene_executor,
            dependency_resolver,
            event_tx,
        })
    }
    
    /// 订阅设备事件
    pub fn subscribe_events(&self) -> broadcast::Receiver<DeviceEvent> {
        self.event_tx.subscribe()
    }
    
    /// 写入单个节点（带依赖检查）
    pub async fn write_node(&self, global_id: u32, value: i32) -> Result<()> {
        debug!("写入节点 {} = {}", global_id, value);
        
        // 获取节点配置
        let node = self.node_manager.get_node(global_id)
            .ok_or_else(|| DeviceError::DeviceNotFound(format!("节点 {}", global_id)))?;
        
        // 检查是否有依赖
        if let Some(dependencies) = &node.depend {
            // 检查依赖是否满足
            let deps_met = self.dependency_resolver
                .check_dependencies(dependencies)
                .await?;
            
            if !deps_met {
                // 依赖未满足，提交任务到调度器
                info!("节点 {} 依赖未满足，加入任务队列", global_id);
                return self.task_scheduler.submit_task(node, value).await;
            }
            
            // 如果策略是自动，先满足依赖
            if node.depend_strategy.as_deref() == Some("auto") {
                self.dependency_resolver
                    .fulfill_dependencies(dependencies, self)
                    .await?;
            }
        }
        
        // 如果节点有 data_point 配置（Modbus数据点），使用特殊写入逻辑
        if let Some(data_point) = &node.data_point {
            // 应用反向缩放（如果有scale）
            let actual_value = if let Some(scale) = data_point.scale {
                (value as f64 / scale) as i32
            } else {
                value
            };
            
            self.channel_manager.execute(
                node.channel_id,
                "write_typed",
                serde_json::json!({
                    "addr": data_point.addr,
                    "type": data_point.r#type,
                    "value": actual_value
                })
            ).await?;
            
            // 更新节点状态
            self.node_manager.update_value(global_id, value);
            
            return Ok(());
        }
        
        // 普通节点，直接执行写入
        self.execute_write(node.channel_id, node.id, value).await
    }
    
    /// 执行实际的写入操作（内部方法）
    pub(crate) async fn execute_write(&self, channel_id: u32, device_id: u32, value: i32) -> Result<()> {
        self.channel_manager.write(channel_id, device_id, value).await
    }
    
    /// 读取节点当前值
    pub async fn read_node(&self, global_id: u32) -> Result<f64> {
        let node = self.node_manager.get_node(global_id)
            .ok_or_else(|| DeviceError::DeviceNotFound(format!("节点 {}", global_id)))?;
        
        // 如果节点有 data_point 配置（Modbus数据点），使用特殊读取逻辑
        if let Some(data_point) = &node.data_point {
            let result = self.channel_manager.execute(
                node.channel_id,
                "read_typed",
                serde_json::json!({
                    "addr": data_point.addr,
                    "type": data_point.r#type,
                    "use_cache": true
                })
            ).await?;
            
            // 从结果中提取值
            if let Some(value) = result.get("value") {
                let raw_value = if value.is_number() {
                    value.as_f64().unwrap_or(0.0)
                } else {
                    0.0
                };
                
                // 应用缩放比例
                let final_value = if let Some(scale) = data_point.scale {
                    raw_value * scale
                } else {
                    raw_value
                };
                
                // 更新节点状态（存储为整数）
                self.node_manager.update_value(global_id, final_value as i32);
                
                return Ok(final_value);
            }
        }
        
        // 普通节点，使用传统方式
        let value = self.channel_manager.read(node.channel_id, node.id).await?;
        self.node_manager.update_value(global_id, value);
        Ok(value as f64)
    }
    
    /// 获取节点状态
    pub fn get_node_state(&self, global_id: u32) -> Option<NodeState> {
        self.node_manager.get_state(global_id)
    }
    
    /// 获取所有节点状态
    pub fn get_all_node_states(&self) -> Vec<(u32, NodeState)> {
        self.node_manager.get_all_states()
    }
    
    /// 执行场景
    pub async fn execute_scene(&self, scene_name: &str) -> Result<()> {
        info!("执行场景: {}", scene_name);
        self.scene_executor.execute(scene_name, self).await
    }
    
    /// 获取所有通道状态
    pub async fn get_all_channel_status(&self) -> Result<serde_json::Value> {
        self.channel_manager.get_all_status().await
    }
    
    /// 执行通道命令
    pub async fn execute_channel_command(
        &self,
        channel_id: u32,
        command: &str,
        params: serde_json::Value
    ) -> Result<serde_json::Value> {
        self.channel_manager.execute(channel_id, command, params).await
    }
    
    /// 调用通道的自定义方法
    pub async fn call_channel_method(
        &self,
        channel_id: u32,
        method_name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.channel_manager.call_method(channel_id, method_name, args).await
    }
    
    /// 获取通道支持的方法列表
    pub async fn get_channel_methods(&self, channel_id: u32) -> Result<Vec<String>> {
        self.channel_manager.get_channel_methods(channel_id).await
    }
}
