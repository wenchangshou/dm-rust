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

use crate::tcp::{
    protocols::{ModbusValues, RegisterConfig, RegisterType, SlaveConfig},
    ProtocolInfo, SimulatorInfo, SimulatorStatus, TcpSimulatorConfig, TcpSimulatorManager,
};
use super::response::ApiResponse;
use axum::extract::Query;

#[cfg(feature = "swagger")]
use utoipa::ToSchema;

/// 创建模拟器请求
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct CreateSimulatorRequest {
    /// 显示名称
    #[cfg_attr(feature = "swagger", schema(example = "PLC 模拟器"))]
    pub name: String,
    /// 描述
    #[serde(default)]
    pub description: String,
    /// 协议类型
    #[cfg_attr(feature = "swagger", schema(example = "modbus"))]
    pub protocol: String,
    /// 绑定地址（可选，默认 0.0.0.0）
    #[serde(default = "default_bind_addr")]
    #[cfg_attr(feature = "swagger", schema(example = "0.0.0.0"))]
    pub bind_addr: String,
    /// 监听端口
    #[cfg_attr(feature = "swagger", schema(example = 502))]
    pub port: u16,
    /// 初始状态（可选）
    #[serde(default)]
    pub initial_state: Value,
    /// 创建后自动启动（可选，默认 true）
    #[serde(default = "default_auto_start")]
    pub auto_start: bool,
    /// 协议配置（可选）
    #[serde(default)]
    pub protocol_config: Option<Value>,
}

fn default_bind_addr() -> String {
    "0.0.0.0".to_string()
}

fn default_auto_start() -> bool {
    true
}

/// 更新状态请求
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
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
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct TriggerFaultRequest {
    /// 故障类型
    #[cfg_attr(feature = "swagger", schema(example = "communication_error"))]
    pub fault_type: String,
}

/// 获取支持的协议列表
///
/// GET /lspcapi/tcp-simulator/protocols
#[cfg_attr(feature = "swagger", utoipa::path(
    get,
    path = "/lspcapi/tcp-simulator/protocols",
    tag = "TCP Simulator",
    responses(
        (status = 200, description = "成功返回协议列表", body = Vec<ProtocolInfo>)
    )
))]
pub async fn get_protocols(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
) -> Json<ApiResponse<Vec<ProtocolInfo>>> {
    let protocols = manager.get_protocols();
    Json(ApiResponse::success("获取协议列表成功", protocols))
}

/// 创建模拟器
///
/// 创建一个新的 TCP 协议模拟器实例，可选择自动启动
#[cfg_attr(feature = "swagger", utoipa::path(
    post,
    path = "/lspcapi/tcp-simulator/create",
    tag = "TCP Simulator",
    request_body = CreateSimulatorRequest,
    responses(
        (status = 200, description = "创建成功", body = Value),
        (status = 400, description = "参数错误", body = Value),
        (status = 500, description = "创建失败", body = Value)
    )
))]
pub async fn create_simulator(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Json(req): Json<CreateSimulatorRequest>,
) -> Json<Value> {
    let config = TcpSimulatorConfig {
        id: String::new(), // 将由 manager 生成
        name: req.name,
        description: req.description,
        protocol: req.protocol,
        transport: "tcp".to_string(), // Default to tcp for create_simulator API
        bind_addr: req.bind_addr,
        port: req.port,
        initial_state: req.initial_state,
        protocol_config: req.protocol_config,
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
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct AddModbusSlaveRequest {
    /// Slave ID (1-247)
    #[serde(rename = "slaveId")]
    #[cfg_attr(feature = "swagger", schema(example = 1))]
    pub slave_id: u8,
    /// 寄存器配置列表（可选）
    #[serde(default)]
    pub registers: Vec<RegisterConfig>,
}

/// 设置 Modbus 寄存器请求
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct SetModbusRegisterRequest {
    /// Slave ID
    #[serde(rename = "slaveId")]
    #[cfg_attr(feature = "swagger", schema(example = 1))]
    pub slave_id: u8,
    /// 寄存器配置
    pub register: RegisterConfig,
}

/// 删除 Modbus 寄存器请求
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct DeleteModbusRegisterRequest {
    /// Slave ID
    #[serde(rename = "slaveId")]
    #[cfg_attr(feature = "swagger", schema(example = 1))]
    pub slave_id: u8,
    /// 寄存器类型
    #[serde(rename = "registerType")]
    #[cfg_attr(feature = "swagger", schema(example = "holding_register"))]
    pub register_type: String,
    /// 寄存器地址
    #[cfg_attr(feature = "swagger", schema(example = 0))]
    pub address: u16,
}

/// 更新 Modbus 寄存器值请求
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct UpdateModbusRegisterValueRequest {
    /// Slave ID
    #[serde(rename = "slaveId")]
    #[cfg_attr(feature = "swagger", schema(example = 1))]
    pub slave_id: u8,
    /// 寄存器类型
    #[serde(rename = "registerType")]
    #[cfg_attr(feature = "swagger", schema(example = "holding_register"))]
    pub register_type: String,
    /// 寄存器地址
    #[cfg_attr(feature = "swagger", schema(example = 0))]
    pub address: u16,
    /// 新值
    pub value: Value,
}

/// 批量更新 Modbus 寄存器请求
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct BatchUpdateModbusRegistersRequest {
    /// 更新列表
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
                if let Some(reg) = slave
                    .registers
                    .iter_mut()
                    .find(|r| r.register_type == register_type && r.address == req.address)
                {
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
    let result =
        manager
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
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct GetPacketsQuery {
    /// 获取此 ID 之后的报文（增量获取）
    #[serde(rename = "afterId")]
    pub after_id: Option<u64>,
    /// 限制返回数量（获取最近 N 条）
    #[cfg_attr(feature = "swagger", schema(example = 100))]
    pub limit: Option<usize>,
}

/// 报文监控设置请求
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct PacketMonitorSettingsRequest {
    /// 是否启用监控
    pub enabled: Option<bool>,
    /// 最大报文数量
    #[serde(rename = "maxPackets")]
    #[cfg_attr(feature = "swagger", schema(example = 1000))]
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

/// 更新模拟器配置请求
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct UpdateSimulatorConfigRequest {
    /// 协议配置
    pub protocol_config: Option<Value>,
}

/// 更新模拟器配置
///
/// POST /lspcapi/tcp-simulator/:id/config
pub async fn update_simulator_config(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSimulatorConfigRequest>,
) -> Json<Value> {
    match manager.update_config(&id, req.protocol_config).await {
        Ok(info) => Json(serde_json::json!({
            "state": 0,
            "message": "配置更新成功，请重启模拟器以生效",
            "data": info
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 更新模拟器信息请求（名称和描述）
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct UpdateSimulatorInfoRequest {
    /// 显示名称
    pub name: Option<String>,
    /// 描述
    pub description: Option<String>,
}

/// 更新模拟器基本信息（名称和描述）
///
/// POST /lspcapi/tcp-simulator/:id/info
pub async fn update_simulator_info(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSimulatorInfoRequest>,
) -> Json<Value> {
    match manager.update_info(&id, req.name, req.description).await {
        Ok(info) => Json(serde_json::json!({
            "state": 0,
            "message": "信息更新成功",
            "data": info
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

// ============ 客户端连接管理 API ============

use crate::tcp::state::ClientConnection;

// ============ 模板管理 API ============

use crate::tcp::{CreateFromTemplateRequest, SimulatorTemplate};

/// 保存为模板请求
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct SaveAsTemplateRequest {
    /// 模板名称
    pub name: String,
    /// 模板描述
    #[serde(default)]
    pub description: String,
}

use crate::tcp::template::UpdateTemplateRequest;

/// 获取模板列表
///
/// GET /lspcapi/tcp-simulator/templates
pub async fn list_templates(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
) -> Json<ApiResponse<Vec<SimulatorTemplate>>> {
    let templates = manager.template_manager.list().await;
    Json(ApiResponse::success("获取模板列表成功", templates))
}

/// 删除模板
///
/// DELETE /lspcapi/tcp-simulator/templates/:id
pub async fn delete_template(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    match manager.template_manager.delete(&id).await {
        Ok(_) => Json(ApiResponse::<()>::success("模板删除成功", ())),
        Err(e) => Json(ApiResponse::<()>::error(500, &e)),
    }
}

/// 更新模板
///
/// PUT /lspcapi/tcp-simulator/templates/:id
pub async fn update_template(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTemplateRequest>,
) -> Json<ApiResponse<SimulatorTemplate>> {
    match manager.template_manager.update(&id, req).await {
        Ok(template) => Json(ApiResponse::success("模板更新成功", template)),
        Err(e) => Json(ApiResponse {
            state: 500,
            message: e, // e is String from update error
            data: None,
        }),
    }
}

/// 创建模板 (直接)
///
/// POST /lspcapi/tcp-simulator/templates
pub async fn create_template_direct(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Json(req): Json<crate::tcp::template::CreateTemplateRequest>,
) -> Json<ApiResponse<SimulatorTemplate>> {
    match manager.template_manager.create(req).await {
        Ok(template) => Json(ApiResponse::success("模板创建成功", template)),
        Err(e) => Json(ApiResponse {
            state: 500,
            message: e,
            data: None,
        }),
    }
}

/// 从模板创建模拟器
///
/// POST /lspcapi/tcp-simulator/create-from-template
pub async fn create_from_template(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Json(req): Json<CreateFromTemplateRequest>,
) -> Json<Value> {
    match manager.create_from_template(req).await {
        Ok(mut info) => {
            // 自动启动
            if let Err(e) = manager.start(&info.id).await {
                return Json(serde_json::json!({
                    "state": 30006,
                    "message": format!("模拟器已创建但启动失败: {}", e),
                    "data": info
                }));
            }
            info.status = SimulatorStatus::Running;

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

/// 将模拟器保存为模板
///
/// POST /lspcapi/tcp-simulator/:id/save-as-template
pub async fn save_as_template(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
    Json(req): Json<SaveAsTemplateRequest>,
) -> Json<Value> {
    match manager
        .save_as_template(&id, req.name, req.description)
        .await
    {
        Ok(template) => Json(serde_json::json!({
            "state": 0,
            "message": "模板保存成功",
            "data": template
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}
///
/// GET /lspcapi/tcp-simulator/:id/clients
pub async fn list_clients(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match manager.get(&id).await {
        Some(info) => {
            let clients: Vec<&ClientConnection> = info.state.clients.values().collect();
            Json(serde_json::json!({
                "state": 0,
                "message": "获取客户端列表成功",
                "data": clients
            }))
        }
        None => Json(serde_json::json!({
            "state": 30001,
            "message": format!("模拟器 '{}' 不存在", id)
        })),
    }
}

/// 断开指定客户端连接
///
/// POST /lspcapi/tcp-simulator/:id/clients/:clientId/disconnect
pub async fn disconnect_client(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path((id, client_id)): Path<(String, String)>,
) -> Json<Value> {
    match manager.disconnect_client(&id, &client_id).await {
        Ok(_) => Json(serde_json::json!({
            "state": 0,
            "message": format!("客户端 '{}' 已断开连接", client_id)
        })),
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

// ============ Debug 模式 API ============

/// 设置 Debug 模式请求
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct SetDebugModeRequest {
    /// 是否启用 Debug 模式
    pub enabled: bool,
}

/// 设置 Debug 模式
///
/// POST /lspcapi/tcp-simulator/:id/debug
pub async fn set_debug_mode(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
    Json(req): Json<SetDebugModeRequest>,
) -> Json<Value> {
    match manager.set_debug_mode(&id, req.enabled).await {
        Ok(info) => {
            let message = if req.enabled {
                "Debug 模式已启用"
            } else {
                "Debug 模式已关闭"
            };
            Json(serde_json::json!({
                "state": 0,
                "message": message,
                "data": {
                    "debug_mode": req.enabled,
                    "log_path": info.state.packet_monitor.debug_log_path
                }
            }))
        }
        Err(e) => Json(serde_json::json!({
            "state": 30006,
            "message": e
        })),
    }
}

/// 下载 Debug 日志
///
/// GET /lspcapi/tcp-simulator/:id/debug/log
pub async fn download_debug_log(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
) -> axum::response::Response {
    use axum::http::{header, StatusCode};
    use axum::response::IntoResponse;

    match manager.get_debug_log(&id).await {
        Ok(content) => {
            let filename = format!("simulator_{}_debug.log", id);
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                    (
                        header::CONTENT_DISPOSITION,
                        &format!("attachment; filename=\"{}\"", filename),
                    ),
                ],
                content,
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "state": 30006,
                "message": e
            })),
        )
            .into_response(),
    }
}

/// 获取 Debug 状态
///
/// GET /lspcapi/tcp-simulator/:id/debug
pub async fn get_debug_status(
    Extension(manager): Extension<Arc<TcpSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match manager.get(&id).await {
        Some(info) => Json(serde_json::json!({
            "state": 0,
            "message": "获取 Debug 状态成功",
            "data": {
                "debug_mode": info.state.packet_monitor.debug_mode,
                "log_path": info.state.packet_monitor.debug_log_path
            }
        })),
        None => Json(serde_json::json!({
            "state": 30001,
            "message": format!("模拟器 '{}' 不存在", id)
        })),
    }
}
