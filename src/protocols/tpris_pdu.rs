// 深圳特普瑞斯科技有限公司 PDU (Power Distribution Unit) 协议实现
// 基于 Modbus TCP，支持 8 路开关控制
//
// 协议说明：
// - 读取8位开关状态: 功能码 03, 寄存器地址 0x0030, 读1个寄存器
//   返回值低字节为位掩码, bit0=开关1, bit7=开关8, 1=开 0=关
//
// - 批量写入8位开关 (组合命令): 功能码 06, 寄存器地址 0x0030
//   值为位掩码, bit0=开关1, bit7=开关8, 1=开 0=关
//   例: 1234开启5678关闭 -> 值=0x0F (二进制 00001111)
//
// - 单独控制开关: 功能码 06, 寄存器地址 0x0034
//   值的高字节=开关编号(1-8), 低字节=动作(01=关闭, 02=开启)
//   例: 第1个开关关闭 -> 值=0x0101, 第1个开关开启 -> 值=0x0102

use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio_modbus::prelude::*;
use tracing::{debug, info};

use crate::protocols::Protocol;
use crate::utils::{DeviceError, Result};

// 寄存器地址常量
/// 8位开关状态/批量控制寄存器
const ADDR_SWITCH_STATUS: u16 = 0x0030;
/// 单独开关控制寄存器
const ADDR_SWITCH_SINGLE: u16 = 0x0034;

/// 单独控制动作码
const ACTION_OFF: u8 = 0x01;
const ACTION_ON: u8 = 0x02;

/// 特普瑞斯 PDU 协议
pub struct TprisPduProtocol {
    addr: String,
    port: u16,
    slave_id: u8,
}

impl TprisPduProtocol {
    pub fn new(addr: String, port: u16, slave_id: u8) -> Self {
        Self {
            addr,
            port,
            slave_id,
        }
    }

    /// 创建 Modbus TCP 连接
    async fn connect(&self) -> Result<client::Context> {
        let socket_addr = format!("{}:{}", self.addr, self.port);
        debug!("连接到 Tpris PDU: {}", socket_addr);

        let socket_addr = socket_addr
            .parse()
            .map_err(|e| DeviceError::ConfigError(format!("无效的地址: {}", e)))?;

        let ctx = tcp::connect_slave(socket_addr, Slave(self.slave_id))
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("Modbus TCP 连接失败: {}", e)))?;

        Ok(ctx)
    }

    /// 读取8位开关状态
    ///
    /// 读取寄存器 0x0030，返回值低字节解析为8位开关状态
    /// bit0 = 开关1, bit1 = 开关2, ..., bit7 = 开关8
    /// 1 = 开启, 0 = 关闭
    async fn read_switch_status(&self, addr: u16) -> Result<(u8, HashMap<String, bool>)> {
        let mut ctx = self.connect().await?;

        let registers = ctx
            .read_holding_registers(addr, 1)
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("读取开关状态失败: {}", e)))?
            .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

        let raw_value = (registers[0] & 0xFF) as u8;

        let mut switches = HashMap::new();
        for i in 0..8u8 {
            let is_on = (raw_value >> i) & 1 == 1;
            switches.insert(format!("{}", i + 1), is_on);
        }

        info!(
            "读取开关状态成功: raw=0x{:02X}, switches={:?}",
            raw_value, switches
        );

        Ok((raw_value, switches))
    }

    /// 批量写入8位开关 (组合命令)
    ///
    /// 写入寄存器 0x0030，值为8位位掩码
    /// bit0 = 开关1, bit1 = 开关2, ..., bit7 = 开关8
    /// 1 = 开启, 0 = 关闭
    ///
    /// 例: 开关1-4开启, 5-8关闭 -> value = 0x0F
    async fn write_switch_all(&self, addr: u16, value: u16) -> Result<()> {
        let mut ctx = self.connect().await?;

        ctx.write_single_register(addr, value)
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("批量写入开关失败: {}", e)))?
            .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

        info!(
            "批量写入开关成功: value=0x{:04X} (binary={:08b})",
            value, value
        );

        Ok(())
    }

    /// 单独控制某个开关
    ///
    /// 写入寄存器 0x0034，值的高字节=开关编号(1-8)，低字节=动作码
    /// 动作码: 0x01 = 关闭, 0x02 = 开启
    async fn write_switch_single(&self, addr: u16, switch_id: u8, on: bool) -> Result<()> {
        if switch_id < 1 || switch_id > 8 {
            return Err(DeviceError::ConfigError(
                "开关编号必须在1-8之间".to_string(),
            ));
        }

        let action = if on { ACTION_ON } else { ACTION_OFF };
        let value = ((switch_id as u16) << 8) | (action as u16);

        let mut ctx = self.connect().await?;

        ctx.write_single_register(addr, value)
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("单独控制开关失败: {}", e)))?
            .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

        info!(
            "单独控制开关成功: switch_id={}, action={}, value=0x{:04X}",
            switch_id,
            if on { "开启" } else { "关闭" },
            value
        );

        Ok(())
    }
}

#[async_trait]
impl Protocol for TprisPduProtocol {
    fn from_config(_channel_id: u32, params: &HashMap<String, Value>) -> Result<Box<dyn Protocol>>
    where
        Self: Sized,
    {
        let addr = params
            .get("addr")
            .or_else(|| params.get("ip"))
            .and_then(|v| v.as_str())
            .ok_or(DeviceError::ConfigError("缺少 addr 参数".to_string()))?
            .to_string();

        let port = params.get("port").and_then(|v| v.as_u64()).unwrap_or(502) as u16;

        let slave_id = params.get("slave_id").and_then(|v| v.as_u64()).unwrap_or(2) as u8; // 默认 slave_id=2 (根据协议示例)

        info!(
            "创建 Tpris PDU 协议: {}:{}, slave_id={}",
            addr, port, slave_id
        );

        Ok(Box::new(Self::new(addr, port, slave_id)))
    }

    async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
        info!("Tpris PDU 执行命令: {}, 参数: {:?}", command, params);

        match command {
            // 读取8位开关状态
            "read_switch_status" => {
                let addr = params
                    .get("addr")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(ADDR_SWITCH_STATUS as u64) as u16;

                let (raw_value, switches) = self.read_switch_status(addr).await?;

                // 构建有序的开关状态对象
                let mut switch_obj = serde_json::Map::new();
                for i in 1..=8u8 {
                    let key = format!("{}", i);
                    let is_on = switches.get(&key).copied().unwrap_or(false);
                    switch_obj.insert(key, Value::Bool(is_on));
                }

                Ok(json!({
                    "status": "success",
                    "raw_value": raw_value,
                    "switches": Value::Object(switch_obj)
                }))
            }

            // 批量写入8位开关 (组合命令)
            "write_switch_all" => {
                let addr = params
                    .get("addr")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(ADDR_SWITCH_STATUS as u64) as u16;

                // 支持两种输入方式：
                // 方式1: 直接给 value 数值 (位掩码)
                // 方式2: 给 switches 对象 {"1": true, "2": false, ...}
                let value = if let Some(v) = params.get("value").and_then(|v| v.as_u64()) {
                    v as u16
                } else if let Some(switches) = params.get("switches").and_then(|v| v.as_object()) {
                    // 从 switches 对象构造位掩码
                    let mut mask: u16 = 0;
                    for i in 1..=8u8 {
                        let key = format!("{}", i);
                        if let Some(on) = switches.get(&key).and_then(|v| v.as_bool()) {
                            if on {
                                mask |= 1 << (i - 1);
                            }
                        }
                    }
                    mask
                } else {
                    return Err(DeviceError::ConfigError(
                        "缺少 value 或 switches 参数".to_string(),
                    ));
                };

                self.write_switch_all(addr, value).await?;

                Ok(json!({
                    "status": "success",
                    "value": value,
                    "binary": format!("{:08b}", value & 0xFF)
                }))
            }

            // 单独控制某个开关
            "write_switch_single" => {
                let addr = params
                    .get("addr")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(ADDR_SWITCH_SINGLE as u64) as u16;

                let switch_id = params
                    .get("switch_id")
                    .and_then(|v| v.as_u64())
                    .ok_or(DeviceError::ConfigError("缺少 switch_id 参数".to_string()))?
                    as u8;

                // 支持多种输入格式:
                // action: "on"/"off" 或 bool true/false
                let on = if let Some(action_str) = params.get("action").and_then(|v| v.as_str()) {
                    match action_str {
                        "on" | "open" | "1" => true,
                        "off" | "close" | "0" => false,
                        _ => {
                            return Err(DeviceError::ConfigError(format!(
                                "无效的 action 值: {}, 支持 on/off",
                                action_str
                            )))
                        }
                    }
                } else if let Some(action_bool) = params.get("action").and_then(|v| v.as_bool()) {
                    action_bool
                } else if let Some(on_bool) = params.get("on").and_then(|v| v.as_bool()) {
                    on_bool
                } else {
                    return Err(DeviceError::ConfigError(
                        "缺少 action 或 on 参数".to_string(),
                    ));
                };

                self.write_switch_single(addr, switch_id, on).await?;

                Ok(json!({
                    "status": "success",
                    "switch_id": switch_id,
                    "action": if on { "on" } else { "off" }
                }))
            }

            _ => Err(DeviceError::Other(format!("未知命令: {}", command))),
        }
    }

    async fn get_status(&self) -> Result<Value> {
        // 尝试连接以检查连接状态
        match self.connect().await {
            Ok(_) => Ok(json!({
                "protocol": "tpris-pdu",
                "connected": true,
                "addr": self.addr,
                "port": self.port,
                "slave_id": self.slave_id
            })),
            Err(e) => Ok(json!({
                "protocol": "tpris-pdu",
                "connected": false,
                "addr": self.addr,
                "port": self.port,
                "slave_id": self.slave_id,
                "error": e.to_string()
            })),
        }
    }

    async fn write(&mut self, id: u32, value: i32) -> Result<()> {
        // id: 开关编号 (1-8)
        // value: 1=开启, 0=关闭
        if id < 1 || id > 8 {
            return Err(DeviceError::ConfigError(format!(
                "开关编号必须在1-8之间，当前: {}",
                id
            )));
        }

        let on = value > 0;
        self.write_switch_single(ADDR_SWITCH_SINGLE, id as u8, on)
            .await
    }

    async fn read(&self, id: u32) -> Result<i32> {
        // id: 开关编号 (1-8)
        // 返回: 1=开, 0=关
        if id < 1 || id > 8 {
            return Err(DeviceError::ConfigError(format!(
                "开关编号必须在1-8之间，当前: {}",
                id
            )));
        }

        let (raw_value, _) = self.read_switch_status(ADDR_SWITCH_STATUS).await?;
        let bit = (raw_value >> (id - 1)) & 1;
        Ok(bit as i32)
    }

    fn name(&self) -> &str {
        "tpris-pdu"
    }

    fn get_methods(&self) -> Vec<String> {
        vec![
            "read_switch_status".to_string(),
            "write_switch_all".to_string(),
            "write_switch_single".to_string(),
        ]
    }
}
