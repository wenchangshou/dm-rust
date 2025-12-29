/// MQTT 模拟器 REST API
use axum::{
    extract::{Extension, Path, Query},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

use crate::mqtt::manager::MqttSimulatorManager;
use crate::mqtt::rules::{MqttRule, MqttRuleAction};
use crate::mqtt::state::{
    CreateMqttSimulatorRequest, MqttPacketRecord, MqttSimulatorInfo,
};
use super::response::ApiResponse;

/// 创建 MQTT 模拟器 API 路由
pub fn mqtt_simulator_routes() -> Router {
    Router::new()
        .route("/create", post(create_simulator))
        .route("/list", get(list_simulators))
        .route("/:id", get(get_simulator))
        .route("/:id", delete(delete_simulator))
        .route("/:id/start", post(start_simulator))
        .route("/:id/stop", post(stop_simulator))
        .route("/:id/packets", get(get_packets))
        .route("/:id/packets", delete(clear_packets))
        .route("/:id/rules", get(list_rules))
        .route("/:id/rules", post(add_rule))
        .route("/:id/rules/:rule_id", delete(remove_rule))
        .route("/export", get(export_config))
        .route("/import", post(import_config))
}

/// 创建模拟器
pub async fn create_simulator(
    Extension(manager): Extension<Arc<MqttSimulatorManager>>,
    Json(request): Json<CreateMqttSimulatorRequest>,
) -> Json<ApiResponse<MqttSimulatorInfo>> {
    info!("Creating MQTT simulator: {}", request.name);

    match manager.create(request).await {
        Ok(info) => Json(ApiResponse::success("MQTT模拟器创建成功", info)),
        Err(e) => Json(ApiResponse {
            state: 30006,
            message: e,
            data: None,
        }),
    }
}

/// 获取模拟器列表
pub async fn list_simulators(
    Extension(manager): Extension<Arc<MqttSimulatorManager>>,
) -> Json<ApiResponse<Vec<MqttSimulatorInfo>>> {
    let simulators = manager.list().await;
    Json(ApiResponse::success("获取成功", simulators))
}

/// 获取单个模拟器
pub async fn get_simulator(
    Extension(manager): Extension<Arc<MqttSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<MqttSimulatorInfo>> {
    match manager.get(&id).await {
        Some(info) => Json(ApiResponse::success("获取成功", info)),
        None => Json(ApiResponse {
            state: 30001,
            message: "模拟器不存在".to_string(),
            data: None,
        }),
    }
}

/// 删除模拟器
pub async fn delete_simulator(
    Extension(manager): Extension<Arc<MqttSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    match manager.delete(&id).await {
        Ok(_) => Json(ApiResponse::<()>::success_empty("删除成功")),
        Err(e) => Json(ApiResponse::<()>::error(30006, e)),
    }
}

/// 启动模拟器
pub async fn start_simulator(
    Extension(manager): Extension<Arc<MqttSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    match manager.start(&id).await {
        Ok(_) => Json(ApiResponse::<()>::success_empty("启动成功")),
        Err(e) => Json(ApiResponse::<()>::error(30006, e)),
    }
}

/// 停止模拟器
pub async fn stop_simulator(
    Extension(manager): Extension<Arc<MqttSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    match manager.stop(&id).await {
        Ok(_) => Json(ApiResponse::<()>::success_empty("停止成功")),
        Err(e) => Json(ApiResponse::<()>::error(30006, e)),
    }
}

/// 报文查询参数
#[derive(Debug, Deserialize)]
pub struct PacketQuery {
    pub limit: Option<usize>,
    pub after_id: Option<u64>,
}

/// 获取报文记录
pub async fn get_packets(
    Extension(manager): Extension<Arc<MqttSimulatorManager>>,
    Path(id): Path<String>,
    Query(query): Query<PacketQuery>,
) -> Json<ApiResponse<Vec<MqttPacketRecord>>> {
    let limit = query.limit.unwrap_or(100);
    match manager.get_packets(&id, limit, query.after_id).await {
        Ok(packets) => Json(ApiResponse::success("获取成功", packets)),
        Err(e) => Json(ApiResponse {
            state: 30001,
            message: e,
            data: None,
        }),
    }
}

/// 清空报文记录
pub async fn clear_packets(
    Extension(manager): Extension<Arc<MqttSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    match manager.clear_packets(&id).await {
        Ok(_) => Json(ApiResponse::<()>::success_empty("清空成功")),
        Err(e) => Json(ApiResponse::<()>::error(30001, e)),
    }
}

/// 获取规则列表
pub async fn list_rules(
    Extension(manager): Extension<Arc<MqttSimulatorManager>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Vec<MqttRule>>> {
    match manager.get_rules(&id).await {
        Ok(rules) => Json(ApiResponse::success("获取成功", rules)),
        Err(e) => Json(ApiResponse {
            state: 30001,
            message: e,
            data: None,
        }),
    }
}

/// 添加规则请求
#[derive(Debug, Deserialize)]
pub struct AddRuleRequest {
    pub name: String,
    pub topic_pattern: String,
    pub action: MqttRuleAction,
}

/// 添加规则
pub async fn add_rule(
    Extension(manager): Extension<Arc<MqttSimulatorManager>>,
    Path(id): Path<String>,
    Json(request): Json<AddRuleRequest>,
) -> Json<ApiResponse<()>> {
    let rule = MqttRule::new(request.name, request.topic_pattern, request.action);
    match manager.add_rule(&id, rule).await {
        Ok(_) => Json(ApiResponse::<()>::success_empty("添加成功")),
        Err(e) => Json(ApiResponse::<()>::error(30006, e)),
    }
}

/// 删除规则路径参数
#[derive(Debug, Deserialize)]
pub struct RulePathParams {
    pub id: String,
    pub rule_id: String,
}

/// 删除规则
pub async fn remove_rule(
    Extension(manager): Extension<Arc<MqttSimulatorManager>>,
    Path(params): Path<RulePathParams>,
) -> Json<ApiResponse<()>> {
    match manager.remove_rule(&params.id, &params.rule_id).await {
        Ok(_) => Json(ApiResponse::<()>::success_empty("删除成功")),
        Err(e) => Json(ApiResponse::<()>::error(30006, e)),
    }
}

/// 导出配置
pub async fn export_config(
    Extension(manager): Extension<Arc<MqttSimulatorManager>>,
) -> Json<ApiResponse<Vec<MqttSimulatorInfo>>> {
    let simulators = manager.list().await;
    Json(ApiResponse::success("导出成功", simulators))
}

/// 导入配置请求
#[derive(Debug, Deserialize)]
pub struct ImportConfigRequest {
    pub simulators: Vec<CreateMqttSimulatorRequest>,
    pub replace_existing: Option<bool>,
}

/// 导入配置
pub async fn import_config(
    Extension(manager): Extension<Arc<MqttSimulatorManager>>,
    Json(request): Json<ImportConfigRequest>,
) -> Json<ApiResponse<Vec<MqttSimulatorInfo>>> {
    let replace_existing = request.replace_existing.unwrap_or(false);

    // 如果需要替换，先删除所有现有模拟器
    if replace_existing {
        let existing = manager.list().await;
        for sim in existing {
            let _ = manager.delete(&sim.id).await;
        }
    }

    // 创建新的模拟器
    let mut created = Vec::new();
    let mut errors = Vec::new();

    for req in request.simulators {
        match manager.create(req).await {
            Ok(info) => created.push(info),
            Err(e) => errors.push(e),
        }
    }

    if !errors.is_empty() {
        Json(ApiResponse {
            state: 30006,
            message: format!(
                "部分导入失败: 成功 {}, 失败 {} - 错误: {:?}",
                created.len(),
                errors.len(),
                errors
            ),
            data: Some(created),
        })
    } else {
        Json(ApiResponse::success(
            &format!("成功导入 {} 个模拟器", created.len()),
            created,
        ))
    }
}
