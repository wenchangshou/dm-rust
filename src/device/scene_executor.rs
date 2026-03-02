/// 场景执行器 - 负责场景的编排和执行
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};
use tracing::{info, warn};

use super::{ChannelManager, DeviceController, DeviceEvent, NodeManager};
use crate::config::SceneConfig;
use crate::utils::{DeviceError, Result};

/// 场景执行状态
#[derive(Debug, Clone, Default)]
pub struct SceneExecutionStatus {
    /// 是否正在执行场景
    pub is_executing: bool,
    /// 当前执行的场景名称（如果有）
    pub current_scene: Option<String>,
    /// 当前执行步骤索引（0-based）
    pub current_step_index: Option<usize>,
    /// 当前执行场景总步骤数
    pub total_steps: Option<usize>,
}

/// 场景执行器
pub struct SceneExecutor {
    scenes: Vec<SceneConfig>,
    channel_manager: Arc<ChannelManager>,
    node_manager: Arc<NodeManager>,
    event_tx: broadcast::Sender<DeviceEvent>,
    /// 当前场景执行状态
    execution_status: Arc<Mutex<SceneExecutionStatus>>,
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
            execution_status: Arc::new(Mutex::new(SceneExecutionStatus::default())),
        }
    }

    /// 执行场景（异步启动，立即返回）
    pub async fn execute(&self, scene_name: &str, controller: &DeviceController) -> Result<()> {
        // 查找场景
        let scene = self
            .scenes
            .iter()
            .find(|s| s.name == scene_name)
            .ok_or_else(|| DeviceError::Other(format!("场景 '{}' 不存在", scene_name)))?;

        // 检查是否已有场景正在执行
        let mut status = self.execution_status.lock().await;
        if status.is_executing {
            let executing_scene = status
                .current_scene
                .clone()
                .unwrap_or_else(|| "未知场景".to_string());
            return Err(DeviceError::Other(format!(
                "场景 '{}' 正在执行中，无法同时执行场景 '{}'",
                executing_scene, scene_name
            )));
        }

        // 标记当前场景为正在执行
        status.is_executing = true;
        status.current_scene = Some(scene_name.to_string());
        status.current_step_index = None;
        status.total_steps = Some(scene.nodes.len());
        drop(status); // 释放锁

        info!("开始执行场景: {}", scene_name);

        // 克隆需要的数据用于异步任务
        let scene_name_str = scene_name.to_string();
        let scene_nodes = scene.nodes.clone();
        let controller_clone = controller.clone();
        let execution_status = self.execution_status.clone();
        let event_tx = self.event_tx.clone();

        // 发送场景开始事件
        let _ = event_tx.send(DeviceEvent::SceneStarted {
            scene_name: scene_name_str.clone(),
        });

        // 在后台异步执行场景
        tokio::spawn(async move {
            let mut success = true;

            // 按顺序执行场景中的所有成员
            for (index, member) in scene_nodes.iter().enumerate() {
                {
                    let mut status = execution_status.lock().await;
                    status.current_step_index = Some(index);
                }

                // 延迟执行（如果有配置）
                if let Some(delay) = member.delay {
                    tokio::time::sleep(Duration::from_millis(delay as u64)).await;
                }

                // 执行写入
                match controller_clone.write_node(member.id, member.value).await {
                    Ok(_) => {
                        info!(
                            "场景 '{}': 节点 {} 设置为 {}",
                            scene_name_str, member.id, member.value
                        );
                    }
                    Err(e) => {
                        warn!(
                            "场景 '{}': 节点 {} 设置失败: {:?}",
                            scene_name_str, member.id, e
                        );
                        success = false;
                    }
                }
            }

            // 清除执行状态
            let mut status = execution_status.lock().await;
            *status = SceneExecutionStatus::default();
            drop(status);

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
        self.execution_status.lock().await.clone()
    }
}
