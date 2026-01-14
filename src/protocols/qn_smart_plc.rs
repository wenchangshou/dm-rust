// QN Smart PLC 协议实现
// 基于 Modbus TCP，支持 40 路开关控制和传感器数据读取

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

// Modbus 功能码
const FC_READ_COILS: u8 = 0x01;
const FC_READ_HOLDING_REGISTERS: u8 = 0x03;
const FC_WRITE_SINGLE_COIL: u8 = 0x05;

// 特殊控制地址
const ADDR_ONE_KEY_START: u16 = 0x03E8; // 一键启动
const ADDR_ONE_KEY_STOP: u16 = 0x03E9; // 一键停止
const ADDR_EMERGENCY_STOP: u16 = 0x03EA; // 急停

// 通道基础地址 (每个通道占2个地址: 启动/关闭)
const ADDR_CHANNEL_BASE: u16 = 0x03EB;

// 传感器读取地址
const ADDR_INTERNAL_TEMP_HUMIDITY: u16 = 0x025A; // 内部温湿度
const ADDR_ZERO_LINE_TEMP: u16 = 0x035B; // 零线温度
const ADDR_EXTERNAL_SENSORS: u16 = 0x04B0; // 外部温湿度、电压、电量
const ADDR_CURRENT: u16 = 0x05DC; // 电流

pub struct QnSmartPlcProtocol {
    addr: String,
    port: u16,
    slave_id: u8,
    transaction_id: u16,
}

impl QnSmartPlcProtocol {
    pub fn new(addr: String, port: u16, slave_id: u8) -> Self {
        Self {
            addr,
            port,
            slave_id,
            transaction_id: 0,
        }
    }

    /// 获取下一个事务ID
    fn next_transaction_id(&mut self) -> u16 {
        self.transaction_id = self.transaction_id.wrapping_add(1);
        self.transaction_id
    }

    /// 构建 Modbus TCP 请求帧
    fn build_request(&mut self, function_code: u8, data: &[u8]) -> Vec<u8> {
        let transaction_id = self.next_transaction_id();
        let length = 2 + data.len() as u16; // Unit ID + Function Code + Data

        let mut frame = Vec::with_capacity(7 + data.len());
        frame.extend_from_slice(&transaction_id.to_be_bytes()); // Transaction ID
        frame.extend_from_slice(&[0x00, 0x00]); // Protocol ID (Modbus)
        frame.extend_from_slice(&length.to_be_bytes()); // Length
        frame.push(self.slave_id); // Unit ID
        frame.push(function_code); // Function Code
        frame.extend_from_slice(data); // Data

        frame
    }

    /// 发送请求并接收响应
    async fn send_request(&mut self, request: &[u8]) -> Result<Vec<u8>> {
        let addr = format!("{}:{}", self.addr, self.port);
        info!("连接到 QN Smart PLC: {}", addr);

        let mut stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("连接失败: {}", e)))?;

        debug!("发送请求: {:02X?}", request);
        stream
            .write_all(request)
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("发送失败: {}", e)))?;
        stream.flush().await?;

        // 读取响应
        let mut response = vec![0u8; 256];
        let timeout = Duration::from_secs(3);

        match tokio::time::timeout(timeout, stream.read(&mut response)).await {
            Ok(Ok(n)) if n > 0 => {
                response.truncate(n);
                debug!("接收响应: {:02X?}", response);
                Ok(response)
            }
            Ok(Ok(_)) => Err(DeviceError::ConnectionError("连接已关闭".to_string())),
            Ok(Err(e)) => Err(DeviceError::ConnectionError(format!("读取失败: {}", e))),
            Err(_) => Err(DeviceError::Other("响应超时".to_string())),
        }
    }

    /// 写单个线圈 (Function Code 0x05)
    async fn write_coil(&mut self, address: u16, value: bool) -> Result<()> {
        let coil_value: u16 = if value { 0xFF00 } else { 0x0000 };
        let data = [
            (address >> 8) as u8,
            address as u8,
            (coil_value >> 8) as u8,
            coil_value as u8,
        ];

        let request = self.build_request(FC_WRITE_SINGLE_COIL, &data);
        let response = self.send_request(&request).await?;

        // 验证响应
        if response.len() >= 12 && response[7] == FC_WRITE_SINGLE_COIL {
            info!("写入线圈成功: addr=0x{:04X}, value={}", address, value);
            Ok(())
        } else if response.len() >= 9 && response[7] == (FC_WRITE_SINGLE_COIL | 0x80) {
            let error_code = response[8];
            error!("Modbus异常: 0x{:02X}", error_code);
            Err(DeviceError::ProtocolError(format!(
                "Modbus异常: 0x{:02X}",
                error_code
            )))
        } else {
            error!("无效响应: {:02X?}", response);
            Err(DeviceError::ProtocolError("无效响应".to_string()))
        }
    }

    /// 读取线圈状态 (Function Code 0x01)
    async fn read_coils(&mut self, address: u16, count: u16) -> Result<Vec<bool>> {
        let data = [
            (address >> 8) as u8,
            address as u8,
            (count >> 8) as u8,
            count as u8,
        ];

        let request = self.build_request(FC_READ_COILS, &data);
        let response = self.send_request(&request).await?;

        if response.len() >= 9 && response[7] == FC_READ_COILS {
            let _byte_count = response[8] as usize;
            let mut coils = Vec::new();

            for i in 0..count as usize {
                let byte_idx = 9 + (i / 8);
                let bit_idx = i % 8;
                if byte_idx < response.len() {
                    let bit = (response[byte_idx] >> bit_idx) & 0x01;
                    coils.push(bit == 1);
                }
            }
            info!("读取线圈(0x{:04X})成功: {:?}", address, coils);

            Ok(coils)
        } else {
            Err(DeviceError::ProtocolError("读取线圈失败".to_string()))
        }
    }

    /// 读取保持寄存器 (Function Code 0x03)
    async fn read_holding_registers(&mut self, address: u16, count: u16) -> Result<Vec<u16>> {
        let data = [
            (address >> 8) as u8,
            address as u8,
            (count >> 8) as u8,
            count as u8,
        ];

        let request = self.build_request(FC_READ_HOLDING_REGISTERS, &data);
        let response = self.send_request(&request).await?;

        if response.len() >= 9 && response[7] == FC_READ_HOLDING_REGISTERS {
            let byte_count = response[8] as usize;
            let mut registers = Vec::new();

            for i in (0..byte_count).step_by(2) {
                if 9 + i + 1 < response.len() {
                    let value = ((response[9 + i] as u16) << 8) | (response[9 + i + 1] as u16);
                    registers.push(value);
                }
            }
            info!("读取寄存器(0x{:04X})成功: {:?}", address, registers);

            Ok(registers)
        } else {
            Err(DeviceError::ProtocolError("读取寄存器失败".to_string()))
        }
    }

    /// 计算通道的启动/关闭地址
    fn get_channel_address(channel: u32, on: bool) -> Result<u16> {
        if channel < 1 || channel > 40 {
            return Err(DeviceError::ConfigError("通道号必须在1-40之间".to_string()));
        }

        let channel = channel as u16;

        // 根据通道号段计算地址
        let addr = match channel {
            // 1-8路: 0x03EB - 0x03FA
            1..=8 => {
                let base = ADDR_CHANNEL_BASE;
                let offset = (channel - 1) * 2;
                if on {
                    base + offset
                } else {
                    base + offset + 1
                }
            }
            // 9-12路: 0x03FB - 0x0402
            9..=12 => {
                let base = 0x03FB;
                let offset = (channel - 9) * 2;
                if on {
                    base + offset
                } else {
                    base + offset + 1
                }
            }
            // 13-16路: 0x040B - 0x0412
            13..=16 => {
                let base = 0x040B;
                let offset = (channel - 13) * 2;
                if on {
                    base + offset
                } else {
                    base + offset + 1
                }
            }
            // 17-20路: 0x041B - 0x0422
            17..=20 => {
                let base = 0x041B;
                let offset = (channel - 17) * 2;
                if on {
                    base + offset
                } else {
                    base + offset + 1
                }
            }
            // 21-24路: 0x042B - 0x0432
            21..=24 => {
                let base = 0x042B;
                let offset = (channel - 21) * 2;
                if on {
                    base + offset
                } else {
                    base + offset + 1
                }
            }
            // 25-28路: 0x043B - 0x0442
            25..=28 => {
                let base = 0x043B;
                let offset = (channel - 25) * 2;
                if on {
                    base + offset
                } else {
                    base + offset + 1
                }
            }
            // 29-32路: 0x044B - 0x0452
            29..=32 => {
                let base = 0x044B;
                let offset = (channel - 29) * 2;
                if on {
                    base + offset
                } else {
                    base + offset + 1
                }
            }
            // 33-36路: 0x045B - 0x0462
            33..=36 => {
                let base = 0x045B;
                let offset = (channel - 33) * 2;
                if on {
                    base + offset
                } else {
                    base + offset + 1
                }
            }
            // 37-40路: 0x046B - 0x0472
            37..=40 => {
                let base = 0x046B;
                let offset = (channel - 37) * 2;
                if on {
                    base + offset
                } else {
                    base + offset + 1
                }
            }
            _ => return Err(DeviceError::ConfigError("无效通道号".to_string())),
        };

        Ok(addr)
    }

    /// 控制单个通道
    pub async fn control_channel(&mut self, channel: u32, on: bool) -> Result<()> {
        let addr = Self::get_channel_address(channel, on)?;
        info!(
            "控制通道{}: {} (addr=0x{:04X})",
            channel,
            if on { "启动" } else { "关闭" },
            addr
        );
        self.write_coil(addr, true).await
    }

    /// 一键启动
    pub async fn one_key_start(&mut self) -> Result<()> {
        info!("执行一键启动");
        self.write_coil(ADDR_ONE_KEY_START, true).await
    }

    /// 一键停止
    pub async fn one_key_stop(&mut self) -> Result<()> {
        info!("执行一键停止");
        self.write_coil(ADDR_ONE_KEY_STOP, true).await
    }

    /// 急停控制
    pub async fn emergency_stop(&mut self, pressed: bool) -> Result<()> {
        info!("急停: {}", if pressed { "按下" } else { "取消" });
        self.write_coil(ADDR_EMERGENCY_STOP, pressed).await
    }

    /// 读取通道状态 (每次读4个通道)
    pub async fn read_channel_status(&mut self, start_channel: u32) -> Result<Vec<bool>> {
        if start_channel < 1 || start_channel > 40 || (start_channel - 1) % 4 != 0 {
            return Err(DeviceError::ConfigError(
                "起始通道必须是1,5,9,...,37".to_string(),
            ));
        }

        // 计算读取地址
        let read_addr = ((start_channel - 1) / 4) * 4;
        let data = [0x00, (read_addr >> 8) as u8, (read_addr as u8), 0x00, 0x04];

        let request = self.build_request(FC_READ_COILS, &data[1..]);
        let response = self.send_request(&request).await?;

        if response.len() >= 10 {
            let status_byte = response[9];
            let mut statuses = Vec::new();
            // Bit 解析: Bit0=第4路, Bit1=第3路, Bit2=第2路, Bit3=第1路
            for i in (0..4).rev() {
                statuses.push((status_byte >> i) & 0x01 == 1);
            }
            info!(
                "读取状态成功: start={}, status_byte=0x{:02X}, statuses={:?}",
                start_channel, status_byte, statuses
            );
            Ok(statuses)
        } else {
            Err(DeviceError::ProtocolError("读取状态失败".to_string()))
        }
    }

    /// 读取内部温湿度
    pub async fn read_internal_temp_humidity(&mut self) -> Result<(f32, f32)> {
        let registers = self
            .read_holding_registers(ADDR_INTERNAL_TEMP_HUMIDITY, 2)
            .await?;
        if registers.len() >= 2 {
            let temp = registers[0] as f32; // 温度 (℃)
            let humidity = registers[1] as f32; // 湿度 (%)
            info!("内部温湿度: temp={}, humidity={}", temp, humidity);
            Ok((temp, humidity))
        } else {
            Err(DeviceError::ProtocolError("读取温湿度失败".to_string()))
        }
    }

    /// 读取零线温度
    pub async fn read_zero_line_temp(&mut self) -> Result<f32> {
        let registers = self.read_holding_registers(ADDR_ZERO_LINE_TEMP, 1).await?;
        if !registers.is_empty() {
            let temp = registers[0] as f32 * 0.1; // 温度 * 0.1 = ℃
            info!("零线温度: {}", temp);
            Ok(temp)
        } else {
            Err(DeviceError::ProtocolError("读取零线温度失败".to_string()))
        }
    }

    /// 读取外部传感器数据 (温湿度、电压、电量)
    pub async fn read_external_sensors(&mut self) -> Result<Value> {
        let registers = self
            .read_holding_registers(ADDR_EXTERNAL_SENSORS, 10)
            .await?;
        if registers.len() >= 7 {
            let ext_humidity = registers[0] as f32 * 0.1; // 外部湿度 %
            let ext_temp = registers[1] as f32 * 0.1; // 外部温度 ℃
            let energy = ((registers[2] as u32) << 16 | registers[3] as u32) as f32 * 0.1; // 电量 kWh
            let voltage_a = registers[4] as f32 * 0.1; // A相电压 V
            let voltage_b = registers[5] as f32 * 0.1; // B相电压 V
            let voltage_c = registers[6] as f32 * 0.1; // C相电压 V

            info!(
                "外部传感器数据: 湿度={}, 温度={}, 电量={}, VA={}, VB={}, VC={}",
                ext_humidity, ext_temp, energy, voltage_a, voltage_b, voltage_c
            );

            Ok(json!({
                "external_humidity": ext_humidity,
                "external_temp": ext_temp,
                "energy_kwh": energy,
                "voltage_a": voltage_a,
                "voltage_b": voltage_b,
                "voltage_c": voltage_c
            }))
        } else {
            Err(DeviceError::ProtocolError("读取外部传感器失败".to_string()))
        }
    }

    /// 读取电流
    pub async fn read_current(&mut self) -> Result<Value> {
        let registers = self.read_holding_registers(ADDR_CURRENT, 6).await?;
        if registers.len() >= 6 {
            let current_a = registers[0] as f32 * 0.1; // A相电流 A
            let current_b = registers[2] as f32 * 0.1; // B相电流 A
            let current_c = registers[4] as f32 * 0.1; // C相电流 A

            info!(
                "电流数据: A={}, B={}, C={}",
                current_a, current_b, current_c
            );

            Ok(json!({
                "current_a": current_a,
                "current_b": current_b,
                "current_c": current_c
            }))
        } else {
            Err(DeviceError::ProtocolError("读取电流失败".to_string()))
        }
    }
}

#[async_trait]
impl Protocol for QnSmartPlcProtocol {
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

        let slave_id = params
            .get("slave_id")
            .and_then(|v| v.as_u64())
            .unwrap_or(0x32) as u8; // 默认 0x32 (50)

        info!(
            "创建 QN Smart PLC 协议: {}:{}, slave_id=0x{:02X}",
            addr, port, slave_id
        );
        Ok(Box::new(Self::new(addr, port, slave_id)))
    }

    async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
        info!("执行命令: {}, 参数: {:?}", command, params);
        match command {
            "one_key_start" => {
                self.one_key_start().await?;
                Ok(json!({"status": "success", "message": "一键启动成功"}))
            }
            "one_key_stop" => {
                self.one_key_stop().await?;
                Ok(json!({"status": "success", "message": "一键停止成功"}))
            }
            "emergency_stop" => {
                let pressed = params
                    .get("pressed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                self.emergency_stop(pressed).await?;
                Ok(
                    json!({"status": "success", "message": if pressed { "急停按下" } else { "急停取消" }}),
                )
            }
            "control_channel" => {
                let channel = params
                    .get("channel")
                    .and_then(|v| v.as_u64())
                    .ok_or(DeviceError::ConfigError("缺少 channel 参数".to_string()))?
                    as u32;
                let on = params.get("on").and_then(|v| v.as_bool()).unwrap_or(true);
                self.control_channel(channel, on).await?;
                Ok(json!({"status": "success", "channel": channel, "on": on}))
            }
            "read_status" => {
                let start = params.get("start").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                let statuses = self.read_channel_status(start).await?;
                Ok(json!({"status": "success", "channels": statuses}))
            }
            "read_temp_humidity" => {
                let (temp, humidity) = self.read_internal_temp_humidity().await?;
                Ok(json!({"status": "success", "temperature": temp, "humidity": humidity}))
            }
            "read_zero_line_temp" => {
                let temp = self.read_zero_line_temp().await?;
                Ok(json!({"status": "success", "zero_line_temp": temp}))
            }
            "read_external_sensors" => self.read_external_sensors().await,
            "read_current" => self.read_current().await,
            _ => Err(DeviceError::Other(format!("未知命令: {}", command))),
        }
    }

    async fn get_status(&self) -> Result<Value> {
        Ok(json!({
            "protocol": "qn-smart-plc",
            "addr": self.addr,
            "port": self.port,
            "slave_id": self.slave_id
        }))
    }

    async fn write(&mut self, id: u32, value: i32) -> Result<()> {
        // id: 通道号 (1-40)
        // value: 1=启动, 0=关闭
        //
        // 例如:
        //   write(1, 1) -> 第1路启动 (0x03EB)
        //   write(1, 0) -> 第1路关闭 (0x03EC)
        //   write(2, 1) -> 第2路启动 (0x03ED)
        //   write(2, 0) -> 第2路关闭 (0x03EE)

        if id < 1 || id > 40 {
            return Err(DeviceError::ConfigError(format!(
                "通道号必须在1-40之间，当前: {}",
                id
            )));
        }

        let on = value > 0;
        info!(
            "写入通道 {}: {} (value={})",
            id,
            if on { "开" } else { "关" },
            value
        );
        self.control_channel(id, on).await
    }

    async fn read(&self, id: u32) -> Result<i32> {
        // id: 通道号 (1-40)
        // 返回: 1=开, 0=关
        //
        // 读取地址规则（每次读4个通道）:
        //   Ch 1-4:   addr=0x0000
        //   Ch 5-8:   addr=0x0004
        //   Ch 9-12:  addr=0x0008
        //   Ch 13-16: addr=0x000C
        //   ...
        //
        // 响应解析: 最后一个字节的 Bit 对应通道状态
        //   Bit3 = 组内第1路, Bit2 = 第2路, Bit1 = 第3路, Bit0 = 第4路
        info!("读取通道 {} 状态", id);

        if id < 1 || id > 40 {
            return Err(DeviceError::ConfigError(format!(
                "通道号必须在1-40之间，当前: {}",
                id
            )));
        }

        // 计算组号和组内位置
        let group = (id - 1) / 4; // 0-9
        let pos_in_group = (id - 1) % 4; // 0-3 (第1-4路)
        let read_addr = group * 4; // 0x0000, 0x0004, 0x0008...

        // 构建读取请求
        // 注意：这里需要 &mut self，但 trait 定义是 &self
        // 所以我们需要手动构建请求
        let addr_str = format!("{}:{}", self.addr, self.port);

        let mut stream = TcpStream::connect(&addr_str).await.map_err(|e| {
            error!("连接失败: {}", e);
            DeviceError::ConnectionError(format!("连接失败: {}", e))
        })?;

        // 构建 Modbus TCP 读线圈请求
        // 00 00 00 00 00 06 32 01 XX XX 00 04
        let request = [
            0x00,
            0x00, // Transaction ID
            0x00,
            0x00, // Protocol ID
            0x00,
            0x06,                   // Length
            self.slave_id,          // Unit ID
            FC_READ_COILS,          // Function Code
            (read_addr >> 8) as u8, // Start Address High
            read_addr as u8,        // Start Address Low
            0x00,
            0x04, // Quantity (4 coils)
        ];

        debug!("读取通道 {} 状态，请求: {:02X?}", id, request);

        stream
            .write_all(&request)
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("发送失败: {}", e)))?;
        stream.flush().await?;

        // 读取响应
        let mut response = vec![0u8; 32];
        let timeout = Duration::from_secs(3);

        let n = tokio::time::timeout(timeout, stream.read(&mut response))
            .await
            .map_err(|_| DeviceError::Other("响应超时".to_string()))?
            .map_err(|e| DeviceError::ConnectionError(format!("读取失败: {}", e)))?;

        response.truncate(n);
        debug!("读取通道 {} 响应: {:02X?}", id, response);

        // 解析响应: 00 00 00 00 00 04 32 01 01 XX
        // 响应长度至少 10 字节，最后一个字节是状态
        if response.len() >= 10 && response[7] == FC_READ_COILS {
            let status_byte = response[9];

            // Bit 解析 (根据协议规范):
            // 0111 = 第四路关闭，第一/二/三路开启
            // Bit0=第1路, Bit1=第2路, Bit2=第3路, Bit3=第4路
            let bit_pos = pos_in_group; // 第1路->Bit0, 第2路->Bit1, 第3路->Bit2, 第4路->Bit3
            let is_on = (status_byte >> bit_pos) & 0x01 == 1;

            info!(
                "通道 {} 状态: {} (byte=0x{:02X}, bit={})",
                id,
                if is_on { "开" } else { "关" },
                status_byte,
                bit_pos
            );

            Ok(if is_on { 1 } else { 0 })
        } else {
            Err(DeviceError::ProtocolError(format!(
                "读取通道 {} 状态失败，响应: {:02X?}",
                id, response
            )))
        }
    }

    fn name(&self) -> &str {
        "qn-smart-plc"
    }

    fn get_methods(&self) -> Vec<String> {
        vec![
            "one_key_start".to_string(),
            "one_key_stop".to_string(),
            "emergency_stop".to_string(),
            "control_channel".to_string(),
            "read_status".to_string(),
            "read_temp_humidity".to_string(),
            "read_zero_line_temp".to_string(),
            "read_external_sensors".to_string(),
            "read_current".to_string(),
        ]
    }
}
