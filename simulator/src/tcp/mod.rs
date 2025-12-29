/// TCP 协议模拟服务器模块
///
/// 提供真实的 TCP 服务器来模拟各种设备协议，用于开发测试。
///
/// # 架构
///
/// ```text
/// TcpSimulatorManager
///     │
///     ├── TcpSimulatorServer (实例1)
///     │       └── ProtocolHandler (scene_loader)
///     │
///     └── TcpSimulatorServer (实例2)
///             └── ProtocolHandler (pjlink)
/// ```
///
/// # 使用示例
///
/// ```rust,ignore
/// use tcp_simulator::{TcpSimulatorManager, TcpSimulatorConfig};
///
/// let manager = TcpSimulatorManager::new();
///
/// // 创建模拟器
/// let config = TcpSimulatorConfig {
///     name: "场景加载模拟器".to_string(),
///     protocol: "scene_loader".to_string(),
///     bind_addr: "0.0.0.0".to_string(),
///     port: 5000,
///     ..Default::default()
/// };
/// let info = manager.create(config).await?;
///
/// // 启动模拟器
/// manager.start(&info.id).await?;
///
/// // 现在可以通过 TCP 连接到 0.0.0.0:5000
/// ```
///
/// # 添加新协议
///
/// 1. 在 `protocols/` 目录创建新文件
/// 2. 实现 `ProtocolHandler` trait
/// 3. 在 `protocols/mod.rs` 中注册
pub mod handler;
pub mod manager;
pub mod persistence;
pub mod protocols;
pub mod server;
pub mod state;
pub mod template;
pub mod transport;
pub mod udp_server;

pub use manager::TcpSimulatorManager;
pub use state::{ProtocolInfo, SimulatorInfo, SimulatorStatus, TcpSimulatorConfig};
pub use template::{CreateFromTemplateRequest, SimulatorTemplate};
