/// 场景执行器 - 负责场景的编排和执行
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};
use tracing::{info, warn};

use crate::config::SceneConfig;
use crate::utils::{Result, DeviceError};
use super::{ChannelManager, NodeManager, DeviceEvent, DeviceController};

/// 场景执行状态
#[derive(Debug, Clone)]
pub struct SceneExecutionStatus {
    /// 是否正在执行场景
    pub is_executing: bool,
    /// 当前执行的场景名称（如果有）
    pub current_scene: Option<String>,
}

/// 场景执行器
pub struct SceneExecutor {
    scenes: Vec<SceneConfig>,
    channel_manager: Arc<ChannelManager>,
    node_manager: Arc<NodeManager>,
    event_tx: broadcast::Sender<DeviceEvent>,
    /// 当前执行的场景名称（用于互斥控制）
    current_executing: Arc<Mutex<Option<String>>>,
}

impl SceneExecutor {
    /// 创建场景执行器
    pub fn new(
        scenes: Vec<SceneConfig>,
        channel_manager: Arc<ChannelManager>,
        node_manager: Arc<NodeManager>,
        event_tx: broadcast::Sender<DeviceEvent>,
    ) -> Self {
        info!("场景执行器初始化，共 {} 个场景", scenes.len());
        Self {
            scenes,
            channel_manager,
            node_manager,
            event_tx,
            current_executing: Arc::new(Mutex::new(None)),
        }
    }
    
    /// 执行场景（异步启动，立即返回）
    pub async fn execute(
        &self,
        scene_name: &str,
        controller: &DeviceController,
    ) -> Result<()> {
        // 查找场景
        let scene = self.scenes.iter()
            .find(|s| s.name == scene_name)
            .ok_or_else(|| DeviceError::Other(format!("场景 '{}' 不存在", scene_name)))?;

        // 检查是否已有场景正在执行
        let mut current = self.current_executing.lock().await;
        if let Some(ref executing_scene) = *current {
            return Err(DeviceError::Other(
                format!("场景 '{}' 正在执行中，无法同时执行场景 '{}'", executing_scene, scene_name)
            ));
        }

        // 标记当前场景为正在执行
        *current = Some(scene_name.to_string());
        drop(current); // 释放锁

        info!("开始执行场景: {}", scene_name);

        // 克隆需要的数据用于异步任务
        let scene_name_str = scene_name.to_string();
        let scene_nodes = scene.nodes.clone();
        let controller_clone = controller.clone();
        let current_executing = self.current_executing.clone();
        let event_tx = self.event_tx.clone();

        // 发送场景开始事件
        let _ = event_tx.send(DeviceEvent::SceneStarted {
            scene_name: scene_name_str.clone(),
        });

        // 在后台异步执行场景
        tokio::spawn(async move {
            let mut success = true;

            // 按顺序执行场景中的所有成员
            for member in &scene_nodes {
                // 延迟执行（如果有配置）
                if let Some(delay) = member.delay {
                    tokio::time::sleep(Duration::from_millis(delay as u64)).await;
                }

                // 执行写入
                match controller_clone.write_node(member.id, member.value).await {
                    Ok(_) => {
                        info!("场景 '{}': 节点 {} 设置为 {}", scene_name_str, member.id, member.value);
                    }
                    Err(e) => {
                        warn!("场景 '{}': 节点 {} 设置失败: {:?}", scene_name_str, member.id, e);
                        success = false;
                    }
                }
            }

            // 清除执行状态
            let mut current = current_executing.lock().await;
            *current = None;
            drop(current);

            // 发送场景完成事件
            let _ = event_tx.send(DeviceEvent::SceneCompleted {
                scene_name: scene_name_str.clone(),
                success,
            });

            if success {
                info!("场景 '{}' 执行成功", scene_name_str);
            } else {
                warn!("场景 '{}' 执行部分失败", scene_name_str);
            }
        });

        // 立即返回，不等待场景执行完成
        Ok(())
    }
    
    /// 获取所有场景名称
    pub fn list_scenes(&self) -> Vec<String> {
        self.scenes.iter().map(|s| s.name.clone()).collect()
    }

    /// 获取场景详情
    pub fn get_scene(&self, scene_name: &str) -> Option<&SceneConfig> {
        self.scenes.iter().find(|s| s.name == scene_name)
    }

    /// 获取当前场景执行状态
    pub async fn get_execution_status(&self) -> SceneExecutionStatus {
        let current = self.current_executing.lock().await;
        SceneExecutionStatus {
            is_executing: current.is_some(),
            current_scene: current.clone(),
        }
    }
}
