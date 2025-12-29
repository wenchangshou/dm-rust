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

use crate::tcp_simulator::{
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
