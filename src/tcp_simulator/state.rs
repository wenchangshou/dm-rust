/// 模拟器状态和配置定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 模拟器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpSimulatorConfig {
    /// 唯一标识（自动生成）
    #[serde(default)]
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 协议类型
    pub protocol: String,
    /// 绑定地址
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,
    /// 监听端口
    pub port: u16,
    /// 协议特定的初始状态
    #[serde(default)]
    pub initial_state: Value,
}

fn default_bind_addr() -> String {
    "0.0.0.0".to_string()
}

/// 模拟器运行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SimulatorStatus {
    /// 已停止
    Stopped,
    /// 运行中
    Running,
    /// 错误状态
    Error,
}

impl Default for SimulatorStatus {
    fn default() -> Self {
        Self::Stopped
    }
}

/// 连接统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectionStats {
    /// 总连接次数
    pub total_connections: u64,
    /// 当前活动连接数
    pub active_connections: u32,
    /// 接收字节数
    pub bytes_received: u64,
    /// 发送字节数
    pub bytes_sent: u64,
    /// 最后活动时间
    pub last_activity: Option<DateTime<Utc>>,
}

impl ConnectionStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_connection(&mut self) {
        self.total_connections += 1;
        self.active_connections += 1;
        self.last_activity = Some(Utc::now());
    }

    pub fn record_disconnection(&mut self) {
        if self.active_connections > 0 {
            self.active_connections -= 1;
        }
    }

    pub fn record_received(&mut self, bytes: u64) {
        self.bytes_received += bytes;
        self.last_activity = Some(Utc::now());
    }

    pub fn record_sent(&mut self, bytes: u64) {
        self.bytes_sent += bytes;
        self.last_activity = Some(Utc::now());
    }
}

/// 模拟器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorState {
    /// 是否在线（模拟设备在线状态）
    pub online: bool,
    /// 故障类型（None 表示正常）
    pub fault: Option<String>,
    /// 协议特定的值存储
    pub values: HashMap<String, Value>,
    /// 连接统计
    #[serde(default)]
    pub stats: ConnectionStats,
}

impl Default for SimulatorState {
    fn default() -> Self {
        Self {
            online: true,
            fault: None,
            values: HashMap::new(),
            stats: ConnectionStats::new(),
        }
    }
}

impl SimulatorState {
    pub fn new() -> Self {
        Self::default()
    }

    /// 从初始状态创建
    pub fn from_initial(initial: &Value) -> Self {
        let mut state = Self::default();

        if let Some(obj) = initial.as_object() {
            for (key, value) in obj {
                state.values.insert(key.clone(), value.clone());
            }
        }

        state
    }

    /// 设置值
    pub fn set_value<S: Into<String>>(&mut self, key: S, value: Value) {
        self.values.insert(key.into(), value);
    }

    /// 获取值
    pub fn get_value(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    /// 获取整数值
    pub fn get_i32(&self, key: &str) -> Option<i32> {
        self.values.get(key).and_then(|v| v.as_i64()).map(|v| v as i32)
    }

    /// 获取布尔值
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.values.get(key).and_then(|v| v.as_bool())
    }

    /// 设置故障
    pub fn set_fault(&mut self, fault_type: &str) {
        self.fault = Some(fault_type.to_string());
    }

    /// 清除故障
    pub fn clear_fault(&mut self) {
        self.fault = None;
    }

    /// 检查是否有故障
    pub fn has_fault(&self) -> bool {
        self.fault.is_some()
    }
}

/// 模拟器信息（用于 API 响应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorInfo {
    pub id: String,
    pub name: String,
    pub protocol: String,
    pub bind_addr: String,
    pub port: u16,
    pub status: SimulatorStatus,
    pub state: SimulatorState,
}

impl SimulatorInfo {
    pub fn new(config: &TcpSimulatorConfig, status: SimulatorStatus, state: SimulatorState) -> Self {
        Self {
            id: config.id.clone(),
            name: config.name.clone(),
            protocol: config.protocol.clone(),
            bind_addr: config.bind_addr.clone(),
            port: config.port,
            status,
            state,
        }
    }
}

/// 协议信息（用于列出支持的协议）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolInfo {
    pub name: String,
    pub description: String,
    pub default_port: u16,
    pub commands: Vec<String>,
}
