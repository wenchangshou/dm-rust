/// MQTT 模拟器状态管理
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MQTT 模拟器运行模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MqttMode {
    /// Broker 模式 - 作为独立 MQTT 服务器
    Broker,
    /// 代理模式 - 转发到上游 Broker
    Proxy,
}

impl Default for MqttMode {
    fn default() -> Self {
        MqttMode::Broker
    }
}

/// MQTT 模拟器状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum MqttSimulatorStatus {
    #[default]
    Stopped,
    Running,
    Error(String),
}

/// MQTT 客户端连接信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttClientInfo {
    pub client_id: String,
    pub connected_at: DateTime<Utc>,
    pub subscriptions: Vec<String>,
    pub last_activity: Option<DateTime<Utc>>,
}

/// MQTT 报文记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttPacketRecord {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub direction: PacketDirection,
    pub client_id: Option<String>,
    pub packet_type: String,
    pub topic: Option<String>,
    pub payload: Option<String>,
    pub payload_hex: Option<String>,
    pub qos: Option<u8>,
}

/// 报文方向
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PacketDirection {
    Received,
    Sent,
    Forwarded,
}

/// MQTT 统计信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MqttStats {
    pub total_connections: u64,
    pub active_connections: u32,
    pub messages_received: u64,
    pub messages_sent: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub last_activity: Option<DateTime<Utc>>,
}

/// MQTT 模拟器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttSimulatorState {
    pub mode: MqttMode,
    pub clients: Vec<MqttClientInfo>,
    pub subscriptions: HashMap<String, Vec<String>>,
    pub stats: MqttStats,
    /// 报文记录（内存中保留最近的 N 条）
    #[serde(skip)]
    pub packets: Vec<MqttPacketRecord>,
    pub packet_counter: u64,
}

impl Default for MqttSimulatorState {
    fn default() -> Self {
        Self {
            mode: MqttMode::Broker,
            clients: Vec::new(),
            subscriptions: HashMap::new(),
            stats: MqttStats::default(),
            packets: Vec::new(),
            packet_counter: 0,
        }
    }
}

impl MqttSimulatorState {
    pub fn new(mode: MqttMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    /// 添加报文记录
    pub fn add_packet(&mut self, record: MqttPacketRecord) {
        self.packet_counter += 1;
        self.packets.push(record);
        // 保留最近 1000 条
        if self.packets.len() > 1000 {
            self.packets.remove(0);
        }
    }

    /// 获取报文记录（分页）
    pub fn get_packets(&self, limit: usize, after_id: Option<u64>) -> Vec<&MqttPacketRecord> {
        self.packets
            .iter()
            .filter(|p| after_id.map_or(true, |id| p.id > id))
            .take(limit)
            .collect()
    }

    /// 清空报文记录
    pub fn clear_packets(&mut self) {
        self.packets.clear();
    }
}

/// 代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub upstream_host: String,
    pub upstream_port: u16,
    pub upstream_username: Option<String>,
    pub upstream_password: Option<String>,
    pub client_id_prefix: String,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            upstream_host: "localhost".to_string(),
            upstream_port: 1883,
            upstream_username: None,
            upstream_password: None,
            client_id_prefix: "proxy_".to_string(),
        }
    }
}

/// MQTT 模拟器完整信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttSimulatorInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub mode: MqttMode,
    pub port: u16,
    pub bind_addr: String,
    pub status: MqttSimulatorStatus,
    pub state: MqttSimulatorState,
    pub proxy_config: Option<ProxyConfig>,
    pub auto_start: bool,
    pub created_at: DateTime<Utc>,
}

impl MqttSimulatorInfo {
    pub fn new(name: String, mode: MqttMode, port: u16) -> Self {
        Self {
            id: format!(
                "mqtt_{}",
                uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
            ),
            name,
            description: None,
            mode: mode.clone(),
            port,
            bind_addr: "0.0.0.0".to_string(),
            status: MqttSimulatorStatus::Stopped,
            state: MqttSimulatorState::new(mode),
            proxy_config: None,
            auto_start: false,
            created_at: Utc::now(),
        }
    }
}

/// 创建 MQTT 模拟器请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMqttSimulatorRequest {
    pub name: String,
    pub description: Option<String>,
    pub mode: MqttMode,
    pub port: u16,
    pub bind_addr: Option<String>,
    pub proxy_config: Option<ProxyConfig>,
    pub auto_start: Option<bool>,
}
