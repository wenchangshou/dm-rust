/// 任务调度器 - 负责依赖任务的队列管理和调度
use std::sync::Arc;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, broadcast};
use tracing::{info, warn, debug};
use uuid::Uuid;

use crate::config::{TaskSettings, NodeConfig};
use crate::utils::Result;
use super::{ChannelManager, NodeManager, DependencyResolver, DeviceEvent};

/// 任务状态
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,      // 等待依赖满足
    Executing,    // 正在执行
    Completed,    // 已完成
    Failed,       // 失败
    Timeout,      // 超时
}

/// 任务
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub global_id: u32,
    pub channel_id: u32,
    pub device_id: u32,
    pub value: i32,
    pub alias: String,
    pub status: TaskStatus,
    pub created_at: Instant,
    pub retry_count: u32,
    pub node_config: NodeConfig,
}

impl Task {
    fn new(node: NodeConfig, value: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            global_id: node.global_id,
            channel_id: node.channel_id,
            device_id: node.id,
            value,
            alias: node.alias.clone(),
            status: TaskStatus::Pending,
            created_at: Instant::now(),
            retry_count: 0,
            node_config: node,
        }
    }
}

/// 任务调度器
pub struct TaskScheduler {
    queue: Arc<RwLock<VecDeque<Task>>>,
    settings: TaskSettings,
    channel_manager: Arc<ChannelManager>,
    node_manager: Arc<NodeManager>,
    dependency_resolver: Arc<DependencyResolver>,
    event_tx: broadcast::Sender<DeviceEvent>,
}

impl TaskScheduler {
    /// 创建任务调度器
    pub async fn new(
        settings: TaskSettings,
        channel_manager: Arc<ChannelManager>,
        node_manager: Arc<NodeManager>,
        dependency_resolver: Arc<DependencyResolver>,
        event_tx: broadcast::Sender<DeviceEvent>,
    ) -> Self {
        let scheduler = Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            settings,
            channel_manager,
            node_manager,
            dependency_resolver,
            event_tx,
        };
        
        // 启动后台调度循环
        scheduler.start_scheduler_loop();
        
        scheduler
    }
    
    /// 提交任务到队列
    pub async fn submit_task(&self, node: NodeConfig, value: i32) -> Result<()> {
        let task = Task::new(node, value);
        info!("提交任务: {} ({})", task.alias, task.id);
        
        let mut queue = self.queue.write().await;
        queue.push_back(task);
        
        Ok(())
    }
    
    /// 启动调度循环
    fn start_scheduler_loop(&self) {
        let queue = self.queue.clone();
        let settings = self.settings.clone();
        let channel_manager = self.channel_manager.clone();
        let node_manager = self.node_manager.clone();
        let dependency_resolver = self.dependency_resolver.clone();
        let event_tx = self.event_tx.clone();
        
        tokio::spawn(async move {
            let check_interval = Duration::from_millis(settings.check_interval_ms);
            let timeout = Duration::from_millis(settings.timeout_ms);
            
            loop {
                tokio::time::sleep(check_interval).await;
                
                let mut queue = queue.write().await;
                if queue.is_empty() {
                    continue;
                }
                
                let mut completed_indices = Vec::new();
                
                for (idx, task) in queue.iter_mut().enumerate() {
                    // 检查超时
                    if task.created_at.elapsed() > timeout {
                        warn!("任务 {} ({}) 超时", task.alias, task.id);
                        task.status = TaskStatus::Timeout;
                        completed_indices.push(idx);
                        
                        let _ = event_tx.send(DeviceEvent::TaskCompleted {
                            task_id: task.id.clone(),
                            success: false,
                        });
                        continue;
                    }
                    
                    // 检查重试次数
                    if task.retry_count >= settings.max_retries {
                        warn!("任务 {} ({}) 达到最大重试次数", task.alias, task.id);
                        task.status = TaskStatus::Failed;
                        completed_indices.push(idx);
                        
                        let _ = event_tx.send(DeviceEvent::TaskCompleted {
                            task_id: task.id.clone(),
                            success: false,
                        });
                        continue;
                    }
                    
                    // 检查依赖
                    if let Some(dependencies) = &task.node_config.depend {
                        match dependency_resolver.check_dependencies(dependencies).await {
                            Ok(true) => {
                                // 依赖满足，执行任务
                                debug!("任务 {} 依赖已满足，开始执行", task.alias);
                                task.status = TaskStatus::Executing;
                                
                                match channel_manager.write(
                                    task.channel_id,
                                    task.device_id,
                                    task.value
                                ).await {
                                    Ok(_) => {
                                        info!("任务 {} ({}) 执行成功", task.alias, task.id);
                                        task.status = TaskStatus::Completed;
                                        node_manager.update_value(task.global_id, task.value);
                                        completed_indices.push(idx);
                                        
                                        let _ = event_tx.send(DeviceEvent::TaskCompleted {
                                            task_id: task.id.clone(),
                                            success: true,
                                        });
                                    }
                                    Err(e) => {
                                        warn!("任务 {} 执行失败: {:?}", task.alias, e);
                                        task.retry_count += 1;
                                        task.status = TaskStatus::Pending;
                                    }
                                }
                            }
                            Ok(false) => {
                                // 依赖未满足，继续等待
                                debug!("任务 {} 依赖未满足，继续等待", task.alias);
                            }
                            Err(e) => {
                                warn!("任务 {} 依赖检查失败: {:?}", task.alias, e);
                                task.retry_count += 1;
                            }
                        }
                    }
                }
                
                // 移除已完成的任务（从后向前删除）
                for idx in completed_indices.iter().rev() {
                    queue.remove(*idx);
                }
            }
        });
    }
    
    /// 获取队列长度
    pub async fn queue_length(&self) -> usize {
        self.queue.read().await.len()
    }
    
    /// 获取所有待处理任务
    pub async fn get_pending_tasks(&self) -> Vec<Task> {
        self.queue.read().await.iter().cloned().collect()
    }
}
