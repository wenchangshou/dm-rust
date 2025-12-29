/// Modbus TCP 协议模拟器
///
/// 支持 Modbus TCP 协议，可配置多个 Slave ID 和各种寄存器类型。
///
/// # 功能码支持
/// - 0x01: 读线圈 (Read Coils)
/// - 0x02: 读离散输入 (Read Discrete Inputs)
/// - 0x03: 读保持寄存器 (Read Holding Registers)
/// - 0x04: 读输入寄存器 (Read Input Registers)
/// - 0x05: 写单个线圈 (Write Single Coil)
/// - 0x06: 写单个寄存器 (Write Single Register)
/// - 0x0F: 写多个线圈 (Write Multiple Coils)
/// - 0x10: 写多个寄存器 (Write Multiple Registers)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, warn};

use crate::tcp_simulator::handler::{HandleResult, ProtocolHandler};
use crate::tcp_simulator::state::SimulatorState;

/// Modbus 功能码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FunctionCode {
    ReadCoils = 0x01,
    ReadDiscreteInputs = 0x02,
    ReadHoldingRegisters = 0x03,
    ReadInputRegisters = 0x04,
    WriteSingleCoil = 0x05,
    WriteSingleRegister = 0x06,
    WriteMultipleCoils = 0x0F,
    WriteMultipleRegisters = 0x10,
}

impl FunctionCode {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::ReadCoils),
            0x02 => Some(Self::ReadDiscreteInputs),
            0x03 => Some(Self::ReadHoldingRegisters),
            0x04 => Some(Self::ReadInputRegisters),
            0x05 => Some(Self::WriteSingleCoil),
            0x06 => Some(Self::WriteSingleRegister),
            0x0F => Some(Self::WriteMultipleCoils),
            0x10 => Some(Self::WriteMultipleRegisters),
            _ => None,
        }
    }
}

/// Modbus 异常码
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ExceptionCode {
    IllegalFunction = 0x01,
    IllegalDataAddress = 0x02,
    IllegalDataValue = 0x03,
    SlaveDeviceFailure = 0x04,
}

/// 寄存器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegisterType {
    Coil,
    DiscreteInput,
    HoldingRegister,
    InputRegister,
}

impl RegisterType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Coil => "coil",
            Self::DiscreteInput => "discrete_input",
            Self::HoldingRegister => "holding_register",
            Self::InputRegister => "input_register",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "coil" => Some(Self::Coil),
            "discrete_input" => Some(Self::DiscreteInput),
            "holding_register" => Some(Self::HoldingRegister),
            "input_register" => Some(Self::InputRegister),
            _ => None,
        }
    }
}

/// 数据类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    Bit,
    Uint16,
    Int16,
    Uint32,
    Int32,
    Float32,
}

/// 值生成模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratorMode {
    /// 固定值（默认）
    Fixed,
    /// 随机值
    Random,
    /// 递增
    Increment,
    /// 递减
    Decrement,
    /// 正弦波
    Sine,
    /// 开关切换（用于 Bit 类型）
    Toggle,
    /// 序列循环
    Sequence,
}

impl Default for GeneratorMode {
    fn default() -> Self {
        Self::Fixed
    }
}

/// 值生成器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    /// 生成模式
    pub mode: GeneratorMode,
    /// 最小值（用于 random, increment, decrement, sine）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// 最大值（用于 random, increment, decrement, sine）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// 步长（用于 increment, decrement）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    /// 周期（毫秒，用于 sine, toggle）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<u64>,
    /// 序列值（用于 sequence）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<Vec<f64>>,
    /// 更新间隔（毫秒）
    #[serde(default = "default_interval")]
    pub interval: u64,
}

fn default_interval() -> u64 {
    1000
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            mode: GeneratorMode::Fixed,
            min: None,
            max: None,
            step: None,
            period: None,
            sequence: None,
            interval: 1000,
        }
    }
}

impl GeneratorConfig {
    /// 创建固定值生成器
    pub fn fixed() -> Self {
        Self::default()
    }

    /// 创建随机值生成器
    pub fn random(min: f64, max: f64, interval: u64) -> Self {
        Self {
            mode: GeneratorMode::Random,
            min: Some(min),
            max: Some(max),
            interval,
            ..Default::default()
        }
    }

    /// 创建递增生成器
    pub fn increment(min: f64, max: f64, step: f64, interval: u64) -> Self {
        Self {
            mode: GeneratorMode::Increment,
            min: Some(min),
            max: Some(max),
            step: Some(step),
            interval,
            ..Default::default()
        }
    }

    /// 创建正弦波生成器
    pub fn sine(min: f64, max: f64, period: u64, interval: u64) -> Self {
        Self {
            mode: GeneratorMode::Sine,
            min: Some(min),
            max: Some(max),
            period: Some(period),
            interval,
            ..Default::default()
        }
    }

    /// 创建切换生成器
    pub fn toggle(period: u64) -> Self {
        Self {
            mode: GeneratorMode::Toggle,
            period: Some(period),
            interval: period,
            ..Default::default()
        }
    }
}

/// 寄存器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterConfig {
    pub address: u16,
    #[serde(rename = "type")]
    pub register_type: RegisterType,
    #[serde(rename = "dataType")]
    pub data_type: DataType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub value: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub readonly: Option<bool>,
    /// 值生成器配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<GeneratorConfig>,
    /// 生成器内部状态：序列索引
    #[serde(rename = "seqIndex", default, skip_serializing_if = "is_zero")]
    pub seq_index: usize,
    /// 生成器内部状态：上次更新时间戳
    #[serde(rename = "lastUpdate", default, skip_serializing_if = "is_zero_u64")]
    pub last_update: u64,
}

fn is_zero(v: &usize) -> bool {
    *v == 0
}

fn is_zero_u64(v: &u64) -> bool {
    *v == 0
}

/// Slave 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaveConfig {
    #[serde(rename = "slaveId")]
    pub slave_id: u8,
    pub registers: Vec<RegisterConfig>,
}

/// Modbus 模拟器状态值
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModbusValues {
    pub slaves: Vec<SlaveConfig>,
    #[serde(rename = "defaultSlaveId", skip_serializing_if = "Option::is_none")]
    pub default_slave_id: Option<u8>,
}

impl ModbusValues {
    /// 从状态中获取 Modbus 值
    pub fn from_state(state: &SimulatorState) -> Self {
        // 直接解析 slaves 数组
        let slaves = state
            .get_value("slaves")
            .and_then(|v| serde_json::from_value::<Vec<SlaveConfig>>(v.clone()).ok())
            .unwrap_or_default();

        let default_slave_id = state
            .get_value("defaultSlaveId")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8);

        Self {
            slaves,
            default_slave_id,
        }
    }

    /// 保存到状态
    pub fn save_to_state(&self, state: &mut SimulatorState) {
        state.set_value("slaves", json!(self.slaves));
        if let Some(default_id) = self.default_slave_id {
            state.set_value("defaultSlaveId", json!(default_id));
        }
    }

    /// 获取 Slave
    pub fn get_slave(&self, slave_id: u8) -> Option<&SlaveConfig> {
        self.slaves.iter().find(|s| s.slave_id == slave_id)
    }

    /// 获取 Slave (可变)
    pub fn get_slave_mut(&mut self, slave_id: u8) -> Option<&mut SlaveConfig> {
        self.slaves.iter_mut().find(|s| s.slave_id == slave_id)
    }

    /// 添加 Slave
    pub fn add_slave(&mut self, slave_id: u8) -> bool {
        if self.get_slave(slave_id).is_some() {
            return false;
        }
        self.slaves.push(SlaveConfig {
            slave_id,
            registers: Vec::new(),
        });
        true
    }

    /// 删除 Slave
    pub fn remove_slave(&mut self, slave_id: u8) -> bool {
        let len_before = self.slaves.len();
        self.slaves.retain(|s| s.slave_id != slave_id);
        self.slaves.len() < len_before
    }

    /// 读取线圈值
    pub fn read_coils(&self, slave_id: u8, start_addr: u16, quantity: u16) -> Option<Vec<bool>> {
        let slave = self.get_slave(slave_id)?;
        let mut result = vec![false; quantity as usize];

        for i in 0..quantity {
            let addr = start_addr + i;
            if let Some(reg) = slave
                .registers
                .iter()
                .find(|r| r.register_type == RegisterType::Coil && r.address == addr)
            {
                result[i as usize] = reg.value.as_bool().unwrap_or(false);
            }
        }

        Some(result)
    }

    /// 读取离散输入
    pub fn read_discrete_inputs(
        &self,
        slave_id: u8,
        start_addr: u16,
        quantity: u16,
    ) -> Option<Vec<bool>> {
        let slave = self.get_slave(slave_id)?;
        let mut result = vec![false; quantity as usize];

        for i in 0..quantity {
            let addr = start_addr + i;
            if let Some(reg) = slave
                .registers
                .iter()
                .find(|r| r.register_type == RegisterType::DiscreteInput && r.address == addr)
            {
                result[i as usize] = reg.value.as_bool().unwrap_or(false);
            }
        }

        Some(result)
    }

    /// 读取保持寄存器
    pub fn read_holding_registers(
        &self,
        slave_id: u8,
        start_addr: u16,
        quantity: u16,
    ) -> Option<Vec<u16>> {
        let slave = self.get_slave(slave_id)?;
        let mut result = vec![0u16; quantity as usize];

        for i in 0..quantity {
            let addr = start_addr + i;
            if let Some(reg) = slave
                .registers
                .iter()
                .find(|r| r.register_type == RegisterType::HoldingRegister && r.address == addr)
            {
                result[i as usize] = value_to_u16(&reg.value);
            }
        }

        Some(result)
    }

    /// 读取输入寄存器
    pub fn read_input_registers(
        &self,
        slave_id: u8,
        start_addr: u16,
        quantity: u16,
    ) -> Option<Vec<u16>> {
        let slave = self.get_slave(slave_id)?;
        let mut result = vec![0u16; quantity as usize];

        for i in 0..quantity {
            let addr = start_addr + i;
            if let Some(reg) = slave
                .registers
                .iter()
                .find(|r| r.register_type == RegisterType::InputRegister && r.address == addr)
            {
                result[i as usize] = value_to_u16(&reg.value);
            }
        }

        Some(result)
    }

    /// 写单个线圈
    pub fn write_single_coil(&mut self, slave_id: u8, address: u16, value: bool) -> bool {
        if let Some(slave) = self.get_slave_mut(slave_id) {
            if let Some(reg) = slave
                .registers
                .iter_mut()
                .find(|r| r.register_type == RegisterType::Coil && r.address == address)
            {
                reg.value = json!(value);
                return true;
            }
            // 自动创建寄存器
            slave.registers.push(RegisterConfig {
                address,
                register_type: RegisterType::Coil,
                data_type: DataType::Bit,
                name: None,
                value: json!(value),
                readonly: None,
                generator: None,
                seq_index: 0,
                last_update: 0,
            });
            return true;
        }
        false
    }

    /// 写单个保持寄存器
    pub fn write_single_register(&mut self, slave_id: u8, address: u16, value: u16) -> bool {
        if let Some(slave) = self.get_slave_mut(slave_id) {
            if let Some(reg) = slave
                .registers
                .iter_mut()
                .find(|r| r.register_type == RegisterType::HoldingRegister && r.address == address)
            {
                reg.value = json!(value);
                return true;
            }
            // 自动创建寄存器
            slave.registers.push(RegisterConfig {
                address,
                register_type: RegisterType::HoldingRegister,
                data_type: DataType::Uint16,
                name: None,
                value: json!(value),
                readonly: None,
                generator: None,
                seq_index: 0,
                last_update: 0,
            });
            return true;
        }
        false
    }

    /// 写多个线圈
    pub fn write_multiple_coils(
        &mut self,
        slave_id: u8,
        start_addr: u16,
        values: &[bool],
    ) -> bool {
        for (i, &value) in values.iter().enumerate() {
            if !self.write_single_coil(slave_id, start_addr + i as u16, value) {
                return false;
            }
        }
        true
    }

    /// 写多个保持寄存器
    pub fn write_multiple_registers(
        &mut self,
        slave_id: u8,
        start_addr: u16,
        values: &[u16],
    ) -> bool {
        for (i, &value) in values.iter().enumerate() {
            if !self.write_single_register(slave_id, start_addr + i as u16, value) {
                return false;
            }
        }
        true
    }

    /// 设置寄存器配置
    pub fn set_register(&mut self, slave_id: u8, register: RegisterConfig) -> bool {
        if let Some(slave) = self.get_slave_mut(slave_id) {
            // 查找并更新，或添加新的
            if let Some(existing) = slave.registers.iter_mut().find(|r| {
                r.register_type == register.register_type && r.address == register.address
            }) {
                *existing = register;
            } else {
                slave.registers.push(register);
            }
            return true;
        }
        false
    }

    /// 删除寄存器
    pub fn delete_register(
        &mut self,
        slave_id: u8,
        register_type: RegisterType,
        address: u16,
    ) -> bool {
        if let Some(slave) = self.get_slave_mut(slave_id) {
            let len_before = slave.registers.len();
            slave
                .registers
                .retain(|r| !(r.register_type == register_type && r.address == address));
            return slave.registers.len() < len_before;
        }
        false
    }

    /// 更新所有寄存器的生成值
    /// 返回是否有值被更新
    pub fn tick_generators(&mut self) -> bool {
        use rand::Rng;
        use std::f64::consts::PI;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let mut updated = false;

        for slave in &mut self.slaves {
            for reg in &mut slave.registers {
                if let Some(ref gen) = reg.generator {
                    // 检查是否需要更新
                    if now < reg.last_update + gen.interval {
                        continue;
                    }

                    let new_value = match gen.mode {
                        GeneratorMode::Fixed => continue, // 固定值不更新

                        GeneratorMode::Random => {
                            let min = gen.min.unwrap_or(0.0);
                            let max = gen.max.unwrap_or(100.0);
                            let mut rng = rand::thread_rng();
                            if reg.data_type == DataType::Bit {
                                json!(rng.gen_bool(0.5))
                            } else {
                                let val = rng.gen_range(min..=max);
                                convert_to_data_type(val, reg.data_type)
                            }
                        }

                        GeneratorMode::Increment => {
                            let min = gen.min.unwrap_or(0.0);
                            let max = gen.max.unwrap_or(100.0);
                            let step = gen.step.unwrap_or(1.0);
                            let current = reg.value.as_f64().unwrap_or(min);
                            let mut next = current + step;
                            if next > max {
                                next = min; // 循环
                            }
                            convert_to_data_type(next, reg.data_type)
                        }

                        GeneratorMode::Decrement => {
                            let min = gen.min.unwrap_or(0.0);
                            let max = gen.max.unwrap_or(100.0);
                            let step = gen.step.unwrap_or(1.0);
                            let current = reg.value.as_f64().unwrap_or(max);
                            let mut next = current - step;
                            if next < min {
                                next = max; // 循环
                            }
                            convert_to_data_type(next, reg.data_type)
                        }

                        GeneratorMode::Sine => {
                            let min = gen.min.unwrap_or(0.0);
                            let max = gen.max.unwrap_or(100.0);
                            let period = gen.period.unwrap_or(10000) as f64; // 默认10秒周期
                            let amplitude = (max - min) / 2.0;
                            let offset = min + amplitude;
                            let phase = (now as f64 / period) * 2.0 * PI;
                            let val = offset + amplitude * phase.sin();
                            convert_to_data_type(val, reg.data_type)
                        }

                        GeneratorMode::Toggle => {
                            let current = reg.value.as_bool().unwrap_or(false);
                            json!(!current)
                        }

                        GeneratorMode::Sequence => {
                            if let Some(ref seq) = gen.sequence {
                                if seq.is_empty() {
                                    continue;
                                }
                                let idx = reg.seq_index % seq.len();
                                reg.seq_index = (idx + 1) % seq.len();
                                convert_to_data_type(seq[idx], reg.data_type)
                            } else {
                                continue;
                            }
                        }
                    };

                    reg.value = new_value;
                    reg.last_update = now;
                    updated = true;
                }
            }
        }

        updated
    }
}

/// 将 f64 转换为对应数据类型的 JSON 值
fn convert_to_data_type(val: f64, data_type: DataType) -> Value {
    match data_type {
        DataType::Bit => json!(val != 0.0),
        DataType::Uint16 => json!(val.round() as u16),
        DataType::Int16 => json!(val.round() as i16),
        DataType::Uint32 => json!(val.round() as u32),
        DataType::Int32 => json!(val.round() as i32),
        DataType::Float32 => json!(val as f32),
    }
}

/// 将 JSON Value 转换为 u16
fn value_to_u16(value: &Value) -> u16 {
    if let Some(n) = value.as_u64() {
        n as u16
    } else if let Some(n) = value.as_i64() {
        n as u16
    } else if let Some(n) = value.as_f64() {
        n as u16
    } else {
        0
    }
}

/// Modbus TCP 协议处理器
pub struct ModbusHandler;

impl ModbusHandler {
    pub fn new() -> Self {
        Self
    }

    /// 解析 Modbus TCP 帧
    fn parse_request(data: &[u8]) -> Option<ModbusRequest> {
        if data.len() < 8 {
            return None; // MBAP header (7) + at least 1 byte PDU
        }

        // MBAP Header
        let transaction_id = u16::from_be_bytes([data[0], data[1]]);
        let protocol_id = u16::from_be_bytes([data[2], data[3]]);
        let length = u16::from_be_bytes([data[4], data[5]]) as usize;
        let unit_id = data[6];

        // 验证协议 ID (必须为 0)
        if protocol_id != 0 {
            return None;
        }

        // 验证数据长度
        if data.len() < 6 + length {
            return None;
        }

        // PDU
        let function_code = data[7];
        let pdu_data = &data[8..6 + length];

        Some(ModbusRequest {
            transaction_id,
            unit_id,
            function_code,
            data: pdu_data.to_vec(),
        })
    }

    /// 构建响应帧
    fn build_response(
        transaction_id: u16,
        unit_id: u8,
        function_code: u8,
        data: &[u8],
    ) -> Vec<u8> {
        let length = 2 + data.len(); // unit_id + function_code + data
        let mut response = Vec::with_capacity(7 + length);

        // MBAP Header
        response.extend_from_slice(&transaction_id.to_be_bytes());
        response.extend_from_slice(&0u16.to_be_bytes()); // Protocol ID
        response.extend_from_slice(&(length as u16).to_be_bytes());
        response.push(unit_id);

        // PDU
        response.push(function_code);
        response.extend_from_slice(data);

        response
    }

    /// 构建异常响应
    fn build_exception(
        transaction_id: u16,
        unit_id: u8,
        function_code: u8,
        exception_code: ExceptionCode,
    ) -> Vec<u8> {
        Self::build_response(
            transaction_id,
            unit_id,
            function_code | 0x80,
            &[exception_code as u8],
        )
    }

    /// 处理读线圈请求
    fn handle_read_coils(
        &self,
        req: &ModbusRequest,
        values: &ModbusValues,
    ) -> HandleResult {
        if req.data.len() < 4 {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        let start_addr = u16::from_be_bytes([req.data[0], req.data[1]]);
        let quantity = u16::from_be_bytes([req.data[2], req.data[3]]);

        if quantity == 0 || quantity > 2000 {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        match values.read_coils(req.unit_id, start_addr, quantity) {
            Some(coils) => {
                let byte_count = (quantity + 7) / 8;
                let mut data = vec![byte_count as u8];

                for byte_idx in 0..byte_count as usize {
                    let mut byte_val = 0u8;
                    for bit_idx in 0..8 {
                        let coil_idx = byte_idx * 8 + bit_idx;
                        if coil_idx < coils.len() && coils[coil_idx] {
                            byte_val |= 1 << bit_idx;
                        }
                    }
                    data.push(byte_val);
                }

                HandleResult::Response(Self::build_response(
                    req.transaction_id,
                    req.unit_id,
                    req.function_code,
                    &data,
                ))
            }
            None => HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::SlaveDeviceFailure,
            )),
        }
    }

    /// 处理读离散输入请求
    fn handle_read_discrete_inputs(
        &self,
        req: &ModbusRequest,
        values: &ModbusValues,
    ) -> HandleResult {
        if req.data.len() < 4 {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        let start_addr = u16::from_be_bytes([req.data[0], req.data[1]]);
        let quantity = u16::from_be_bytes([req.data[2], req.data[3]]);

        if quantity == 0 || quantity > 2000 {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        match values.read_discrete_inputs(req.unit_id, start_addr, quantity) {
            Some(inputs) => {
                let byte_count = (quantity + 7) / 8;
                let mut data = vec![byte_count as u8];

                for byte_idx in 0..byte_count as usize {
                    let mut byte_val = 0u8;
                    for bit_idx in 0..8 {
                        let input_idx = byte_idx * 8 + bit_idx;
                        if input_idx < inputs.len() && inputs[input_idx] {
                            byte_val |= 1 << bit_idx;
                        }
                    }
                    data.push(byte_val);
                }

                HandleResult::Response(Self::build_response(
                    req.transaction_id,
                    req.unit_id,
                    req.function_code,
                    &data,
                ))
            }
            None => HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::SlaveDeviceFailure,
            )),
        }
    }

    /// 处理读保持寄存器请求
    fn handle_read_holding_registers(
        &self,
        req: &ModbusRequest,
        values: &ModbusValues,
    ) -> HandleResult {
        if req.data.len() < 4 {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        let start_addr = u16::from_be_bytes([req.data[0], req.data[1]]);
        let quantity = u16::from_be_bytes([req.data[2], req.data[3]]);

        if quantity == 0 || quantity > 125 {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        match values.read_holding_registers(req.unit_id, start_addr, quantity) {
            Some(registers) => {
                let byte_count = quantity * 2;
                let mut data = vec![byte_count as u8];

                for reg in registers {
                    data.extend_from_slice(&reg.to_be_bytes());
                }

                HandleResult::Response(Self::build_response(
                    req.transaction_id,
                    req.unit_id,
                    req.function_code,
                    &data,
                ))
            }
            None => HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::SlaveDeviceFailure,
            )),
        }
    }

    /// 处理读输入寄存器请求
    fn handle_read_input_registers(
        &self,
        req: &ModbusRequest,
        values: &ModbusValues,
    ) -> HandleResult {
        if req.data.len() < 4 {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        let start_addr = u16::from_be_bytes([req.data[0], req.data[1]]);
        let quantity = u16::from_be_bytes([req.data[2], req.data[3]]);

        if quantity == 0 || quantity > 125 {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        match values.read_input_registers(req.unit_id, start_addr, quantity) {
            Some(registers) => {
                let byte_count = quantity * 2;
                let mut data = vec![byte_count as u8];

                for reg in registers {
                    data.extend_from_slice(&reg.to_be_bytes());
                }

                HandleResult::Response(Self::build_response(
                    req.transaction_id,
                    req.unit_id,
                    req.function_code,
                    &data,
                ))
            }
            None => HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::SlaveDeviceFailure,
            )),
        }
    }

    /// 处理写单个线圈请求
    fn handle_write_single_coil(
        &self,
        req: &ModbusRequest,
        values: &mut ModbusValues,
    ) -> HandleResult {
        if req.data.len() < 4 {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        let address = u16::from_be_bytes([req.data[0], req.data[1]]);
        let value = u16::from_be_bytes([req.data[2], req.data[3]]);

        // 值必须是 0x0000 (OFF) 或 0xFF00 (ON)
        let coil_value = match value {
            0x0000 => false,
            0xFF00 => true,
            _ => {
                return HandleResult::Response(Self::build_exception(
                    req.transaction_id,
                    req.unit_id,
                    req.function_code,
                    ExceptionCode::IllegalDataValue,
                ));
            }
        };

        if values.write_single_coil(req.unit_id, address, coil_value) {
            // 响应与请求相同
            HandleResult::Response(Self::build_response(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                &req.data[0..4],
            ))
        } else {
            HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::SlaveDeviceFailure,
            ))
        }
    }

    /// 处理写单个寄存器请求
    fn handle_write_single_register(
        &self,
        req: &ModbusRequest,
        values: &mut ModbusValues,
    ) -> HandleResult {
        if req.data.len() < 4 {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        let address = u16::from_be_bytes([req.data[0], req.data[1]]);
        let value = u16::from_be_bytes([req.data[2], req.data[3]]);

        if values.write_single_register(req.unit_id, address, value) {
            // 响应与请求相同
            HandleResult::Response(Self::build_response(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                &req.data[0..4],
            ))
        } else {
            HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::SlaveDeviceFailure,
            ))
        }
    }

    /// 处理写多个线圈请求
    fn handle_write_multiple_coils(
        &self,
        req: &ModbusRequest,
        values: &mut ModbusValues,
    ) -> HandleResult {
        if req.data.len() < 5 {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        let start_addr = u16::from_be_bytes([req.data[0], req.data[1]]);
        let quantity = u16::from_be_bytes([req.data[2], req.data[3]]);
        let byte_count = req.data[4] as usize;

        if quantity == 0 || quantity > 1968 {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        let expected_bytes = ((quantity + 7) / 8) as usize;
        if byte_count != expected_bytes || req.data.len() < 5 + byte_count {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        // 解析线圈值
        let mut coils = Vec::with_capacity(quantity as usize);
        for i in 0..quantity as usize {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            let value = (req.data[5 + byte_idx] >> bit_idx) & 1 == 1;
            coils.push(value);
        }

        if values.write_multiple_coils(req.unit_id, start_addr, &coils) {
            HandleResult::Response(Self::build_response(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                &req.data[0..4], // 起始地址和数量
            ))
        } else {
            HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::SlaveDeviceFailure,
            ))
        }
    }

    /// 处理写多个寄存器请求
    fn handle_write_multiple_registers(
        &self,
        req: &ModbusRequest,
        values: &mut ModbusValues,
    ) -> HandleResult {
        if req.data.len() < 5 {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        let start_addr = u16::from_be_bytes([req.data[0], req.data[1]]);
        let quantity = u16::from_be_bytes([req.data[2], req.data[3]]);
        let byte_count = req.data[4] as usize;

        if quantity == 0 || quantity > 123 {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        let expected_bytes = (quantity * 2) as usize;
        if byte_count != expected_bytes || req.data.len() < 5 + byte_count {
            return HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::IllegalDataValue,
            ));
        }

        // 解析寄存器值
        let mut registers = Vec::with_capacity(quantity as usize);
        for i in 0..quantity as usize {
            let value = u16::from_be_bytes([req.data[5 + i * 2], req.data[6 + i * 2]]);
            registers.push(value);
        }

        if values.write_multiple_registers(req.unit_id, start_addr, &registers) {
            HandleResult::Response(Self::build_response(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                &req.data[0..4], // 起始地址和数量
            ))
        } else {
            HandleResult::Response(Self::build_exception(
                req.transaction_id,
                req.unit_id,
                req.function_code,
                ExceptionCode::SlaveDeviceFailure,
            ))
        }
    }
}

/// Modbus 请求结构
struct ModbusRequest {
    transaction_id: u16,
    unit_id: u8,
    function_code: u8,
    data: Vec<u8>,
}

#[async_trait]
impl ProtocolHandler for ModbusHandler {
    fn name(&self) -> &str {
        "modbus"
    }

    fn description(&self) -> &str {
        "Modbus TCP 协议模拟器，支持多 Slave 和各种寄存器类型"
    }

    fn default_port(&self) -> u16 {
        502
    }

    async fn handle(&self, data: &[u8], state: &mut SimulatorState) -> HandleResult {
        // 检查设备状态
        if !state.online {
            debug!("Modbus: 设备离线，不响应");
            return HandleResult::NoResponse;
        }

        if let Some(fault) = &state.fault {
            debug!("Modbus: 设备故障 ({})", fault);
            match fault.as_str() {
                "timeout" => return HandleResult::NoResponse,
                "protocol_error" => {
                    return HandleResult::Response(vec![0xFF; 8]); // 无效响应
                }
                _ => {}
            }
        }

        // 解析请求
        let req = match Self::parse_request(data) {
            Some(r) => r,
            None => {
                if data.len() < 8 {
                    return HandleResult::NeedMoreData;
                }
                warn!("Modbus: 无法解析请求");
                return HandleResult::Error("无法解析 Modbus 请求".to_string());
            }
        };

        debug!(
            "Modbus: 收到请求 - Transaction: {}, Unit: {}, FC: 0x{:02X}",
            req.transaction_id, req.unit_id, req.function_code
        );

        // 获取 Modbus 值
        let mut values = ModbusValues::from_state(state);

        // 检查 Slave 是否存在，如果不存在则自动创建
        if values.get_slave(req.unit_id).is_none() {
            values.add_slave(req.unit_id);
        }

        // 处理功能码
        let result = match FunctionCode::from_u8(req.function_code) {
            Some(FunctionCode::ReadCoils) => self.handle_read_coils(&req, &values),
            Some(FunctionCode::ReadDiscreteInputs) => {
                self.handle_read_discrete_inputs(&req, &values)
            }
            Some(FunctionCode::ReadHoldingRegisters) => {
                self.handle_read_holding_registers(&req, &values)
            }
            Some(FunctionCode::ReadInputRegisters) => {
                self.handle_read_input_registers(&req, &values)
            }
            Some(FunctionCode::WriteSingleCoil) => {
                self.handle_write_single_coil(&req, &mut values)
            }
            Some(FunctionCode::WriteSingleRegister) => {
                self.handle_write_single_register(&req, &mut values)
            }
            Some(FunctionCode::WriteMultipleCoils) => {
                self.handle_write_multiple_coils(&req, &mut values)
            }
            Some(FunctionCode::WriteMultipleRegisters) => {
                self.handle_write_multiple_registers(&req, &mut values)
            }
            None => {
                warn!("Modbus: 不支持的功能码 0x{:02X}", req.function_code);
                HandleResult::Response(Self::build_exception(
                    req.transaction_id,
                    req.unit_id,
                    req.function_code,
                    ExceptionCode::IllegalFunction,
                ))
            }
        };

        // 保存更新后的值
        values.save_to_state(state);

        result
    }

    fn supported_commands(&self) -> Vec<String> {
        vec![
            "0x01: Read Coils".to_string(),
            "0x02: Read Discrete Inputs".to_string(),
            "0x03: Read Holding Registers".to_string(),
            "0x04: Read Input Registers".to_string(),
            "0x05: Write Single Coil".to_string(),
            "0x06: Write Single Register".to_string(),
            "0x0F: Write Multiple Coils".to_string(),
            "0x10: Write Multiple Registers".to_string(),
        ]
    }

    fn metadata(&self) -> Value {
        json!({
            "name": self.name(),
            "description": self.description(),
            "default_port": self.default_port(),
            "commands": self.supported_commands(),
            "register_types": ["coil", "discrete_input", "holding_register", "input_register"],
            "data_types": ["bit", "uint16", "int16", "uint32", "int32", "float32"]
        })
    }
}

impl Default for ModbusHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request() {
        // Read Holding Registers: Transaction 1, Unit 1, FC 0x03, Addr 0, Qty 10
        let data = vec![
            0x00, 0x01, // Transaction ID
            0x00, 0x00, // Protocol ID
            0x00, 0x06, // Length
            0x01, // Unit ID
            0x03, // Function Code
            0x00, 0x00, // Start Address
            0x00, 0x0A, // Quantity
        ];

        let req = ModbusHandler::parse_request(&data).unwrap();
        assert_eq!(req.transaction_id, 1);
        assert_eq!(req.unit_id, 1);
        assert_eq!(req.function_code, 0x03);
        assert_eq!(req.data, vec![0x00, 0x00, 0x00, 0x0A]);
    }

    #[test]
    fn test_build_response() {
        let response = ModbusHandler::build_response(1, 1, 0x03, &[0x02, 0x00, 0x64]);

        assert_eq!(response[0..2], [0x00, 0x01]); // Transaction ID
        assert_eq!(response[2..4], [0x00, 0x00]); // Protocol ID
        assert_eq!(response[4..6], [0x00, 0x05]); // Length
        assert_eq!(response[6], 0x01); // Unit ID
        assert_eq!(response[7], 0x03); // Function Code
        assert_eq!(response[8..], [0x02, 0x00, 0x64]); // Data
    }

    #[test]
    fn test_modbus_values() {
        let mut values = ModbusValues::default();

        // 添加 Slave
        assert!(values.add_slave(1));
        assert!(!values.add_slave(1)); // 重复添加

        // 写入寄存器
        assert!(values.write_single_register(1, 0, 100));
        assert!(values.write_single_coil(1, 0, true));

        // 读取寄存器
        let registers = values.read_holding_registers(1, 0, 1).unwrap();
        assert_eq!(registers[0], 100);

        let coils = values.read_coils(1, 0, 1).unwrap();
        assert!(coils[0]);
    }

    #[tokio::test]
    async fn test_handler() {
        let handler = ModbusHandler::new();
        let mut state = SimulatorState::default();

        // 初始化 Slave
        let mut values = ModbusValues::default();
        values.add_slave(1);
        values.write_single_register(1, 0, 12345);
        values.save_to_state(&mut state);

        // Read Holding Registers
        let request = vec![
            0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01, 0x03, 0x00, 0x00, 0x00, 0x01,
        ];

        let result = handler.handle(&request, &mut state).await;
        if let HandleResult::Response(response) = result {
            assert_eq!(response[7], 0x03); // Function Code
            assert_eq!(response[8], 0x02); // Byte Count
            let value = u16::from_be_bytes([response[9], response[10]]);
            assert_eq!(value, 12345);
        } else {
            panic!("Expected Response");
        }
    }
}
