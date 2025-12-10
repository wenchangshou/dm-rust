/// 场景执行器 - 负责场景的编排和执行
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::config::SceneConfig;
use crate::utils::{Result, DeviceError};
use super::{ChannelManager, NodeManager, DeviceEvent, DeviceController};

/// 场景执行器
pub struct SceneExecutor {
    scenes: Vec<SceneConfig>,
    channel_manager: Arc<ChannelManager>,
    node_manager: Arc<NodeManager>,
    event_tx: broadcast::Sender<DeviceEvent>,
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
        }
    }
    
    /// 执行场景
    pub async fn execute(
        &self,
        scene_name: &str,
        controller: &DeviceController,
    ) -> Result<()> {
        // 查找场景
        let scene = self.scenes.iter()
            .find(|s| s.name == scene_name)
            .ok_or_else(|| DeviceError::Other(format!("场景 '{}' 不存在", scene_name)))?;
        
        info!("开始执行场景: {}", scene_name);
        
        // 发送场景开始事件
        let _ = self.event_tx.send(DeviceEvent::SceneStarted {
            scene_name: scene_name.to_string(),
        });
        
        let mut success = true;
        
        // 按顺序执行场景中的所有成员
        for member in &scene.nodes {
            // 延迟执行（如果有配置）
            if let Some(delay) = member.delay {
                tokio::time::sleep(Duration::from_millis(delay as u64)).await;
            }
            
            // 执行写入
            match controller.write_node(member.id, member.value).await {
                Ok(_) => {
                    info!("场景 '{}': 节点 {} 设置为 {}", scene_name, member.id, member.value);
                }
                Err(e) => {
                    warn!("场景 '{}': 节点 {} 设置失败: {:?}", scene_name, member.id, e);
                    success = false;
                }
            }
        }
        
        // 发送场景完成事件
        let _ = self.event_tx.send(DeviceEvent::SceneCompleted {
            scene_name: scene_name.to_string(),
            success,
        });
        
        if success {
            info!("场景 '{}' 执行成功", scene_name);
            Ok(())
        } else {
            Err(DeviceError::Other(format!("场景 '{}' 执行部分失败", scene_name)))
        }
    }
    
    /// 获取所有场景名称
    pub fn list_scenes(&self) -> Vec<String> {
        self.scenes.iter().map(|s| s.name.clone()).collect()
    }
    
    /// 获取场景详情
    pub fn get_scene(&self, scene_name: &str) -> Option<&SceneConfig> {
        self.scenes.iter().find(|s| s.name == scene_name)
    }
}
