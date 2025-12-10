//! 数据库模型定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Screen 类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "enum", rename_all = "PascalCase")]
pub enum ScreenType {
    Clean,
    Close,
    Normal,
    Pause,
    Register,
    Vote,
}

impl std::fmt::Display for ScreenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScreenType::Clean => write!(f, "Clean"),
            ScreenType::Close => write!(f, "Close"),
            ScreenType::Normal => write!(f, "Normal"),
            ScreenType::Pause => write!(f, "Pause"),
            ScreenType::Register => write!(f, "Register"),
            ScreenType::Vote => write!(f, "Vote"),
        }
    }
}

impl std::str::FromStr for ScreenType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Clean" => Ok(ScreenType::Clean),
            "Close" => Ok(ScreenType::Close),
            "Normal" => Ok(ScreenType::Normal),
            "Pause" => Ok(ScreenType::Pause),
            "Register" => Ok(ScreenType::Register),
            "Vote" => Ok(ScreenType::Vote),
            _ => Err(anyhow::anyhow!("无效的 ScreenType: {}", s)),
        }
    }
}

/// lspc_screen 表模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Screen {
    /// 屏幕唯一标识
    pub id: String,
    /// 屏幕类型
    pub screen_type: String,
    /// 屏幕名称
    pub name: String,
    /// 屏幕内容（JSON格式）
    pub content: String,
    /// 是否激活
    pub active: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 创建 Screen 请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateScreenRequest {
    /// 屏幕ID（可选，不提供则自动生成）
    #[serde(default = "generate_uuid")]
    pub id: String,
    /// 屏幕类型：Clean, Close, Normal, Pause, Register, Vote
    #[serde(rename = "type")]
    pub screen_type: String,
    /// 屏幕名称
    pub name: String,
    /// 屏幕内容（JSON格式）
    pub content: String,
    /// 是否激活
    #[serde(default)]
    pub active: bool,
}

fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// 更新 Screen 请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateScreenRequest {
    /// 屏幕类型（可选）
    #[serde(rename = "type")]
    pub screen_type: Option<String>,
    /// 屏幕名称（可选）
    pub name: Option<String>,
    /// 屏幕内容（可选）
    pub content: Option<String>,
    /// 是否激活（可选）
    pub active: Option<bool>,
}

/// lspc_material 表模型（合并了素材和资源信息）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Material {
    /// 素材唯一标识
    pub id: String,
    /// 素材名称
    pub name: String,
    /// 关联的屏幕ID
    pub screen_id: String,
    /// 是否为预设素材
    pub preset: bool,
    /// 文件路径（相对于静态目录）
    pub path: String,
    /// 资源类型（image, video, audio, document, other）
    pub resource_type: String,
    /// 文件大小（字节）
    pub size: i64,
    /// MIME 类型
    pub mime_type: String,
    /// 原始文件名
    pub original_name: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// MaterialResponse 为 Material 的类型别名（API 响应使用）
pub type MaterialResponse = Material;

/// 创建 Material 请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMaterialRequest {
    /// 素材ID（可选，不提供则自动生成）
    #[serde(default = "generate_uuid")]
    pub id: String,
    /// 素材名称
    pub name: String,
    /// 关联的屏幕ID
    #[serde(default)]
    pub screen_id: String,
    /// 是否为预设素材
    #[serde(default)]
    pub preset: bool,
    /// 文件路径
    pub path: String,
    /// 资源类型
    #[serde(default)]
    pub resource_type: String,
    /// 文件大小（字节）
    #[serde(default)]
    pub size: i64,
    /// MIME 类型
    #[serde(default)]
    pub mime_type: String,
    /// 原始文件名
    #[serde(default)]
    pub original_name: String,
}

/// 更新 Material 请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateMaterialRequest {
    /// 素材名称（可选）
    pub name: Option<String>,
    /// 关联的屏幕ID（可选）
    pub screen_id: Option<String>,
    /// 是否为预设素材（可选）
    pub preset: Option<bool>,
}

/// 批量覆盖请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BatchReplaceScreensRequest {
    /// 屏幕列表
    pub screens: Vec<CreateScreenRequest>,
}

/// 批量覆盖 Material 请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BatchReplaceMaterialsRequest {
    /// 素材列表
    pub materials: Vec<CreateMaterialRequest>,
}

/// 上传素材响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UploadMaterialResponse {
    /// 素材ID
    pub id: String,
    /// 文件名称
    pub name: String,
    /// 关联的屏幕ID
    pub screen_id: String,
    /// 是否为预设素材
    pub preset: bool,
    /// 访问路径
    pub path: String,
    /// 创建时间
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

/// 上传素材请求（用于 Swagger 文档）
#[derive(Debug, ToSchema)]
pub struct UploadMaterialRequest {
    /// 屏幕ID
    #[schema(example = "screen-001")]
    pub screen_id: String,
    /// 上传的文件
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
}
