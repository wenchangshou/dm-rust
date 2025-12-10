/// 南京龙港 PLC 屏幕控制协议
/// 基于 Modbus ASCII 变种
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

use crate::protocols::Protocol;
use crate::utils::{DeviceError, Result};

/// 协议常量
const START_BYTE: u8 = 0x3A; // ':'
const END_BYTES: [u8; 2] = [0x0D, 0x0A]; // '\r\n'
const COMMAND_LENGTH: usize = 21;
const RESPONSE_LENGTH: usize = 17;

/// 操作码
const OP_OPEN: &str = "01";
const OP_CLOSE: &str = "02";

/// 南京龙港 PLC 协议配置
#[derive(Debug)]
struct NjlgPlcConfig {
    addr: String,
    port: u16,
    timeout: u64,
}

/// 南京龙港 PLC 协议实现
pub struct ScreenNjlgPlcProtocol {
    channel_id: u32,
    addr: String,
    port: u16,
    timeout: std::time::Duration,
}

impl ScreenNjlgPlcProtocol {
    /// 构建控制命令
    /// 
    /// # 参数
    /// - device_id: 设备编号 (1-10)
    /// - operation: 操作 ("01"=开, "02"=关)
    /// 
    /// # 返回
    /// 21字节的完整命令
    /// 
    /// # 命令格式
    /// :00100B[X]000100[Y][CK]\r\n
    /// 其中:
    /// - :: 起始符
    /// - 00100B: 固定前缀
    /// - [X]: 设备编号 (0-9 对应设备1-10)
    /// - 000100: 固定部分
    /// - [Y]: 操作值 (1=开, 2=关)
    /// - [CK]: LRC校验和（2个字符）
    /// - \r\n: 结束符
    fn build_command(device_id: u32, operation: &str) -> Result<Vec<u8>> {
        if device_id < 1 || device_id > 10 {
            return Err(DeviceError::ConfigError(
                format!("设备编号必须在 1-10 之间，实际: {}", device_id)
            ));
        }

        if operation != OP_OPEN && operation != OP_CLOSE {
            return Err(DeviceError::ConfigError(
                format!("无效的操作码: {}, 应为 '01'(开) 或 '02'(关)", operation)
            ));
        }

        // 设备编号转 ASCII (0-9)
        let device_char = ((device_id - 1) as u8 + b'0') as char;
        
        // 操作码取第二个字符 (01 -> 1, 02 -> 2)
        let op_char = operation.chars().nth(1).unwrap();

        // 构建完整命令数据（不含起始符和结束符）
        // 格式: 00100B[device]000100[op]
        let data_str = format!("00100B{}000100{}", device_char, op_char);

        // 计算 LRC 校验和
        let lrc = Self::calculate_lrc_from_ascii(&data_str)?;

        // 转换为 ASCII 十六进制字符串
        let lrc_str = format!("{:02X}", lrc);

        // 完整命令: : + 数据 + 校验和 + \r\n
        let mut cmd = Vec::with_capacity(21);
        cmd.push(b':');
        cmd.extend_from_slice(data_str.as_bytes());
        cmd.extend_from_slice(lrc_str.as_bytes());
        cmd.push(b'\r');
        cmd.push(b'\n');

        debug!(
            "构建命令 - 设备{}, 操作{}: {:02X?}",
            device_id, operation, cmd
        );

        Ok(cmd)
    }

    /// 从 ASCII 十六进制字符串计算 LRC 校验和
    /// 
    /// # 参数
    /// - ascii_str: ASCII 十六进制字符串（不含起始符、校验和、结束符）
    ///   例如: "0010000B0000100001"
    /// 
    /// # 返回
    /// LRC 校验和（8位）
    /// 
    /// # 算法
    /// 1. 将 ASCII 十六进制字符串转换为字节数组
    /// 2. 对所有字节求和
    /// 3. 取反加1（二进制补码）
    /// 4. 保留低8位
    fn calculate_lrc_from_ascii(ascii_str: &str) -> Result<u8> {
        if ascii_str.len() % 2 != 0 {
            return Err(DeviceError::ConfigError(
                format!("ASCII 字符串长度必须是偶数: {}", ascii_str.len())
            ));
        }

        let mut sum: u8 = 0;

        // 每2个 ASCII 字符转换为1个字节
        for i in (0..ascii_str.len()).step_by(2) {
            let hex_byte = &ascii_str[i..i + 2];
            let byte = u8::from_str_radix(hex_byte, 16).map_err(|e| {
                DeviceError::ConfigError(format!("无效的十六进制字符串 '{}': {}", hex_byte, e))
            })?;
            sum = sum.wrapping_add(byte);
        }

        // LRC = 负的和的二进制补码 = (!sum) + 1
        let lrc = (!sum).wrapping_add(1);

        debug!(
            "LRC 计算 - 输入: {}, 字节和: 0x{:02X}, LRC: 0x{:02X}",
            ascii_str, sum, lrc
        );

        Ok(lrc)
    }

    /// 解析响应
    /// 
    /// # 参数
    /// - response: 17字节响应数据
    /// 
    /// # 返回
    /// 成功返回 true，失败返回 false
    fn parse_response(response: &[u8]) -> Result<bool> {
        if response.len() < RESPONSE_LENGTH {
            return Err(DeviceError::ProtocolError(
                format!("响应长度不足: {} < {}", response.len(), RESPONSE_LENGTH)
            ));
        }

        // 检查起始符
        if response[0] != START_BYTE {
            return Err(DeviceError::ProtocolError(
                format!("起始符错误: 0x{:02X} != 0x3A", response[0])
            ));
        }

        // 检查结束符
        if response[15] != END_BYTES[0] || response[16] != END_BYTES[1] {
            return Err(DeviceError::ProtocolError(
                "结束符错误".to_string()
            ));
        }

        // 提取数据部分 (不含起始符、校验和、结束符)
        let data_str = std::str::from_utf8(&response[1..13])
            .map_err(|e| DeviceError::ProtocolError(format!("响应解析错误: {}", e)))?;

        // 提取校验和
        let checksum_str = std::str::from_utf8(&response[13..15])
            .map_err(|e| DeviceError::ProtocolError(format!("校验和解析错误: {}", e)))?;

        // 验证校验和
        let calculated_lrc = Self::calculate_lrc_from_ascii(data_str)?;
        let received_lrc = u8::from_str_radix(checksum_str, 16)
            .map_err(|e| DeviceError::ProtocolError(format!("校验和格式错误: {}", e)))?;

        if calculated_lrc != received_lrc {
            warn!(
                "校验和不匹配 - 计算: 0x{:02X}, 接收: 0x{:02X}",
                calculated_lrc, received_lrc
            );
        }

        debug!("响应解析成功: {}", data_str);
        Ok(true)
    }

    /// 执行控制命令
    /// 
    /// # 参数
    /// - device_id: 设备编号
    /// - operation: 操作码
    async fn execute_control(&self, device_id: u32, operation: &str) -> Result<bool> {
        // 构建命令
        let command = Self::build_command(device_id, operation)?;

        // 连接设备
        let addr = format!("{}:{}", self.addr, self.port);
        debug!("连接到 {}", addr);

        let mut stream = tokio::time::timeout(
            self.timeout,
            TcpStream::connect(&addr)
        )
        .await
        .map_err(|_| DeviceError::Timeout)?
        .map_err(|e| DeviceError::ConnectionError(format!("连接失败: {}", e)))?;

        // 发送命令
        debug!("发送命令: {:02X?}", command);
        stream.write_all(&command).await
            .map_err(|e| DeviceError::ConnectionError(format!("发送失败: {}", e)))?;

        // 读取响应
        let mut response = vec![0u8; RESPONSE_LENGTH];
        let n = tokio::time::timeout(
            self.timeout,
            stream.read(&mut response)
        )
        .await
        .map_err(|_| DeviceError::Timeout)?
        .map_err(|e| DeviceError::ConnectionError(format!("读取失败: {}", e)))?;

        response.truncate(n);
        debug!("接收响应: {:02X?}", response);

        // 解析响应
        Self::parse_response(&response)
    }
}

#[async_trait]
impl Protocol for ScreenNjlgPlcProtocol {
    fn from_config(channel_id: u32, params: &HashMap<String, Value>) -> Result<Box<dyn Protocol>>
    where
        Self: Sized,
    {
        let addr = params
            .get("addr")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DeviceError::ConfigError("缺少 addr 参数".to_string()))?
            .to_string();

        let port = params
            .get("port")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| DeviceError::ConfigError("缺少 port 参数".to_string()))?
            as u16;

        let timeout_ms = params
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(3000);

        info!(
            "初始化南京龙港PLC协议 - 通道{}, 地址: {}:{}",
            channel_id, addr, port
        );

        Ok(Box::new(Self {
            channel_id,
            addr,
            port,
            timeout: std::time::Duration::from_millis(timeout_ms),
        }))
    }

    async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
        match command {
            "control" => {
                let device_id = params["device_id"]
                    .as_u64()
                    .ok_or_else(|| DeviceError::Other("缺少 device_id 参数".to_string()))?
                    as u32;

                let value = params["value"]
                    .as_i64()
                    .ok_or_else(|| DeviceError::Other("缺少 value 参数".to_string()))?;

                let operation = if value == 1 { OP_OPEN } else { OP_CLOSE };

                let success = self.execute_control(device_id, operation).await?;

                Ok(serde_json::json!({
                    "success": success,
                    "device_id": device_id,
                    "value": value
                }))
            }
            _ => Err(DeviceError::ProtocolError(format!(
                "不支持的命令: {}",
                command
            ))),
        }
    }

    async fn get_status(&self) -> Result<Value> {
        Ok(serde_json::json!({
            "protocol": "screen_njlg_plc",
            "channel_id": self.channel_id,
            "addr": format!("{}:{}", self.addr, self.port),
            "connected": true
        }))
    }

    async fn write(&mut self, device_id: u32, value: i32) -> Result<()> {
        let operation = if value == 1 { OP_OPEN } else { OP_CLOSE };
        self.execute_control(device_id, operation).await?;
        Ok(())
    }

    async fn read(&self, _device_id: u32) -> Result<i32> {
        // 该协议不支持读取状态
        Err(DeviceError::ProtocolError(
            "南京龙港PLC协议不支持读取操作".to_string()
        ))
    }

    fn name(&self) -> &str {
        "ScreenNjlgPlc"
    }

    async fn call_method(&mut self, method_name: &str, args: Value) -> Result<Value> {
        match method_name {
            "open_device" => {
                let device_id = args["device_id"]
                    .as_u64()
                    .ok_or_else(|| DeviceError::Other("缺少 device_id 参数".to_string()))?
                    as u32;

                self.execute_control(device_id, OP_OPEN).await?;

                Ok(serde_json::json!({
                    "result": "ok",
                    "device_id": device_id,
                    "action": "open"
                }))
            }
            "close_device" => {
                let device_id = args["device_id"]
                    .as_u64()
                    .ok_or_else(|| DeviceError::Other("缺少 device_id 参数".to_string()))?
                    as u32;

                self.execute_control(device_id, OP_CLOSE).await?;

                Ok(serde_json::json!({
                    "result": "ok",
                    "device_id": device_id,
                    "action": "close"
                }))
            }
            "batch_control" => {
                let devices = args["devices"]
                    .as_array()
                    .ok_or_else(|| DeviceError::Other("devices 必须是数组".to_string()))?;

                let action = args["action"]
                    .as_str()
                    .ok_or_else(|| DeviceError::Other("缺少 action 参数".to_string()))?;

                let operation = match action {
                    "open" => OP_OPEN,
                    "close" => OP_CLOSE,
                    _ => return Err(DeviceError::Other(
                        format!("无效的 action: {}, 应为 'open' 或 'close'", action)
                    )),
                };

                let mut results = Vec::new();
                for device in devices {
                    let device_id = device
                        .as_u64()
                        .ok_or_else(|| DeviceError::Other("设备ID必须是数字".to_string()))?
                        as u32;

                    match self.execute_control(device_id, operation).await {
                        Ok(_) => results.push(serde_json::json!({
                            "device_id": device_id,
                            "success": true
                        })),
                        Err(e) => results.push(serde_json::json!({
                            "device_id": device_id,
                            "success": false,
                            "error": format!("{:?}", e)
                        })),
                    }

                    // 设备间延迟（如果配置了）
                    if let Some(delay) = args.get("delay").and_then(|v| v.as_u64()) {
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    }
                }

                Ok(serde_json::json!({
                    "result": "ok",
                    "total": devices.len(),
                    "results": results
                }))
            }
            _ => Err(DeviceError::Other(format!(
                "协议 {} 不支持自定义方法: {}",
                self.name(),
                method_name
            ))),
        }
    }

    fn get_methods(&self) -> Vec<String> {
        vec![
            "open_device".to_string(),
            "close_device".to_string(),
            "batch_control".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_device1_open() {
        let cmd = ScreenNjlgPlcProtocol::build_command(1, "01").unwrap();
        let expected = b":00100B0000100013E\r\n";
        assert_eq!(cmd, expected, "设备1打开命令错误");
    }

    #[test]
    fn test_build_command_device1_close() {
        let cmd = ScreenNjlgPlcProtocol::build_command(1, "02").unwrap();
        let expected = b":00100B0000100023D\r\n";
        assert_eq!(cmd, expected, "设备1关闭命令错误");
    }

    #[test]
    fn test_build_command_device2_open() {
        let cmd = ScreenNjlgPlcProtocol::build_command(2, "01").unwrap();
        let expected = b":00100B1000100013D\r\n";
        assert_eq!(cmd, expected, "设备2打开命令错误");
    }

    #[test]
    fn test_lrc_calculation() {
        // 测试设备1打开的 LRC
        let lrc = ScreenNjlgPlcProtocol::calculate_lrc_from_ascii("00100B00001001").unwrap();
        assert_eq!(lrc, 0x3E, "LRC 计算错误");
    }

    #[test]
    fn test_invalid_device_id() {
        let result = ScreenNjlgPlcProtocol::build_command(0, "01");
        assert!(result.is_err(), "应拒绝设备ID为0");

        let result = ScreenNjlgPlcProtocol::build_command(11, "01");
        assert!(result.is_err(), "应拒绝设备ID大于10");
    }

    #[test]
    fn test_invalid_operation() {
        let result = ScreenNjlgPlcProtocol::build_command(1, "99");
        assert!(result.is_err(), "应拒绝无效操作码");
    }
}
