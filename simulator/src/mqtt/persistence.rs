/// MQTT 模拟器配置持久化
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs;
use tracing::{error, info, warn};

use super::state::MqttSimulatorInfo;

/// 持久化配置文件
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MqttSimulatorPersistence {
    pub mqtt_simulators: Vec<MqttSimulatorInfo>,
}

impl MqttSimulatorPersistence {
    /// 从文件加载配置
    pub async fn load(file_path: &PathBuf) -> Result<Self, String> {
        info!("Loading MQTT simulator config from: {:?}", file_path);

        if !file_path.exists() {
            info!("Config file not found, starting with empty config");
            return Ok(Self::default());
        }

        match fs::read_to_string(file_path).await {
            Ok(content) => {
                // 先解析为通用Value
                let mut json: Value = match serde_json::from_str(&content) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Failed to parse config file as JSON: {}", e);
                        return Ok(Self::default());
                    }
                };

                // 提取mqtt_simulators字段
                let mqtt_simulators = if let Some(obj) = json.as_object() {
                    if let Some(mqtt_sims) = obj.get("mqtt_simulators") {
                        match serde_json::from_value(mqtt_sims.clone()) {
                            Ok(sims) => sims,
                            Err(e) => {
                                warn!("Failed to parse mqtt_simulators: {}", e);
                                Vec::new()
                            }
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };

                info!("Successfully loaded {} MQTT simulators", mqtt_simulators.len());
                Ok(Self { mqtt_simulators })
            }
            Err(e) => {
                error!("Failed to read config file: {}", e);
                Err(format!("Failed to read config: {}", e))
            }
        }
    }

    /// 保存配置到文件
    pub async fn save(&self, file_path: &PathBuf) -> Result<(), String> {
        info!("Saving MQTT simulator config to: {:?}", file_path);

        // 确保目录存在
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent).await {
                    error!("Failed to create config directory: {}", e);
                    return Err(format!("Failed to create directory: {}", e));
                }
            }
        }

        // 读取现有配置
        let mut json = if file_path.exists() {
            match fs::read_to_string(file_path).await {
                Ok(content) => match serde_json::from_str::<Value>(&content) {
                    Ok(v) => v,
                    Err(_) => serde_json::json!({}),
                },
                Err(_) => serde_json::json!({}),
            }
        } else {
            serde_json::json!({})
        };

        // 更新mqtt_simulators字段
        if let Some(obj) = json.as_object_mut() {
            obj.insert(
                "mqtt_simulators".to_string(),
                serde_json::to_value(&self.mqtt_simulators).map_err(|e| {
                    error!("Failed to serialize mqtt_simulators: {}", e);
                    format!("Failed to serialize: {}", e)
                })?,
            );
        }

        // 序列化为 JSON
        let content = match serde_json::to_string_pretty(&json) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to serialize config: {}", e);
                return Err(format!("Failed to serialize: {}", e));
            }
        };

        // 写入文件
        match fs::write(file_path, content).await {
            Ok(_) => {
                info!("Successfully saved MQTT simulator config");
                Ok(())
            }
            Err(e) => {
                error!("Failed to write config file: {}", e);
                Err(format!("Failed to write config: {}", e))
            }
        }
    }
}
