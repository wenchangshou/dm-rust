//! 资源管理 API 路由
//!
//! 提供素材上传、静态资源服务等操作接口

use axum::{
    extract::{Extension, Multipart, Path},
    body::StreamBody,
    http::{header, StatusCode, HeaderValue},
    response::IntoResponse,
    Json,
};
use axum::body::Bytes;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::config::ResourceConfig;
use crate::db::{
    Database, CreateMaterialRequest, UploadMaterialResponse,
};
use super::response::ApiResponse;

/// 错误码
mod error_codes {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const NOT_FOUND: i32 = 404;
    pub const INVALID_PARAMS: i32 = 400;
}

/// 资源管理器状态
#[derive(Clone)]
pub struct ResourceManagerState {
    pub config: ResourceConfig,
}

// ==================== 工具函数 ====================

/// 获取安全的文件路径，防止路径遍历攻击
fn get_safe_path(base_path: &str, relative_path: &str) -> Option<PathBuf> {
    let base = PathBuf::from(base_path).canonicalize().ok()?;
    let relative = relative_path.trim_start_matches('/');
    let full_path = base.join(relative);

    // 确保路径在基础路径内
    if let Ok(canonical) = full_path.canonicalize() {
        if canonical.starts_with(&base) {
            return Some(canonical);
        }
    }

    // 对于不存在的路径，检查父目录
    if let Some(parent) = full_path.parent() {
        if let Ok(canonical_parent) = parent.canonicalize() {
            if canonical_parent.starts_with(&base) {
                return Some(full_path);
            }
        }
    }

    // 如果路径就是基础路径本身
    if relative.is_empty() || relative == "." {
        return Some(base);
    }

    None
}

/// 根据文件扩展名获取资源类型
fn get_resource_type(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        // 图片
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico" => "image",
        // 视频
        "mp4" | "webm" | "ogg" | "avi" | "mkv" | "mov" | "flv" | "wmv" => "video",
        // 音频
        "mp3" | "wav" | "flac" | "aac" | "m4a" | "wma" => "audio",
        // 文档
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "md" => "document",
        // 其他
        _ => "other",
    }
}

/// 根据文件扩展名获取 MIME 类型
fn get_mime_type(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        // 图片
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        // 视频
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogg" => "video/ogg",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",
        "mov" => "video/quicktime",
        // 音频
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        "m4a" => "audio/mp4",
        // 文档
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "json" => "application/json",
        // 默认
        _ => "application/octet-stream",
    }
}

// ==================== API 处理函数 ====================

/// 上传素材（POST /lspcapi/materials）
///
/// 需要传入文件和 screenId，首先验证 screenId 对应的屏幕是否存在，
/// 然后保存文件并创建素材记录
#[utoipa::path(
    post,
    path = "/lspcapi/materials",
    request_body(content_type = "multipart/form-data", content = UploadMaterialRequest),
    responses(
        (status = 200, description = "上传成功", body = UploadMaterialApiResponse),
        (status = 400, description = "无效的请求"),
        (status = 404, description = "屏幕不存在")
    ),
    tag = "Material"
)]
pub async fn upload_material(
    Extension(state): Extension<ResourceManagerState>,
    Extension(db): Extension<Arc<Database>>,
    mut multipart: Multipart,
) -> Json<ApiResponse<UploadMaterialResponse>> {
    // 解析 multipart 表单，提取 screenId 和文件
    let mut screen_id: Option<String> = None;
    let mut file_data: Option<(String, Bytes)> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or_default().to_string();

        if field_name == "screenId" || field_name == "screen_id" {
            match field.text().await {
                Ok(text) => screen_id = Some(text),
                Err(e) => return Json(ApiResponse {
                    state: error_codes::GENERAL_ERROR,
                    message: format!("读取 screenId 失败: {}", e),
                    data: None,
                }),
            }
        } else if field.file_name().is_some() {
            let original_name = field.file_name().unwrap().to_string();
            match field.bytes().await {
                Ok(data) => file_data = Some((original_name, data)),
                Err(e) => return Json(ApiResponse {
                    state: error_codes::GENERAL_ERROR,
                    message: format!("读取上传数据失败: {}", e),
                    data: None,
                }),
            }
        }
    }

    // 验证必需参数
    let screen_id = match screen_id {
        Some(id) if !id.is_empty() => id,
        _ => return Json(ApiResponse {
            state: error_codes::INVALID_PARAMS,
            message: "缺少 screenId 参数".to_string(),
            data: None,
        }),
    };

    let (original_name, data) = match file_data {
        Some(f) => f,
        None => return Json(ApiResponse {
            state: error_codes::INVALID_PARAMS,
            message: "未找到上传文件".to_string(),
            data: None,
        }),
    };

    // 验证 screenId 对应的屏幕是否存在
    match db.screens().find_by_id(&screen_id).await {
        Ok(Some(_)) => {},
        Ok(None) => return Json(ApiResponse {
            state: error_codes::NOT_FOUND,
            message: format!("屏幕不存在: {}", screen_id),
            data: None,
        }),
        Err(e) => return Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("查询屏幕失败: {}", e),
            data: None,
        }),
    }

    // 获取文件扩展名
    let ext = original_name.rsplit('.').next().unwrap_or("").to_lowercase();

    // 生成唯一文件名
    let file_id = Uuid::new_v4().to_string();
    let new_filename = if ext.is_empty() {
        file_id.clone()
    } else {
        format!("{}.{}", file_id, ext)
    };

    // 按日期创建子目录
    let resource_type = get_resource_type(&ext);
    let sub_dir = chrono::Local::now().format("%Y%m").to_string();
    let relative_path = format!("{}/{}", sub_dir, new_filename);

    // 确保目录存在
    let full_dir = PathBuf::from(&state.config.path).join(&sub_dir);
    if let Err(e) = fs::create_dir_all(&full_dir).await {
        return Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("创建目录失败: {}", e),
            data: None,
        });
    }

    // 保存文件
    let file_path = full_dir.join(&new_filename);
    let file_size = data.len() as i64;

    if let Err(e) = fs::write(&file_path, &data).await {
        return Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("保存文件失败: {}", e),
            data: None,
        });
    }

    // 创建素材数据库记录
    let mime_type = get_mime_type(&ext);
    let name = original_name
        .rsplit('/')
        .next()
        .unwrap_or(&original_name)
        .rsplit('\\')
        .next()
        .unwrap_or(&original_name)
        .to_string();

    let material_req = CreateMaterialRequest {
        id: file_id.clone(),
        name: name.clone(),
        screen_id: screen_id.clone(),
        preset: false,
        path: relative_path.clone(),
        resource_type: resource_type.to_string(),
        size: file_size,
        mime_type: mime_type.to_string(),
        original_name: original_name.clone(),
    };

    let material = match db.materials().create(&material_req).await {
        Ok(m) => m,
        Err(e) => {
            // 回滚：删除已上传的文件
            let _ = fs::remove_file(&file_path).await;
            return Json(ApiResponse {
                state: error_codes::GENERAL_ERROR,
                message: format!("创建素材记录失败: {}", e),
                data: None,
            });
        }
    };

    let url = format!("{}/{}", state.config.url_prefix.trim_end_matches('/'), relative_path);
    Json(ApiResponse {
        state: error_codes::SUCCESS,
        message: "上传成功".to_string(),
        data: Some(UploadMaterialResponse {
            id: material.id,
            name: material.name,
            screen_id: material.screen_id,
            preset: material.preset,
            path: url,
            created_at: material.created_at,
        }),
    })
}

/// 预览/下载静态资源
#[utoipa::path(
    get,
    path = "/static/{path}",
    params(("path" = String, Path, description = "资源文件路径")),
    responses(
        (status = 200, description = "文件内容"),
        (status = 404, description = "文件不存在")
    ),
    tag = "Resource"
)]
pub async fn serve_static_resource(
    Extension(state): Extension<ResourceManagerState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let full_path = match get_safe_path(&state.config.path, &path) {
        Some(p) => p,
        None => return Err((StatusCode::BAD_REQUEST, "无效的路径")),
    };

    if !full_path.is_file() {
        return Err((StatusCode::NOT_FOUND, "文件不存在"));
    }

    let file = match tokio::fs::File::open(&full_path).await {
        Ok(f) => f,
        Err(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "无法打开文件")),
    };

    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);

    // 根据文件扩展名确定 Content-Type
    let ext = full_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let content_type = get_mime_type(ext);

    let headers = [
        (header::CONTENT_TYPE, HeaderValue::from_str(content_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"))),
    ];

    Ok((headers, body))
}
