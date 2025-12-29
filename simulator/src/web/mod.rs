//! Web API 模块

pub mod mqtt_api;
pub mod response;
pub mod tcp_api;

pub use response::ApiResponse;

use axum::{
    routing::{delete, get, post},
    Extension, Router,
};
use std::sync::Arc;

use crate::mqtt::MqttSimulatorManager;
use crate::tcp::TcpSimulatorManager;

/// TCP 模拟器路由
pub fn tcp_routes(manager: Arc<TcpSimulatorManager>) -> Router {
    Router::new()
        .route("/protocols", get(tcp_api::get_protocols))
        .route("/create", post(tcp_api::create_simulator))
        .route("/list", get(tcp_api::list_simulators))
        .route(
            "/:id",
            get(tcp_api::get_simulator).delete(tcp_api::delete_simulator),
        )
        .route("/:id/start", post(tcp_api::start_simulator))
        .route("/:id/stop", post(tcp_api::stop_simulator))
        .route("/:id/state", post(tcp_api::update_simulator_state))
        .route("/:id/fault", post(tcp_api::trigger_fault))
        .route("/:id/clear-fault", post(tcp_api::clear_fault))
        .route("/:id/online", post(tcp_api::set_online))
        // Modbus
        .route("/:id/modbus/slaves", get(tcp_api::get_modbus_slaves))
        .route("/:id/modbus/slave", post(tcp_api::add_modbus_slave))
        .route(
            "/:id/modbus/slave/:slaveId",
            delete(tcp_api::delete_modbus_slave),
        )
        .route("/:id/modbus/register", post(tcp_api::set_modbus_register))
        .route(
            "/:id/modbus/register/delete",
            post(tcp_api::delete_modbus_register),
        )
        .route(
            "/:id/modbus/register/value",
            post(tcp_api::update_modbus_register_value),
        )
        .route(
            "/:id/modbus/registers/batch",
            post(tcp_api::batch_update_modbus_registers),
        )
        // 报文
        .route(
            "/:id/packets",
            get(tcp_api::get_packets).delete(tcp_api::clear_packets),
        )
        .route(
            "/:id/packets/settings",
            post(tcp_api::set_packet_monitor_settings),
        )
        // 模板管理
        .route(
            "/templates",
            get(tcp_api::list_templates).post(tcp_api::create_template_direct),
        )
        .route(
            "/templates/:id",
            delete(tcp_api::delete_template).put(tcp_api::update_template),
        )
        .route("/create-from-template", post(tcp_api::create_from_template))
        .route("/:id/save-as-template", post(tcp_api::save_as_template))
        .layer(Extension(manager))
}

/// MQTT 模拟器路由
pub fn mqtt_routes(manager: Arc<MqttSimulatorManager>) -> Router {
    Router::new()
        .route("/create", post(mqtt_api::create_simulator))
        .route("/list", get(mqtt_api::list_simulators))
        .route(
            "/:id",
            get(mqtt_api::get_simulator).delete(mqtt_api::delete_simulator),
        )
        .route("/:id/start", post(mqtt_api::start_simulator))
        .route("/:id/stop", post(mqtt_api::stop_simulator))
        .route(
            "/:id/packets",
            get(mqtt_api::get_packets).delete(mqtt_api::clear_packets),
        )
        .route(
            "/:id/rules",
            get(mqtt_api::list_rules).post(mqtt_api::add_rule),
        )
        .route("/:id/rules/:rule_id", delete(mqtt_api::remove_rule))
        .route("/export", get(mqtt_api::export_config))
        .route("/import", post(mqtt_api::import_config))
        .layer(Extension(manager))
}
