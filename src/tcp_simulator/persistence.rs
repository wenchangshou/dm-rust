/// 模拟器持久化存储
///
/// 将模拟器配置和状态保存到 JSON 文件，支持启动时自动加载。
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info};

use super::state::TcpSimulatorConfig;

/// 持久化的模拟器数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSimulator {
    /// 模拟器配置
    pub config: TcpSimulatorConfig,
    /// 协议特定的值（如 Modbus slaves）
    #[serde(default)]
    pub values: HashMap<String, Value>,
    /// 是否自动启动
    #[serde(default = "default_auto_start")]
    pub auto_start: bool,
}

fn default_auto_start() -> bool {
    true
}

/// 持久化存储的数据结构
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistedData {
    /// 版本号（用于未来兼容性）
    #[serde(default = "default_version")]
    pub version: u32,
    /// 模拟器列表
    #[serde(default)]
    pub simulators: Vec<PersistedSimulator>,
}

fn default_version() -> u32 {
    1
}

/// 持久化管理器
pub struct PersistenceManager {
    /// 存储文件路径
    file_path: PathBuf,
}

impl PersistenceManager {
    /// 创建持久化管理器
    pub fn new<P: AsRef<Path>>(file_path: P) -> Self {
        Self {
            file_path: file_path.as_ref().to_path_buf(),
        }
    }

    /// 使用默认路径创建
    pub fn with_default_path() -> Self {
        Self::new("simulators.json")
    }

    /// 获取文件路径
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// 加载持久化数据
    pub async fn load(&self) -> Result<PersistedData, String> {
        if !self.file_path.exists() {
            debug!("持久化文件不存在: {:?}", self.file_path);
            return Ok(PersistedData::default());
        }

        let content = fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| format!("读取持久化文件失败: {}", e))?;

        let data: PersistedData =
            serde_json::from_str(&content).map_err(|e| format!("解析持久化文件失败: {}", e))?;

        info!("已加载 {} 个模拟器配置", data.simulators.len());

        Ok(data)
    }

    /// 保存持久化数据
    pub async fn save(&self, data: &PersistedData) -> Result<(), String> {
        let content =
            serde_json::to_string_pretty(data).map_err(|e| format!("序列化失败: {}", e))?;

        // 确保目录存在
        if let Some(parent) = self.file_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }
        }

        fs::write(&self.file_path, content)
            .await
            .map_err(|e| format!("写入持久化文件失败: {}", e))?;

        debug!(
            "已保存 {} 个模拟器配置到 {:?}",
            data.simulators.len(),
            self.file_path
        );

        Ok(())
    }

    /// 添加或更新模拟器
    pub async fn upsert_simulator(&self, simulator: PersistedSimulator) -> Result<(), String> {
        let mut data = self.load().await.unwrap_or_default();

        // 查找并更新，或添加新的
        if let Some(existing) = data
            .simulators
            .iter_mut()
            .find(|s| s.config.id == simulator.config.id)
        {
            *existing = simulator;
        } else {
            data.simulators.push(simulator);
        }

        self.save(&data).await
    }

    /// 删除模拟器
    pub async fn remove_simulator(&self, id: &str) -> Result<(), String> {
        let mut data = self.load().await.unwrap_or_default();
        let len_before = data.simulators.len();
        data.simulators.retain(|s| s.config.id != id);

        if data.simulators.len() < len_before {
            self.save(&data).await?;
        }

        Ok(())
    }

    /// 更新模拟器的 values
    pub async fn update_values(
        &self,
        id: &str,
        values: HashMap<String, Value>,
    ) -> Result<(), String> {
        let mut data = self.load().await.unwrap_or_default();

        if let Some(simulator) = data.simulators.iter_mut().find(|s| s.config.id == id) {
            simulator.values = values;
            self.save(&data).await?;
        }

        Ok(())
    }
}

impl Default for PersistenceManager {
    fn default() -> Self {
        Self::with_default_path()
    }
}
