//! Mock 协议 - 用于调试和测试
//!
//! 这是一个模拟设备协议，支持：
//! - 模拟读写操作
//! - 内存状态存储
//! - 延迟模拟
//! - 错误模拟
//! - 自定义方法调用
//!
//! # 配置示例
//! ```json
//! {
//!   "type": "mock",
//!   "delay_ms": 100,        // 可选，模拟延迟（毫秒）
//!   "error_rate": 0.0,      // 可选，错误率（0.0-1.0）
//!   "initial_values": {     // 可选，初始值
//!     "1": 100,
//!     "2": 200
//!   }
//! }
//! ```
//!
//! # 支持的命令
//! - `ping`: 测试连接
//! - `reset`: 重置所有值为0
//! - `set_error_rate`: 设置错误率
//! - `get_all_values`: 获取所有存储的值
//! - `batch_write`: 批量写入
//! - `batch_read`: 批量读取
//!
//! # 自定义方法
//! - `simulate_fault`: 模拟设备故障
//! - `clear_fault`: 清除故障状态
//! - `get_statistics`: 获取统计信息

use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use rand::Rng;

use crate::protocols::Protocol;
use crate::utils::{DeviceError, Result};

/// Mock 协议内部状态
#[derive(Debug, Clone)]
struct MockState {
    /// 存储的值（地址 -> 值）
    values: HashMap<u32, i32>,
    /// 是否处于故障状态
    fault: bool,
    /// 统计信息
    read_count: u64,
    write_count: u64,
    error_count: u64,
}

impl MockState {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
            fault: false,
            read_count: 0,
            write_count: 0,
            error_count: 0,
        }
    }
}

/// Mock 协议实现
pub struct MockProtocol {
    channel_id: u32,
    delay_ms: u64,
    error_rate: f64,
    state: Arc<Mutex<MockState>>,
}

impl MockProtocol {
    /// 创建新的 Mock 协议实例
    pub fn new(channel_id: u32) -> Self {
        Self {
            channel_id,
            delay_ms: 0,
            error_rate: 0.0,
            state: Arc::new(Mutex::new(MockState::new())),
        }
    }

    /// 模拟延迟
    async fn simulate_delay(&self) {
        if self.delay_ms > 0 {
            sleep(Duration::from_millis(self.delay_ms)).await;
        }
    }

    /// 检查是否应该模拟错误
    fn should_simulate_error(&self) -> bool {
        if self.error_rate <= 0.0 {
            return false;
        }
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < self.error_rate
    }

    /// 检查设备故障状态
    fn check_fault(&self) -> Result<()> {
        let state = self.state.lock().unwrap();
        if state.fault {
            Err(DeviceError::Other("设备处于故障状态".to_string()))
        } else {
            Ok(())
        }
    }

    /// 记录错误
    fn record_error(&self) {
        let mut state = self.state.lock().unwrap();
        state.error_count += 1;
    }
}

#[async_trait]
impl Protocol for MockProtocol {
    fn from_config(channel_id: u32, params: &HashMap<String, Value>) -> Result<Box<dyn Protocol>> {
        let mut protocol = Self::new(channel_id);

        // 解析延迟配置
        if let Some(delay) = params.get("delay_ms") {
            if let Some(d) = delay.as_u64() {
                protocol.delay_ms = d;
            }
        }

        // 解析错误率配置
        if let Some(error_rate) = params.get("error_rate") {
            if let Some(er) = error_rate.as_f64() {
                protocol.error_rate = er.clamp(0.0, 1.0);
            }
        }

        // 解析初始值
        if let Some(initial_values) = params.get("initial_values") {
            if let Some(obj) = initial_values.as_object() {
                let mut state = protocol.state.lock().unwrap();
                for (key, value) in obj {
                    if let Ok(addr) = key.parse::<u32>() {
                        if let Some(val) = value.as_i64() {
                            state.values.insert(addr, val as i32);
                        }
                    }
                }
            }
        }

        tracing::info!(
            "Mock 协议初始化 [通道{}] 延迟:{}ms 错误率:{:.2}%",
            channel_id,
            protocol.delay_ms,
            protocol.error_rate * 100.0
        );

        Ok(Box::new(protocol))
    }

    async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
        self.simulate_delay().await;
        self.check_fault()?;

        if self.should_simulate_error() {
            self.record_error();
            return Err(DeviceError::Other(format!("模拟错误: 命令 '{}' 执行失败", command)));
        }

        match command {
            "ping" => {
                Ok(json!({
                    "status": "ok",
                    "message": "pong",
                    "channel_id": self.channel_id
                }))
            }
            "reset" => {
                let mut state = self.state.lock().unwrap();
                state.values.clear();
                Ok(json!({
                    "status": "ok",
                    "message": "所有值已重置"
                }))
            }
            "set_error_rate" => {
                if let Some(rate) = params.get("rate").and_then(|v| v.as_f64()) {
                    self.error_rate = rate.clamp(0.0, 1.0);
                    Ok(json!({
                        "status": "ok",
                        "error_rate": self.error_rate
                    }))
                } else {
                    Err(DeviceError::Other("需要参数 'rate' (0.0-1.0)".to_string()))
                }
            }
            "get_all_values" => {
                let state = self.state.lock().unwrap();
                Ok(json!({
                    "status": "ok",
                    "values": state.values
                }))
            }
            "batch_write" => {
                if let Some(writes) = params.get("writes").and_then(|v| v.as_array()) {
                    let mut state = self.state.lock().unwrap();
                    let mut count = 0;
                    for item in writes {
                        if let (Some(addr), Some(value)) = (
                            item.get("addr").and_then(|v| v.as_u64()),
                            item.get("value").and_then(|v| v.as_i64())
                        ) {
                            state.values.insert(addr as u32, value as i32);
                            count += 1;
                        }
                    }
                    state.write_count += count;
                    Ok(json!({
                        "status": "ok",
                        "written": count
                    }))
                } else {
                    Err(DeviceError::Other("需要参数 'writes' 数组".to_string()))
                }
            }
            "batch_read" => {
                if let Some(addrs) = params.get("addrs").and_then(|v| v.as_array()) {
                    let mut state = self.state.lock().unwrap();
                    let mut results = Vec::new();
                    for addr_val in addrs {
                        if let Some(addr) = addr_val.as_u64() {
                            let value = state.values.get(&(addr as u32)).copied().unwrap_or(0);
                            results.push(json!({
                                "addr": addr,
                                "value": value
                            }));
                        }
                    }
                    state.read_count += results.len() as u64;
                    Ok(json!({
                        "status": "ok",
                        "results": results
                    }))
                } else {
                    Err(DeviceError::Other("需要参数 'addrs' 数组".to_string()))
                }
            }
            _ => Err(DeviceError::Other(format!("不支持的命令: {}", command))),
        }
    }

    async fn get_status(&self) -> Result<Value> {
        self.simulate_delay().await;

        let state = self.state.lock().unwrap();
        Ok(json!({
            "connected": !state.fault,
            "channel_id": self.channel_id,
            "fault": state.fault,
            "delay_ms": self.delay_ms,
            "error_rate": self.error_rate,
            "statistics": {
                "read_count": state.read_count,
                "write_count": state.write_count,
                "error_count": state.error_count,
                "stored_values": state.values.len()
            }
        }))
    }

    async fn write(&mut self, id: u32, value: i32) -> Result<()> {
        self.simulate_delay().await;
        self.check_fault()?;

        if self.should_simulate_error() {
            self.record_error();
            return Err(DeviceError::Other(format!("模拟错误: 写入地址 {} 失败", id)));
        }

        let mut state = self.state.lock().unwrap();
        state.values.insert(id, value);
        state.write_count += 1;

        tracing::debug!("Mock 写入 [通道{}] 地址:{} 值:{}", self.channel_id, id, value);
        Ok(())
    }

    async fn read(&self, id: u32) -> Result<i32> {
        self.simulate_delay().await;
        self.check_fault()?;

        if self.should_simulate_error() {
            self.record_error();
            return Err(DeviceError::Other(format!("模拟错误: 读取地址 {} 失败", id)));
        }

        let mut state = self.state.lock().unwrap();
        let value = state.values.get(&id).copied().unwrap_or(0);
        state.read_count += 1;

        tracing::debug!("Mock 读取 [通道{}] 地址:{} 值:{}", self.channel_id, id, value);
        Ok(value)
    }

    fn name(&self) -> &str {
        "mock"
    }

    async fn call_method(&mut self, method_name: &str, args: Value) -> Result<Value> {
        self.simulate_delay().await;

        match method_name {
            "simulate_fault" => {
                let mut state = self.state.lock().unwrap();
                state.fault = true;
                Ok(json!({
                    "status": "ok",
                    "message": "设备故障已模拟"
                }))
            }
            "clear_fault" => {
                let mut state = self.state.lock().unwrap();
                state.fault = false;
                Ok(json!({
                    "status": "ok",
                    "message": "设备故障已清除"
                }))
            }
            "get_statistics" => {
                let state = self.state.lock().unwrap();
                Ok(json!({
                    "read_count": state.read_count,
                    "write_count": state.write_count,
                    "error_count": state.error_count,
                    "stored_values": state.values.len(),
                    "total_operations": state.read_count + state.write_count
                }))
            }
            "set_delay" => {
                if let Some(delay) = args.get("delay_ms").and_then(|v| v.as_u64()) {
                    self.delay_ms = delay;
                    Ok(json!({
                        "status": "ok",
                        "delay_ms": self.delay_ms
                    }))
                } else {
                    Err(DeviceError::Other("需要参数 'delay_ms'".to_string()))
                }
            }
            "get_value" => {
                if let Some(addr) = args.get("addr").and_then(|v| v.as_u64()) {
                    let state = self.state.lock().unwrap();
                    let value = state.values.get(&(addr as u32)).copied().unwrap_or(0);
                    Ok(json!({
                        "addr": addr,
                        "value": value
                    }))
                } else {
                    Err(DeviceError::Other("需要参数 'addr'".to_string()))
                }
            }
            "set_value" => {
                if let (Some(addr), Some(value)) = (
                    args.get("addr").and_then(|v| v.as_u64()),
                    args.get("value").and_then(|v| v.as_i64())
                ) {
                    let mut state = self.state.lock().unwrap();
                    state.values.insert(addr as u32, value as i32);
                    Ok(json!({
                        "status": "ok",
                        "addr": addr,
                        "value": value
                    }))
                } else {
                    Err(DeviceError::Other("需要参数 'addr' 和 'value'".to_string()))
                }
            }
            _ => Err(DeviceError::Other(format!("不支持的方法: {}", method_name))),
        }
    }

    fn get_methods(&self) -> Vec<String> {
        vec![
            "simulate_fault".to_string(),
            "clear_fault".to_string(),
            "get_statistics".to_string(),
            "set_delay".to_string(),
            "get_value".to_string(),
            "set_value".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_protocol_basic() {
        let mut protocol = MockProtocol::new(1);

        // 测试写入
        protocol.write(100, 42).await.unwrap();

        // 测试读取
        let value = protocol.read(100).await.unwrap();
        assert_eq!(value, 42);
    }

    #[tokio::test]
    async fn test_mock_protocol_commands() {
        let params = HashMap::new();
        let mut protocol = MockProtocol::from_config(1, &params).unwrap();

        // 测试 ping
        let result = protocol.execute("ping", json!({})).await.unwrap();
        assert_eq!(result["status"], "ok");

        // 测试批量写入
        let result = protocol.execute("batch_write", json!({
            "writes": [
                {"addr": 1, "value": 100},
                {"addr": 2, "value": 200}
            ]
        })).await.unwrap();
        assert_eq!(result["written"], 2);

        // 测试批量读取
        let result = protocol.execute("batch_read", json!({
            "addrs": [1, 2]
        })).await.unwrap();
        assert_eq!(result["results"][0]["value"], 100);
        assert_eq!(result["results"][1]["value"], 200);
    }

    #[tokio::test]
    async fn test_mock_protocol_fault_simulation() {
        let params = HashMap::new();
        let mut protocol = MockProtocol::from_config(1, &params).unwrap();

        // 模拟故障
        protocol.call_method("simulate_fault", json!({})).await.unwrap();

        // 此时读写应该失败
        assert!(protocol.read(1).await.is_err());
        assert!(protocol.write(1, 100).await.is_err());

        // 清除故障
        protocol.call_method("clear_fault", json!({})).await.unwrap();

        // 此时读写应该成功
        assert!(protocol.write(1, 100).await.is_ok());
        assert!(protocol.read(1).await.is_ok());
    }
}
