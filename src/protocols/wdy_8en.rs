// WDY-8EN 8路电源时序器协议实现
// 通信方式: TCP (串口转网口)
// 协议格式: 固定6字节指令帧, 帧头 0x55, 帧尾 0xAA
//
// 指令格式: [55] [5A] [ID] [CMD] [DATA] [AA]
//   - 55: 帧头
//   - 5A: 固定字节 (控制命令) / FF (查询命令)
//   - ID: 设备地址 (默认 0x00)
//   - CMD: 功能码
//   - DATA: 数据
//   - AA: 帧尾
//
// 状态查询返回 56 字节数据

use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error, info, warn};

use crate::protocols::Protocol;
use crate::utils::error::DeviceError;
use crate::utils::Result;

// 帧常量
const FRAME_HEADER: u8 = 0x55;
const FRAME_CONTROL: u8 = 0x5A;
const FRAME_QUERY: u8 = 0xFF;
const FRAME_TAIL: u8 = 0xAA;

// 功能码
const CMD_POWER: u8 = 0x09;       // 开机/关机
const CMD_SET_ID: u8 = 0xFF;      // 设置ID (特殊: 55 5A 00 FF 00 AA)
const CMD_GET_STATUS: u8 = 0xFF;  // 获取状态 (特殊: 55 FF 00 FF FF AA)

// 状态响应长度
const STATUS_RESPONSE_LEN: usize = 56;

pub struct Wdy8enProtocol {
    addr: String,
    port: u16,
    device_id: u8,
}

impl Wdy8enProtocol {
    pub fn new(addr: String, port: u16, device_id: u8) -> Self {
        Self {
            addr,
            port,
            device_id,
        }
    }

    /// 构建控制命令帧: 55 5A [ID] [CMD] [DATA] AA
    fn build_control_frame(&self, cmd: u8, data: u8) -> Vec<u8> {
        vec![FRAME_HEADER, FRAME_CONTROL, self.device_id, cmd, data, FRAME_TAIL]
    }

    /// 构建查询命令帧: 55 FF [ID] FF FF AA
    fn build_query_frame(&self) -> Vec<u8> {
        vec![FRAME_HEADER, FRAME_QUERY, self.device_id, 0xFF, 0xFF, FRAME_TAIL]
    }

    /// 发送命令并接收响应
    async fn send_command(&self, command: &[u8], expect_len: usize) -> Result<Vec<u8>> {
        let addr = format!("{}:{}", self.addr, self.port);
        debug!("连接到 WDY-8EN: {}", addr);

        let mut stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("连接失败: {}", e)))?;

        debug!("发送命令: {:02X?}", command);
        stream
            .write_all(command)
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("发送失败: {}", e)))?;
        stream.flush().await?;

        // 读取响应
        let mut response = vec![0u8; expect_len];
        let timeout = Duration::from_secs(3);

        match tokio::time::timeout(timeout, stream.read_exact(&mut response)).await {
            Ok(Ok(_)) => {
                debug!("接收响应: {:02X?}", response);
                Ok(response)
            }
            Ok(Err(e)) => {
                // 尝试读取可能不完整的响应
                let mut buf = vec![0u8; 256];
                if let Ok(Ok(n)) = tokio::time::timeout(
                    Duration::from_millis(500),
                    stream.read(&mut buf),
                ).await {
                    buf.truncate(n);
                    debug!("接收到部分响应: {:02X?}", buf);
                    return Ok(buf);
                }
                error!("读取失败: {}", e);
                Err(DeviceError::ConnectionError(format!("读取失败: {}", e)))
            }
            Err(_) => {
                warn!("响应超时");
                Err(DeviceError::Other("响应超时".to_string()))
            }
        }
    }

    /// 发送控制命令（发送和返回内容相同）
    async fn send_control(&self, cmd: u8, data: u8) -> Result<()> {
        let frame = self.build_control_frame(cmd, data);
        let response = self.send_command(&frame, 6).await?;

        // 控制命令的返回和发送内容相同
        if response == frame {
            info!("控制命令成功: cmd=0x{:02X}, data=0x{:02X}", cmd, data);
            Ok(())
        } else {
            warn!(
                "响应不匹配: 期望={:02X?}, 实际={:02X?}",
                frame, response
            );
            // 即使不完全匹配也视为成功（兼容性考虑）
            Ok(())
        }
    }

    /// 开机
    pub async fn power_on(&self) -> Result<()> {
        info!("WDY-8EN 开机");
        self.send_control(CMD_POWER, 0x01).await
    }

    /// 关机
    pub async fn power_off(&self) -> Result<()> {
        info!("WDY-8EN 关机");
        self.send_control(CMD_POWER, 0x00).await
    }

    /// 通道开 (1-8)
    pub async fn channel_on(&self, channel: u8) -> Result<()> {
        if channel < 1 || channel > 8 {
            return Err(DeviceError::ConfigError("通道号必须在1-8之间".to_string()));
        }
        info!("WDY-8EN 通道 {} 开", channel);
        self.send_control(channel, 0x01).await
    }

    /// 通道关 (1-8)
    pub async fn channel_off(&self, channel: u8) -> Result<()> {
        if channel < 1 || channel > 8 {
            return Err(DeviceError::ConfigError("通道号必须在1-8之间".to_string()));
        }
        info!("WDY-8EN 通道 {} 关", channel);
        self.send_control(channel, 0x00).await
    }

    /// 设置设备ID
    pub async fn set_device_id(&self, new_id: u8) -> Result<()> {
        info!("WDY-8EN 设置设备ID: 0x{:02X}", new_id);
        // 设置ID: 55 5A 00 FF 00 AA
        let frame = vec![FRAME_HEADER, FRAME_CONTROL, self.device_id, CMD_SET_ID, new_id, FRAME_TAIL];
        let response = self.send_command(&frame, 6).await?;

        if response == frame {
            info!("设置设备ID成功: 0x{:02X}", new_id);
        }
        Ok(())
    }

    /// 获取设备状态
    /// 返回56字节的状态数据
    pub async fn get_device_status(&self) -> Result<Value> {
        info!("WDY-8EN 获取设备状态");
        let frame = self.build_query_frame();
        let response = self.send_command(&frame, STATUS_RESPONSE_LEN).await?;

        if response.len() < STATUS_RESPONSE_LEN {
            return Err(DeviceError::ProtocolError(format!(
                "状态响应长度不足: 期望={}, 实际={}",
                STATUS_RESPONSE_LEN,
                response.len()
            )));
        }

        // 验证帧头和帧尾
        if response[0] != FRAME_HEADER || response[1] != FRAME_CONTROL {
            return Err(DeviceError::ProtocolError(format!(
                "状态响应帧头无效: {:02X} {:02X}",
                response[0], response[1]
            )));
        }
        if response[55] != FRAME_TAIL {
            return Err(DeviceError::ProtocolError(format!(
                "状态响应帧尾无效: {:02X}",
                response[55]
            )));
        }

        // 解析状态数据
        let device_id = response[2];

        // CH1-CH8 开机延时时间 (bytes 3-18, 每通道2字节, 高位在前)
        let mut on_delays = Vec::new();
        for i in 0..8 {
            let offset = 3 + i * 2;
            let delay = ((response[offset] as u16) << 8) | (response[offset + 1] as u16);
            on_delays.push(delay);
        }

        // 隔位符 FF (byte 19)

        // CH1-CH8 关机延时时间 (bytes 20-35, 每通道2字节, 高位在前)
        let mut off_delays = Vec::new();
        for i in 0..8 {
            let offset = 20 + i * 2;
            let delay = ((response[offset] as u16) << 8) | (response[offset + 1] as u16);
            off_delays.push(delay);
        }

        // 隔位符 FF (byte 36)

        // CH1-CH8 通道开关状态 (bytes 37-44)
        let mut channel_states = Vec::new();
        for i in 0..8 {
            let state = response[37 + i] != 0x00;
            channel_states.push(state);
        }

        // 隔位符 FF (byte 45)

        // 上电自启状态 (byte 46)
        let auto_start = response[46] != 0x00;

        // 隔位符 FF (byte 47)

        // 电压 (bytes 48-49, 高位在前, 单位V)
        let voltage = ((response[48] as u16) << 8) | (response[49] as u16);

        // 电流 (bytes 50-51, 高位在前, 单位mA)
        let current_ma = ((response[50] as u16) << 8) | (response[51] as u16);

        // 功率 (bytes 52-53, 高位在前, 单位W)
        let power = ((response[52] as u16) << 8) | (response[53] as u16);

        // 功率因素 (byte 54, 例如 63 = 0.99)
        let power_factor_raw = response[54];
        let power_factor = power_factor_raw as f64 / 100.0;

        info!(
            "WDY-8EN 状态: 通道={:?}, 电压={}V, 电流={}mA, 功率={}W, PF={}",
            channel_states, voltage, current_ma, power, power_factor
        );

        Ok(json!({
            "device_id": device_id,
            "channels": {
                "1": { "on": channel_states[0], "on_delay": on_delays[0], "off_delay": off_delays[0] },
                "2": { "on": channel_states[1], "on_delay": on_delays[1], "off_delay": off_delays[1] },
                "3": { "on": channel_states[2], "on_delay": on_delays[2], "off_delay": off_delays[2] },
                "4": { "on": channel_states[3], "on_delay": on_delays[3], "off_delay": off_delays[3] },
                "5": { "on": channel_states[4], "on_delay": on_delays[4], "off_delay": off_delays[4] },
                "6": { "on": channel_states[5], "on_delay": on_delays[5], "off_delay": off_delays[5] },
                "7": { "on": channel_states[6], "on_delay": on_delays[6], "off_delay": off_delays[6] },
                "8": { "on": channel_states[7], "on_delay": on_delays[7], "off_delay": off_delays[7] },
            },
            "auto_start": auto_start,
            "voltage": voltage,
            "current_ma": current_ma,
            "power_w": power,
            "power_factor": power_factor,
        }))
    }

    /// 读取指定通道状态 (从完整状态中提取)
    pub async fn read_channel_state(&self, channel: u8) -> Result<bool> {
        if channel < 1 || channel > 8 {
            return Err(DeviceError::ConfigError("通道号必须在1-8之间".to_string()));
        }

        let frame = self.build_query_frame();
        let response = self.send_command(&frame, STATUS_RESPONSE_LEN).await?;

        if response.len() < STATUS_RESPONSE_LEN {
            return Err(DeviceError::ProtocolError(format!(
                "状态响应长度不足: 期望={}, 实际={}",
                STATUS_RESPONSE_LEN,
                response.len()
            )));
        }

        // 通道状态在 bytes 37-44 (CH1-CH8)
        let idx = 37 + (channel - 1) as usize;
        let is_on = response[idx] != 0x00;

        info!("WDY-8EN 通道 {} 状态: {}", channel, if is_on { "开" } else { "关" });
        Ok(is_on)
    }
}

#[async_trait]
impl Protocol for Wdy8enProtocol {
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

        let port = params.get("port").and_then(|v| v.as_u64()).unwrap_or(4196) as u16;

        let device_id = params
            .get("device_id")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;

        info!(
            "创建 WDY-8EN 协议: {}:{}, device_id=0x{:02X}",
            addr, port, device_id
        );
        Ok(Box::new(Self::new(addr, port, device_id)))
    }

    async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
        info!("WDY-8EN 执行命令: {}, 参数: {:?}", command, params);

        match command {
            "power_on" => {
                self.power_on().await?;
                Ok(json!({"status": "success", "message": "开机成功"}))
            }
            "power_off" => {
                self.power_off().await?;
                Ok(json!({"status": "success", "message": "关机成功"}))
            }
            "channel_on" => {
                let channel = params
                    .get("channel")
                    .and_then(|v| v.as_u64())
                    .ok_or(DeviceError::ConfigError("缺少 channel 参数".to_string()))?
                    as u8;
                self.channel_on(channel).await?;
                Ok(json!({"status": "success", "channel": channel, "action": "on"}))
            }
            "channel_off" => {
                let channel = params
                    .get("channel")
                    .and_then(|v| v.as_u64())
                    .ok_or(DeviceError::ConfigError("缺少 channel 参数".to_string()))?
                    as u8;
                self.channel_off(channel).await?;
                Ok(json!({"status": "success", "channel": channel, "action": "off"}))
            }
            "set_device_id" => {
                let new_id = params
                    .get("id")
                    .and_then(|v| v.as_u64())
                    .ok_or(DeviceError::ConfigError("缺少 id 参数".to_string()))?
                    as u8;
                self.set_device_id(new_id).await?;
                Ok(json!({"status": "success", "new_id": new_id}))
            }
            "get_status" | "read_status" => {
                self.get_device_status().await
            }
            _ => Err(DeviceError::Other(format!("未知命令: {}", command))),
        }
    }

    async fn get_status(&self) -> Result<Value> {
        Ok(json!({
            "protocol": "wdy-8en",
            "addr": self.addr,
            "port": self.port,
            "device_id": self.device_id
        }))
    }

    async fn write(&mut self, id: u32, value: i32) -> Result<()> {
        // id: 通道号 (1-8)
        // value: 1=开, 0=关
        if id < 1 || id > 8 {
            return Err(DeviceError::ConfigError(format!(
                "通道号必须在1-8之间，当前: {}",
                id
            )));
        }

        let channel = id as u8;
        if value > 0 {
            self.channel_on(channel).await
        } else {
            self.channel_off(channel).await
        }
    }

    async fn read(&self, id: u32) -> Result<i32> {
        // id: 通道号 (1-8)
        // 返回: 1=开, 0=关
        // 只返回第一路的状态
        if id < 1 || id > 8 {
            return Err(DeviceError::ConfigError(format!(
                "通道号必须在1-8之间，当前: {}",
                id
            )));
        }

        let frame = self.build_query_frame();
        let response = self.send_command(&frame, STATUS_RESPONSE_LEN).await?;

        if response.len() < STATUS_RESPONSE_LEN {
            return Err(DeviceError::ProtocolError(format!(
                "状态响应长度不足: 期望={}, 实际={}",
                STATUS_RESPONSE_LEN,
                response.len()
            )));
        }

        // 通道状态在 bytes 37-44 (CH1-CH8)
        let idx = 37 + (id - 1) as usize;
        let is_on = response[idx] != 0x00;

        info!(
            "WDY-8EN 通道 {} 状态: {}",
            id,
            if is_on { "开" } else { "关" }
        );

        Ok(if is_on { 1 } else { 0 })
    }

    fn name(&self) -> &str {
        "wdy-8en"
    }

    fn get_methods(&self) -> Vec<String> {
        vec![
            "power_on".to_string(),
            "power_off".to_string(),
            "channel_on".to_string(),
            "channel_off".to_string(),
            "set_device_id".to_string(),
            "get_status".to_string(),
        ]
    }
}
