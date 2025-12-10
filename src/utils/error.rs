/// 错误类型定义
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("设备未找到: {0}")]
    DeviceNotFound(String),

    #[error("通道未找到: {0}")]
    ChannelNotFound(u32),

    #[error("协议错误: {0}")]
    ProtocolError(String),

    #[error("连接错误: {0}")]
    ConnectionError(String),

    #[error("超时错误")]
    Timeout,

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("依赖条件未满足")]
    DependencyNotMet,

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("其他错误: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DeviceError>;

/// 错误代码常量
pub mod error_codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 30006;
    pub const DEVICE_NOT_FOUND: i32 = 30001;
    pub const CHANNEL_NOT_FOUND: i32 = 30002;
    pub const TIMEOUT: i32 = 30003;
    pub const DEPENDENCY_NOT_MET: i32 = 30004;
    pub const INVALID_PARAMS: i32 = 400;
    pub const CROSSING: i32 = 30005;
}
