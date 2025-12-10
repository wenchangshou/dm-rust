use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_modbus::prelude::*;
use tracing::{debug, info, warn};

use crate::protocols::Protocol;
use crate::utils::{Result, DeviceError};
use crate::config::AutoCallConfig;

/// Modbus 数据类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModbusDataType {
    /// 无符号16位整数 (1个寄存器)
    UInt16,
    /// 有符号16位整数 (1个寄存器)
    Int16,
    /// 无符号32位整数 (2个寄存器, Big Endian)
    UInt32,
    /// 有符号32位整数 (2个寄存器, Big Endian)
    Int32,
    /// 无符号32位整数 (2个寄存器, Little Endian)
    UInt32LE,
    /// 有符号32位整数 (2个寄存器, Little Endian)
    Int32LE,
    /// 32位浮点数 (2个寄存器, Big Endian)
    Float32,
    /// 32位浮点数 (2个寄存器, Little Endian)
    Float32LE,
    /// 64位浮点数 (4个寄存器, Big Endian)
    Float64,
    /// 布尔值 (线圈)
    Bool,
}

impl ModbusDataType {
    /// 从字符串解析数据类型
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "uint16" | "u16" => Ok(Self::UInt16),
            "int16" | "i16" => Ok(Self::Int16),
            "uint32" | "u32" => Ok(Self::UInt32),
            "int32" | "i32" => Ok(Self::Int32),
            "uint32le" | "u32le" => Ok(Self::UInt32LE),
            "int32le" | "i32le" => Ok(Self::Int32LE),
            "float32" | "float" | "f32" => Ok(Self::Float32),
            "float32le" | "floatle" | "f32le" => Ok(Self::Float32LE),
            "float64" | "double" | "f64" => Ok(Self::Float64),
            "bool" | "boolean" | "bit" => Ok(Self::Bool),
            _ => Err(DeviceError::ConfigError(format!(
                "不支持的数据类型: {}",
                s
            ))),
        }
    }

    /// 获取该数据类型需要的寄存器数量
    pub fn register_count(&self) -> u16 {
        match self {
            Self::UInt16 | Self::Int16 => 1,
            Self::UInt32 | Self::Int32 | Self::UInt32LE | Self::Int32LE 
            | Self::Float32 | Self::Float32LE => 2,
            Self::Float64 => 4,
            Self::Bool => 1,
        }
    }

    /// 是否为线圈类型
    pub fn is_coil(&self) -> bool {
        matches!(self, Self::Bool)
    }
}

/// Modbus协议实现
pub struct ModbusProtocol {
    channel_id: u32,
    addr: String,
    port: u16,
    slave_id: u8,
    /// 数据缓存：地址 -> (值, 数据类型, 时间戳)
    cache: Arc<RwLock<HashMap<u16, (Value, String, std::time::Instant)>>>,
    /// 自动召唤配置
    auto_call_configs: Vec<AutoCallConfig>,
}

impl ModbusProtocol {
    pub fn new(addr: String, port: u16, slave_id: u8) -> Self {
        Self {
            channel_id: 0,
            addr,
            port,
            slave_id,
            cache: Arc::new(RwLock::new(HashMap::new())),
            auto_call_configs: Vec::new(),
        }
    }

    /// 启动自动召唤任务
    pub fn start_auto_call_tasks(&self) {
        for config in &self.auto_call_configs {
            let addr = self.addr.clone();
            let port = self.port;
            let slave_id = self.slave_id;
            let cache = Arc::clone(&self.cache);
            let config = config.clone();

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_millis(config.interval_ms));
                
                loop {
                    interval.tick().await;
                    
                    if let Err(e) = Self::auto_call_task(&addr, port, slave_id, &config, &cache).await {
                        warn!("自动召唤失败 (function={}, start_addr={}, count={}): {}", 
                            config.function, config.start_addr, config.count, e);
                    }
                }
            });
        }
    }

    /// 执行单次自动召唤任务
    async fn auto_call_task(
        addr: &str,
        port: u16,
        slave_id: u8,
        config: &AutoCallConfig,
        cache: &Arc<RwLock<HashMap<u16, (Value, String, std::time::Instant)>>>,
    ) -> Result<()> {
        let socket_addr = format!("{}:{}", addr, port);
        let socket_addr = socket_addr
            .parse()
            .map_err(|e| DeviceError::ConfigError(format!("无效的地址: {}", e)))?;

        let mut ctx = tcp::connect_slave(socket_addr, Slave(slave_id))
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("Modbus TCP 连接失败: {}", e)))?;

        let now = std::time::Instant::now();

        match config.function.as_str() {
            "holding" => {
                let registers = ctx.read_holding_registers(config.start_addr, config.count).await
                    .map_err(|e| DeviceError::ConnectionError(format!("读取失败: {}", e)))?
                    .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

                let mut cache_write = cache.write().await;
                let mut updated_addrs = Vec::new();
                for (i, &value) in registers.iter().enumerate() {
                    let addr = config.start_addr + i as u16;
                    cache_write.insert(addr, (Value::Number(value.into()), "uint16".to_string(), now));
                    updated_addrs.push((addr, value));
                }
                drop(cache_write);
                
                info!(
                    "自动召唤成功: 保持寄存器 [{}-{}], 更新了 {} 个地址。样例数据: [addr={}, val={}], [addr={}, val={}]", 
                    config.start_addr, 
                    config.start_addr + config.count - 1,
                    updated_addrs.len(),
                    updated_addrs.first().map(|x| x.0).unwrap_or(0),
                    updated_addrs.first().map(|x| x.1).unwrap_or(0),
                    updated_addrs.get(updated_addrs.len().saturating_sub(1)).map(|x| x.0).unwrap_or(0),
                    updated_addrs.get(updated_addrs.len().saturating_sub(1)).map(|x| x.1).unwrap_or(0)
                );
            }
            "input" => {
                let registers = ctx.read_input_registers(config.start_addr, config.count).await
                    .map_err(|e| DeviceError::ConnectionError(format!("读取失败: {}", e)))?
                    .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

                let mut cache_write = cache.write().await;
                let mut updated_addrs = Vec::new();
                for (i, &value) in registers.iter().enumerate() {
                    let addr = config.start_addr + i as u16;
                    cache_write.insert(addr, (Value::Number(value.into()), "uint16".to_string(), now));
                    updated_addrs.push((addr, value));
                }
                drop(cache_write);
                
                info!(
                    "自动召唤成功: 输入寄存器 [{}-{}], 更新了 {} 个地址。样例数据: [addr={}, val={}], [addr={}, val={}]", 
                    config.start_addr, 
                    config.start_addr + config.count - 1,
                    updated_addrs.len(),
                    updated_addrs.first().map(|x| x.0).unwrap_or(0),
                    updated_addrs.first().map(|x| x.1).unwrap_or(0),
                    updated_addrs.get(updated_addrs.len().saturating_sub(1)).map(|x| x.0).unwrap_or(0),
                    updated_addrs.get(updated_addrs.len().saturating_sub(1)).map(|x| x.1).unwrap_or(0)
                );
            }
            "coil" => {
                let coils = ctx.read_coils(config.start_addr, config.count).await
                    .map_err(|e| DeviceError::ConnectionError(format!("读取失败: {}", e)))?
                    .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

                let mut cache_write = cache.write().await;
                let mut updated_addrs = Vec::new();
                for (i, &value) in coils.iter().enumerate() {
                    let addr = config.start_addr + i as u16;
                    cache_write.insert(addr, (Value::Bool(value), "bool".to_string(), now));
                    updated_addrs.push(addr);
                }
                drop(cache_write);
                
                info!(
                    "自动召唤成功: 线圈 [{}-{}], 更新了 {} 个地址", 
                    config.start_addr, 
                    config.start_addr + config.count - 1,
                    updated_addrs.len()
                );
            }
            "discrete" => {
                let inputs = ctx.read_discrete_inputs(config.start_addr, config.count).await
                    .map_err(|e| DeviceError::ConnectionError(format!("读取失败: {}", e)))?
                    .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

                let mut cache_write = cache.write().await;
                let mut updated_addrs = Vec::new();
                for (i, &value) in inputs.iter().enumerate() {
                    let addr = config.start_addr + i as u16;
                    cache_write.insert(addr, (Value::Bool(value), "bool".to_string(), now));
                    updated_addrs.push(addr);
                }
                drop(cache_write);
                
                info!(
                    "自动召唤成功: 离散输入 [{}-{}], 更新了 {} 个地址", 
                    config.start_addr, 
                    config.start_addr + config.count - 1,
                    updated_addrs.len()
                );
            }
            _ => {
                return Err(DeviceError::ConfigError(format!("不支持的功能码: {}", config.function)));
            }
        }

        Ok(())
    }

    /// 从缓存读取数据
    pub async fn read_from_cache(&self, addr: u16, data_type: &str) -> Result<Option<Value>> {

        println!("尝试从缓存读取: addr={} type={}", addr, data_type);
        let cache = self.cache.read().await;
        // 打印cache所有的值
            for (k, v) in cache.iter() {
                println!("缓存地址: {}, 值: {:?}, 类型: {}, 时间: {:?}", k, v.0, v.1, v.2);
            }
        
        // 根据数据类型需要读取的寄存器数量
        let data_type_enum = ModbusDataType::from_str(data_type)?;
        let count = data_type_enum.register_count() as usize;
        
        if data_type_enum.is_coil() {
            // 线圈类型直接从缓存读取
            if let Some((value, _, _)) = cache.get(&addr) {
                return Ok(Some(value.clone()));
            }
        } else {
            // 寄存器类型需要读取多个连续地址
            let mut registers = Vec::new();
            for i in 0..count {
                if let Some((value, _, _)) = cache.get(&(addr + i as u16)) {
                    println!("缓存命中: addr={} value={:?}", addr, value);
                    if let Some(num) = value.as_u64() {
                        registers.push(num as u16);
                    } else {
                        return Ok(None); // 缓存数据格式不正确
                    }
                } else {
                    return Ok(None); // 缓存中缺少数据
                }
            }
            
            // 将寄存器数据转换为指定类型
            let converted = Self::registers_to_value(&registers, data_type_enum)?;
            return Ok(Some(converted));
        }
        
        Ok(None)
    }

    /// 获取所有缓存数据
    pub async fn get_all_cache(&self) -> HashMap<u16, (Value, String, std::time::Instant)> {
        self.cache.read().await.clone()
    }

    /// 创建 Modbus TCP 连接
    async fn connect(&self) -> Result<client::Context> {
        let socket_addr = format!("{}:{}", self.addr, self.port);
        debug!("连接到 Modbus TCP 服务器: {}", socket_addr);

        let socket_addr = socket_addr
            .parse()
            .map_err(|e| DeviceError::ConfigError(format!("无效的地址: {}", e)))?;

        let ctx = tcp::connect_slave(socket_addr, Slave(self.slave_id))
            .await
            .map_err(|e| DeviceError::ConnectionError(format!("Modbus TCP 连接失败: {}", e)))?;

        info!("Modbus TCP 连接成功");
        Ok(ctx)
    }

    /// 将寄存器数据转换为指定类型的值
    fn registers_to_value(registers: &[u16], data_type: ModbusDataType) -> Result<Value> {
        match data_type {
            ModbusDataType::UInt16 => {
                Ok(Value::Number(registers.get(0).copied().unwrap_or(0).into()))
            }
            ModbusDataType::Int16 => {
                let val = registers.get(0).copied().unwrap_or(0) as i16;
                Ok(Value::Number(val.into()))
            }
            ModbusDataType::UInt32 => {
                if registers.len() < 2 {
                    return Err(DeviceError::ProtocolError("寄存器数据不足".into()));
                }
                let val = ((registers[0] as u32) << 16) | (registers[1] as u32);
                Ok(Value::Number(val.into()))
            }
            ModbusDataType::Int32 => {
                if registers.len() < 2 {
                    return Err(DeviceError::ProtocolError("寄存器数据不足".into()));
                }
                let val = (((registers[0] as u32) << 16) | (registers[1] as u32)) as i32;
                Ok(Value::Number(val.into()))
            }
            ModbusDataType::UInt32LE => {
                if registers.len() < 2 {
                    return Err(DeviceError::ProtocolError("寄存器数据不足".into()));
                }
                let val = ((registers[1] as u32) << 16) | (registers[0] as u32);
                Ok(Value::Number(val.into()))
            }
            ModbusDataType::Int32LE => {
                if registers.len() < 2 {
                    return Err(DeviceError::ProtocolError("寄存器数据不足".into()));
                }
                let val = (((registers[1] as u32) << 16) | (registers[0] as u32)) as i32;
                Ok(Value::Number(val.into()))
            }
            ModbusDataType::Float32 => {
                if registers.len() < 2 {
                    return Err(DeviceError::ProtocolError("寄存器数据不足".into()));
                }
                let bytes = [
                    (registers[0] >> 8) as u8,
                    registers[0] as u8,
                    (registers[1] >> 8) as u8,
                    registers[1] as u8,
                ];
                let val = f32::from_be_bytes(bytes);
                Ok(serde_json::Number::from_f64(val as f64)
                    .map(Value::Number)
                    .unwrap_or(Value::Null))
            }
            ModbusDataType::Float32LE => {
                if registers.len() < 2 {
                    return Err(DeviceError::ProtocolError("寄存器数据不足".into()));
                }
                let bytes = [
                    registers[1] as u8,
                    (registers[1] >> 8) as u8,
                    registers[0] as u8,
                    (registers[0] >> 8) as u8,
                ];
                let val = f32::from_le_bytes(bytes);
                Ok(serde_json::Number::from_f64(val as f64)
                    .map(Value::Number)
                    .unwrap_or(Value::Null))
            }
            ModbusDataType::Float64 => {
                if registers.len() < 4 {
                    return Err(DeviceError::ProtocolError("寄存器数据不足".into()));
                }
                let bytes = [
                    (registers[0] >> 8) as u8,
                    registers[0] as u8,
                    (registers[1] >> 8) as u8,
                    registers[1] as u8,
                    (registers[2] >> 8) as u8,
                    registers[2] as u8,
                    (registers[3] >> 8) as u8,
                    registers[3] as u8,
                ];
                let val = f64::from_be_bytes(bytes);
                Ok(serde_json::Number::from_f64(val)
                    .map(Value::Number)
                    .unwrap_or(Value::Null))
            }
            ModbusDataType::Bool => {
                Err(DeviceError::ProtocolError("Bool类型应使用线圈操作".into()))
            }
        }
    }

    /// 将值转换为寄存器数据
    fn value_to_registers(value: Value, data_type: ModbusDataType) -> Result<Vec<u16>> {
        match data_type {
            ModbusDataType::UInt16 => {
                let val = value
                    .as_u64()
                    .ok_or_else(|| DeviceError::ConfigError("无效的UInt16值".into()))?;
                if val > u16::MAX as u64 {
                    return Err(DeviceError::ConfigError("值超出UInt16范围".into()));
                }
                Ok(vec![val as u16])
            }
            ModbusDataType::Int16 => {
                let val = value
                    .as_i64()
                    .ok_or_else(|| DeviceError::ConfigError("无效的Int16值".into()))?;
                if val < i16::MIN as i64 || val > i16::MAX as i64 {
                    return Err(DeviceError::ConfigError("值超出Int16范围".into()));
                }
                Ok(vec![val as i16 as u16])
            }
            ModbusDataType::UInt32 => {
                let val = value
                    .as_u64()
                    .ok_or_else(|| DeviceError::ConfigError("无效的UInt32值".into()))?;
                if val > u32::MAX as u64 {
                    return Err(DeviceError::ConfigError("值超出UInt32范围".into()));
                }
                let val = val as u32;
                Ok(vec![(val >> 16) as u16, val as u16])
            }
            ModbusDataType::Int32 => {
                let val = value
                    .as_i64()
                    .ok_or_else(|| DeviceError::ConfigError("无效的Int32值".into()))?;
                if val < i32::MIN as i64 || val > i32::MAX as i64 {
                    return Err(DeviceError::ConfigError("值超出Int32范围".into()));
                }
                let val = val as i32 as u32;
                Ok(vec![(val >> 16) as u16, val as u16])
            }
            ModbusDataType::UInt32LE => {
                let val = value
                    .as_u64()
                    .ok_or_else(|| DeviceError::ConfigError("无效的UInt32值".into()))?;
                if val > u32::MAX as u64 {
                    return Err(DeviceError::ConfigError("值超出UInt32范围".into()));
                }
                let val = val as u32;
                Ok(vec![val as u16, (val >> 16) as u16])
            }
            ModbusDataType::Int32LE => {
                let val = value
                    .as_i64()
                    .ok_or_else(|| DeviceError::ConfigError("无效的Int32值".into()))?;
                if val < i32::MIN as i64 || val > i32::MAX as i64 {
                    return Err(DeviceError::ConfigError("值超出Int32范围".into()));
                }
                let val = val as i32 as u32;
                Ok(vec![val as u16, (val >> 16) as u16])
            }
            ModbusDataType::Float32 => {
                let val = value
                    .as_f64()
                    .ok_or_else(|| DeviceError::ConfigError("无效的Float32值".into()))?
                    as f32;
                let bytes = val.to_be_bytes();
                Ok(vec![
                    u16::from_be_bytes([bytes[0], bytes[1]]),
                    u16::from_be_bytes([bytes[2], bytes[3]]),
                ])
            }
            ModbusDataType::Float32LE => {
                let val = value
                    .as_f64()
                    .ok_or_else(|| DeviceError::ConfigError("无效的Float32值".into()))?
                    as f32;
                let bytes = val.to_le_bytes();
                Ok(vec![
                    u16::from_le_bytes([bytes[2], bytes[3]]),
                    u16::from_le_bytes([bytes[0], bytes[1]]),
                ])
            }
            ModbusDataType::Float64 => {
                let val = value
                    .as_f64()
                    .ok_or_else(|| DeviceError::ConfigError("无效的Float64值".into()))?;
                let bytes = val.to_be_bytes();
                Ok(vec![
                    u16::from_be_bytes([bytes[0], bytes[1]]),
                    u16::from_be_bytes([bytes[2], bytes[3]]),
                    u16::from_be_bytes([bytes[4], bytes[5]]),
                    u16::from_be_bytes([bytes[6], bytes[7]]),
                ])
            }
            ModbusDataType::Bool => {
                Err(DeviceError::ProtocolError("Bool类型应使用线圈操作".into()))
            }
        }
    }
}

#[async_trait]
impl Protocol for ModbusProtocol {
    fn from_config(channel_id: u32, params: &HashMap<String, Value>) -> Result<Box<dyn Protocol>> {
        let conn_type = params
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DeviceError::ConfigError("Modbus缺少type参数".into()))?;

        match conn_type {
            "tcp" => {
                let addr = params
                    .get("addr")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        DeviceError::ConfigError("Modbus TCP模式缺少addr参数".into())
                    })?
                    .to_string();

                let port = params
                    .get("port")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| {
                        DeviceError::ConfigError("Modbus TCP模式缺少port参数".into())
                    })? as u16;

                let slave_id = params
                    .get("slave_id")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as u8;

                // 读取自动召唤配置
                let auto_call_configs = if let Some(auto_call_arr) = params.get("auto_call").and_then(|v| v.as_array()) {
                    auto_call_arr.iter().filter_map(|item| {
                        serde_json::from_value::<AutoCallConfig>(item.clone()).ok()
                    }).collect()
                } else {
                    Vec::new()
                };

                let protocol = Self {
                    channel_id,
                    addr,
                    port,
                    slave_id,
                    cache: Arc::new(RwLock::new(HashMap::new())),
                    auto_call_configs: auto_call_configs.clone(),
                };

                // 启动自动召唤任务
                if !auto_call_configs.is_empty() {
                    info!("启动 {} 个自动召唤任务", auto_call_configs.len());
                    protocol.start_auto_call_tasks();
                }

                Ok(Box::new(protocol))
            }
            "serial" => {
                Err(DeviceError::ConfigError(
                    "Modbus串口模式暂未实现".into(),
                ))
            }
            _ => Err(DeviceError::ConfigError(format!(
                "不支持的Modbus连接类型: {}",
                conn_type
            ))),
        }
    }

    async fn execute(&mut self, command: &str, params: Value) -> Result<Value> {
        let mut ctx = self.connect().await?;

        match command {
            "read" | "read_typed" => {
                // 支持指定数据类型的读取
                let addr = params
                    .get("addr")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| DeviceError::ConfigError("缺少addr参数".into()))?
                    as u16;

                let data_type_str = params
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("uint16");

                let use_cache = params
                    .get("use_cache")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true); // 默认使用缓存

                // 尝试从缓存读取
                if use_cache {
                    if let Some(cached_value) = self.read_from_cache(addr, data_type_str).await? {
                        debug!("从缓存读取数据: addr={} type={}", addr, data_type_str);
                        return Ok(serde_json::json!({
                            "status": "success",
                            "value": cached_value,
                            "type": data_type_str,
                            "from_cache": true
                        }));
                    }
                }

                let data_type = ModbusDataType::from_str(data_type_str)?;

                // 根据数据类型读取
                if data_type.is_coil() {
                    let coil = ctx.read_coils(addr, 1).await
                        .map_err(|e| DeviceError::ConnectionError(format!("读取失败: {}", e)))?
                        .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;
                    
                    let value = coil.get(0).copied().unwrap_or(false);
                    
                    // 更新缓存
                    let mut cache = self.cache.write().await;
                    cache.insert(addr, (Value::Bool(value), data_type_str.to_string(), std::time::Instant::now()));
                    
                    Ok(serde_json::json!({
                        "status": "success",
                        "value": value,
                        "type": data_type_str,
                        "from_cache": false
                    }))
                } else {
                    let count = data_type.register_count();
                    let registers = ctx.read_holding_registers(addr, count).await
                        .map_err(|e| DeviceError::ConnectionError(format!("读取失败: {}", e)))?
                        .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

                    let value = Self::registers_to_value(&registers, data_type)?;

                    // 更新缓存
                    let mut cache = self.cache.write().await;
                    let now = std::time::Instant::now();
                    for (i, &reg) in registers.iter().enumerate() {
                        cache.insert(addr + i as u16, (Value::Number(reg.into()), "uint16".to_string(), now));
                    }

                    Ok(serde_json::json!({
                        "status": "success",
                        "value": value,
                        "type": data_type_str,
                        "registers": registers,
                        "from_cache": false
                    }))
                }
            }
            "write" | "write_typed" => {
                // 支持指定数据类型的写入
                let addr = params
                    .get("addr")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| DeviceError::ConfigError("缺少addr参数".into()))?
                    as u16;

                let value = params
                    .get("value")
                    .ok_or_else(|| DeviceError::ConfigError("缺少value参数".into()))?
                    .clone();

                let data_type_str = params
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("uint16");

                let data_type = ModbusDataType::from_str(data_type_str)?;

                // 根据数据类型写入
                if data_type.is_coil() {
                    let bool_val = value.as_bool()
                        .ok_or_else(|| DeviceError::ConfigError("Bool类型需要布尔值".into()))?;
                    
                    ctx.write_single_coil(addr, bool_val).await
                        .map_err(|e| DeviceError::ConnectionError(format!("写入失败: {}", e)))?
                        .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;
                } else {
                    let registers = Self::value_to_registers(value, data_type)?;
                    
                    if registers.len() == 1 {
                        ctx.write_single_register(addr, registers[0]).await
                            .map_err(|e| DeviceError::ConnectionError(format!("写入失败: {}", e)))?
                            .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;
                    } else {
                        ctx.write_multiple_registers(addr, &registers).await
                            .map_err(|e| DeviceError::ConnectionError(format!("写入失败: {}", e)))?
                            .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;
                    }
                }

                Ok(serde_json::json!({
                    "status": "success"
                }))
            }
            "read_holding_registers" | "read_holding" => {
                let addr = params
                    .get("addr")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| DeviceError::ConfigError("缺少addr参数".into()))?
                    as u16;

                let count = params
                    .get("count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as u16;

                let data = ctx.read_holding_registers(addr, count).await
                    .map_err(|e| DeviceError::ConnectionError(format!("读取失败: {}", e)))?
                    .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

                Ok(serde_json::json!({
                    "status": "success",
                    "data": data
                }))
            }
            "read_input_registers" | "read_input" => {
                let addr = params
                    .get("addr")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| DeviceError::ConfigError("缺少addr参数".into()))?
                    as u16;

                let count = params
                    .get("count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as u16;

                let data = ctx.read_input_registers(addr, count).await
                    .map_err(|e| DeviceError::ConnectionError(format!("读取失败: {}", e)))?
                    .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

                Ok(serde_json::json!({
                    "status": "success",
                    "data": data
                }))
            }
            "write_single_register" | "write_single" => {
                let addr = params
                    .get("addr")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| DeviceError::ConfigError("缺少addr参数".into()))?
                    as u16;

                let value = params
                    .get("value")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| DeviceError::ConfigError("缺少value参数".into()))?
                    as u16;

                ctx.write_single_register(addr, value).await
                    .map_err(|e| DeviceError::ConnectionError(format!("写入失败: {}", e)))?
                    .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

                Ok(serde_json::json!({"status": "success"}))
            }
            "write_multiple_registers" | "write_multiple" => {
                let addr = params
                    .get("addr")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| DeviceError::ConfigError("缺少addr参数".into()))?
                    as u16;

                let values: Vec<u16> = params
                    .get("values")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| DeviceError::ConfigError("缺少values参数".into()))?
                    .iter()
                    .filter_map(|v| v.as_u64().map(|n| n as u16))
                    .collect();

                ctx.write_multiple_registers(addr, &values).await
                    .map_err(|e| DeviceError::ConnectionError(format!("写入失败: {}", e)))?
                    .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

                Ok(serde_json::json!({"status": "success"}))
            }
            "read_coils" => {
                let addr = params
                    .get("addr")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| DeviceError::ConfigError("缺少addr参数".into()))?
                    as u16;

                let count = params
                    .get("count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as u16;

                let data = ctx.read_coils(addr, count).await
                    .map_err(|e| DeviceError::ConnectionError(format!("读取失败: {}", e)))?
                    .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

                Ok(serde_json::json!({
                    "status": "success",
                    "data": data
                }))
            }
            "read_discrete_inputs" | "read_discrete" => {
                let addr = params
                    .get("addr")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| DeviceError::ConfigError("缺少addr参数".into()))?
                    as u16;

                let count = params
                    .get("count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as u16;

                let data = ctx.read_discrete_inputs(addr, count).await
                    .map_err(|e| DeviceError::ConnectionError(format!("读取失败: {}", e)))?
                    .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

                Ok(serde_json::json!({
                    "status": "success",
                    "data": data
                }))
            }
            "write_single_coil" => {
                let addr = params
                    .get("addr")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| DeviceError::ConfigError("缺少addr参数".into()))?
                    as u16;

                let value = params
                    .get("value")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| DeviceError::ConfigError("缺少value参数".into()))?;

                ctx.write_single_coil(addr, value).await
                    .map_err(|e| DeviceError::ConnectionError(format!("写入失败: {}", e)))?
                    .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

                Ok(serde_json::json!({"status": "success"}))
            }
            "write_multiple_coils" => {
                let addr = params
                    .get("addr")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| DeviceError::ConfigError("缺少addr参数".into()))?
                    as u16;

                let values: Vec<bool> = params
                    .get("values")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| DeviceError::ConfigError("缺少values参数".into()))?
                    .iter()
                    .filter_map(|v| v.as_bool())
                    .collect();

                ctx.write_multiple_coils(addr, &values).await
                    .map_err(|e| DeviceError::ConnectionError(format!("写入失败: {}", e)))?
                    .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;

                Ok(serde_json::json!({"status": "success"}))
            }
            _ => Err(DeviceError::ProtocolError(format!(
                "不支持的命令: {}",
                command
            ))),
        }
    }

    async fn get_status(&self) -> Result<Value> {
        match self.connect().await {
            Ok(_) => Ok(serde_json::json!({
                "connected": true,
                "addr": self.addr,
                "port": self.port,
                "slave_id": self.slave_id
            })),
            Err(e) => Ok(serde_json::json!({
                "connected": false,
                "error": e.to_string()
            })),
        }
    }

    async fn write(&mut self, id: u32, value: i32) -> Result<()> {
        let mut ctx = self.connect().await?;
        ctx.write_single_register(id as u16, value as u16).await
            .map_err(|e| DeviceError::ConnectionError(format!("写入失败: {}", e)))?
            .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;
        Ok(())
    }

    async fn read(&self, id: u32) -> Result<i32> {
        // 优先从缓存读取
        if let Some(cached) = self.read_from_cache(id as u16, "int16").await? {
            if let Some(num) = cached.as_i64() {
                return Ok(num as i32);
            }
        }
        
        // 缓存未命中，从设备读取
        let mut ctx = self.connect().await?;
        let data = ctx.read_holding_registers(id as u16, 1).await
            .map_err(|e| DeviceError::ConnectionError(format!("读取失败: {}", e)))?
            .map_err(|e| DeviceError::ProtocolError(format!("Modbus异常: {:?}", e)))?;
        
        let value = data.get(0).copied().unwrap_or(0) as i32;
        
        // 更新缓存
        let mut cache = self.cache.write().await;
        cache.insert(id as u16, (Value::Number(value.into()), "int16".to_string(), std::time::Instant::now()));
        
        Ok(value)
    }

    fn name(&self) -> &str {
        "modbus"
    }
}
