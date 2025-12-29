//! 数据库 API 路由
//!
//! 提供 Screen 和 Material 表的 CRUD 操作接口

use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::IntoParams;

use crate::db::{
    Database,
    CreateScreenRequest, UpdateScreenRequest, Screen,
    CreateMaterialRequest, UpdateMaterialRequest, MaterialResponse,
    BatchReplaceScreensRequest, BatchReplaceMaterialsRequest,
};
use super::resource_api::ResourceManagerState;
use super::response::ApiResponse;

/// 错误码
mod error_codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const NOT_FOUND: i32 = 404;
    pub const INVALID_PARAMS: i32 = 400;
}

// ==================== Screen API ====================

/// Screen 查询参数
#[derive(Debug, Deserialize, IntoParams)]
pub struct ScreenQuery {
    /// 按屏幕类型过滤
    #[serde(rename = "type")]
    pub screen_type: Option<String>,
    /// 按激活状态过滤
    pub active: Option<bool>,
}

/// 获取所有 Screen
#[utoipa::path(
    get,
    path = "/lspcapi/screens",
    params(ScreenQuery),
    responses(
        (status = 200, description = "查询成功", body = ScreenListApiResponse)
    ),
    tag = "Screen"
)]
pub async fn list_screens(
    Extension(db): Extension<Arc<Database>>,
    Query(query): Query<ScreenQuery>,
) -> Json<ApiResponse<Vec<Screen>>> {
    let result = if let (Some(ref screen_type), Some(active)) = (&query.screen_type, query.active) {
        // 同时指定类型和激活状态：AND 查询
        db.screens().find_by_type_and_active(screen_type, active).await
    } else if let Some(ref screen_type) = query.screen_type {
        // 只指定类型
        db.screens().find_by_type(screen_type).await
    } else if query.active == Some(true) {
        // 只指定激活状态
        db.screens().find_active().await
    } else {
        // 无过滤条件
        db.screens().find_all().await
    };

    match result {
        Ok(screens) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "成功".to_string(),
            data: Some(screens),
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("查询失败: {}", e),
            data: None,
        }),
    }
}

/// 获取单个 Screen
#[utoipa::path(
    get,
    path = "/lspcapi/screens/{id}",
    params(("id" = String, Path, description = "Screen ID")),
    responses(
        (status = 200, description = "查询成功", body = ScreenApiResponse),
        (status = 404, description = "Screen 不存在")
    ),
    tag = "Screen"
)]
pub async fn get_screen(
    Extension(db): Extension<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Screen>> {
    match db.screens().find_by_id(&id).await {
        Ok(Some(screen)) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "成功".to_string(),
            data: Some(screen),
        }),
        Ok(None) => Json(ApiResponse {
            state: error_codes::NOT_FOUND,
            message: "Screen 不存在".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("查询失败: {}", e),
            data: None,
        }),
    }
}

/// 创建 Screen
#[utoipa::path(
    post,
    path = "/lspcapi/screens",
    request_body = CreateScreenRequest,
    responses(
        (status = 200, description = "创建成功", body = ScreenApiResponse)
    ),
    tag = "Screen"
)]
pub async fn create_screen(
    Extension(db): Extension<Arc<Database>>,
    Json(req): Json<CreateScreenRequest>,
) -> Json<ApiResponse<Screen>> {
    match db.screens().create(&req).await {
        Ok(screen) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "创建成功".to_string(),
            data: Some(screen),
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("创建失败: {}", e),
            data: None,
        }),
    }
}

/// 更新 Screen
#[utoipa::path(
    put,
    path = "/lspcapi/screens/{id}",
    params(("id" = String, Path, description = "Screen ID")),
    request_body = UpdateScreenRequest,
    responses(
        (status = 200, description = "更新成功", body = ScreenApiResponse),
        (status = 404, description = "Screen 不存在")
    ),
    tag = "Screen"
)]
pub async fn update_screen(
    Extension(db): Extension<Arc<Database>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateScreenRequest>,
) -> Json<ApiResponse<Screen>> {
    match db.screens().update(&id, &req).await {
        Ok(Some(screen)) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "更新成功".to_string(),
            data: Some(screen),
        }),
        Ok(None) => Json(ApiResponse {
            state: error_codes::NOT_FOUND,
            message: "Screen 不存在".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("更新失败: {}", e),
            data: None,
        }),
    }
}

/// 删除 Screen
#[utoipa::path(
    delete,
    path = "/lspcapi/screens/{id}",
    params(("id" = String, Path, description = "Screen ID")),
    responses(
        (status = 200, description = "删除成功"),
        (status = 404, description = "Screen 不存在")
    ),
    tag = "Screen"
)]
pub async fn delete_screen(
    Extension(db): Extension<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    match db.screens().delete(&id).await {
        Ok(true) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "删除成功".to_string(),
            data: None,
        }),
        Ok(false) => Json(ApiResponse {
            state: error_codes::NOT_FOUND,
            message: "Screen 不存在".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("删除失败: {}", e),
            data: None,
        }),
    }
}

/// 批量覆盖 Screen（删除所有后重新创建）
#[utoipa::path(
    post,
    path = "/lspcapi/screens/replace",
    request_body = BatchReplaceScreensRequest,
    responses(
        (status = 200, description = "覆盖成功", body = ScreenListApiResponse)
    ),
    tag = "Screen"
)]
pub async fn replace_all_screens(
    Extension(db): Extension<Arc<Database>>,
    Json(req): Json<BatchReplaceScreensRequest>,
) -> Json<ApiResponse<Vec<Screen>>> {
    match db.screens().replace_all(&req.screens).await {
        Ok(screens) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: format!("覆盖成功，共 {} 条记录", screens.len()),
            data: Some(screens),
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("覆盖失败: {}", e),
            data: None,
        }),
    }
}

/// 设置指定 Screen 为激活状态（同一类型的其他 Screen 自动设为非激活）
///
/// 注意：只会将相同类型（type）的其他屏幕设为非激活。
/// 例如：激活一个 Vote 类型的屏幕时，只有其他 Vote 类型的屏幕会被设为非激活，
/// Normal、Register 等其他类型的屏幕不受影响。
#[utoipa::path(
    put,
    path = "/lspcapi/screens/{id}/active",
    params(("id" = String, Path, description = "Screen ID")),
    responses(
        (status = 200, description = "设置成功（同一类型的其他屏幕自动设为非激活）", body = ScreenApiResponse),
        (status = 404, description = "Screen 不存在")
    ),
    tag = "Screen"
)]
pub async fn set_screen_active(
    Extension(db): Extension<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Screen>> {
    match db.screens().set_active(&id).await {
        Ok(Some(screen)) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "设置成功".to_string(),
            data: Some(screen),
        }),
        Ok(None) => Json(ApiResponse {
            state: error_codes::NOT_FOUND,
            message: "Screen 不存在".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("设置失败: {}", e),
            data: None,
        }),
    }
}

// ==================== Material API ====================

/// Material 查询参数
#[derive(Debug, Deserialize, IntoParams)]
pub struct MaterialQuery {
    /// 按素材名称过滤（模糊匹配）
    pub name: Option<String>,
    /// 按是否为预设素材过滤
    pub preset: Option<bool>,
}

/// 获取所有 Material
#[utoipa::path(
    get,
    path = "/lspcapi/materials",
    params(MaterialQuery),
    responses(
        (status = 200, description = "查询成功", body = MaterialArrayApiResponse)
    ),
    tag = "Material"
)]
pub async fn list_materials(
    Extension(db): Extension<Arc<Database>>,
    resource_state: Option<Extension<ResourceManagerState>>,
    Query(query): Query<MaterialQuery>,
) -> Json<ApiResponse<Vec<MaterialResponse>>> {
    let result = if let Some(preset) = query.preset {
        db.materials().find_by_preset(preset).await
    } else if let Some(ref name) = query.name {
        db.materials().find_by_name(name).await
    } else {
        db.materials().find_all().await
    };

    match result {
        Ok(materials) => {
            // 添加完整的静态资源路径前缀
            let materials = materials.into_iter().map(|mut m| {
                if !m.path.is_empty() {
                    if let Some(Extension(ref state)) = resource_state {
                        m.path = format!("{}/{}", state.config.url_prefix.trim_end_matches('/'), m.path);
                    }
                }
                m
            }).collect();
            Json(ApiResponse {
                state: error_codes::SUCCESS,
                message: "成功".to_string(),
                data: Some(materials),
            })
        }
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("查询失败: {}", e),
            data: None,
        }),
    }
}

/// 获取单个 Material
#[utoipa::path(
    get,
    path = "/lspcapi/materials/{id}",
    params(("id" = String, Path, description = "Material ID")),
    responses(
        (status = 200, description = "查询成功", body = MaterialSingleApiResponse),
        (status = 404, description = "Material 不存在")
    ),
    tag = "Material"
)]
pub async fn get_material(
    Extension(db): Extension<Arc<Database>>,
    resource_state: Option<Extension<ResourceManagerState>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<MaterialResponse>> {
    match db.materials().find_by_id(&id).await {
        Ok(Some(mut material)) => {
            // 添加完整的静态资源路径前缀
            if !material.path.is_empty() {
                if let Some(Extension(ref state)) = resource_state {
                    material.path = format!("{}/{}", state.config.url_prefix.trim_end_matches('/'), material.path);
                }
            }
            Json(ApiResponse {
                state: error_codes::SUCCESS,
                message: "成功".to_string(),
                data: Some(material),
            })
        }
        Ok(None) => Json(ApiResponse {
            state: error_codes::NOT_FOUND,
            message: "Material 不存在".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("查询失败: {}", e),
            data: None,
        }),
    }
}

/// 创建 Material
#[utoipa::path(
    post,
    path = "/lspcapi/materials",
    request_body = CreateMaterialRequest,
    responses(
        (status = 200, description = "创建成功", body = MaterialSingleApiResponse)
    ),
    tag = "Material"
)]
pub async fn create_material(
    Extension(db): Extension<Arc<Database>>,
    Json(req): Json<CreateMaterialRequest>,
) -> Json<ApiResponse<MaterialResponse>> {
    match db.materials().create(&req).await {
        Ok(material) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "创建成功".to_string(),
            data: Some(material),
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("创建失败: {}", e),
            data: None,
        }),
    }
}

/// 更新 Material
#[utoipa::path(
    put,
    path = "/lspcapi/materials/{id}",
    params(("id" = String, Path, description = "Material ID")),
    request_body = UpdateMaterialRequest,
    responses(
        (status = 200, description = "更新成功", body = MaterialSingleApiResponse),
        (status = 404, description = "Material 不存在")
    ),
    tag = "Material"
)]
pub async fn update_material(
    Extension(db): Extension<Arc<Database>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateMaterialRequest>,
) -> Json<ApiResponse<MaterialResponse>> {
    match db.materials().update(&id, &req).await {
        Ok(Some(material)) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "更新成功".to_string(),
            data: Some(material),
        }),
        Ok(None) => Json(ApiResponse {
            state: error_codes::NOT_FOUND,
            message: "Material 不存在".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("更新失败: {}", e),
            data: None,
        }),
    }
}

/// 删除 Material（同时删除关联的资源文件）
#[utoipa::path(
    delete,
    path = "/lspcapi/materials/{id}",
    params(("id" = String, Path, description = "Material ID")),
    responses(
        (status = 200, description = "删除成功"),
        (status = 404, description = "Material 不存在")
    ),
    tag = "Material"
)]
pub async fn delete_material(
    Extension(db): Extension<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    match db.materials().delete(&id).await {
        Ok(true) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "删除成功".to_string(),
            data: None,
        }),
        Ok(false) => Json(ApiResponse {
            state: error_codes::NOT_FOUND,
            message: "Material 不存在".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("删除失败: {}", e),
            data: None,
        }),
    }
}

/// 根据 Screen ID 获取所有 Material
#[utoipa::path(
    get,
    path = "/lspcapi/screens/{id}/materials",
    params(("id" = String, Path, description = "Screen ID")),
    responses(
        (status = 200, description = "查询成功", body = MaterialArrayApiResponse),
        (status = 404, description = "Screen 不存在")
    ),
    tag = "Material"
)]
pub async fn get_materials_by_screen_id(
    Extension(db): Extension<Arc<Database>>,
    resource_state: Option<Extension<ResourceManagerState>>,
    Path(screen_id): Path<String>,
) -> Json<ApiResponse<Vec<MaterialResponse>>> {
    // 先检查 screen 是否存在
    match db.screens().find_by_id(&screen_id).await {
        Ok(None) => {
            return Json(ApiResponse {
                state: error_codes::NOT_FOUND,
                message: "Screen 不存在".to_string(),
                data: None,
            });
        }
        Err(e) => {
            return Json(ApiResponse {
                state: error_codes::GENERAL_ERROR,
                message: format!("查询 Screen 失败: {}", e),
                data: None,
            });
        }
        Ok(Some(_)) => {}
    }

    // 获取该 screen 关联的所有素材
    match db.materials().find_by_screen_id(&screen_id).await {
        Ok(materials) => {
            // 添加完整的静态资源路径前缀
            let materials = materials.into_iter().map(|mut m| {
                if !m.path.is_empty() {
                    if let Some(Extension(ref state)) = resource_state {
                        m.path = format!("{}/{}", state.config.url_prefix.trim_end_matches('/'), m.path);
                    }
                }
                m
            }).collect();
            Json(ApiResponse {
                state: error_codes::SUCCESS,
                message: "成功".to_string(),
                data: Some(materials),
            })
        }
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("查询失败: {}", e),
            data: None,
        }),
    }
}

/// 批量覆盖 Material（删除所有后重新创建）
#[utoipa::path(
    post,
    path = "/lspcapi/materials/replace",
    request_body = BatchReplaceMaterialsRequest,
    responses(
        (status = 200, description = "覆盖成功", body = MaterialArrayApiResponse)
    ),
    tag = "Material"
)]
pub async fn replace_all_materials(
    Extension(db): Extension<Arc<Database>>,
    Json(req): Json<BatchReplaceMaterialsRequest>,
) -> Json<ApiResponse<Vec<MaterialResponse>>> {
    match db.materials().replace_all(&req.materials).await {
        Ok(materials) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: format!("覆盖成功，共 {} 条记录", materials.len()),
            data: Some(materials),
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("覆盖失败: {}", e),
            data: None,
        }),
    }
}
