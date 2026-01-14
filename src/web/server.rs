//! Web 服务器主模块
//!
//! 负责路由配置和服务器启动

use axum::{
    extract::Extension,
    response::Html,
    routing::{delete, get, post, put},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::config::{Config, ResourceConfig};
use crate::db::Database;
use crate::device::DeviceController;

// 导入子模块
use super::db_api::{
    create_screen, delete_material, delete_screen, get_material, get_materials_by_screen_id,
    get_screen, list_materials, list_screens, replace_all_materials, replace_all_screens,
    set_screen_active, update_material, update_screen,
};
use super::device_api::{
    batch_read, call_method, execute_channel_command, execute_scene, get_all_node_states,
    get_all_settings, get_all_status, get_methods, get_node_state, get_scene_status, read_device,
    read_many, write_device, write_many,
};
use super::file_api::{
    file_delete, file_download, file_info, file_list, file_mkdir, file_preview, file_rename,
    file_upload, file_view, FileManagerState,
};
use super::file_page::{CONFIG_MANAGER_HTML, DEBUG_CONSOLE_HTML, FILE_MANAGER_HTML};
use super::resource_api::{serve_static_resource, upload_material, ResourceManagerState};
#[cfg(feature = "swagger")]
use super::swagger::swagger_routes;

/// API 路由前缀
const API_PREFIX: &str = "/lspcapi";

/// Web 服务器
#[derive(Clone)]
pub struct WebServer {
    config: Config,
    config_path: String,
    controller: DeviceController,
    database: Option<Arc<Database>>,
    resource_config: Option<ResourceConfig>,
}

impl WebServer {
    /// 创建新的 Web 服务器实例
    pub fn new(config: Config, config_path: String, controller: DeviceController) -> Self {
        let resource_config = config.resource.clone();
        Self {
            config,
            config_path,
            controller,
            database: None,
            resource_config,
        }
    }

    /// 创建带数据库的 Web 服务器实例
    pub fn with_database(
        config: Config,
        config_path: String,
        controller: DeviceController,
        database: Database,
    ) -> Self {
        let resource_config = config.resource.clone();
        Self {
            config,
            config_path,
            controller,
            database: Some(Arc::new(database)),
            resource_config,
        }
    }

    /// 运行 Web 服务器
    pub async fn run(self) -> anyhow::Result<()> {
        let controller = Arc::new(self.controller);
        let file_config = self.config.file.clone();
        let file_manager_state = FileManagerState {
            config: file_config.clone(),
        };
        let db_ref = self.database.clone();

        // 设备控制路由
        let config_clone = self.config.clone();
        let mut device_routes = Router::new()
            .route("/getAllStatus", post(get_all_status))
            .route("/getAllNodeStates", post(get_all_node_states))
            .route("/getNodeState", post(get_node_state))
            .route("/write", post(write_device))
            .route("/writeMany", post(write_many))
            .route("/read", post(read_device))
            .route("/readMany", post(read_many))
            .route("/scene", post(execute_scene))
            .route("/sceneStatus", get(get_scene_status))
            .route("/executeCommand", post(execute_channel_command))
            .route("/callMethod", post(call_method))
            .route("/getMethods", post(get_methods))
            .route("/batchRead", post(batch_read))
            .route("/config", get(move || get_config(config_clone.clone())))
            .layer(Extension(controller));

        // 如果有数据库，添加需要数据库的路由
        if let Some(ref db) = db_ref {
            device_routes = device_routes
                .route("/getAll", get(get_all_settings))
                .layer(Extension(db.clone()));
        }

        // 基础应用路由
        let config_path = self.config_path.clone();
        let _config_for_save = self.config.clone();
        let mut app = Router::new()
            .route("/", get(hello))
            .route(&format!("{}/debug", API_PREFIX), get(debug_console_page))
            .route(
                &format!("{}/config-manager", API_PREFIX),
                get(config_manager_page),
            )
            .route(
                &format!("{}/config/save", API_PREFIX),
                post(move |body| save_config(body, config_path.clone())),
            )
            .nest(&format!("{}/device", API_PREFIX), device_routes)
            .layer(CorsLayer::permissive());

        // 文件管理路由（可选）
        if let Some(ref fc) = file_config {
            if fc.enable {
                tracing::info!("文件管理功能已启用, 根路径: {}", fc.path);
                let file_routes = Router::new()
                    .route("/list", get(file_list))
                    .route("/upload", post(file_upload))
                    .route("/download", get(file_download))
                    .route("/delete", post(file_delete))
                    .route("/mkdir", post(file_mkdir))
                    .route("/rename", post(file_rename))
                    .route("/info", get(file_info))
                    .route("/preview", get(file_preview))
                    .route("/view", get(file_view))
                    .layer(Extension(file_manager_state.clone()));

                app = app
                    .nest(&format!("{}/file", API_PREFIX), file_routes)
                    .route(&format!("{}/files", API_PREFIX), get(file_manager_page))
                    .layer(Extension(file_manager_state));
            }
        }

        // Swagger UI（仅在 swagger feature 启用时）
        #[cfg(feature = "swagger")]
        {
            app = app.merge(swagger_routes());
            tracing::info!("Swagger UI 已启用: /swagger-ui/ (开发环境)");
        }

        // 数据库 API 路由（可选）
        if let Some(ref db) = self.database {
            tracing::info!("数据库 API 已启用");

            // Material API 路由（基础路由，不含上传）
            let material_base_routes = Router::new()
                .route("/replace", post(replace_all_materials))
                .route("/:id", put(update_material))
                .route("/:id", delete(delete_material))
                .layer(Extension(db.clone()));

            // 素材管理 API（需要数据库和资源配置支持）
            if let Some(ref rc) = self.resource_config {
                if rc.enable {
                    tracing::info!(
                        "素材管理功能已启用, 根路径: {}, URL前缀: {}",
                        rc.path,
                        rc.url_prefix
                    );
                    let resource_state = ResourceManagerState { config: rc.clone() };

                    // Screen API 路由（带 ResourceManagerState 以支持素材路径）
                    let screen_routes = Router::new()
                        .route("/", get(list_screens))
                        .route("/", post(create_screen))
                        .route("/replace", post(replace_all_screens))
                        .route("/:id", get(get_screen))
                        .route("/:id", put(update_screen))
                        .route("/:id", delete(delete_screen))
                        .route("/:id/active", put(set_screen_active))
                        .route("/:id/materials", get(get_materials_by_screen_id))
                        .layer(Extension(resource_state.clone()))
                        .layer(Extension(db.clone()));

                    // Material 路由（需要 ResourceManagerState 来拼接完整路径）
                    let material_with_state_routes: Router = Router::new()
                        .route("/", get(list_materials))
                        .route("/", post(upload_material))
                        .route("/:id", get(get_material))
                        .layer(Extension(resource_state.clone()))
                        .layer(Extension(db.clone()));

                    app = app
                        .nest(&format!("{}/screens", API_PREFIX), screen_routes)
                        .nest(
                            &format!("{}/materials", API_PREFIX),
                            material_base_routes.merge(material_with_state_routes),
                        )
                        // 静态资源访问（支持子路径）
                        .route("/static/*path", get(serve_static_resource))
                        .layer(Extension(resource_state))
                        .layer(Extension(db.clone()));
                } else {
                    // 没有资源配置时，screen 和 material 路由不带完整路径
                    let screen_routes = Router::new()
                        .route("/", get(list_screens))
                        .route("/", post(create_screen))
                        .route("/replace", post(replace_all_screens))
                        .route("/:id", get(get_screen))
                        .route("/:id", put(update_screen))
                        .route("/:id", delete(delete_screen))
                        .route("/:id/active", put(set_screen_active))
                        .route("/:id/materials", get(get_materials_by_screen_id))
                        .layer(Extension(db.clone()));

                    let material_routes = material_base_routes
                        .route("/", get(list_materials))
                        .route("/:id", get(get_material));
                    app = app
                        .nest(&format!("{}/screens", API_PREFIX), screen_routes)
                        .nest(&format!("{}/materials", API_PREFIX), material_routes);
                }
            } else {
                // 没有资源配置时，screen 和 material 路由不带完整路径
                let screen_routes = Router::new()
                    .route("/", get(list_screens))
                    .route("/", post(create_screen))
                    .route("/replace", post(replace_all_screens))
                    .route("/:id", get(get_screen))
                    .route("/:id", put(update_screen))
                    .route("/:id", delete(delete_screen))
                    .route("/:id/active", put(set_screen_active))
                    .route("/:id/materials", get(get_materials_by_screen_id))
                    .layer(Extension(db.clone()));

                let material_routes = material_base_routes
                    .route("/", get(list_materials))
                    .route("/:id", get(get_material));
                app = app
                    .nest(&format!("{}/screens", API_PREFIX), screen_routes)
                    .nest(&format!("{}/materials", API_PREFIX), material_routes);
            }
        }

        let addr: SocketAddr = format!("0.0.0.0:{}", self.config.web_server.port).parse()?;
        tracing::info!("HTTP 控制服务器监听于 {}", addr);
        tracing::info!("API 前缀: {}", API_PREFIX);

        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}

/// 根路由处理
async fn hello() -> &'static str {
    "Device Control System (Rust Version)"
}

/// 文件管理器页面
async fn file_manager_page() -> Html<&'static str> {
    Html(FILE_MANAGER_HTML)
}

/// 调试控制台页面
async fn debug_console_page() -> Html<&'static str> {
    Html(DEBUG_CONSOLE_HTML)
}

/// 获取配置信息（用于调试控制台）
async fn get_config(config: Config) -> axum::Json<serde_json::Value> {
    tracing::info!("[调试] 获取配置信息请求");

    let response = serde_json::json!({
        "state": 0,
        "message": "成功",
        "data": {
            "nodes": config.nodes,
            "scenes": config.scenes,
            "channels": config.channels.iter().map(|c| {
                serde_json::json!({
                    "channel_id": c.channel_id,
                    "enable": c.enable,
                    "statute": c.statute
                })
            }).collect::<Vec<_>>()
        }
    });

    tracing::info!(
        "[调试] 返回配置: {} 个节点, {} 个场景, {} 个通道",
        config.nodes.len(),
        config.scenes.len(),
        config.channels.len()
    );

    axum::Json(response)
}

/// 配置管理页面
async fn config_manager_page() -> Html<&'static str> {
    Html(CONFIG_MANAGER_HTML)
}

/// 保存配置到文件
async fn save_config(
    axum::Json(payload): axum::Json<serde_json::Value>,
    config_path: String,
) -> axum::Json<serde_json::Value> {
    tracing::info!("[配置] 保存配置请求");

    // 将配置写入文件
    match serde_json::to_string_pretty(&payload) {
        Ok(json_str) => match std::fs::write(&config_path, json_str) {
            Ok(_) => {
                tracing::info!("[配置] 配置已保存到: {}", config_path);
                axum::Json(serde_json::json!({
                    "state": 0,
                    "message": format!("配置已保存到 {}", config_path)
                }))
            }
            Err(e) => {
                tracing::error!("[配置] 保存失败: {}", e);
                axum::Json(serde_json::json!({
                    "state": 1,
                    "message": format!("保存失败: {}", e)
                }))
            }
        },
        Err(e) => {
            tracing::error!("[配置] 序列化失败: {}", e);
            axum::Json(serde_json::json!({
                "state": 1,
                "message": format!("序列化失败: {}", e)
            }))
        }
    }
}
