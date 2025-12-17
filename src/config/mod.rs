use serde::{Deserialize, Serialize};

/// 主配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub channels: Vec<ChannelConfig>,
    pub nodes: Vec<NodeConfig>,
    pub scenes: Vec<SceneConfig>,
    #[serde(default)]
    pub task_settings: TaskSettings,
    pub web_server: WebServerConfig,
    /// 文件管理配置（可选）
    #[serde(default)]
    pub file: Option<FileConfig>,
    /// 数据库配置（可选）
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    /// 资源管理配置（可选）
    #[serde(default)]
    pub resource: Option<ResourceConfig>,
    /// 日志配置（可选）
    #[serde(default)]
    pub log: Option<LogConfig>,
}

/// 文件管理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    /// 是否启用文件管理功能
    #[serde(default)]
    pub enable: bool,
    /// 文件存储根路径
    pub path: String,
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 是否启用数据库功能
    #[serde(default)]
    pub enable: bool,
    /// 数据库连接URL
    /// 格式: mysql://username:password@host:port/database
    pub url: String,
}

/// 资源管理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// 是否启用资源管理功能
    #[serde(default)]
    pub enable: bool,
    /// 静态文件存储根路径
    pub path: String,
    /// URL 前缀（用于生成访问路径）
    #[serde(default = "default_resource_url_prefix")]
    pub url_prefix: String,
}

fn default_resource_url_prefix() -> String {
    "/static".to_string()
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// 日志级别: trace, debug, info, warn, error
    #[serde(default = "default_log_level")]
    pub level: String,
    /// 日志输出目标: console, file, both
    #[serde(default = "default_log_target")]
    pub target: String,
    /// 日志文件路径（当 target 为 file 或 both 时使用）
    #[serde(default = "default_log_file")]
    pub file: String,
    /// 是否追加到现有文件
    #[serde(default = "default_log_append")]
    pub append: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_target() -> String {
    "console".to_string()
}

fn default_log_file() -> String {
    "logs/dm-rust.log".to_string()
}

fn default_log_append() -> bool {
    true
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            target: default_log_target(),
            file: default_log_file(),
            append: default_log_append(),
        }
    }
}

/// 任务调度配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSettings {
    #[serde(default = "default_task_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_check_interval")]
    pub check_interval_ms: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_task_timeout() -> u64 { 5000 }
fn default_check_interval() -> u64 { 500 }
fn default_max_retries() -> u32 { 3 }

impl Default for TaskSettings {
    fn default() -> Self {
        Self {
            timeout_ms: default_task_timeout(),
            check_interval_ms: default_check_interval(),
            max_retries: default_max_retries(),
        }
    }
}

/// Web服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebServerConfig {
    pub port: u16,
}

impl Default for WebServerConfig {
    fn default() -> Self {
        Self { port: 8080 }
    }
}

/// 通道配置 - 通用结构，协议特定参数由各协议自行解析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// 通道ID
    pub channel_id: u32,
    /// 是否启用
    pub enable: bool,
    /// 协议类型
    pub statute: StatuteType,
    /// 协议参数（推荐使用 arguments，兼容旧的 flatten 方式）
    #[serde(default)]
    pub arguments: Option<serde_json::Value>,
    /// 自定义方法定义
    #[serde(default)]
    pub methods: Option<Vec<MethodConfig>>,
    /// 自动召唤配置（Modbus专用）
    #[serde(default)]
    pub auto_call: Option<Vec<AutoCallConfig>>,
    /// 其余字段（兼容旧配置）
    #[serde(flatten)]
    pub params: std::collections::HashMap<String, serde_json::Value>,
}

/// 自动召唤配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoCallConfig {
    /// 功能码类型: "holding", "input", "coil", "discrete"
    pub function: String,
    /// 起始地址
    pub start_addr: u16,
    /// 数据位数（寄存器数量或线圈数量）
    pub count: u16,
    /// 召唤间隔（毫秒）
    pub interval_ms: u64,
}

/// 方法配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodConfig {
    /// 方法名称
    pub name: String,
    /// 方法描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 方法参数定义
    #[serde(default)]
    pub arguments: Vec<MethodArgument>,
}

/// 方法参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodArgument {
    /// 参数名
    pub name: String,
    /// 参数类型（string, number, boolean, object）
    pub r#type: String,
    /// 是否必需
    #[serde(default)]
    pub required: bool,
    /// 默认值（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// 参数描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 规约类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StatuteType {
    Pjlink,
    Modbus,
    #[serde(rename = "modbus-slave")]
    ModbusSlave,
    #[serde(rename = "xinkeQ1")]
    XinkeQ1,
    #[serde(rename = "computerControl")]
    ComputerControl,
    Custom,
    #[serde(rename = "screen-njlg-plc")]
    ScreenNjlgPlc,
    #[serde(rename = "hs-power-sequencer")]
    HsPowerSequencer,
    Novastar,
    Mock,
    #[serde(rename = "BFHD1")]
    BFHD1,
    #[serde(rename = "nmDk")]
    NmDk,
    Vivitek,
    #[serde(rename = "hikvisionLed")]
    HikvisionLed,
}

/// 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub global_id: u32,
    pub channel_id: u32,
    pub id: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    pub alias: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depend: Option<Vec<Dependency>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depend_strategy: Option<String>, // "auto" or "manual"
    /// Modbus数据点配置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_point: Option<DataPointConfig>,
}

/// 数据点配置（用于Modbus节点）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPointConfig {
    /// 数据类型
    pub r#type: String,
    /// 寄存器地址
    pub addr: u16,
    /// 缩放比例（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    /// 单位（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

/// 依赖配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i32>,
}

/// 场景配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
    pub nodes: Vec<SceneNode>,
}

/// 场景节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: u32,
    pub value: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<u32>,  // 延迟毫秒数
}

/// 加载配置文件
pub fn load_config() -> anyhow::Result<Config> {
    load_config_from_file("config.json")
}

/// 从指定文件加载配置
pub fn load_config_from_file(path: &str) -> anyhow::Result<Config> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(path);

    // 检查文件是否存在
    if !path.exists() {
        anyhow::bail!("配置文件不存在: {}", path.display());
    }

    // 读取文件内容
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("读取配置文件失败: {}", e))?;

    // 解析JSON
    let config: Config = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("解析配置文件失败: {}", e))?;

    Ok(config)
}
