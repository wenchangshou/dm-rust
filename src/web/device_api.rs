//! 设备控制 API 处理器

use axum::{Json, extract::Extension};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::device::DeviceController;
use crate::db::Database;
use crate::utils::error::error_codes;
use super::response::ApiResponse;

// ===== 请求/响应类型定义 =====

/// 写入请求
#[derive(Deserialize, ToSchema)]
pub struct WriteRequest {
    /// 节点全局 ID
    pub global_id: u32,
    /// 写入值
    pub value: i32,
}

/// 批量写入项
#[derive(Deserialize, ToSchema)]
pub struct WriteManyItem {
    /// 节点全局 ID
    pub id: u32,
    /// 写入值
    pub value: i32,
}

/// 批量写入请求
#[derive(Deserialize, ToSchema)]
pub struct WriteManyRequest {
    /// 写入项列表
    pub items: Vec<WriteManyItem>,
}

/// 批量写入结果项
#[derive(Serialize, ToSchema)]
pub struct WriteManyResultItem {
    /// 节点全局 ID
    pub id: u32,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 读取请求
#[derive(Deserialize, ToSchema)]
pub struct ReadRequest {
    /// 节点全局 ID
    pub global_id: u32,
}

/// 批量读取请求
#[derive(Deserialize, ToSchema)]
pub struct ReadManyRequest {
    /// 节点全局 ID 列表
    pub ids: Vec<u32>,
}

/// 批量读取结果项
#[derive(Serialize, ToSchema)]
pub struct ReadManyResultItem {
    /// 节点全局 ID
    pub id: u32,
    /// 是否成功
    pub success: bool,
    /// 读取到的值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 状态查询请求
#[derive(Deserialize, ToSchema)]
pub struct StatusRequest {
    /// 节点全局 ID（可选）
    pub id: Option<u32>,
}

/// 场景执行请求
#[derive(Deserialize, ToSchema)]
pub struct SceneRequest {
    /// 场景名称
    pub name: String,
}

/// 通道命令请求
#[derive(Deserialize, ToSchema)]
pub struct ChannelCommandRequest {
    /// 通道 ID
    pub channel_id: u32,
    /// 命令名称
    pub command: String,
    /// 命令参数（JSON）
    pub params: serde_json::Value,
}

/// 调用方法请求
#[derive(Deserialize, ToSchema)]
pub struct CallMethodRequest {
    /// 通道 ID
    pub channel_id: u32,
    /// 方法名称
    pub method_name: String,
    /// 方法参数（JSON）
    pub arguments: serde_json::Value,
}

/// 获取方法列表请求
#[derive(Deserialize, ToSchema)]
pub struct GetMethodsRequest {
    /// 通道 ID
    pub channel_id: u32,
}

/// 批量读取项
#[derive(Deserialize, ToSchema)]
pub struct BatchReadItem {
    /// 读取项名称
    pub name: String,
    /// 通道 ID
    pub channel_id: u32,
    /// 读取参数（JSON）
    #[serde(flatten)]
    pub params: serde_json::Value,
}

/// 批量读取请求
#[derive(Deserialize, ToSchema)]
pub struct BatchReadRequest {
    /// 读取项列表
    pub items: Vec<BatchReadItem>,
}

/// 批量读取结果项
#[derive(Serialize, ToSchema)]
pub struct BatchReadResultItem {
    /// 读取项名称
    pub name: String,
    /// 是否成功
    pub success: bool,
    /// 读取到的值（JSON）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 系统设置响应
#[derive(Serialize, ToSchema)]
pub struct SystemSettingsResponse {
    /// PID PDUs设置
    #[serde(rename = "pidPdus")]
    pub pid_pdus: bool,
    /// 大屏设置
    #[serde(rename = "bigScreen")]
    pub big_screen: bool,
    /// 大屏机架设置
    #[serde(rename = "bigScreenRack")]
    pub big_screen_rack: bool,
    /// 音频机架设置
    #[serde(rename = "audioRack")]
    pub audio_rack: bool,
}

// ===== API 处理函数 =====

/// 获取系统设置
#[utoipa::path(
    get,
    path = "/lspcapi/device/getAll",
    responses(
        (status = 200, description = "获取成功", body = inline(ApiResponse<SystemSettingsResponse>))
    ),
    tag = "Device"
)]
pub async fn get_all_settings(
    Extension(db): Extension<Arc<Database>>,
) -> Json<ApiResponse<SystemSettingsResponse>> {
    // 辅助函数：解析boolean值
    let parse_bool = |value: &str| -> bool {
        match value.to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            _ => false,
        }
    };

    // 从数据库读取settings
    let result = sqlx::query_as::<_, (String, String)>(
        "SELECT name, value FROM settings WHERE name IN ('pidPdus', 'bigScreen', 'bigScreenRack', 'audioRack')"
    )
    .fetch_all(&db.pool)
    .await;

    match result {
        Ok(rows) => {
            let mut pid_pdus = false;
            let mut big_screen = false;
            let mut big_screen_rack = false;
            let mut audio_rack = false;

            for (name, value) in rows {
                match name.as_str() {
                    "pidPdus" => pid_pdus = parse_bool(&value),
                    "bigScreen" => big_screen = parse_bool(&value),
                    "bigScreenRack" => big_screen_rack = parse_bool(&value),
                    "audioRack" => audio_rack = parse_bool(&value),
                    _ => {}
                }
            }

            Json(ApiResponse {
                state: error_codes::SUCCESS,
                message: "成功".to_string(),
                data: Some(SystemSettingsResponse {
                    pid_pdus,
                    big_screen,
                    big_screen_rack,
                    audio_rack,
                }),
            })
        }
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("查询失败: {:?}", e),
            data: None,
        }),
    }
}

/// 获取所有通道状态
#[utoipa::path(
    post,
    path = "/lspcapi/device/getAllStatus",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "获取成功", body = inline(ApiResponse<serde_json::Value>))
    ),
    tag = "Device"
)]
pub async fn get_all_status(
    Extension(controller): Extension<Arc<DeviceController>>,
) -> Json<ApiResponse<serde_json::Value>> {
    match controller.get_all_channel_status().await {
        Ok(data) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "成功".to_string(),
            data: Some(data),
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("错误: {:?}", e),
            data: None,
        }),
    }
}

/// 获取所有节点状态
#[utoipa::path(
    post,
    path = "/lspcapi/device/getAllNodeStates",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "获取成功", body = inline(ApiResponse<serde_json::Value>))
    ),
    tag = "Device"
)]
pub async fn get_all_node_states(
    Extension(controller): Extension<Arc<DeviceController>>,
) -> Json<ApiResponse<serde_json::Value>> {
    let states = controller.get_all_node_states();
    let data: Vec<_> = states.into_iter().map(|(global_id, state)| {
        serde_json::json!({
            "global_id": global_id,
            "channel_id": state.channel_id,
            "device_id": state.device_id,
            "category": state.category,
            "alias": state.alias,
            "current_value": state.current_value,
            "online": state.online,
        })
    }).collect();

    Json(ApiResponse {
        state: error_codes::SUCCESS,
        message: "成功".to_string(),
        data: Some(serde_json::json!(data)),
    })
}

/// 获取单个节点状态
#[utoipa::path(
    post,
    path = "/lspcapi/device/getNodeState",
    request_body = StatusRequest,
    responses(
        (status = 200, description = "获取成功", body = inline(ApiResponse<serde_json::Value>)),
        (status = 404, description = "节点不存在")
    ),
    tag = "Device"
)]
pub async fn get_node_state(
    Extension(controller): Extension<Arc<DeviceController>>,
    Json(payload): Json<StatusRequest>,
) -> Json<ApiResponse<serde_json::Value>> {
    if let Some(id) = payload.id {
        match controller.get_node_state(id) {
            Some(state) => Json(ApiResponse {
                state: error_codes::SUCCESS,
                message: "成功".to_string(),
                data: Some(serde_json::json!({
                    "global_id": state.global_id,
                    "channel_id": state.channel_id,
                    "device_id": state.device_id,
                    "category": state.category,
                    "alias": state.alias,
                    "current_value": state.current_value,
                    "online": state.online,
                })),
            }),
            None => Json(ApiResponse {
                state: error_codes::DEVICE_NOT_FOUND,
                message: format!("节点 {} 不存在", id),
                data: None,
            }),
        }
    } else {
        Json(ApiResponse {
            state: error_codes::INVALID_PARAMS,
            message: "缺少参数 id".to_string(),
            data: None,
        })
    }
}

/// 读取设备值
#[utoipa::path(
    post,
    path = "/lspcapi/device/read",
    request_body = ReadRequest,
    responses(
        (status = 200, description = "读取成功", body = inline(ApiResponse<f64>))
    ),
    tag = "Device"
)]
pub async fn read_device(
    Extension(controller): Extension<Arc<DeviceController>>,
    Json(payload): Json<ReadRequest>,
) -> Json<ApiResponse<f64>> {
    match controller.read_node(payload.global_id).await {
        Ok(value) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "读取成功".to_string(),
            data: Some(value),
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("读取失败: {:?}", e),
            data: None,
        }),
    }
}

/// 批量读取设备值
#[utoipa::path(
    post,
    path = "/lspcapi/device/readMany",
    request_body = ReadManyRequest,
    responses(
        (status = 200, description = "批量读取完成", body = inline(ApiResponse<Vec<ReadManyResultItem>>))
    ),
    tag = "Device"
)]
pub async fn read_many(
    Extension(controller): Extension<Arc<DeviceController>>,
    Json(payload): Json<ReadManyRequest>,
) -> Json<ApiResponse<Vec<ReadManyResultItem>>> {
    let mut results = Vec::new();
    let mut success_count = 0;
    let mut fail_count = 0;

    for id in payload.ids {
        match controller.read_node(id).await {
            Ok(value) => {
                results.push(ReadManyResultItem {
                    id,
                    success: true,
                    value: Some(value),
                    error: None,
                });
                success_count += 1;
            }
            Err(e) => {
                results.push(ReadManyResultItem {
                    id,
                    success: false,
                    value: None,
                    error: Some(format!("{:?}", e)),
                });
                fail_count += 1;
            }
        }
    }

    Json(ApiResponse {
        state: error_codes::SUCCESS,
        message: format!("批量读取完成: 成功 {}, 失败 {}", success_count, fail_count),
        data: Some(results),
    })
}

/// 写入设备值
#[utoipa::path(
    post,
    path = "/lspcapi/device/write",
    request_body = WriteRequest,
    responses(
        (status = 200, description = "写入成功", body = inline(ApiResponse<()>))
    ),
    tag = "Device"
)]
pub async fn write_device(
    Extension(controller): Extension<Arc<DeviceController>>,
    Json(payload): Json<WriteRequest>,
) -> Json<ApiResponse<()>> {
    match controller.write_node(payload.global_id, payload.value).await {
        Ok(_) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "操作成功".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("操作失败: {:?}", e),
            data: None,
        }),
    }
}

/// 批量写入设备值
#[utoipa::path(
    post,
    path = "/lspcapi/device/writeMany",
    request_body = WriteManyRequest,
    responses(
        (status = 200, description = "批量写入完成", body = inline(ApiResponse<Vec<WriteManyResultItem>>))
    ),
    tag = "Device"
)]
pub async fn write_many(
    Extension(controller): Extension<Arc<DeviceController>>,
    Json(payload): Json<WriteManyRequest>,
) -> Json<ApiResponse<Vec<WriteManyResultItem>>> {
    let mut results = Vec::new();
    let mut success_count = 0;
    let mut fail_count = 0;

    for item in payload.items {
        match controller.write_node(item.id, item.value).await {
            Ok(_) => {
                results.push(WriteManyResultItem {
                    id: item.id,
                    success: true,
                    error: None,
                });
                success_count += 1;
            }
            Err(e) => {
                results.push(WriteManyResultItem {
                    id: item.id,
                    success: false,
                    error: Some(format!("{:?}", e)),
                });
                fail_count += 1;
            }
        }
    }

    Json(ApiResponse {
        state: error_codes::SUCCESS,
        message: format!("批量写入完成: 成功 {}, 失败 {}", success_count, fail_count),
        data: Some(results),
    })
}

/// 执行场景
#[utoipa::path(
    post,
    path = "/lspcapi/device/scene",
    request_body = SceneRequest,
    responses(
        (status = 200, description = "场景执行成功", body = inline(ApiResponse<()>))
    ),
    tag = "Device"
)]
pub async fn execute_scene(
    Extension(controller): Extension<Arc<DeviceController>>,
    Json(payload): Json<SceneRequest>,
) -> Json<ApiResponse<()>> {
    match controller.execute_scene(&payload.name).await {
        Ok(_) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: format!("场景 '{}' 执行成功", payload.name),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("场景执行失败: {:?}", e),
            data: None,
        }),
    }
}

/// 执行通道命令
#[utoipa::path(
    post,
    path = "/lspcapi/device/executeCommand",
    request_body = ChannelCommandRequest,
    responses(
        (status = 200, description = "命令执行成功", body = inline(ApiResponse<serde_json::Value>))
    ),
    tag = "Device"
)]
pub async fn execute_channel_command(
    Extension(controller): Extension<Arc<DeviceController>>,
    Json(payload): Json<ChannelCommandRequest>,
) -> Json<ApiResponse<serde_json::Value>> {
    match controller.execute_channel_command(
        payload.channel_id,
        &payload.command,
        payload.params
    ).await {
        Ok(result) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "命令执行成功".to_string(),
            data: Some(result),
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("命令执行失败: {:?}", e),
            data: None,
        }),
    }
}

/// 调用通道方法
#[utoipa::path(
    post,
    path = "/lspcapi/device/callMethod",
    request_body = CallMethodRequest,
    responses(
        (status = 200, description = "方法调用成功", body = inline(ApiResponse<serde_json::Value>))
    ),
    tag = "Device"
)]
pub async fn call_method(
    Extension(controller): Extension<Arc<DeviceController>>,
    Json(payload): Json<CallMethodRequest>,
) -> Json<ApiResponse<serde_json::Value>> {
    match controller.call_channel_method(
        payload.channel_id,
        &payload.method_name,
        payload.arguments
    ).await {
        Ok(result) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "方法调用成功".to_string(),
            data: Some(result),
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("方法调用失败: {:?}", e),
            data: None,
        }),
    }
}

/// 获取通道方法列表
#[utoipa::path(
    post,
    path = "/lspcapi/device/getMethods",
    request_body = GetMethodsRequest,
    responses(
        (status = 200, description = "获取成功", body = inline(ApiResponse<Vec<String>>))
    ),
    tag = "Device"
)]
pub async fn get_methods(
    Extension(controller): Extension<Arc<DeviceController>>,
    Json(payload): Json<GetMethodsRequest>,
) -> Json<ApiResponse<Vec<String>>> {
    match controller.get_channel_methods(payload.channel_id).await {
        Ok(methods) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "获取方法列表成功".to_string(),
            data: Some(methods),
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("获取方法列表失败: {:?}", e),
            data: None,
        }),
    }
}

/// 批量读取（通过通道命令）
#[utoipa::path(
    post,
    path = "/lspcapi/device/batchRead",
    request_body = BatchReadRequest,
    responses(
        (status = 200, description = "批量读取完成", body = inline(ApiResponse<Vec<BatchReadResultItem>>))
    ),
    tag = "Device"
)]
pub async fn batch_read(
    Extension(controller): Extension<Arc<DeviceController>>,
    Json(payload): Json<BatchReadRequest>,
) -> Json<ApiResponse<Vec<BatchReadResultItem>>> {
    let mut results = Vec::new();

    for item in payload.items {
        let result = controller.execute_channel_command(
            item.channel_id,
            "read",
            item.params
        ).await;

        match result {
            Ok(value) => {
                results.push(BatchReadResultItem {
                    name: item.name,
                    success: true,
                    value: Some(value),
                    error: None,
                });
            }
            Err(e) => {
                results.push(BatchReadResultItem {
                    name: item.name,
                    success: false,
                    value: None,
                    error: Some(format!("{:?}", e)),
                });
            }
        }
    }

    Json(ApiResponse {
        state: error_codes::SUCCESS,
        message: format!("批量读取完成: 成功 {}, 失败 {}",
            results.iter().filter(|r| r.success).count(),
            results.iter().filter(|r| !r.success).count()
        ),
        data: Some(results),
    })
}
