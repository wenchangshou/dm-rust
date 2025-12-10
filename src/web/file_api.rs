//! 文件管理 API 处理器

use axum::{
    Json,
    extract::{Extension, Multipart, Query},
    body::StreamBody,
    http::{header, StatusCode, HeaderValue},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio_util::io::ReaderStream;

use crate::config::FileConfig;
use crate::utils::error::error_codes;
use super::response::ApiResponse;

// ===== 状态和类型定义 =====

/// 文件管理器状态
#[derive(Clone)]
pub struct FileManagerState {
    pub config: Option<FileConfig>,
}

/// 文件路径查询参数
#[derive(Deserialize)]
pub struct FilePathQuery {
    pub path: Option<String>,
}

/// 文件信息
#[derive(Serialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
}

/// 创建目录请求
#[derive(Deserialize)]
pub struct CreateDirRequest {
    pub path: String,
}

/// 删除请求
#[derive(Deserialize)]
pub struct DeleteRequest {
    pub path: String,
}

/// 重命名请求
#[derive(Deserialize)]
pub struct RenameRequest {
    pub old_path: String,
    pub new_path: String,
}

// ===== 工具函数 =====

/// 获取安全的文件路径，防止路径遍历攻击
pub fn get_safe_path(base_path: &str, relative_path: &str) -> Option<PathBuf> {
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

/// 根据文件扩展名获取 MIME 类型
pub fn get_mime_type(path: &PathBuf) -> &'static str {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        // 文本类型
        "txt" => "text/plain; charset=utf-8",
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",
        "yaml" | "yml" => "text/yaml; charset=utf-8",
        "toml" => "text/plain; charset=utf-8",
        "ini" | "conf" | "cfg" => "text/plain; charset=utf-8",
        "log" => "text/plain; charset=utf-8",
        "sh" | "bash" => "text/x-shellscript; charset=utf-8",
        "py" => "text/x-python; charset=utf-8",
        "rs" => "text/x-rust; charset=utf-8",
        "c" | "h" => "text/x-c; charset=utf-8",
        "cpp" | "hpp" | "cc" => "text/x-c++; charset=utf-8",
        "java" => "text/x-java; charset=utf-8",
        "go" => "text/x-go; charset=utf-8",
        "sql" => "text/x-sql; charset=utf-8",

        // 图片类型
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",

        // 视频类型
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogg" => "video/ogg",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",
        "mov" => "video/quicktime",

        // 音频类型
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        "m4a" => "audio/mp4",

        // 文档类型
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",

        // 压缩文件
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" | "gzip" => "application/gzip",
        "rar" => "application/vnd.rar",
        "7z" => "application/x-7z-compressed",

        // 字体
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",

        // 默认
        _ => "application/octet-stream",
    }
}

/// 检查文件管理配置是否启用
fn check_config(state: &FileManagerState) -> Result<&FileConfig, Json<ApiResponse<()>>> {
    match &state.config {
        Some(c) if c.enable => Ok(c),
        _ => Err(Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: "文件管理功能未启用".to_string(),
            data: None,
        })),
    }
}

// ===== API 处理函数 =====

/// 列出目录内容
pub async fn file_list(
    Extension(state): Extension<FileManagerState>,
    Query(query): Query<FilePathQuery>,
) -> Json<ApiResponse<Vec<FileInfo>>> {
    let config = match check_config(&state) {
        Ok(c) => c,
        Err(e) => return Json(ApiResponse {
            state: e.0.state,
            message: e.0.message,
            data: None,
        }),
    };

    let relative_path = query.path.unwrap_or_default();
    let full_path = match get_safe_path(&config.path, &relative_path) {
        Some(p) => p,
        None => return Json(ApiResponse {
            state: error_codes::INVALID_PARAMS,
            message: "无效的路径".to_string(),
            data: None,
        }),
    };

    let mut entries = match fs::read_dir(&full_path).await {
        Ok(e) => e,
        Err(e) => return Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("读取目录失败: {}", e),
            data: None,
        }),
    };

    let mut files = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let metadata = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue,
        };

        let name = entry.file_name().to_string_lossy().to_string();
        let entry_path = if relative_path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", relative_path.trim_end_matches('/'), name)
        };

        let modified = metadata.modified().ok().map(|t| {
            let datetime: chrono::DateTime<chrono::Local> = t.into();
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        });

        files.push(FileInfo {
            name,
            path: entry_path,
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified,
        });
    }

    // 按类型和名称排序：目录在前，文件在后
    files.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Json(ApiResponse {
        state: error_codes::SUCCESS,
        message: "成功".to_string(),
        data: Some(files),
    })
}

/// 上传文件
pub async fn file_upload(
    Extension(state): Extension<FileManagerState>,
    Query(query): Query<FilePathQuery>,
    mut multipart: Multipart,
) -> Json<ApiResponse<Vec<String>>> {
    let config = match check_config(&state) {
        Ok(c) => c,
        Err(e) => return Json(ApiResponse {
            state: e.0.state,
            message: e.0.message,
            data: None,
        }),
    };

    let relative_path = query.path.unwrap_or_default();
    let base_path = match get_safe_path(&config.path, &relative_path) {
        Some(p) => p,
        None => return Json(ApiResponse {
            state: error_codes::INVALID_PARAMS,
            message: "无效的路径".to_string(),
            data: None,
        }),
    };

    let mut uploaded_files = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = match field.file_name() {
            Some(name) => name.to_string(),
            None => continue,
        };

        let file_path = base_path.join(&file_name);

        // 再次验证路径安全性
        if !file_path.starts_with(&PathBuf::from(&config.path).canonicalize().unwrap_or_default()) {
            continue;
        }

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                tracing::error!("读取上传数据失败: {}", e);
                continue;
            }
        };

        match fs::write(&file_path, &data).await {
            Ok(_) => {
                uploaded_files.push(file_name);
            }
            Err(e) => {
                tracing::error!("保存文件失败: {}", e);
            }
        }
    }

    Json(ApiResponse {
        state: error_codes::SUCCESS,
        message: format!("上传完成，成功 {} 个文件", uploaded_files.len()),
        data: Some(uploaded_files),
    })
}

/// 下载文件
pub async fn file_download(
    Extension(state): Extension<FileManagerState>,
    Query(query): Query<FilePathQuery>,
) -> impl IntoResponse {
    let config = match &state.config {
        Some(c) if c.enable => c,
        _ => return Err((StatusCode::SERVICE_UNAVAILABLE, "文件管理功能未启用")),
    };

    let relative_path = match &query.path {
        Some(p) => p,
        None => return Err((StatusCode::BAD_REQUEST, "缺少 path 参数")),
    };

    let full_path = match get_safe_path(&config.path, relative_path) {
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

    let file_name = full_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download")
        .to_string();

    let content_disposition = format!("attachment; filename=\"{}\"", file_name);

    let headers = [
        (header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream")),
        (header::CONTENT_DISPOSITION, HeaderValue::from_str(&content_disposition).unwrap_or_else(|_| HeaderValue::from_static("attachment"))),
    ];

    Ok((headers, body))
}

/// 删除文件或目录
pub async fn file_delete(
    Extension(state): Extension<FileManagerState>,
    Json(payload): Json<DeleteRequest>,
) -> Json<ApiResponse<()>> {
    let config = match check_config(&state) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let full_path = match get_safe_path(&config.path, &payload.path) {
        Some(p) => p,
        None => return Json(ApiResponse {
            state: error_codes::INVALID_PARAMS,
            message: "无效的路径".to_string(),
            data: None,
        }),
    };

    // 防止删除根目录
    if full_path == PathBuf::from(&config.path).canonicalize().unwrap_or_default() {
        return Json(ApiResponse {
            state: error_codes::INVALID_PARAMS,
            message: "不能删除根目录".to_string(),
            data: None,
        });
    }

    let result = if full_path.is_dir() {
        fs::remove_dir_all(&full_path).await
    } else {
        fs::remove_file(&full_path).await
    };

    match result {
        Ok(_) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "删除成功".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("删除失败: {}", e),
            data: None,
        }),
    }
}

/// 创建目录
pub async fn file_mkdir(
    Extension(state): Extension<FileManagerState>,
    Json(payload): Json<CreateDirRequest>,
) -> Json<ApiResponse<()>> {
    let config = match check_config(&state) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let full_path = match get_safe_path(&config.path, &payload.path) {
        Some(p) => p,
        None => return Json(ApiResponse {
            state: error_codes::INVALID_PARAMS,
            message: "无效的路径".to_string(),
            data: None,
        }),
    };

    match fs::create_dir_all(&full_path).await {
        Ok(_) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "目录创建成功".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("创建目录失败: {}", e),
            data: None,
        }),
    }
}

/// 重命名文件或目录
pub async fn file_rename(
    Extension(state): Extension<FileManagerState>,
    Json(payload): Json<RenameRequest>,
) -> Json<ApiResponse<()>> {
    let config = match check_config(&state) {
        Ok(c) => c,
        Err(e) => return e,
    };

    let old_path = match get_safe_path(&config.path, &payload.old_path) {
        Some(p) => p,
        None => return Json(ApiResponse {
            state: error_codes::INVALID_PARAMS,
            message: "无效的源路径".to_string(),
            data: None,
        }),
    };

    let new_path = match get_safe_path(&config.path, &payload.new_path) {
        Some(p) => p,
        None => return Json(ApiResponse {
            state: error_codes::INVALID_PARAMS,
            message: "无效的目标路径".to_string(),
            data: None,
        }),
    };

    match fs::rename(&old_path, &new_path).await {
        Ok(_) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "重命名成功".to_string(),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("重命名失败: {}", e),
            data: None,
        }),
    }
}

/// 获取文件信息
pub async fn file_info(
    Extension(state): Extension<FileManagerState>,
    Query(query): Query<FilePathQuery>,
) -> Json<ApiResponse<FileInfo>> {
    let config = match check_config(&state) {
        Ok(c) => c,
        Err(e) => return Json(ApiResponse {
            state: e.0.state,
            message: e.0.message,
            data: None,
        }),
    };

    let relative_path = match &query.path {
        Some(p) => p,
        None => return Json(ApiResponse {
            state: error_codes::INVALID_PARAMS,
            message: "缺少 path 参数".to_string(),
            data: None,
        }),
    };

    let full_path = match get_safe_path(&config.path, relative_path) {
        Some(p) => p,
        None => return Json(ApiResponse {
            state: error_codes::INVALID_PARAMS,
            message: "无效的路径".to_string(),
            data: None,
        }),
    };

    let metadata = match fs::metadata(&full_path).await {
        Ok(m) => m,
        Err(e) => return Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("获取文件信息失败: {}", e),
            data: None,
        }),
    };

    let name = full_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let modified = metadata.modified().ok().map(|t| {
        let datetime: chrono::DateTime<chrono::Local> = t.into();
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    });

    Json(ApiResponse {
        state: error_codes::SUCCESS,
        message: "成功".to_string(),
        data: Some(FileInfo {
            name,
            path: relative_path.clone(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified,
        }),
    })
}

/// 预览文件（返回适当的 Content-Type）
pub async fn file_preview(
    Extension(state): Extension<FileManagerState>,
    Query(query): Query<FilePathQuery>,
) -> impl IntoResponse {
    let config = match &state.config {
        Some(c) if c.enable => c,
        _ => return Err((StatusCode::SERVICE_UNAVAILABLE, "文件管理功能未启用")),
    };

    let relative_path = match &query.path {
        Some(p) => p,
        None => return Err((StatusCode::BAD_REQUEST, "缺少 path 参数")),
    };

    let full_path = match get_safe_path(&config.path, relative_path) {
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
    let content_type = get_mime_type(&full_path);

    let headers = [
        (header::CONTENT_TYPE, HeaderValue::from_str(content_type).unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"))),
    ];

    Ok((headers, body))
}

/// 查看文本文件内容
pub async fn file_view(
    Extension(state): Extension<FileManagerState>,
    Query(query): Query<FilePathQuery>,
) -> Json<ApiResponse<String>> {
    let config = match check_config(&state) {
        Ok(c) => c,
        Err(e) => return Json(ApiResponse {
            state: e.0.state,
            message: e.0.message,
            data: None,
        }),
    };

    let relative_path = match &query.path {
        Some(p) => p,
        None => return Json(ApiResponse {
            state: error_codes::INVALID_PARAMS,
            message: "缺少 path 参数".to_string(),
            data: None,
        }),
    };

    let full_path = match get_safe_path(&config.path, relative_path) {
        Some(p) => p,
        None => return Json(ApiResponse {
            state: error_codes::INVALID_PARAMS,
            message: "无效的路径".to_string(),
            data: None,
        }),
    };

    if !full_path.is_file() {
        return Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: "文件不存在".to_string(),
            data: None,
        });
    }

    // 限制文件大小（最大 10MB）
    let metadata = match fs::metadata(&full_path).await {
        Ok(m) => m,
        Err(e) => return Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("获取文件信息失败: {}", e),
            data: None,
        }),
    };

    if metadata.len() > 10 * 1024 * 1024 {
        return Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: "文件过大，无法预览（最大 10MB）".to_string(),
            data: None,
        });
    }

    match fs::read_to_string(&full_path).await {
        Ok(content) => Json(ApiResponse {
            state: error_codes::SUCCESS,
            message: "成功".to_string(),
            data: Some(content),
        }),
        Err(_) => {
            // 尝试以二进制方式读取并转为 base64
            match fs::read(&full_path).await {
                Ok(bytes) => {
                    use base64::{Engine as _, engine::general_purpose::STANDARD};
                    let base64_content = STANDARD.encode(&bytes);
                    Json(ApiResponse {
                        state: error_codes::SUCCESS,
                        message: "base64".to_string(),
                        data: Some(base64_content),
                    })
                }
                Err(e) => Json(ApiResponse {
                    state: error_codes::GENERAL_ERROR,
                    message: format!("读取文件失败: {}", e),
                    data: None,
                }),
            }
        }
    }
}
