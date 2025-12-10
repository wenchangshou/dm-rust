use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::protocols::Protocol;
use crate::utils::{Result, DeviceError};

/// PJLink协议配置参数
#[derive(Debug, Deserialize, Serialize)]
struct PjlinkConfig {
    addr: String,
    port: u16,
    #[serde(default)]
    password: Option<String>,
}

/// PJLink协议实现
pub struct PjlinkProtocol {
    channel_id: u32,
    addr: String,
    port: u16,
    password: Option<String>,
}

impl PjlinkProtocol {
    pub fn new(addr: String, port: u16, password: Option<String>) -> Self {
        Self { 
            channel_id: 0,
            addr, 
            port, 
            password 
        }
    }

    async fn send_command(&self, cmd: &str) -> Result<String> {
        let mut stream = TcpStream::connect(format!("{}:{}", self.addr, self.port))
            .await
            .map_err(|e| DeviceError::ConnectionError(e.to_string()))?;

        // PJLink协议实现
        let mut buffer = [0u8; 1024];
        let n = stream.read(&mut buffer).await?;
        
        // 发送命令
        let command = format!("%1{} ?\r", cmd);
        stream.write_all(command.as_bytes()).await?;
        
        // 读取响应
        let n = stream.read(&mut buffer).await?;
        let response = String::from_utf8_lossy(&buffer[..n]).to_string();
        
        Ok(response)
    }
}

#[async_trait]
impl Protocol for PjlinkProtocol {
    /// 从配置创建PJLink协议实例
    /// 
    /// 期望的配置格式:
    /// ```json
    /// {
    ///   "addr": "192.168.1.100",
    ///   "port": 4352,
    ///   "password": "optional_password"
    /// }
    /// ```
    fn from_config(channel_id: u32, params: &HashMap<String, Value>) -> Result<Box<dyn Protocol>> {
        // 从 HashMap 中提取参数
        let addr = params.get("addr")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DeviceError::ConfigError("PJLink缺少addr参数".into()))?
            .to_string();
        
        let port = params.get("port")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| DeviceError::ConfigError("PJLink缺少port参数".into()))? as u16;
        
        let password = params.get("password")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        Ok(Box::new(Self {
            channel_id,
            addr,
            port,
            password,
        }))
    }
    
    async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
        match command {
            "powerOn" => {
                self.send_command("POWR 1").await?;
                Ok(serde_json::json!({"status": "ok"}))
            }
            "powerOff" => {
                self.send_command("POWR 0").await?;
                Ok(serde_json::json!({"status": "ok"}))
            }
            "getPowerState" => {
                let resp = self.send_command("POWR").await?;
                Ok(serde_json::json!({"state": resp}))
            }
            _ => Err(DeviceError::ProtocolError(format!("未知命令: {}", command))),
        }
    }

    async fn get_status(&self) -> Result<Value> {
        Ok(serde_json::json!({"connected": true}))
    }

    async fn write(&mut self, _id: u32, _value: i32) -> Result<()> {
        Err(DeviceError::ProtocolError("PJLink不支持write操作".to_string()))
    }

    async fn read(&self, _id: u32) -> Result<i32> {
        Err(DeviceError::ProtocolError("PJLink不支持read操作".to_string()))
    }

    fn name(&self) -> &str {
        "pjlink"
    }
}
