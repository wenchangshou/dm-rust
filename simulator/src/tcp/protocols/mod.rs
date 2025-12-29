/// 协议注册表
///
/// 管理所有可用的协议处理器，支持通过名称创建实例。
///
/// # 添加新协议
/// 1. 创建新文件 `my_protocol.rs`
/// 2. 实现 `ProtocolHandler` trait
/// 3. 在 `ProtocolRegistry::new()` 中注册
mod custom;
mod modbus;
mod scene_loader;

pub use custom::{CustomProtocolConfig, CustomProtocolHandler};
pub use modbus::{ModbusHandler, ModbusValues, RegisterConfig, RegisterType, SlaveConfig};
pub use scene_loader::SceneLoaderHandler;

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use super::handler::ProtocolHandler;
use super::state::ProtocolInfo;

/// 协议工厂函数类型
type ProtocolFactory = Box<dyn Fn(Option<Value>) -> Arc<dyn ProtocolHandler> + Send + Sync>;

/// 协议注册表
pub struct ProtocolRegistry {
    factories: HashMap<String, ProtocolFactory>,
}

impl ProtocolRegistry {
    /// 创建注册表并注册所有内置协议
    pub fn new() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
        };

        // 注册内置协议
        registry.register("scene_loader", |_| Arc::new(SceneLoaderHandler::new()));
        registry.register("modbus", |_| Arc::new(ModbusHandler::new()));

        // 注册自定义协议
        registry.register("custom", |config| {
            if let Some(cfg_value) = config {
                if let Ok(cfg) = serde_json::from_value::<CustomProtocolConfig>(cfg_value) {
                    return Arc::new(CustomProtocolHandler::new(cfg));
                }
            }
            // Fallback/Error case: return a dummy or error handler?
            // Since we return Arc<dyn ProtocolHandler>, we must return *something*.
            // Only for "custom" protocol without config this happens.
            // Ideally we should return Result, but that changes the signature significantly.
            // For now, let's return a default empty custom handler.
            Arc::new(CustomProtocolHandler::new(CustomProtocolConfig {
                name: "custom".to_string(),
                description: "Unconfigured Custom Protocol".to_string(),
                default_port: 0,
                rules: vec![],
                checksum: None,
            }))
        });

        registry
    }

    /// 注册协议
    pub fn register<F>(&mut self, name: &str, factory: F)
    where
        F: Fn(Option<Value>) -> Arc<dyn ProtocolHandler> + Send + Sync + 'static,
    {
        self.factories.insert(name.to_string(), Box::new(factory));
    }

    /// 创建协议处理器实例
    pub fn create(
        &self,
        protocol: &str,
        config: Option<Value>,
    ) -> Option<Arc<dyn ProtocolHandler>> {
        self.factories.get(protocol).map(|f| f(config))
    }

    /// 检查协议是否存在
    pub fn contains(&self, protocol: &str) -> bool {
        self.factories.contains_key(protocol)
    }

    /// 获取所有支持的协议名称
    pub fn list_protocols(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }

    /// 获取协议详细信息列表
    pub fn get_protocol_infos(&self) -> Vec<ProtocolInfo> {
        self.factories
            .iter()
            .map(|(name, factory)| {
                let handler = factory(None); // Use default/no config for metadata
                ProtocolInfo {
                    name: name.clone(),
                    description: handler.description().to_string(),
                    default_port: handler.default_port(),
                    commands: handler.supported_commands(),
                }
            })
            .collect()
    }
}

impl Default for ProtocolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ProtocolRegistry::new();
        assert!(registry.contains("scene_loader"));
    }

    #[test]
    fn test_create_protocol() {
        let registry = ProtocolRegistry::new();
        let handler = registry.create("scene_loader", None);
        assert!(handler.is_some());
        assert_eq!(handler.unwrap().name(), "scene_loader");
    }

    #[test]
    fn test_unknown_protocol() {
        let registry = ProtocolRegistry::new();
        assert!(registry.create("unknown", None).is_none());
    }
}
