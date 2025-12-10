//! Swagger UI 和 OpenAPI 文档配置
//!
//! 提供 API 文档和交互式测试界面

use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::db::{
    Screen, CreateScreenRequest, UpdateScreenRequest, BatchReplaceScreensRequest,
    Material, MaterialResponse, CreateMaterialRequest, UpdateMaterialRequest, BatchReplaceMaterialsRequest,
    UploadMaterialResponse, UploadMaterialRequest,
};
use super::response::{
    ScreenApiResponse, ScreenListApiResponse,
    MaterialSingleApiResponse, MaterialArrayApiResponse,
    UploadMaterialApiResponse,
};
use super::device_api::{
    WriteRequest, WriteManyRequest, WriteManyItem, WriteManyResultItem,
    ReadRequest, ReadManyRequest, ReadManyResultItem,
    StatusRequest, SceneRequest, ChannelCommandRequest,
    CallMethodRequest, GetMethodsRequest,
    BatchReadRequest, BatchReadItem, BatchReadResultItem,
};

/// OpenAPI 文档定义
#[derive(OpenApi)]
#[openapi(
    info(
        title = "设备控制系统 API",
        version = "1.0.0",
        description = "工业设备统一控制系统 REST API 文档",
        contact(name = "Device Control Team")
    ),
    paths(
        // Screen API
        crate::web::db_api::list_screens,
        crate::web::db_api::get_screen,
        crate::web::db_api::create_screen,
        crate::web::db_api::update_screen,
        crate::web::db_api::delete_screen,
        crate::web::db_api::replace_all_screens,
        crate::web::db_api::set_screen_active,
        crate::web::db_api::get_materials_by_screen_id,
        // Material API
        crate::web::db_api::list_materials,
        crate::web::db_api::get_material,
        crate::web::db_api::update_material,
        crate::web::db_api::delete_material,
        crate::web::db_api::replace_all_materials,
        crate::web::resource_api::upload_material,
        crate::web::resource_api::serve_static_resource,
        // Device API
        crate::web::device_api::get_all_status,
        crate::web::device_api::get_all_node_states,
        crate::web::device_api::get_node_state,
        crate::web::device_api::read_device,
        crate::web::device_api::read_many,
        crate::web::device_api::write_device,
        crate::web::device_api::write_many,
        crate::web::device_api::execute_scene,
        crate::web::device_api::execute_channel_command,
        crate::web::device_api::call_method,
        crate::web::device_api::get_methods,
        crate::web::device_api::batch_read,
    ),
    components(
        schemas(
            // Response wrappers
            ScreenApiResponse,
            ScreenListApiResponse,
            MaterialSingleApiResponse,
            MaterialArrayApiResponse,
            UploadMaterialApiResponse,
            // Screen
            Screen,
            CreateScreenRequest,
            UpdateScreenRequest,
            BatchReplaceScreensRequest,
            // Material
            Material,
            MaterialResponse,
            CreateMaterialRequest,
            UpdateMaterialRequest,
            BatchReplaceMaterialsRequest,
            UploadMaterialResponse,
            UploadMaterialRequest,
            // Device API
            WriteRequest,
            WriteManyRequest,
            WriteManyItem,
            WriteManyResultItem,
            ReadRequest,
            ReadManyRequest,
            ReadManyResultItem,
            StatusRequest,
            SceneRequest,
            ChannelCommandRequest,
            CallMethodRequest,
            GetMethodsRequest,
            BatchReadRequest,
            BatchReadItem,
            BatchReadResultItem,
        )
    ),
    tags(
        (name = "Screen", description = "屏幕管理 API"),
        (name = "Material", description = "素材管理 API"),
        (name = "Device", description = "设备控制 API")
    )
)]
pub struct ApiDoc;

/// 获取 Swagger UI 路由
pub fn swagger_routes() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
}
