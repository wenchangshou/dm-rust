//! 模拟器模板管理
//!
//! 提供模板的创建、保存、加载和管理功能。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

/// 模拟器模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorTemplate {
    /// 模板唯一 ID
    pub id: String,
    /// 模板名称
    pub name: String,
    /// 模板描述
    #[serde(default)]
    pub description: String,
    /// 协议类型
    pub protocol: String,
    /// 协议配置 (如 Modbus 寄存器配置)
    #[serde(default)]
    pub config: Value,
    /// 初始状态值
    #[serde(default)]
    pub values: Value,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl SimulatorTemplate {
    /// 创建新模板
    pub fn new(
        name: String,
        description: String,
        protocol: String,
        config: Value,
        values: Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            protocol,
            config,
            values,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 创建模板请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateRequest {
    /// 模板名称
    pub name: String,
    /// 模板描述
    #[serde(default)]
    pub description: String,
    /// 协议类型
    pub protocol: String,
    /// 协议配置
    #[serde(default)]
    pub config: Value,
    /// 初始状态值
    #[serde(default)]
    pub values: Value,
}

/// 从模板创建模拟器请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFromTemplateRequest {
    /// 模板 ID
    pub template_id: String,
    /// 模拟器名称
    pub name: String,
    /// 绑定地址
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,
    /// 端口
    pub port: u16,
}

fn default_bind_addr() -> String {
    "0.0.0.0".to_string()
}

/// 模板管理器
#[derive(Clone)]
pub struct TemplateManager {
    templates: Arc<RwLock<HashMap<String, SimulatorTemplate>>>,
    storage_path: PathBuf,
}

impl TemplateManager {
    /// 创建新的模板管理器
    pub fn new() -> Self {
        Self {
            templates: Arc::new(RwLock::new(HashMap::new())),
            storage_path: PathBuf::from("templates.json"),
        }
    }

    /// 从文件加载模板
    pub async fn load_from_file(&self) -> Result<usize, String> {
        if !self.storage_path.exists() {
            return Ok(0);
        }

        let content = fs::read_to_string(&self.storage_path)
            .map_err(|e| format!("读取模板文件失败: {}", e))?;

        let templates: Vec<SimulatorTemplate> =
            serde_json::from_str(&content).map_err(|e| format!("解析模板文件失败: {}", e))?;

        let count = templates.len();
        let mut store = self.templates.write().await;
        for template in templates {
            store.insert(template.id.clone(), template);
        }

        info!("已加载 {} 个模板", count);
        Ok(count)
    }

    /// 保存模板到文件
    pub async fn save_to_file(&self) -> Result<(), String> {
        let store = self.templates.read().await;
        let templates: Vec<&SimulatorTemplate> = store.values().collect();

        let content = serde_json::to_string_pretty(&templates)
            .map_err(|e| format!("序列化模板失败: {}", e))?;

        fs::write(&self.storage_path, content).map_err(|e| format!("写入模板文件失败: {}", e))?;

        Ok(())
    }

    /// 获取所有模板
    pub async fn list(&self) -> Vec<SimulatorTemplate> {
        let store = self.templates.read().await;
        let mut templates: Vec<SimulatorTemplate> = store.values().cloned().collect();
        templates.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        templates
    }

    /// 获取单个模板
    pub async fn get(&self, id: &str) -> Option<SimulatorTemplate> {
        let store = self.templates.read().await;
        store.get(id).cloned()
    }

    /// 创建模板
    pub async fn create(&self, req: CreateTemplateRequest) -> Result<SimulatorTemplate, String> {
        let template = SimulatorTemplate::new(
            req.name,
            req.description,
            req.protocol,
            req.config,
            req.values,
        );

        let mut store = self.templates.write().await;
        store.insert(template.id.clone(), template.clone());
        drop(store);

        self.save_to_file().await?;
        info!("创建模板: {} ({})", template.name, template.id);
        Ok(template)
    }

    /// 删除模板
    pub async fn delete(&self, id: &str) -> Result<(), String> {
        let mut store = self.templates.write().await;
        if store.remove(id).is_none() {
            return Err(format!("模板 '{}' 不存在", id));
        }
        drop(store);

        self.save_to_file().await?;
        info!("删除模板: {}", id);
        Ok(())
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new()
    }
}
