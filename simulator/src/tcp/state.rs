/// 模拟器状态和配置定义
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};

#[cfg(feature = "swagger")]
use utoipa::ToSchema;

/// 模拟器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct TcpSimulatorConfig {
    /// 唯一标识（自动生成）
    #[serde(default)]
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 描述
    #[serde(default)]
    pub description: String,
    /// 协议类型
    pub protocol: String,
    /// 传输协议 (tcp, udp)
    #[serde(default = "default_transport")]
    pub transport: String,
    /// 绑定地址
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,
    /// 监听端口
    pub port: u16,
    /// 协议特定的初始状态
    #[serde(default)]
    pub initial_state: Value,
    /// 协议配置 (如自定义协议规则)
    #[serde(default)]
    pub protocol_config: Option<Value>,
}

fn default_transport() -> String {
    "tcp".to_string()
}

fn default_bind_addr() -> String {
    "0.0.0.0".to_string()
}

/// 模拟器运行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
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

// ============ 报文监控 ============

/// 报文方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum PacketDirection {
    /// 接收到的数据
    Received,
    /// 发送的数据
    Sent,
}

/// 报文记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct PacketRecord {
    /// 唯一 ID
    pub id: u64,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 方向
    pub direction: PacketDirection,
    /// 客户端地址
    pub peer_addr: String,
    /// 原始数据（十六进制字符串）
    pub hex_data: String,
    /// 数据大小（字节）
    pub size: usize,
    /// 协议解析信息（可选，协议特定）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parsed: Option<Value>,
}

impl PacketRecord {
    /// 创建新的报文记录
    pub fn new(
        id: u64,
        direction: PacketDirection,
        peer_addr: String,
        data: &[u8],
        parsed: Option<Value>,
    ) -> Self {
        Self {
            id,
            timestamp: Utc::now(),
            direction,
            peer_addr,
            hex_data: hex::encode(data),
            size: data.len(),
            parsed,
        }
    }
}

/// 报文监控器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketMonitor {
    /// 是否启用监控
    #[serde(default = "default_monitor_enabled")]
    pub enabled: bool,
    /// 最大记录数量
    #[serde(default = "default_max_packets")]
    pub max_packets: usize,
    /// 报文记录列表
    #[serde(default)]
    pub packets: VecDeque<PacketRecord>,
    /// 下一个报文 ID
    #[serde(default)]
    next_id: u64,
    /// Debug 模式（持久化所有报文到文件）
    #[serde(default)]
    pub debug_mode: bool,
    /// Debug 日志文件路径
    #[serde(skip)]
    pub debug_log_path: Option<String>,
}

fn default_monitor_enabled() -> bool {
    true
}

fn default_max_packets() -> usize {
    1000
}

impl Default for PacketMonitor {
    fn default() -> Self {
        Self {
            enabled: true,
            max_packets: 1000,
            packets: VecDeque::new(),
            next_id: 1,
            debug_mode: false,
            debug_log_path: None,
        }
    }
}

impl PacketMonitor {
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录报文
    pub fn record(
        &mut self,
        direction: PacketDirection,
        peer_addr: &str,
        data: &[u8],
        parsed: Option<Value>,
    ) {
        if !self.enabled || data.is_empty() {
            return;
        }

        let record = PacketRecord::new(
            self.next_id,
            direction.clone(),
            peer_addr.to_string(),
            data,
            parsed,
        );
        self.next_id += 1;

        // Debug 模式：写入文件
        if self.debug_mode {
            if let Some(ref path) = self.debug_log_path {
                self.append_to_log_file(path, &record);
            }
        }

        self.packets.push_back(record);

        // 超过最大数量时移除旧记录
        while self.packets.len() > self.max_packets {
            self.packets.pop_front();
        }
    }

    /// 追加报文到日志文件
    fn append_to_log_file(&self, path: &str, record: &PacketRecord) {
        use std::fs::OpenOptions;
        use std::io::Write;

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let dir_str = match record.direction {
                PacketDirection::Received => "<<<",
                PacketDirection::Sent => ">>>",
            };
            let line = format!(
                "[{}] {} {} {}\n",
                record.timestamp, dir_str, record.peer_addr, record.hex_data
            );
            let _ = file.write_all(line.as_bytes());
        }
    }

    /// 记录接收的报文
    pub fn record_received(&mut self, peer_addr: &str, data: &[u8], parsed: Option<Value>) {
        self.record(PacketDirection::Received, peer_addr, data, parsed);
    }

    /// 记录发送的报文
    pub fn record_sent(&mut self, peer_addr: &str, data: &[u8], parsed: Option<Value>) {
        self.record(PacketDirection::Sent, peer_addr, data, parsed);
    }

    /// 获取所有报文
    pub fn get_packets(&self) -> Vec<PacketRecord> {
        self.packets.iter().cloned().collect()
    }

    /// 获取最近 N 条报文
    pub fn get_recent(&self, count: usize) -> Vec<PacketRecord> {
        self.packets.iter().rev().take(count).cloned().collect()
    }

    /// 获取指定 ID 之后的报文（用于增量获取）
    pub fn get_after(&self, after_id: u64) -> Vec<PacketRecord> {
        self.packets
            .iter()
            .filter(|p| p.id > after_id)
            .cloned()
            .collect()
    }

    /// 清空所有报文
    pub fn clear(&mut self) {
        self.packets.clear();
    }

    /// 设置是否启用
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 设置最大记录数
    pub fn set_max_packets(&mut self, max: usize) {
        self.max_packets = max;
        while self.packets.len() > max {
            self.packets.pop_front();
        }
    }

    /// 设置 Debug 模式
    pub fn set_debug_mode(&mut self, enabled: bool, simulator_id: &str) {
        self.debug_mode = enabled;
        if enabled {
            // 创建日志目录
            let log_dir = std::path::Path::new("logs/simulator");
            let _ = std::fs::create_dir_all(log_dir);

            // 生成日志文件路径
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
            let filename = format!("{}_{}.log", simulator_id, timestamp);
            let path = log_dir.join(filename);
            self.debug_log_path = Some(path.to_string_lossy().to_string());
        } else {
            self.debug_log_path = None;
        }
    }

    /// 获取 Debug 日志文件路径
    pub fn get_debug_log_path(&self) -> Option<&str> {
        self.debug_log_path.as_deref()
    }

    /// 获取报文数量
    pub fn len(&self) -> usize {
        self.packets.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }
}

// ============ 客户端连接追踪 ============

/// 客户端连接信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct ClientConnection {
    /// 唯一标识
    pub id: String,
    /// 客户端地址
    pub peer_addr: String,
    /// 连接时间
    pub connected_at: DateTime<Utc>,
    /// 接收字节数
    pub bytes_received: u64,
    /// 发送字节数
    pub bytes_sent: u64,
    /// 最后活动时间
    pub last_activity: DateTime<Utc>,
}

impl ClientConnection {
    /// 创建新的客户端连接
    pub fn new(id: String, peer_addr: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            peer_addr,
            connected_at: now,
            bytes_received: 0,
            bytes_sent: 0,
            last_activity: now,
        }
    }

    /// 记录接收字节
    pub fn record_received(&mut self, bytes: u64) {
        self.bytes_received += bytes;
        self.last_activity = Utc::now();
    }

    /// 记录发送字节
    pub fn record_sent(&mut self, bytes: u64) {
        self.bytes_sent += bytes;
        self.last_activity = Utc::now();
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
    /// 报文监控器
    #[serde(default)]
    pub packet_monitor: PacketMonitor,
    /// 已连接的客户端列表
    #[serde(default)]
    pub clients: HashMap<String, ClientConnection>,
}

impl Default for SimulatorState {
    fn default() -> Self {
        Self {
            online: true,
            fault: None,
            values: HashMap::new(),
            stats: ConnectionStats::new(),
            packet_monitor: PacketMonitor::new(),
            clients: HashMap::new(),
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
        self.values
            .get(key)
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
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
    pub description: String,
    pub protocol: String,
    pub transport: String,
    pub bind_addr: String,
    pub port: u16,
    pub status: SimulatorStatus,
    pub state: SimulatorState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_config: Option<Value>,
}

impl SimulatorInfo {
    pub fn new(
        config: &TcpSimulatorConfig,
        status: SimulatorStatus,
        state: SimulatorState,
    ) -> Self {
        Self {
            id: config.id.clone(),
            name: config.name.clone(),
            description: config.description.clone(),
            protocol: config.protocol.clone(),
            transport: config.transport.clone(),
            bind_addr: config.bind_addr.clone(),
            port: config.port,
            status,
            state,
            protocol_config: config.protocol_config.clone(),
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
