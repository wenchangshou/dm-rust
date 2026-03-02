use axum::{extract::Path, Json};
use serde_json::Value;

use super::response::ApiResponse;
use crate::utils::error::error_codes;

const SCHEMA_SOURCES: &[(&str, &str)] = &[
    (
        "hs-power-sequencer",
        include_str!("../protocols/schemas/hs-power-sequencer.json"),
    ),
    ("mock", include_str!("../protocols/schemas/mock.json")),
    ("modbus", include_str!("../protocols/schemas/modbus.json")),
    (
        "novastar",
        include_str!("../protocols/schemas/novastar.json"),
    ),
    ("pjlink", include_str!("../protocols/schemas/pjlink.json")),
    (
        "qn-smart-plc",
        include_str!("../protocols/schemas/qn-smart-plc.json"),
    ),
    (
        "screen-njlg-plc",
        include_str!("../protocols/schemas/screen-njlg-plc.json"),
    ),
    (
        "splicer3d",
        include_str!("../protocols/schemas/splicer3d.json"),
    ),
    (
        "tpris-pdu",
        include_str!("../protocols/schemas/tpris-pdu.json"),
    ),
    ("xfusion", include_str!("../protocols/schemas/xfusion.json")),
    ("xinkeQ1", include_str!("../protocols/schemas/xinkeQ1.json")),
    ("yk-vap", include_str!("../protocols/schemas/yk-vap.json")),
];

fn normalize_key(input: &str) -> String {
    input
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn find_schema_by_name(name: &str) -> Option<(&'static str, &'static str)> {
    let normalized = normalize_key(name);

    SCHEMA_SOURCES
        .iter()
        .copied()
        .find(|(schema_name, _)| normalize_key(schema_name) == normalized)
        .or_else(|| {
            match normalized.as_str() {
                // xFusion statute maps to xfusion schema
                "xfusion" => SCHEMA_SOURCES
                    .iter()
                    .copied()
                    .find(|(schema_name, _)| *schema_name == "xfusion"),
                _ => None,
            }
        })
}

pub async fn list_protocol_schemas() -> Json<ApiResponse<Vec<String>>> {
    let mut names = SCHEMA_SOURCES
        .iter()
        .map(|(name, _)| (*name).to_string())
        .collect::<Vec<_>>();
    names.sort();

    Json(ApiResponse::success("成功", names))
}

pub async fn get_protocol_schema(Path(name): Path<String>) -> Json<ApiResponse<Value>> {
    let Some((schema_name, raw_schema)) = find_schema_by_name(&name) else {
        return Json(ApiResponse {
            state: error_codes::INVALID_PARAMS,
            message: format!("schema 不存在: {}", name),
            data: None,
        });
    };

    match serde_json::from_str::<Value>(raw_schema) {
        Ok(schema) => Json(ApiResponse::success(
            "成功",
            serde_json::json!({
                "name": schema_name,
                "schema": schema
            }),
        )),
        Err(error) => Json(ApiResponse {
            state: error_codes::GENERAL_ERROR,
            message: format!("schema 解析失败: {}", error),
            data: None,
        }),
    }
}
