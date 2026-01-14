// 3D拼接处理器通讯协议
// 支持 TCP 和 串口 两种通信方式
// TCP 端口: 默认 5000
// 串口: 115200 8N1 无校验
//
// 功能：切换场景 (SetPreset)
// 指令格式: /SetPreset:d,{scene_id},{group};
// 成功返回: /ack:d,1;
// 失败返回: /ack:d,0;

use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_serial::SerialPortBuilderExt;
use tracing::{debug, info, warn};

use crate::protocols::Protocol;
use crate::utils::error::DeviceError;

// 协议常量
const DEFAULT_TCP_PORT: u16 = 5000;
const DEFAULT_UDP_PORT: u16 = 5002;
const DEFAULT_BAUD_RATE: u32 = 115200;
const DEFAULT_GROUP: u32 = 1;

/// 通信方式枚举
#[derive(Debug, Clone)]
enum ConnectionType {
    Tcp { addr: String, port: u16 },
    Udp {
        addr: String,
        port: u16,
        local_port: Option<u16>,
    },
    Serial { port_name: String, baud_rate: u32 },
}

/// 3D拼接处理器协议
pub struct Splicer3dProtocol {
    connection_type: ConnectionType,
    /// 用户组 (group)
    group: u32,
    /// 当前场景值
    current_scene: AtomicI32,
}

impl Splicer3dProtocol {
    /// 创建 TCP 连接的协议实例
    pub fn new_tcp(addr: String, port: u16, group: u32) -> Self {
        Self {
            connection_type: ConnectionType::Tcp { addr, port },
            group,
            current_scene: AtomicI32::new(0),
        }
    }

    /// 创建 UDP 连接的协议实例
    pub fn new_udp(addr: String, port: u16, group: u32, local_port: Option<u16>) -> Self {
        Self {
            connection_type: ConnectionType::Udp {
                addr,
                port,
                local_port,
            },
            group,
            current_scene: AtomicI32::new(0),
        }
    }

    /// 创建串口连接的协议实例
    pub fn new_serial(port_name: String, baud_rate: u32, group: u32) -> Self {
        Self {
            connection_type: ConnectionType::Serial {
                port_name,
                baud_rate,
            },
            group,
            current_scene: AtomicI32::new(0),
        }
    }

    /// 构建切换场景命令
    /// 格式: /SetPreset:d,{scene_id},{group};
    fn build_set_preset_command(&self, scene_id: u32) -> String {
        format!("/SetPreset:d,{},{};", scene_id, self.group)
    }

    /// 构建设置窗口信号源命令
    /// 格式: /setWinSrc:d,{window},{slot},{interface},{type},{group};
    fn build_set_win_src_command(
        &self,
        window: u32,
        slot: u32,
        interface: u32,
        signal_type: u32,
    ) -> String {
        format!(
            "/setWinSrc:d,{},{},{},{},{};",
            window, slot, interface, signal_type, self.group
        )
    }

    /// 发送命令并接收响应 (TCP)
    async fn send_command_tcp(
        &self,
        addr: &str,
        port: u16,
        command: &str,
    ) -> Result<String, DeviceError> {
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
                return Err(DeviceError::ConnectionError(format!("连接失败: {}", e)));
            }
            Err(_) => {
                warn!("TCP 连接超时");
                return Err(DeviceError::Other("TCP 连接超时 (5秒)".to_string()));
            }
        };

        info!("TCP 连接成功");
        info!("TCP 发送数据: {}", command);

        stream
            .write_all(command.as_bytes())
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("发送命令失败: {}", e)))?;
        stream
            .flush()
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("刷新缓冲区失败: {}", e)))?;

        // 读取响应 (超时 3秒)
        let mut buffer = vec![0u8; 256];
        match tokio::time::timeout(Duration::from_secs(3), stream.read(&mut buffer)).await {
            Ok(Ok(n)) => {
                let response = String::from_utf8_lossy(&buffer[..n]).to_string();
                info!("TCP 收到数据: {}", response);
                Ok(response)
            }
            Ok(Err(e)) => {
                warn!("读取响应失败: {}", e);
                Err(DeviceError::ConnectionError(format!("读取响应失败: {}", e)))
            }
            Err(_) => {
                // 对于某些设备可能"无返回信息"，这里我们认为发送成功
                info!("TCP 收到数据超时，假设命令已执行");
                Ok(String::new())
            }
        }
    }

    /// 发送命令并接收响应 (UDP)
    async fn send_command_udp(
        &self,
        addr: &str,
        port: u16,
        local_port: Option<u16>,
        command: &str,
    ) -> Result<String, DeviceError> {
        info!("UDP 发送数据 {}:{}: {}", addr, port, command);

        let bind_addr = match local_port {
            Some(port) => format!("0.0.0.0:{}", port),
            None => "0.0.0.0:0".to_string(),
        };

        let socket = tokio::net::UdpSocket::bind(&bind_addr)
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("绑定 UDP socket 失败: {}", e)))?;
        debug!("UDP 本地绑定地址: {:?}", socket.local_addr());

        let target_addr = format!("{}:{}", addr, port);

        socket
            .send_to(command.as_bytes(), &target_addr)
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("发送 UDP 数据失败: {}", e)))?;

        // 读取响应 (超时 3秒)
        let mut buffer = vec![0u8; 1024];
        match tokio::time::timeout(Duration::from_secs(3), socket.recv_from(&mut buffer)).await {
            Ok(Ok((n, src_addr))) => {
                debug!("收到来自 {} 的 UDP 数据", src_addr);
                if n == 0 {
                    info!("UDP 收到数据长度为 0");
                    Ok(String::new())
                } else {
                    let response = String::from_utf8_lossy(&buffer[..n]).to_string();
                    info!("UDP 收到数据: {}", response);
                    Ok(response)
                }
            }
            Ok(Err(e)) => {
                warn!("读取 UDP 响应失败: {}", e);
                Err(DeviceError::ConnectionError(format!(
                    "读取 UDP 响应失败: {}",
                    e
                )))
            }
            Err(_) => {
                info!("UDP 收到数据超时，假设命令已执行");
                Ok(String::new())
            }
        }
    }

    /// 发送命令并接收响应 (串口)
    async fn send_command_serial(
        &self,
        port_name: &str,
        baud_rate: u32,
        command: &str,
    ) -> Result<String, DeviceError> {
        debug!("打开串口: {}, 波特率: {}", port_name, baud_rate);

        let mut stream = tokio_serial::new(port_name, baud_rate)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .timeout(Duration::from_millis(1000))
            .open_native_async()
            .map_err(|e| DeviceError::ConnectionError(format!("打开串口失败: {}", e)))?;

        info!("串口连接成功");
        info!("Serial 发送数据: {}", command);

        stream
            .write_all(command.as_bytes())
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("发送命令失败: {}", e)))?;
        stream
            .flush()
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("刷新缓冲区失败: {}", e)))?;

        // 读取响应 (超时 3秒)
        let mut buffer = vec![0u8; 256];
        match tokio::time::timeout(Duration::from_secs(3), stream.read(&mut buffer)).await {
            Ok(Ok(n)) => {
                let response = String::from_utf8_lossy(&buffer[..n]).to_string();
                info!("Serial 收到数据: {}", response);
                Ok(response)
            }
            Ok(Err(e)) => {
                warn!("读取响应失败: {}", e);
                Err(DeviceError::ConnectionError(format!("读取响应失败: {}", e)))
            }
            Err(_) => {
                // 对于某些设备可能"无返回信息"
                info!("Serial 收到数据超时，假设命令已执行");
                Ok(String::new())
            }
        }
    }

    /// 发送命令 (自动选择通信方式)
    async fn send_command(&self, command: &str) -> Result<String, DeviceError> {
        match &self.connection_type {
            ConnectionType::Tcp { addr, port } => self.send_command_tcp(addr, *port, command).await,
            ConnectionType::Udp {
                addr,
                port,
                local_port,
            } => {
                self.send_command_udp(addr, *port, *local_port, command)
                    .await
            }
            ConnectionType::Serial {
                port_name,
                baud_rate,
            } => {
                self.send_command_serial(port_name, *baud_rate, command)
                    .await
            }
        }
    }

    /// 解析响应，检查是否成功
    /// 成功: /ack:d,1;
    /// 失败: /ack:d,0;
    fn parse_ack_response(&self, response: &str) -> bool {
        if response.is_empty() {
            // 某些命令无返回信息，默认认为成功
            return true;
        }
        response.contains("/ack:d,1;") || response.contains("/ack:d,1")
    }

    /// 切换场景
    pub async fn set_preset(&self, scene_id: u32) -> Result<bool, DeviceError> {
        let command = self.build_set_preset_command(scene_id);
        info!("切换场景: scene_id={}, command={}", scene_id, command);

        let response = self.send_command(&command).await?;
        let success = self.parse_ack_response(&response);

        if success {
            info!("场景 {} 切换成功", scene_id);
        } else {
            warn!("场景 {} 切换失败，响应: {}", scene_id, response);
        }

        Ok(success)
    }

    /// 设置窗口信号源
    pub async fn set_win_src(
        &self,
        window: u32,
        slot: u32,
        interface: u32,
        signal_type: u32,
    ) -> Result<(), DeviceError> {
        let command = self.build_set_win_src_command(window, slot, interface, signal_type);
        info!(
            "设置窗口信号源: window={}, slot={}, interface={}, type={}, command={}",
            window, slot, interface, signal_type, command
        );

        // setWinSrc 命令无返回信息
        self.send_command(&command).await?;
        info!("窗口信号源设置命令已发送");
        Ok(())
    }
}

#[async_trait]
impl Protocol for Splicer3dProtocol {
    fn from_config(
        _channel_id: u32,
        params: &HashMap<String, Value>,
    ) -> crate::utils::Result<Box<dyn Protocol>>
    where
        Self: Sized,
    {
        // 检查通信方式 TCP / UDP / 串口
        let conn_type = params
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_lowercase());

        let use_tcp = params
            .get("use_tcp")
            .and_then(|v| v.as_bool())
            .unwrap_or(true); // 默认使用 TCP

        let use_udp = params
            .get("use_udp")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // 获取用户组
        let group = params
            .get("group")
            .and_then(|v| v.as_u64())
            .unwrap_or(DEFAULT_GROUP as u64) as u32;

        if use_udp || conn_type.as_deref() == Some("udp") {
            // UDP 模式
            let addr = params
                .get("addr")
                .or_else(|| params.get("ip"))
                .and_then(|v| v.as_str())
                .ok_or(DeviceError::ConfigError("缺少 addr 或 ip 参数".to_string()))?
                .to_string();

            let port = params
                .get("port")
                .and_then(|v| v.as_u64())
                .unwrap_or(DEFAULT_UDP_PORT as u64) as u16;

            let local_port = params
                .get("local_port")
                .or_else(|| params.get("udp_local_port"))
                .or_else(|| params.get("bind_port"))
                .and_then(|v| v.as_u64())
                .map(|v| v as u16)
                .filter(|v| *v != 0);

            if let Some(local_port) = local_port {
                info!(
                    "创建 3D拼接处理器 UDP 协议: {}:{}, group={}, local_port={}",
                    addr, port, group, local_port
                );
            } else {
                info!(
                    "创建 3D拼接处理器 UDP 协议: {}:{}, group={}",
                    addr, port, group
                );
            }
            Ok(Box::new(Self::new_udp(addr, port, group, local_port)))
        } else if use_tcp && conn_type.as_deref() != Some("serial") {
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
                .unwrap_or(DEFAULT_TCP_PORT as u64) as u16;

            info!(
                "创建 3D拼接处理器 TCP 协议: {}:{}, group={}",
                addr, port, group
            );
            Ok(Box::new(Self::new_tcp(addr, port, group)))
        } else {
            // 串口模式
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
                .unwrap_or(DEFAULT_BAUD_RATE as u64) as u32;

            info!(
                "创建 3D拼接处理器 串口协议: {}, 波特率: {}, group={}",
                port_name, baud_rate, group
            );
            Ok(Box::new(Self::new_serial(port_name, baud_rate, group)))
        }
    }

    async fn execute(&mut self, command: &str, params: Value) -> crate::utils::Result<Value> {
        info!("执行 3D拼接处理器 命令: {}, 参数: {:?}", command, params);

        match command {
            "setPreset" | "set_preset" | "loadScene" | "load_scene" => {
                let scene_id = params["scene_id"]
                    .as_u64()
                    .or_else(|| params["sceneId"].as_u64())
                    .or_else(|| params["value"].as_u64())
                    .ok_or(DeviceError::Other("缺少 scene_id 参数".to_string()))?
                    as u32;

                let success = self.set_preset(scene_id).await?;
                if success {
                    self.current_scene.store(scene_id as i32, Ordering::SeqCst);
                }
                Ok(json!({
                    "success": success,
                    "message": if success {
                        format!("场景 {} 切换成功", scene_id)
                    } else {
                        format!("场景 {} 切换失败", scene_id)
                    }
                }))
            }
            "setWinSrc" | "set_win_src" => {
                let window = params["window"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少 window 参数".to_string()))?
                    as u32;
                let slot = params["slot"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少 slot 参数".to_string()))?
                    as u32;
                let interface = params["interface"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少 interface 参数".to_string()))?
                    as u32;
                let signal_type = params["type"]
                    .as_u64()
                    .or_else(|| params["signal_type"].as_u64())
                    .ok_or(DeviceError::Other("缺少 type 参数".to_string()))?
                    as u32;

                self.set_win_src(window, slot, interface, signal_type)
                    .await?;
                Ok(json!({
                    "success": true,
                    "message": format!("窗口 {} 信号源设置成功", window)
                }))
            }
            _ => Err(DeviceError::Other(format!("未知命令: {}", command))),
        }
    }

    async fn get_status(&self) -> crate::utils::Result<Value> {
        match &self.connection_type {
            ConnectionType::Tcp { addr, port } => Ok(json!({
                "protocol": "3D拼接处理器",
                "connection_type": "TCP",
                "addr": addr,
                "port": port,
                "group": self.group,
                "current_scene": self.current_scene.load(Ordering::SeqCst),
                "online": true
            })),
            ConnectionType::Udp {
                addr,
                port,
                local_port,
            } => Ok(json!({
                "protocol": "3D拼接处理器",
                "connection_type": "UDP",
                "addr": addr,
                "port": port,
                "local_port": local_port,
                "group": self.group,
                "current_scene": self.current_scene.load(Ordering::SeqCst),
                "online": true
            })),
            ConnectionType::Serial {
                port_name,
                baud_rate,
            } => Ok(json!({
                "protocol": "3D拼接处理器",
                "connection_type": "Serial",
                "port_name": port_name,
                "baud_rate": baud_rate,
                "group": self.group,
                "current_scene": self.current_scene.load(Ordering::SeqCst),
                "online": true
            })),
        }
    }

    /// 写入操作 - 通过 value 切换场景
    /// id: 节点ID (1 = 场景控制)
    /// value: 场景编号
    async fn write(&mut self, id: u32, value: i32) -> crate::utils::Result<()> {
        info!("3D拼接处理器 write: id={}, value={}", id, value);

        if value < 1 {
            return Err(DeviceError::Other("场景编号必须大于等于1".to_string()));
        }

        // id 为 1 时切换场景
        if id == 1 {
            let success = self.set_preset(value as u32).await?;
            if success {
                self.current_scene.store(value, Ordering::SeqCst);
                Ok(())
            } else {
                Err(DeviceError::Other(format!("场景 {} 切换失败", value)))
            }
        } else {
            Err(DeviceError::Other(format!("不支持的节点ID: {}", id)))
        }
    }

    /// 读取操作 - 获取当前场景值
    async fn read(&self, id: u32) -> crate::utils::Result<i32> {
        if id == 1 {
            Ok(self.current_scene.load(Ordering::SeqCst))
        } else {
            Err(DeviceError::Other(format!("不支持的节点ID: {}", id)))
        }
    }

    fn name(&self) -> &str {
        "3D拼接处理器"
    }

    async fn call_method(&mut self, method_name: &str, args: Value) -> crate::utils::Result<Value> {
        self.execute(method_name, args).await
    }

    fn get_methods(&self) -> Vec<String> {
        vec![
            "setPreset".to_string(),
            "set_preset".to_string(),
            "loadScene".to_string(),
            "setWinSrc".to_string(),
            "set_win_src".to_string(),
        ]
    }
}
