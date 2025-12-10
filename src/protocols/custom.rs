use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use crate::protocols::Protocol;
use crate::utils::Result;

/// 自定义协议
pub struct CustomProtocol {
    channel_id: u32,
}

impl CustomProtocol {
    pub fn new() -> Self {
        Self { channel_id: 0 }
    }
}

#[async_trait]
impl Protocol for CustomProtocol {
    fn from_config(channel_id: u32, _params: &HashMap<String, Value>) -> Result<Box<dyn Protocol>> {
        // TODO: 解析自定义协议配置
        Ok(Box::new(Self { channel_id }))
    }
    
    async fn execute(&mut self, _command: &str, _params: Value) -> Result<Value> {
        Ok(serde_json::json!({"status": "ok"}))
    }

    async fn get_status(&self) -> Result<Value> {
        Ok(serde_json::json!({"connected": true}))
    }

    async fn write(&mut self, _id: u32, _value: i32) -> Result<()> {
        Ok(())
    }

    async fn read(&self, _id: u32) -> Result<i32> {
        Ok(0)
    }

    fn name(&self) -> &str {
        "custom"
    }
}
