//! 协议持久化存储模块
//! 提供统一的键值存储接口，数据持久化到文件系统

use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// 协议存储管理器
/// 每个通道/协议有独立的存储空间
pub struct ProtocolStorage {
    /// 存储目录
    storage_dir: PathBuf,
    /// 内存缓存 (channel_id -> (key -> value))
    cache: Arc<RwLock<HashMap<u32, HashMap<String, Value>>>>,
}

impl ProtocolStorage {
    /// 创建新的存储管理器
    pub fn new(storage_dir: PathBuf) -> Self {
        // 确保存储目录存在
        if !storage_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&storage_dir) {
                error!("创建存储目录失败: {:?}, 错误: {}", storage_dir, e);
            }
        }

        info!("协议存储初始化, 目录: {:?}", storage_dir);

        Self {
            storage_dir,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取通道的存储文件路径
    fn get_channel_file(&self, channel_id: u32) -> PathBuf {
        self.storage_dir.join(format!("channel_{}.json", channel_id))
    }

    /// 加载通道的存储数据
    pub async fn load_channel(&self, channel_id: u32) -> HashMap<String, Value> {
        let file_path = self.get_channel_file(channel_id);

        if !file_path.exists() {
            debug!("通道 {} 存储文件不存在，返回空数据", channel_id);
            return HashMap::new();
        }

        match tokio::fs::read_to_string(&file_path).await {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(data) => {
                    debug!("通道 {} 存储数据已加载", channel_id);
                    data
                }
                Err(e) => {
                    warn!("通道 {} 存储数据解析失败: {}", channel_id, e);
                    HashMap::new()
                }
            },
            Err(e) => {
                warn!("通道 {} 存储文件读取失败: {}", channel_id, e);
                HashMap::new()
            }
        }
    }

    /// 保存通道的存储数据
    async fn save_channel(&self, channel_id: u32, data: &HashMap<String, Value>) {
        let file_path = self.get_channel_file(channel_id);

        let content = match serde_json::to_string_pretty(data) {
            Ok(c) => c,
            Err(e) => {
                error!("通道 {} 存储数据序列化失败: {}", channel_id, e);
                return;
            }
        };

        if let Err(e) = tokio::fs::write(&file_path, content).await {
            error!("通道 {} 存储文件写入失败: {}", channel_id, e);
        } else {
            debug!("通道 {} 存储数据已保存", channel_id);
        }
    }

    /// 初始化通道存储（加载已有数据到缓存）
    pub async fn init_channel(&self, channel_id: u32) {
        let data = self.load_channel(channel_id).await;
        let mut cache = self.cache.write().await;
        cache.insert(channel_id, data);
        debug!("通道 {} 存储已初始化", channel_id);
    }

    /// 设置值
    pub async fn set(&self, channel_id: u32, key: &str, value: Value) {
        let mut cache = self.cache.write().await;
        let channel_data = cache.entry(channel_id).or_insert_with(HashMap::new);
        channel_data.insert(key.to_string(), value.clone());

        debug!("通道 {} 存储设置: {} = {}", channel_id, key, value);

        // 异步保存到文件
        let data = channel_data.clone();
        drop(cache);
        self.save_channel(channel_id, &data).await;
    }

    /// 获取值
    pub async fn get(&self, channel_id: u32, key: &str) -> Option<Value> {
        let cache = self.cache.read().await;
        cache
            .get(&channel_id)
            .and_then(|data| data.get(key).cloned())
    }

    /// 获取字符串值
    pub async fn get_string(&self, channel_id: u32, key: &str) -> Option<String> {
        self.get(channel_id, key)
            .await
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    /// 获取整数值
    pub async fn get_i64(&self, channel_id: u32, key: &str) -> Option<i64> {
        self.get(channel_id, key).await.and_then(|v| v.as_i64())
    }

    /// 获取布尔值
    pub async fn get_bool(&self, channel_id: u32, key: &str) -> Option<bool> {
        self.get(channel_id, key).await.and_then(|v| v.as_bool())
    }

    /// 删除值
    pub async fn remove(&self, channel_id: u32, key: &str) -> Option<Value> {
        let mut cache = self.cache.write().await;
        let removed = cache
            .get_mut(&channel_id)
            .and_then(|data| data.remove(key));

        if removed.is_some() {
            debug!("通道 {} 存储删除: {}", channel_id, key);
            let data = cache.get(&channel_id).cloned().unwrap_or_default();
            drop(cache);
            self.save_channel(channel_id, &data).await;
        }

        removed
    }

    /// 清空通道的所有存储
    pub async fn clear_channel(&self, channel_id: u32) {
        let mut cache = self.cache.write().await;
        cache.insert(channel_id, HashMap::new());
        drop(cache);

        let file_path = self.get_channel_file(channel_id);
        if file_path.exists() {
            if let Err(e) = tokio::fs::remove_file(&file_path).await {
                warn!("删除通道 {} 存储文件失败: {}", channel_id, e);
            }
        }

        info!("通道 {} 存储已清空", channel_id);
    }

    /// 获取通道的所有键
    pub async fn keys(&self, channel_id: u32) -> Vec<String> {
        let cache = self.cache.read().await;
        cache
            .get(&channel_id)
            .map(|data| data.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// 获取通道的所有数据
    pub async fn get_all(&self, channel_id: u32) -> HashMap<String, Value> {
        let cache = self.cache.read().await;
        cache.get(&channel_id).cloned().unwrap_or_default()
    }

    /// 批量设置值
    pub async fn set_many(&self, channel_id: u32, values: HashMap<String, Value>) {
        let mut cache = self.cache.write().await;
        let channel_data = cache.entry(channel_id).or_insert_with(HashMap::new);

        for (key, value) in values {
            channel_data.insert(key, value);
        }

        let data = channel_data.clone();
        drop(cache);
        self.save_channel(channel_id, &data).await;

        debug!("通道 {} 批量存储完成", channel_id);
    }
}

/// 全局存储实例
static GLOBAL_STORAGE: tokio::sync::OnceCell<Arc<ProtocolStorage>> =
    tokio::sync::OnceCell::const_new();

/// 初始化全局存储
pub async fn init_global_storage(storage_dir: PathBuf) {
    let _ = GLOBAL_STORAGE
        .set(Arc::new(ProtocolStorage::new(storage_dir)))
        .map_err(|_| warn!("全局存储已初始化"));
}

/// 获取全局存储实例
pub fn get_storage() -> Option<Arc<ProtocolStorage>> {
    GLOBAL_STORAGE.get().cloned()
}

/// 获取存储或使用默认路径初始化
pub async fn get_or_init_storage() -> Arc<ProtocolStorage> {
    GLOBAL_STORAGE
        .get_or_init(|| async {
            let storage_dir = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("data")
                .join("protocol_storage");
            Arc::new(ProtocolStorage::new(storage_dir))
        })
        .await
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_storage_basic() {
        let dir = tempdir().unwrap();
        let storage = ProtocolStorage::new(dir.path().to_path_buf());

        // 设置值
        storage
            .set(1, "test_key", serde_json::json!("test_value"))
            .await;
        storage.set(1, "count", serde_json::json!(42)).await;

        // 读取值
        assert_eq!(
            storage.get_string(1, "test_key").await,
            Some("test_value".to_string())
        );
        assert_eq!(storage.get_i64(1, "count").await, Some(42));

        // 删除值
        storage.remove(1, "test_key").await;
        assert_eq!(storage.get_string(1, "test_key").await, None);
    }
}
