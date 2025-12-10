use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use crate::protocols::Protocol;
use crate::utils::Result;

pub struct ModbusSlaveProtocol {
    channel_id: u32,
}

impl ModbusSlaveProtocol {
    pub fn new() -> Self {
        Self { channel_id: 0 }
    }
}

#[async_trait]
impl Protocol for ModbusSlaveProtocol {
    fn from_config(channel_id: u32, _params: &HashMap<String, Value>) -> Result<Box<dyn Protocol>> {
        // TODO: 解析 device_list 和 map 配置
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
        "modbus-slave"
    }
}
