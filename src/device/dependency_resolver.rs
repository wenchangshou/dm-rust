/// 依赖解析器 - 负责依赖条件检查和满足
use std::sync::Arc;
use tracing::{debug, info};

use crate::config::Dependency;
use crate::utils::{Result, DeviceError};
use super::{NodeManager, DeviceController};

/// 依赖解析器
pub struct DependencyResolver {
    node_manager: Arc<NodeManager>,
}

impl DependencyResolver {
    /// 创建依赖解析器
    pub fn new(node_manager: Arc<NodeManager>) -> Self {
        Self { node_manager }
    }
    
    /// 检查依赖列表是否全部满足
    pub async fn check_dependencies(&self, dependencies: &[Dependency]) -> Result<bool> {
        for dep in dependencies {
            if !self.check_single_dependency(dep).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    /// 检查单个依赖条件
    async fn check_single_dependency(&self, dep: &Dependency) -> Result<bool> {
        // 获取依赖节点的全局ID
        let global_id = if let Some(channel_id) = dep.channel_id {
            // 通过channel_id和id查找
            if let Some(node_id) = dep.id {
                self.node_manager.find_global_id(channel_id, node_id)
                    .ok_or_else(|| DeviceError::DeviceNotFound(
                        format!("通道 {} 设备 {}", channel_id, node_id)
                    ))?
            } else {
                return Err(DeviceError::ConfigError("依赖配置缺少id".into()));
            }
        } else if let Some(node_id) = dep.id {
            // 直接使用id作为global_id
            node_id
        } else {
            return Err(DeviceError::ConfigError("依赖配置无效".into()));
        };
        
        // 获取节点状态
        let state = self.node_manager.get_state(global_id)
            .ok_or_else(|| DeviceError::DeviceNotFound(format!("节点 {}", global_id)))?;
        
        // 检查条件
        if let Some(expected_value) = dep.value {
            // 检查值是否匹配
            if let Some(current_value) = state.current_value {
                if current_value != expected_value {
                    debug!(
                        "依赖节点 {} 值不匹配: 期望 {}, 实际 {}",
                        global_id, expected_value, current_value
                    );
                    return Ok(false);
                }
            } else {
                debug!("依赖节点 {} 当前值未知", global_id);
                return Ok(false);
            }
        }
        
        if let Some(expected_status) = dep.status {
            // 检查在线状态
            if state.online != expected_status {
                debug!(
                    "依赖节点 {} 状态不匹配: 期望 {}, 实际 {}",
                    global_id, expected_status, state.online
                );
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// 自动满足依赖条件（用于auto策略）
    pub async fn fulfill_dependencies(
        &self,
        dependencies: &[Dependency],
        controller: &DeviceController,
    ) -> Result<()> {
        info!("自动满足依赖条件...");
        
        for dep in dependencies {
            if let (Some(node_id), Some(target_value)) = (dep.id, dep.value) {
                // 获取当前状态
                let state = self.node_manager.get_state(node_id)
                    .ok_or_else(|| DeviceError::DeviceNotFound(format!("节点 {}", node_id)))?;
                
                // 检查是否需要改变
                if state.current_value != Some(target_value) {
                    info!("设置依赖节点 {} = {}", node_id, target_value);
                    controller.execute_write(state.channel_id, state.device_id, target_value).await?;
                    
                    // 等待一小段时间让设备响应
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
        
        Ok(())
    }
}
