// HS-08R-16R 多功能电源时序器通讯协议
// 通信方式: RS485/RS232 串口
// 波特率: 9600, 数据位: 8, 校验位: 无, 停止位: 1
// 协议版本: V1.1 (支持12路控制)

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tracing::{debug, info, warn};

use crate::protocols::Protocol;
use crate::utils::error::DeviceError;

// 协议常量
const PROTOCOL_HEADER: [u8; 2] = [0x5B, 0xB5]; // 数据帧头

// 功能码
const FUNC_READ: u8 = 0x02;
const FUNC_WRITE_PARAM: u8 = 0x10;
const FUNC_CONTROL: u8 = 0x16;
const FUNC_SET_TIME: u8 = 0x13;
const FUNC_FACTORY_RESET: u8 = 0x77;

// 应答码
const RESP_SUCCESS: u8 = 0xAA;
const RESP_FAIL: u8 = 0xFF;

// 通道地址 (开延时参数)
const ADDR_CH1_ON_DELAY: u16 = 0x4010;
const ADDR_CH2_ON_DELAY: u16 = 0x4012;
const ADDR_CH3_ON_DELAY: u16 = 0x4014;
const ADDR_CH4_ON_DELAY: u16 = 0x4016;
const ADDR_CH5_ON_DELAY: u16 = 0x4018;
const ADDR_CH6_ON_DELAY: u16 = 0x401A;
const ADDR_CH7_ON_DELAY: u16 = 0x401C;
const ADDR_CH8_ON_DELAY: u16 = 0x401E;
const ADDR_CH9_ON_DELAY: u16 = 0x4050;
const ADDR_CH10_ON_DELAY: u16 = 0x4052;
const ADDR_CH11_ON_DELAY: u16 = 0x4054;
const ADDR_CH12_ON_DELAY: u16 = 0x4056;

// 通道地址 (关延时参数)
const ADDR_CH1_OFF_DELAY: u16 = 0x4020;
const ADDR_CH2_OFF_DELAY: u16 = 0x4022;
const ADDR_CH3_OFF_DELAY: u16 = 0x4024;
const ADDR_CH4_OFF_DELAY: u16 = 0x4026;
const ADDR_CH5_OFF_DELAY: u16 = 0x4028;
const ADDR_CH6_OFF_DELAY: u16 = 0x402A;
const ADDR_CH7_OFF_DELAY: u16 = 0x402C;
const ADDR_CH8_OFF_DELAY: u16 = 0x402E;
const ADDR_CH9_OFF_DELAY: u16 = 0x4060;
const ADDR_CH10_OFF_DELAY: u16 = 0x4062;
const ADDR_CH11_OFF_DELAY: u16 = 0x4064;
const ADDR_CH12_OFF_DELAY: u16 = 0x4066;

// 特殊地址
const ADDR_CHANNEL_ENABLE: u16 = 0x4030;
const ADDR_DEVICE_ADDRESS: u16 = 0x2017;
const ADDR_DEVICE_STATUS: u16 = 0x2016;
const ADDR_VOLTAGE_PROTECT_1: u16 = 0x236C;
const ADDR_VOLTAGE_PROTECT_2: u16 = 0x236D;

pub struct HsPowerSequencerProtocol {
    port_name: String,  // 串口设备名,如 /dev/ttyUSB0 或 COM1
    baud_rate: u32,     // 波特率,默认 9600
    device_address: u8, // 设备地址,出厂默认 0x01
}

impl HsPowerSequencerProtocol {
    pub fn new(port_name: String, baud_rate: u32, device_address: u8) -> Self {
        Self {
            port_name,
            baud_rate,
            device_address,
        }
    }

    /// 打开串口连接
    async fn connect(&self) -> Result<SerialStream> {
        debug!("打开串口: {}, 波特率: {}", self.port_name, self.baud_rate);

        let port = tokio_serial::new(&self.port_name, self.baud_rate)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .timeout(Duration::from_millis(1000))
            .open_native_async()?;

        info!("HS 电源时序器串口连接成功");
        Ok(port)
    }

    /// 构建完整数据帧 (带协议头)
    fn build_frame(&self, data: &[u8]) -> Vec<u8> {
        let mut frame = Vec::with_capacity(PROTOCOL_HEADER.len() + data.len());
        frame.extend_from_slice(&PROTOCOL_HEADER);
        frame.extend_from_slice(data);
        frame
    }

    /// 发送命令并接收响应
    async fn send_command(&self, command: &[u8]) -> Result<Vec<u8>> {
        let mut stream = self.connect().await?;

        let frame = self.build_frame(command);
        debug!("发送数据帧: {:02X?}", frame);

        stream.write_all(&frame).await?;
        stream.flush().await?;

        // 设置读取超时 (3秒)
        let timeout = Duration::from_secs(3);

        // 读取响应 (跳过协议头) - 带超时
        let mut header = [0u8; 2];
        match tokio::time::timeout(timeout, stream.read_exact(&mut header)).await {
            Ok(Ok(_)) => {
                if header != PROTOCOL_HEADER {
                    warn!(
                        "响应帧头不匹配: {:02X?}, 期望: {:02X?}",
                        header, PROTOCOL_HEADER
                    );
                    return Err(DeviceError::ProtocolError(format!(
                        "响应帧头不匹配: 收到 {:02X?}, 期望 {:02X?}",
                        header, PROTOCOL_HEADER
                    ))
                    .into());
                }
            }
            Ok(Err(e)) => {
                warn!("读取响应帧头失败: {}", e);
                return Err(DeviceError::ConnectionError(format!("读取响应失败: {}", e)).into());
            }
            Err(_) => {
                warn!("读取响应超时 ({}秒)", timeout.as_secs());
                return Err(DeviceError::Other(
                    format!("设备响应超时 ({}秒), 请检查: 1.串口连接是否正常 2.设备地址是否正确 3.设备是否供电", timeout.as_secs())
                ).into());
            }
        }

        // 读取响应数据 (8字节) - 带超时
        let mut response = vec![0u8; 8];
        match tokio::time::timeout(timeout, stream.read_exact(&mut response)).await {
            Ok(Ok(_)) => {
                debug!("接收响应: {:02X?}", response);
                Ok(response)
            }
            Ok(Err(e)) => {
                warn!("读取响应数据失败: {}", e);
                Err(DeviceError::ConnectionError(format!("读取响应数据失败: {}", e)).into())
            }
            Err(_) => {
                warn!("读取响应数据超时");
                Err(DeviceError::Other("读取响应数据超时, 设备可能未正确响应".to_string()).into())
            }
        }
    }

    /// 通道开 (1-12)
    pub async fn channel_on(&self, channel: u8) -> Result<bool> {
        if channel < 1 || channel > 12 {
            return Err(DeviceError::Other("通道号必须在1-12之间".to_string()).into());
        }

        let command = [
            self.device_address,
            FUNC_CONTROL,
            0x00,
            0x00,
            0x00,
            0x01,
            channel,
            RESP_SUCCESS,
        ];

        let response = self.send_command(&command).await?;

        // 检查响应
        if response[0] == self.device_address && response[1] == RESP_SUCCESS {
            info!("通道 {} 开启成功", channel);
            Ok(true)
        } else if response[1] == RESP_FAIL {
            warn!("通道 {} 开启失败", channel);
            Ok(false)
        } else {
            Err(DeviceError::ProtocolError("无效的响应".to_string()).into())
        }
    }

    /// 通道关 (1-12)
    pub async fn channel_off(&self, channel: u8) -> Result<bool> {
        if channel < 1 || channel > 12 {
            return Err(DeviceError::Other("通道号必须在1-12之间".to_string()).into());
        }

        let command = [
            self.device_address,
            FUNC_CONTROL,
            0x00,
            0x00,
            0x00,
            0x00,
            channel,
            RESP_SUCCESS,
        ];

        let response = self.send_command(&command).await?;

        if response[0] == self.device_address && response[1] == RESP_SUCCESS {
            info!("通道 {} 关闭成功", channel);
            Ok(true)
        } else if response[1] == RESP_FAIL {
            warn!("通道 {} 关闭失败", channel);
            Ok(false)
        } else {
            Err(DeviceError::ProtocolError("无效的响应".to_string()).into())
        }
    }

    /// 延时开
    pub async fn delayed_on(&self) -> Result<bool> {
        let command = [
            self.device_address,
            FUNC_CONTROL,
            0x00,
            0x00,
            0x00,
            0x01,
            0x11,
            RESP_SUCCESS,
        ];
        let response = self.send_command(&command).await?;
        Ok(response[1] == RESP_SUCCESS)
    }

    /// 延时关
    pub async fn delayed_off(&self) -> Result<bool> {
        let command = [
            self.device_address,
            FUNC_CONTROL,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            RESP_SUCCESS,
        ];
        let response = self.send_command(&command).await?;
        Ok(response[1] == RESP_SUCCESS)
    }

    /// 一键开
    pub async fn all_on(&self) -> Result<bool> {
        let command = [
            self.device_address,
            FUNC_CONTROL,
            0x00,
            0x00,
            0x00,
            0x01,
            0x12,
            RESP_SUCCESS,
        ];
        let response = self.send_command(&command).await?;
        Ok(response[1] == RESP_SUCCESS)
    }

    /// 一键关
    pub async fn all_off(&self) -> Result<bool> {
        let command = [
            self.device_address,
            FUNC_CONTROL,
            0x00,
            0x00,
            0x00,
            0x00,
            0x10,
            RESP_SUCCESS,
        ];
        let response = self.send_command(&command).await?;
        Ok(response[1] == RESP_SUCCESS)
    }

    /// 设置通道延时参数 (单位: ms)
    pub async fn set_channel_delay(&self, channel: u8, delay_ms: u32, is_on: bool) -> Result<bool> {
        if channel < 1 || channel > 12 {
            return Err(DeviceError::Other("通道号必须在1-12之间".to_string()).into());
        }

        let addr = if is_on {
            // 开延时地址
            match channel {
                1 => ADDR_CH1_ON_DELAY,
                2 => ADDR_CH2_ON_DELAY,
                3 => ADDR_CH3_ON_DELAY,
                4 => ADDR_CH4_ON_DELAY,
                5 => ADDR_CH5_ON_DELAY,
                6 => ADDR_CH6_ON_DELAY,
                7 => ADDR_CH7_ON_DELAY,
                8 => ADDR_CH8_ON_DELAY,
                9 => ADDR_CH9_ON_DELAY,
                10 => ADDR_CH10_ON_DELAY,
                11 => ADDR_CH11_ON_DELAY,
                12 => ADDR_CH12_ON_DELAY,
                _ => return Err(DeviceError::Other("无效通道号".to_string()).into()),
            }
        } else {
            // 关延时地址
            match channel {
                1 => ADDR_CH1_OFF_DELAY,
                2 => ADDR_CH2_OFF_DELAY,
                3 => ADDR_CH3_OFF_DELAY,
                4 => ADDR_CH4_OFF_DELAY,
                5 => ADDR_CH5_OFF_DELAY,
                6 => ADDR_CH6_OFF_DELAY,
                7 => ADDR_CH7_OFF_DELAY,
                8 => ADDR_CH8_OFF_DELAY,
                9 => ADDR_CH9_OFF_DELAY,
                10 => ADDR_CH10_OFF_DELAY,
                11 => ADDR_CH11_OFF_DELAY,
                12 => ADDR_CH12_OFF_DELAY,
                _ => return Err(DeviceError::Other("无效通道号".to_string()).into()),
            }
        };

        let command = [
            self.device_address,
            FUNC_WRITE_PARAM,
            (addr >> 8) as u8,
            (addr & 0xFF) as u8,
            ((delay_ms >> 24) & 0xFF) as u8,
            ((delay_ms >> 16) & 0xFF) as u8,
            ((delay_ms >> 8) & 0xFF) as u8,
            (delay_ms & 0xFF) as u8,
        ];

        let response = self.send_command(&command).await?;
        Ok(response[1] == RESP_SUCCESS)
    }

    /// 读取设备状态 (返回各通道状态)
    pub async fn read_device_status(&self) -> Result<Vec<bool>> {
        let command = [
            self.device_address,
            FUNC_READ,
            (ADDR_DEVICE_STATUS >> 8) as u8,
            (ADDR_DEVICE_STATUS & 0xFF) as u8,
            0x00,
            0x00,
            0x00,
            0x00,
        ];

        let response = self.send_command(&command).await?;

        // 响应格式: [addr, func, addr_hi, addr_lo, ch1, ch2, ..., chN]
        let mut status = Vec::new();
        for i in 4..response.len() {
            if response[i] == 0x01 {
                status.push(true); // 开启
            } else if response[i] == 0x00 {
                status.push(false); // 关闭
            }
        }

        debug!("设备状态: {:?}", status);
        Ok(status)
    }

    /// 设置设备时间
    pub async fn set_time(
        &self,
        year: u8,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<bool> {
        // 转换为BCD码
        let year_bcd = ((year / 10) << 4) | (year % 10);
        let month_bcd = ((month / 10) << 4) | (month % 10);
        let day_bcd = ((day / 10) << 4) | (day % 10);
        let hour_bcd = ((hour / 10) << 4) | (hour % 10);
        let minute_bcd = ((minute / 10) << 4) | (minute % 10);
        let second_bcd = ((second / 10) << 4) | (second % 10);

        let command = [
            self.device_address,
            FUNC_SET_TIME,
            year_bcd,
            month_bcd,
            day_bcd,
            hour_bcd,
            minute_bcd,
            second_bcd,
        ];

        let response = self.send_command(&command).await?;
        Ok(response[1] == FUNC_SET_TIME)
    }

    /// 读取设备地址
    pub async fn read_device_address(&self) -> Result<u8> {
        let command = [
            0x00, // 广播地址
            FUNC_READ,
            (ADDR_DEVICE_ADDRESS >> 8) as u8,
            (ADDR_DEVICE_ADDRESS & 0xFF) as u8,
            RESP_SUCCESS,
            0x00,
            0x00,
            0x00,
        ];

        let response = self.send_command(&command).await?;
        Ok(response[7]) // 地址在最后一个字节
    }

    /// 写入设备地址
    pub async fn write_device_address(&self, new_address: u8) -> Result<bool> {
        let command = [
            self.device_address,
            FUNC_WRITE_PARAM,
            (ADDR_DEVICE_ADDRESS >> 8) as u8,
            (ADDR_DEVICE_ADDRESS & 0xFF) as u8,
            RESP_SUCCESS,
            0x00,
            0x00,
            new_address,
        ];

        let response = self.send_command(&command).await?;
        Ok(response[1] == FUNC_WRITE_PARAM && response[7] == RESP_SUCCESS)
    }

    /// 恢复出厂设置
    pub async fn factory_reset(&self) -> Result<bool> {
        let command = [
            self.device_address,
            FUNC_FACTORY_RESET,
            0x66,
            0x86,
            0x00,
            0x00,
            0x00,
            RESP_SUCCESS,
        ];

        let response = self.send_command(&command).await?;
        Ok(response[1] == FUNC_FACTORY_RESET && response[4] == RESP_SUCCESS)
    }

    /// 设置电压保护参数
    pub async fn set_voltage_protection(
        &self,
        over_voltage: u16,
        under_voltage: u8,
        hysteresis: u8,
        over_en: bool,
        under_en: bool,
    ) -> Result<bool> {
        // 先设置过压和欠压值
        let command1 = [
            self.device_address,
            FUNC_WRITE_PARAM,
            (ADDR_VOLTAGE_PROTECT_1 >> 8) as u8,
            (ADDR_VOLTAGE_PROTECT_1 & 0xFF) as u8,
            (over_voltage >> 8) as u8,
            (over_voltage & 0xFF) as u8,
            under_voltage,
            RESP_SUCCESS,
        ];

        let response1 = self.send_command(&command1).await?;
        if response1[1] != FUNC_WRITE_PARAM {
            return Ok(false);
        }

        // 再设置回差和使能
        let command2 = [
            self.device_address,
            FUNC_WRITE_PARAM,
            (ADDR_VOLTAGE_PROTECT_2 >> 8) as u8,
            (ADDR_VOLTAGE_PROTECT_2 & 0xFF) as u8,
            hysteresis,
            if over_en { 0x01 } else { 0x00 },
            if under_en { 0x01 } else { 0x00 },
            RESP_SUCCESS,
        ];

        let response2 = self.send_command(&command2).await?;
        Ok(response2[1] == FUNC_WRITE_PARAM)
    }

    /// 执行自定义命令
    pub async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
        info!("执行 HS 电源时序器命令: {}, 参数: {:?}", command, params);
        debug!(
            "设备地址: {}, 串口: {}",
            self.device_address, self.port_name
        );
        match command {
            "channel_on" => {
                let channel = params["channel"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少channel参数".to_string()))?
                    as u8;
                let result = self.channel_on(channel).await?;
                Ok(json!({ "success": result }))
            }
            "channel_off" => {
                let channel = params["channel"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少channel参数".to_string()))?
                    as u8;
                let result = self.channel_off(channel).await?;
                Ok(json!({ "success": result }))
            }
            "all_on" => {
                let result = self.all_on().await?;
                Ok(json!({ "success": result }))
            }
            "all_off" => {
                let result = self.all_off().await?;
                Ok(json!({ "success": result }))
            }
            "delayed_on" => {
                let result = self.delayed_on().await?;
                Ok(json!({ "success": result }))
            }
            "delayed_off" => {
                let result = self.delayed_off().await?;
                Ok(json!({ "success": result }))
            }
            "set_delay" => {
                let channel = params["channel"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少channel参数".to_string()))?
                    as u8;
                let delay_ms = params["delay_ms"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少delay_ms参数".to_string()))?
                    as u32;
                let is_on = params
                    .get("is_on")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let result = self.set_channel_delay(channel, delay_ms, is_on).await?;
                Ok(json!({ "success": result }))
            }
            "read_status" => {
                let status = self.read_device_status().await?;
                Ok(json!({ "channels": status }))
            }
            "set_time" => {
                let year = params["year"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少year参数".to_string()))?
                    as u8;
                let month = params["month"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少month参数".to_string()))?
                    as u8;
                let day = params["day"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少day参数".to_string()))?
                    as u8;
                let hour = params["hour"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少hour参数".to_string()))?
                    as u8;
                let minute = params["minute"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少minute参数".to_string()))?
                    as u8;
                let second = params.get("second").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
                let result = self
                    .set_time(year, month, day, hour, minute, second)
                    .await?;
                Ok(json!({ "success": result }))
            }
            "read_address" => {
                let address = self.read_device_address().await?;
                Ok(json!({ "address": address }))
            }
            "write_address" => {
                let new_address = params["address"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少address参数".to_string()))?
                    as u8;
                let result = self.write_device_address(new_address).await?;
                Ok(json!({ "success": result }))
            }
            "factory_reset" => {
                let result = self.factory_reset().await?;
                Ok(json!({ "success": result }))
            }
            "set_voltage_protection" => {
                let over_voltage = params["over_voltage"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少over_voltage参数".to_string()))?
                    as u16;
                let under_voltage = params["under_voltage"]
                    .as_u64()
                    .ok_or(DeviceError::Other("缺少under_voltage参数".to_string()))?
                    as u8;
                let hysteresis = params
                    .get("hysteresis")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8;
                let over_en = params
                    .get("over_enable")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let under_en = params
                    .get("under_enable")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let result = self
                    .set_voltage_protection(
                        over_voltage,
                        under_voltage,
                        hysteresis,
                        over_en,
                        under_en,
                    )
                    .await?;
                Ok(json!({ "success": result }))
            }
            _ => Err(DeviceError::Other(format!("未知命令: {}", command)).into()),
        }
    }
}

#[async_trait]
impl Protocol for HsPowerSequencerProtocol {
    fn from_config(
        _channel_id: u32,
        params: &HashMap<String, Value>,
    ) -> crate::utils::Result<Box<dyn Protocol>>
    where
        Self: Sized,
    {
        // port_name: 串口设备路径,如 /dev/ttyUSB0 (Linux) 或 COM1 (Windows)
        let port_name = params
            .get("port_name")
            .or_else(|| params.get("port"))
            .and_then(|v| v.as_str())
            .ok_or(DeviceError::ConfigError(
                "缺少port_name或port参数".to_string(),
            ))?
            .to_string();

        // baud_rate: 波特率,默认 9600
        let baud_rate = params
            .get("baud_rate")
            .and_then(|v| v.as_u64())
            .unwrap_or(9600) as u32;

        // device_address: 设备地址,出厂默认 1
        let device_address = params
            .get("device_address")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u8;

        debug!(
            "创建 HS 电源时序器协议: port={}, baud={}, addr={}",
            port_name, baud_rate, device_address
        );

        Ok(Box::new(Self::new(port_name, baud_rate, device_address)))
    }

    async fn execute(&mut self, command: &str, params: Value) -> crate::utils::Result<Value> {
        HsPowerSequencerProtocol::execute(self, command, params)
            .await
            .map_err(|e| DeviceError::Other(e.to_string()))
    }

    async fn get_status(&self) -> crate::utils::Result<Value> {
        let status = self
            .read_device_status()
            .await
            .map_err(|e| DeviceError::Other(e.to_string()))?;
        Ok(json!({ "channels": status }))
    }

    async fn write(&mut self, id: u32, value: i32) -> crate::utils::Result<()> {
        let channel = id as u8;
        if value == 0 {
            self.channel_off(channel)
                .await
                .map_err(|e| DeviceError::Other(e.to_string()))?;
        } else {
            self.channel_on(channel)
                .await
                .map_err(|e| DeviceError::Other(e.to_string()))?;
        }
        Ok(())
    }

    async fn read(&self, id: u32) -> crate::utils::Result<i32> {
        let status = self
            .read_device_status()
            .await
            .map_err(|e| DeviceError::Other(e.to_string()))?;

        let channel_idx = (id as usize).saturating_sub(1);
        if channel_idx < status.len() {
            Ok(if status[channel_idx] { 1 } else { 0 })
        } else {
            Err(DeviceError::Other(format!("通道 {} 超出范围", id)))
        }
    }

    fn name(&self) -> &str {
        "HsPowerSequencer"
    }

    async fn call_method(&mut self, method_name: &str, args: Value) -> crate::utils::Result<Value> {
        HsPowerSequencerProtocol::execute(self, method_name, args)
            .await
            .map_err(|e| DeviceError::Other(e.to_string()))
    }

    fn get_methods(&self) -> Vec<String> {
        vec![
            "channel_on".to_string(),
            "channel_off".to_string(),
            "all_on".to_string(),
            "all_off".to_string(),
            "delayed_on".to_string(),
            "delayed_off".to_string(),
            "set_delay".to_string(),
            "read_status".to_string(),
            "set_time".to_string(),
            "read_address".to_string(),
            "write_address".to_string(),
            "factory_reset".to_string(),
            "set_voltage_protection".to_string(),
        ]
    }
}
