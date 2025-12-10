//! 通用 API 响应类型

use serde::Serialize;
use utoipa::ToSchema;
use crate::db::{Screen, UploadMaterialResponse, MaterialResponse};

/// 统一 API 响应结构
#[derive(Serialize, ToSchema)]
#[aliases(
    ScreenApiResponse = ApiResponse<Screen>,
    ScreenListApiResponse = ApiResponse<Vec<Screen>>,
    MaterialSingleApiResponse = ApiResponse<MaterialResponse>,
    MaterialArrayApiResponse = ApiResponse<Vec<MaterialResponse>>,
    UploadMaterialApiResponse = ApiResponse<UploadMaterialResponse>
)]
pub struct ApiResponse<T> {
    /// 状态码（0 表示成功）
    pub state: i32,
    /// 响应消息
    pub message: String,
    /// 响应数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(message: impl Into<String>, data: T) -> Self {
        Self {
            state: crate::utils::error::error_codes::SUCCESS,
            message: message.into(),
            data: Some(data),
        }
    }

    /// 创建成功响应（无数据）
    pub fn success_empty(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            state: crate::utils::error::error_codes::SUCCESS,
            message: message.into(),
            data: None,
        }
    }

    /// 创建错误响应
    pub fn error(code: i32, message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            state: code,
            message: message.into(),
            data: None,
        }
    }

    /// 创建通用错误响应
    pub fn general_error(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            state: crate::utils::error::error_codes::GENERAL_ERROR,
            message: message.into(),
            data: None,
        }
    }

    /// 创建参数无效错误响应
    pub fn invalid_params(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            state: crate::utils::error::error_codes::INVALID_PARAMS,
            message: message.into(),
            data: None,
        }
    }
}
