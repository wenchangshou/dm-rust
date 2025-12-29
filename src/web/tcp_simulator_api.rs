/// TCP 模拟器 HTTP API
///
/// 提供 REST API 来管理 TCP 协议模拟器。

use axum::{
    extract::{Extension, Path},
    Json,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use axum::extract::Query;
use crate::tcp_simulator::{
    protocols::{ModbusValues, RegisterConfig, RegisterType, SlaveConfig},
    state::PacketRecord,
    ProtocolInfo, SimulatorInfo, SimulatorStatus, TcpSimulatorConfig, TcpSimulatorManager,
};
use crate::web::response::ApiResponse;

/// 创建模拟器请求
#[derive(Debug, Deserialize)]
pub struct CreateSimulatorRequest {
    /// 显示名称
    pub name: String,
    /// 协议类型
    pub protocol: String,
    /// 绑定地址（可选，默认 0.0.0.0）
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,
    /// 监听端口
    pub port: u16,
    /// 初始状态（可选）
    #[serde(default)]
    pub initial_state: Value,
    /// 创建后自动启动（可选，默认 true）
    #[serde(default = "default_auto_start")]
    pub auto_start: bool,
}

fn default_bind_addr() -> String {
    "0.0.0.0".to_string()
}

fn default_auto_start() -> bool {
    true
}

/// 更新状态请求
#[derive(Debug, Deserialize)]
pub struct UpdateStateRequest {
    /// 是否在线
    #[serde(default)]
    pub online: Option<bool>,
    /// 故障类型（空字符串表示清除故障）
    #[serde(default)]
    pub fault: Option<String>,
}

/// 触发故障请求
#[derive(Debug, Deserialize)]
pub struct TriggerFaultRequest {
    /// 故障类型
    pub fault_type: String,
}

/// 获取支持的协议列表
///
/// GET /lspcapi/tcp-simulator/protocols
pub async fn get_protocols(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
) -> Json<ApiResponse<Vec<ProtocolInfo>>> {
    let protocols = manager.get_protocols();
    Json(ApiResponse::success("获取协议列表成功", protocols))
}

/// 创建模拟器
///
/// POST /lspcapi/tcp-simulator/create
pub async fn create_simulator(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Json(req): Json<CreateSimulatorRequest>,
) -> Json<Value> {
    let config = TcpSimulatorConfig {
        id: String::new(),
        name: req.name,
        protocol: req.protocol,
        bind_addr: req.bind_addr,
        port: req.port,
        initial_state: req.initial_state,
    };

    match manager.create(config).await {
        Ok(mut info) => {
            // 如果需要自动启动
            if req.auto_start {
                if let Err(e) = manager.start(&info.id).await {
                    return Json(serde_json::json!({
                        "state": 30006,
                        "message": format!("模拟器已创建但启动失败: {}", e),
                        "data": info
                    }));
                }
                info.status = SimulatorStatus::Running;
            }

            Json(serde_json::json!({
                "state": 0,
                "message": "模拟器创建成功",
                "data": info
            }))
        }
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 列出所有模拟器
///
/// GET /lspcapi/tcp-simulator/list
pub async fn list_simulators(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
) -> Json<ApiResponse<Vec<SimulatorInfo>>> {
    let simulators = manager.list().await;
    Json(ApiResponse::success("获取模拟器列表成功", simulators))
}

/// 获取模拟器详情
///
/// GET /lspcapi/tcp-simulator/:id
pub async fn get_simulator(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match manager.get(&id).await {
        Some(info) => Json(serde_json::json!({
            "state": 0,
            "message": "获取模拟器成功",
            "data": info
        })),
        None => Json(serde_json::json!({
            "state": 30001,
            "message": format!("模拟器 '{}' 不存在", id)
        })),
    }
}

/// 删除模拟器
///
/// DELETE /lspcapi/tcp-simulator/:id
pub async fn delete_simulator(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match manager.delete(&id).await {
        Ok(_) => Json(serde_json::json!({
            "state": 0,
            "message": format!("模拟器 '{}' 已删除", id)
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 启动模拟器
///
/// POST /lspcapi/tcp-simulator/:id/start
pub async fn start_simulator(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match manager.start(&id).await {
        Ok(_) => {
            // 获取最新状态
            if let Some(info) = manager.get(&id).await {
                Json(serde_json::json!({
                    "state": 0,
                    "message": format!("模拟器 '{}' 已启动", id),
                    "data": info
                }))
            } else {
                Json(serde_json::json!({
                    "state": 0,
                    "message": format!("模拟器 '{}' 已启动", id)
                }))
            }
        }
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 停止模拟器
///
/// POST /lspcapi/tcp-simulator/:id/stop
pub async fn stop_simulator(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match manager.stop(&id).await {
        Ok(_) => {
            if let Some(info) = manager.get(&id).await {
                Json(serde_json::json!({
                    "state": 0,
                    "message": format!("模拟器 '{}' 已停止", id),
                    "data": info
                }))
            } else {
                Json(serde_json::json!({
                    "state": 0,
                    "message": format!("模拟器 '{}' 已停止", id)
                }))
            }
        }
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 更新模拟器状态
///
/// POST /lspcapi/tcp-simulator/:id/state
pub async fn update_simulator_state(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateStateRequest>,
) -> Json<Value> {
    match manager.update_state(&id, req.online, req.fault).await {
        Ok(info) => Json(serde_json::json!({
            "state": 0,
            "message": "状态更新成功",
            "data": info
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 触发故障
///
/// POST /lspcapi/tcp-simulator/:id/fault
pub async fn trigger_fault(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
    Json(req): Json<TriggerFaultRequest>,
) -> Json<Value> {
    match manager.trigger_fault(&id, &req.fault_type).await {
        Ok(info) => Json(serde_json::json!({
            "state": 0,
            "message": format!("已触发故障: {}", req.fault_type),
            "data": info
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 清除故障
///
/// POST /lspcapi/tcp-simulator/:id/clear-fault
pub async fn clear_fault(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match manager.clear_fault(&id).await {
        Ok(info) => Json(serde_json::json!({
            "state": 0,
            "message": "故障已清除",
            "data": info
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 设置在线状态
///
/// POST /lspcapi/tcp-simulator/:id/online
pub async fn set_online(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Json<Value> {
    let online = req.get("online").and_then(|v| v.as_bool()).unwrap_or(true);

    match manager.set_online(&id, online).await {
        Ok(info) => Json(serde_json::json!({
            "state": 0,
            "message": if online { "设备已上线" } else { "设备已下线" },
            "data": info
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

// ============ Modbus 模拟器 API ============

/// 添加 Modbus Slave 请求
#[derive(Debug, Deserialize)]
pub struct AddModbusSlaveRequest {
    #[serde(rename = "slaveId")]
    pub slave_id: u8,
    #[serde(default)]
    pub registers: Vec<RegisterConfig>,
}

/// 设置 Modbus 寄存器请求
#[derive(Debug, Deserialize)]
pub struct SetModbusRegisterRequest {
    #[serde(rename = "slaveId")]
    pub slave_id: u8,
    pub register: RegisterConfig,
}

/// 删除 Modbus 寄存器请求
#[derive(Debug, Deserialize)]
pub struct DeleteModbusRegisterRequest {
    #[serde(rename = "slaveId")]
    pub slave_id: u8,
    #[serde(rename = "registerType")]
    pub register_type: String,
    pub address: u16,
}

/// 更新 Modbus 寄存器值请求
#[derive(Debug, Deserialize)]
pub struct UpdateModbusRegisterValueRequest {
    #[serde(rename = "slaveId")]
    pub slave_id: u8,
    #[serde(rename = "registerType")]
    pub register_type: String,
    pub address: u16,
    pub value: Value,
}

/// 批量更新 Modbus 寄存器请求
#[derive(Debug, Deserialize)]
pub struct BatchUpdateModbusRegistersRequest {
    pub updates: Vec<UpdateModbusRegisterValueRequest>,
}

/// 获取 Modbus Slaves
///
/// GET /lspcapi/tcp-simulator/:id/modbus/slaves
pub async fn get_modbus_slaves(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match manager.get(&id).await {
        Some(info) => {
            let values = get_modbus_values_from_info(&info);
            Json(serde_json::json!({
                "state": 0,
                "message": "获取 Slaves 成功",
                "data": values.slaves
            }))
        }
        None => Json(serde_json::json!({
            "state": 30001,
            "message": format!("模拟器 '{}' 不存在", id)
        })),
    }
}

/// 添加 Modbus Slave
///
/// POST /lspcapi/tcp-simulator/:id/modbus/slave
pub async fn add_modbus_slave(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
    Json(req): Json<AddModbusSlaveRequest>,
) -> Json<Value> {
    let result = manager
        .update_modbus_state(&id, |values| {
            if values.get_slave(req.slave_id).is_some() {
                return Err(format!("Slave ID {} 已存在", req.slave_id));
            }
            values.slaves.push(SlaveConfig {
                slave_id: req.slave_id,
                registers: req.registers.clone(),
            });
            Ok(())
        })
        .await;

    match result {
        Ok(info) => Json(serde_json::json!({
            "state": 0,
            "message": format!("Slave {} 添加成功", req.slave_id),
            "data": info
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 删除 Modbus Slave
///
/// DELETE /lspcapi/tcp-simulator/:id/modbus/slave/:slaveId
pub async fn delete_modbus_slave(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path((id, slave_id)): Path<(String, u8)>,
) -> Json<Value> {
    let result = manager
        .update_modbus_state(&id, |values| {
            if values.remove_slave(slave_id) {
                Ok(())
            } else {
                Err(format!("Slave ID {} 不存在", slave_id))
            }
        })
        .await;

    match result {
        Ok(info) => Json(serde_json::json!({
            "state": 0,
            "message": format!("Slave {} 已删除", slave_id),
            "data": info
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 设置 Modbus 寄存器
///
/// POST /lspcapi/tcp-simulator/:id/modbus/register
pub async fn set_modbus_register(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
    Json(req): Json<SetModbusRegisterRequest>,
) -> Json<Value> {
    let result = manager
        .update_modbus_state(&id, |values| {
            if values.set_register(req.slave_id, req.register.clone()) {
                Ok(())
            } else {
                Err(format!("Slave ID {} 不存在", req.slave_id))
            }
        })
        .await;

    match result {
        Ok(info) => Json(serde_json::json!({
            "state": 0,
            "message": "寄存器设置成功",
            "data": info
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 删除 Modbus 寄存器
///
/// POST /lspcapi/tcp-simulator/:id/modbus/register/delete
pub async fn delete_modbus_register(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
    Json(req): Json<DeleteModbusRegisterRequest>,
) -> Json<Value> {
    let register_type = match parse_register_type(&req.register_type) {
        Some(t) => t,
        None => {
            return Json(serde_json::json!({
                "state": 30003,
                "message": format!("无效的寄存器类型: {}", req.register_type)
            }));
        }
    };

    let result = manager
        .update_modbus_state(&id, |values| {
            if values.delete_register(req.slave_id, register_type, req.address) {
                Ok(())
            } else {
                Err("寄存器不存在".to_string())
            }
        })
        .await;

    match result {
        Ok(info) => Json(serde_json::json!({
            "state": 0,
            "message": "寄存器已删除",
            "data": info
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 更新 Modbus 寄存器值
///
/// POST /lspcapi/tcp-simulator/:id/modbus/register/value
pub async fn update_modbus_register_value(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateModbusRegisterValueRequest>,
) -> Json<Value> {
    let register_type = match parse_register_type(&req.register_type) {
        Some(t) => t,
        None => {
            return Json(serde_json::json!({
                "state": 30003,
                "message": format!("无效的寄存器类型: {}", req.register_type)
            }));
        }
    };

    let result = manager
        .update_modbus_state(&id, |values| {
            if let Some(slave) = values.get_slave_mut(req.slave_id) {
                if let Some(reg) = slave.registers.iter_mut().find(|r| {
                    r.register_type == register_type && r.address == req.address
                }) {
                    reg.value = req.value.clone();
                    return Ok(());
                }
            }
            Err("寄存器不存在".to_string())
        })
        .await;

    match result {
        Ok(info) => Json(serde_json::json!({
            "state": 0,
            "message": "寄存器值已更新",
            "data": info
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 批量更新 Modbus 寄存器值
///
/// POST /lspcapi/tcp-simulator/:id/modbus/registers/batch
pub async fn batch_update_modbus_registers(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
    Json(req): Json<BatchUpdateModbusRegistersRequest>,
) -> Json<Value> {
    let result = manager
        .update_modbus_state(&id, |values| {
            for update in &req.updates {
                let register_type = match parse_register_type(&update.register_type) {
                    Some(t) => t,
                    None => continue,
                };

                if let Some(slave) = values.get_slave_mut(update.slave_id) {
                    if let Some(reg) = slave.registers.iter_mut().find(|r| {
                        r.register_type == register_type && r.address == update.address
                    }) {
                        reg.value = update.value.clone();
                    }
                }
            }
            Ok(())
        })
        .await;

    match result {
        Ok(info) => Json(serde_json::json!({
            "state": 0,
            "message": "批量更新成功",
            "data": info
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 从 SimulatorInfo 获取 ModbusValues
fn get_modbus_values_from_info(info: &SimulatorInfo) -> ModbusValues {
    if let Some(slaves_value) = info.state.values.get("slaves") {
        if let Ok(values) = serde_json::from_value::<ModbusValues>(serde_json::json!({
            "slaves": slaves_value,
            "defaultSlaveId": info.state.values.get("defaultSlaveId")
        })) {
            return values;
        }
    }
    ModbusValues::default()
}

/// 从字符串解析 RegisterType
fn parse_register_type(s: &str) -> Option<RegisterType> {
    match s {
        "coil" => Some(RegisterType::Coil),
        "discrete_input" => Some(RegisterType::DiscreteInput),
        "holding_register" => Some(RegisterType::HoldingRegister),
        "input_register" => Some(RegisterType::InputRegister),
        _ => None,
    }
}

// ============ 报文监控 API ============

/// 获取报文列表查询参数
#[derive(Debug, Deserialize)]
pub struct GetPacketsQuery {
    /// 获取此 ID 之后的报文（增量获取）
    #[serde(rename = "afterId")]
    pub after_id: Option<u64>,
    /// 限制返回数量（获取最近 N 条）
    pub limit: Option<usize>,
}

/// 报文监控设置请求
#[derive(Debug, Deserialize)]
pub struct PacketMonitorSettingsRequest {
    /// 是否启用监控
    pub enabled: Option<bool>,
    /// 最大报文数量
    #[serde(rename = "maxPackets")]
    pub max_packets: Option<usize>,
}

/// 获取报文列表
///
/// GET /lspcapi/tcp-simulator/:id/packets
pub async fn get_packets(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
    Query(query): Query<GetPacketsQuery>,
) -> Json<Value> {
    match manager.get_packets(&id, query.after_id, query.limit).await {
        Ok(packets) => Json(serde_json::json!({
            "state": 0,
            "message": "获取报文成功",
            "data": {
                "packets": packets,
                "total": packets.len()
            }
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 清空报文记录
///
/// DELETE /lspcapi/tcp-simulator/:id/packets
pub async fn clear_packets(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match manager.clear_packets(&id).await {
        Ok(_) => Json(serde_json::json!({
            "state": 0,
            "message": "报文已清空"
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 设置报文监控选项
///
/// POST /lspcapi/tcp-simulator/:id/packets/settings
pub async fn set_packet_monitor_settings(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
    Json(req): Json<PacketMonitorSettingsRequest>,
) -> Json<Value> {
    // 处理 enabled 设置
    if let Some(enabled) = req.enabled {
        if let Err(e) = manager.set_packet_monitor_enabled(&id, enabled).await {
            return Json(serde_json::json!({
                "state": 30006,
                "message": e
            }));
        }
    }

    // 处理 max_packets 设置
    if let Some(max) = req.max_packets {
        if let Err(e) = manager.set_packet_monitor_max(&id, max).await {
            return Json(serde_json::json!({
                "state": 30006,
                "message": e
            }));
        }
    }

    // 返回最新状态
    match manager.get(&id).await {
        Some(info) => Json(serde_json::json!({
            "state": 0,
            "message": "设置已更新",
            "data": info
        })),
        None => Json(serde_json::json!({
            "state": 30001,
            "message": format!("模拟器 '{}' 不存在", id)
        })),
    }
}
