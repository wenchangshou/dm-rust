// Novastar LED 控制器通讯协议
// 支持 TCP 和 RS232 两种通信方式
// TCP 端口: 15200
// RS232: 115200 8N1

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_serial::SerialPortBuilderExt;
use tracing::{debug, info, warn};

use crate::protocols::Protocol;
use crate::utils::error::DeviceError;

// 协议常量
const FRAME_HEADER: [u8; 2] = [0x55, 0xAA]; // 帧头
const FRAME_TAIL: u8 = 0x56; // 帧尾
const RESPONSE_HEADER: [u8; 2] = [0xAA, 0x55]; // 响应帧头

const TCP_PORT: u16 = 15200;
const RS232_BAUD: u32 = 115200;

// 命令类型
const CMD_READ_MODE_ID_TCP: [u8; 20] = [
    0x55, 0xAA, 0x00, 0x00, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x57, 0x56,
];

const CMD_READ_MODE_ID_RS232: [u8; 20] = [
    0x55, 0xAA, 0x00, 0x14, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x02,
    0x02, 0x00, 0x6D, 0x56,
];

// 场景加载基础命令（不含场景号和校验和）
const CMD_LOAD_SCENE_BASE: [u8; 18] = [
    0x55, 0xAA, 0x00, 0x00, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x51, 0x13,
    0x01, 0x00,
];

// 场景加载成功响应
const RESP_LOAD_SCENE_SUCCESS: [u8; 20] = [
    0xAA, 0x55, 0x00, 0x00, 0x00, 0xFE, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x51, 0x13,
    0x00, 0x00, 0xB9, 0x56,
];

/// 通信方式枚举
#[derive(Debug, Clone)]
enum ConnectionType {
    Tcp { addr: String, port: u16 },
    Serial { port_name: String, baud_rate: u32 },
}

pub struct NovastarProtocol {
    connection_type: ConnectionType,
}

impl NovastarProtocol {
    pub fn new_tcp(addr: String, port: u16) -> Self {
        Self {
            connection_type: ConnectionType::Tcp { addr, port },
        }
    }

    pub fn new_serial(port_name: String, baud_rate: u32) -> Self {
        Self {
            connection_type: ConnectionType::Serial {
                port_name,
                baud_rate,
            },
        }
    }

    /// 计算校验和
    /// SUM = data[2..] + 0x5555 (从第3个字节到末尾)
    /// SUM_L = SUM & 0xFF (低8位)
    /// SUM_H = (SUM >> 8) & 0xFF (高8位)
    fn calculate_checksum(data: &[u8]) -> (u8, u8) {
        let mut sum: u16 = 0x5555;
        for &byte in &data[2..] {
            // 从第3个字节开始计算到末尾（调用时还未添加校验和）
            sum = sum.wrapping_add(byte as u16);
        }
        let sum_l = (sum & 0xFF) as u8;
        let sum_h = ((sum >> 8) & 0xFF) as u8;
        (sum_l, sum_h)
    }

    /// 构建场景加载命令
    fn build_load_scene_command(scene_id: u8) -> Result<Vec<u8>> {
        println!("构建场景{}加载命令", scene_id);
        if scene_id < 1 || scene_id > 10 {
            return Err(DeviceError::Other("场景编号必须在1-10之间".to_string()).into());
        }

        let mut command = Vec::with_capacity(21);
        command.extend_from_slice(&CMD_LOAD_SCENE_BASE);
        command.push(scene_id - 1); // 场景号从0开始，0x00-0x09 表示 1-10

        // 计算校验和
        let (sum_l, sum_h) = Self::calculate_checksum(&command);
        command.push(sum_l);
        command.push(sum_h);
        // 注意：FRAME_TAIL (0x56) 已包含在校验和计算中，不需要额外添加

        debug!("构建场景{}加载命令: {:02X?}", scene_id, command);
        Ok(command)
    }

    /// 发送命令并接收响应 (TCP)
    async fn send_command_tcp(&self, addr: &str, port: u16, command: &[u8]) -> Result<Vec<u8>> {
        debug!("连接到 TCP 设备: {}:{}", addr, port);

        let mut stream = match tokio::time::timeout(
            Duration::from_secs(5),
            TcpStream::connect(format!("{}:{}", addr, port)),
        )
        .await
        {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                warn!("TCP 连接失败: {}", e);
                return Err(DeviceError::ConnectionError(format!("连接失败: {}", e)).into());
            }
            Err(_) => {
                warn!("TCP 连接超时");
                return Err(DeviceError::Other("TCP 连接超时 (5秒)".to_string()).into());
            }
        };

        info!("TCP 连接成功");
        debug!("发送命令: {:02X?}", command);

        stream.write_all(command).await?;
        stream.flush().await?;

        // 读取响应 (超时 3秒)
        let mut response = vec![0u8; 20]; // Novastar 响应通常是20字节
        match tokio::time::timeout(Duration::from_secs(3), stream.read_exact(&mut response)).await {
            Ok(Ok(_)) => {
                debug!("接收响应: {:02X?}", response);
                Ok(response)
            }
            Ok(Err(e)) => {
                warn!("读取响应失败: {}", e);
                Err(DeviceError::ConnectionError(format!("读取响应失败: {}", e)).into())
            }
            Err(_) => {
                warn!("读取响应超时");
                Err(
                    DeviceError::Other("设备响应超时 (3秒), 请检查设备连接和供电".to_string())
                        .into(),
                )
            }
        }
    }

    /// 发送命令并接收响应 (RS232)
    async fn send_command_serial(
        &self,
        port_name: &str,
        baud_rate: u32,
        command: &[u8],
    ) -> Result<Vec<u8>> {
        debug!("打开串口: {}, 波特率: {}", port_name, baud_rate);

        let mut stream = tokio_serial::new(port_name, baud_rate)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .timeout(Duration::from_millis(1000))
            .open_native_async()?;

        info!("串口连接成功");
        debug!("发送命令: {:02X?}", command);

        stream.write_all(command).await?;
        stream.flush().await?;

        // 读取响应 (超时 3秒)
        let mut response = vec![0u8; 20];
        match tokio::time::timeout(Duration::from_secs(3), stream.read_exact(&mut response)).await {
            Ok(Ok(_)) => {
                debug!("接收响应: {:02X?}", response);
                Ok(response)
            }
            Ok(Err(e)) => {
                warn!("读取响应失败: {}", e);
                Err(DeviceError::ConnectionError(format!("读取响应失败: {}", e)).into())
            }
            Err(_) => {
                warn!("读取响应超时");
                Err(
                    DeviceError::Other("设备响应超时 (3秒), 请检查串口连接和设备供电".to_string())
                        .into(),
                )
            }
        }
    }

    /// 发送命令 (自动选择通信方式)
    async fn send_command(&self, command: &[u8]) -> Result<Vec<u8>> {
        match &self.connection_type {
            ConnectionType::Tcp { addr, port } => self.send_command_tcp(addr, *port, command).await,
            ConnectionType::Serial {
                port_name,
                baud_rate,
            } => {
                self.send_command_serial(port_name, *baud_rate, command)
                    .await
            }
        }
    }

    /// 读取设备 Mode ID
    pub async fn read_mode_id(&self) -> Result<Vec<u8>> {
        let command = match &self.connection_type {
            ConnectionType::Tcp { .. } => &CMD_READ_MODE_ID_TCP[..],
            ConnectionType::Serial { .. } => &CMD_READ_MODE_ID_RS232[..],
        };

        let response = self.send_command(command).await?;

        // 验证响应帧头
        if response.len() >= 2 && &response[0..2] == RESPONSE_HEADER {
            info!("读取 Mode ID 成功");
            Ok(response)
        } else {
            warn!("Mode ID 响应格式错误: {:02X?}", response);
            Err(DeviceError::ProtocolError(format!("响应格式错误: {:02X?}", response)).into())
        }
    }

    /// 加载场景 (1-10)
    pub async fn load_scene(&self, scene_id: u8) -> Result<bool> {
        if scene_id < 1 || scene_id > 10 {
            return Err(DeviceError::Other("场景编号必须在1-10之间".to_string()).into());
        }

        let command = Self::build_load_scene_command(scene_id)?;
        println!("Novastar load scene: {:02X?}", command);
        let response = self.send_command(&command).await?;

        // 检查响应是否为成功
        if response == RESP_LOAD_SCENE_SUCCESS {
            info!("场景 {} 加载成功", scene_id);
            Ok(true)
        } else if response.len() >= 2 && &response[0..2] == RESPONSE_HEADER {
            warn!("场景 {} 加载失败，响应: {:02X?}", scene_id, response);
            Ok(false)
        } else {
            warn!("场景 {} 加载响应格式错误: {:02X?}", scene_id, response);
            Err(DeviceError::ProtocolError(format!("响应格式错误: {:02X?}", response)).into())
        }
    }

    /// 执行自定义命令
    pub async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
        info!("执行 Novastar 命令: {}, 参数: {:?}", command, params);

        match command {
            "read_mode_id" => {
                let response = self.read_mode_id().await?;
                Ok(json!({
                    "success": true,
                    "mode_id": format!("{:02X?}", response)
                }))
            }
            "load_scene" => {
                let scene_id = params["scene_id"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少 scene_id 参数".to_string()))?
                    as u8;

                let success = self.load_scene(scene_id).await?;
                Ok(json!({
                    "success": success,
                    "message": if success {
                        format!("场景 {} 加载成功", scene_id)
                    } else {
                        format!("场景 {} 加载失败", scene_id)
                    }
                }))
            }
            _ => Err(DeviceError::Other(format!("未知命令: {}", command)).into()),
        }
    }
}

#[async_trait]
impl Protocol for NovastarProtocol {
    fn from_config(
        _channel_id: u32,
        params: &HashMap<String, Value>,
    ) -> crate::utils::Result<Box<dyn Protocol>>
    where
        Self: Sized,
    {
        // 检查是使用 TCP 还是 RS232
        let use_tcp = params
            .get("use_tcp")
            .and_then(|v| v.as_bool())
            .unwrap_or(true); // 默认使用 TCP

        if use_tcp {
            // TCP 模式
            let addr = params
                .get("addr")
                .or_else(|| params.get("ip"))
                .and_then(|v| v.as_str())
                .ok_or(DeviceError::ConfigError("缺少 addr 或 ip 参数".to_string()))?
                .to_string();

            let port = params
                .get("port")
                .and_then(|v| v.as_u64())
                .unwrap_or(TCP_PORT as u64) as u16;

            info!("创建 Novastar TCP 协议: {}:{}", addr, port);
            Ok(Box::new(Self::new_tcp(addr, port)))
        } else {
            // RS232 模式
            let port_name = params
                .get("port_name")
                .or_else(|| params.get("serial_port"))
                .and_then(|v| v.as_str())
                .ok_or(DeviceError::ConfigError(
                    "缺少 port_name 或 serial_port 参数".to_string(),
                ))?
                .to_string();

            let baud_rate = params
                .get("baud_rate")
                .and_then(|v| v.as_u64())
                .unwrap_or(RS232_BAUD as u64) as u32;

            info!(
                "创建 Novastar RS232 协议: {}, 波特率: {}",
                port_name, baud_rate
            );
            Ok(Box::new(Self::new_serial(port_name, baud_rate)))
        }
    }

    async fn execute(&mut self, command: &str, params: Value) -> crate::utils::Result<Value> {
        NovastarProtocol::execute(self, command, params)
            .await
            .map_err(|e| DeviceError::Other(e.to_string()))
    }

    async fn get_status(&self) -> crate::utils::Result<Value> {
        // 通过读取 Mode ID 来检查设备状态
        match self.connection_type {
            ConnectionType::Tcp { ref addr, port } => Ok(json!({
                "connection_type": "TCP",
                "addr": addr,
                "port": port,
                "online": true
            })),
            ConnectionType::Serial {
                ref port_name,
                baud_rate,
            } => Ok(json!({
                "connection_type": "RS232",
                "port_name": port_name,
                "baud_rate": baud_rate,
                "online": true
            })),
        }
    }

    async fn write(&mut self, id: u32, value: i32) -> crate::utils::Result<()> {
        println!("Novastar write: id: {}, value: {}", id, value);

        if value < 1 || value > 10 {
            return Err(DeviceError::Other("场景编号必须在1-10之间".to_string()));
        }

        if id == 1 {
            self.load_scene((value - 1) as u8)
                .await
                .map_err(|e| DeviceError::Other(e.to_string()))?;
        }
        Ok(())
    }

    async fn read(&self, _id: u32) -> crate::utils::Result<i32> {
        // Novastar 不支持读取当前场景状态
        Err(DeviceError::Other(
            "Novastar 协议不支持读取场景状态".to_string(),
        ))
    }

    fn name(&self) -> &str {
        "Novastar"
    }

    async fn call_method(&mut self, method_name: &str, args: Value) -> crate::utils::Result<Value> {
        NovastarProtocol::execute(self, method_name, args)
            .await
            .map_err(|e| DeviceError::Other(e.to_string()))
    }

    fn get_methods(&self) -> Vec<String> {
        vec!["read_mode_id".to_string(), "load_scene".to_string()]
    }
}
