/// MQTT 协议模拟器模块
///
/// 提供 MQTT Broker 和代理功能，支持:
/// - 独立 MQTT Broker 服务器
/// - 自定义规则引擎（基于 Topic 匹配）
/// - 代理模式（转发到上游 Broker 并监控）
pub mod broker;
pub mod manager;
pub mod persistence;
pub mod proxy;
pub mod rules;
pub mod state;

pub use manager::MqttSimulatorManager;
pub use state::MqttSimulatorInfo;
