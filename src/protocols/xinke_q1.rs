use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use crate::protocols::Protocol;
use crate::utils::{Result, DeviceError};

#[derive(Debug, Deserialize, Serialize)]
struct XinkeQ1Config {
    addr: String,
    port: u16,
}

pub struct XinkeQ1Protocol {
    channel_id: u32,
    addr: String,
    port: u16,
}

impl XinkeQ1Protocol {
    pub fn new(addr: String, port: u16) -> Self {
        Self { channel_id: 0, addr, port }
    }
}

#[async_trait]
impl Protocol for XinkeQ1Protocol {
    fn from_config(channel_id: u32, params: &HashMap<String, Value>) -> Result<Box<dyn Protocol>> {
        let addr = params.get("addr")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DeviceError::ConfigError("XinkeQ1缺少addr参数".into()))?
            .to_string();
        
        let port = params.get("port")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| DeviceError::ConfigError("XinkeQ1缺少port参数".into()))? as u16;
        
        Ok(Box::new(Self {
            channel_id,
            addr,
            port,
        }))
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
        "xinkeQ1"
    }
}

