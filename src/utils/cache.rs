/// 全局持久化缓存模块
/// 提供按 (channel_id, key) 存储 i32 值的通用离线缓存。
/// 任何协议都可以调用此模块来缓存和读取状态，程序重启后缓存依然有效。
///
/// # 用法
/// ```rust
/// use crate::utils::cache;
///
/// // 写入缓存
/// cache::set(channel_id, node_id, value);
///
/// // 读取缓存
/// let val = cache::get(channel_id, node_id);          // Option<i32>
/// let val = cache::get_or(channel_id, node_id, 0);    // i32
/// ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{debug, info, warn};

/// 缓存条目的键：(channel_id, key)
type CacheKey = (u32, u32);

/// 用于 JSON 序列化的条目结构
#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    channel_id: u32,
    key: u32,
    value: i32,
}

/// 全局缓存文件内容
#[derive(Debug, Serialize, Deserialize)]
struct CacheFile {
    entries: Vec<CacheEntry>,
}

/// 全局缓存（单例）
static CACHE: once_cell::sync::Lazy<PersistentCache> =
    once_cell::sync::Lazy::new(|| PersistentCache::new());

struct PersistentCache {
    data: Mutex<HashMap<CacheKey, i32>>,
    file_path: PathBuf,
}

impl PersistentCache {
    fn new() -> Self {
        let file_path = Self::default_cache_path();
        let data = Self::load_from_disk(&file_path);
        info!(
            "全局持久化缓存已初始化，共 {} 条记录，路径: {:?}",
            data.len(),
            file_path
        );
        Self {
            data: Mutex::new(data),
            file_path,
        }
    }

    /// 缓存文件路径：与可执行文件同目录下的 device_cache.json
    fn default_cache_path() -> PathBuf {
        let mut path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        path.push("device_cache.json");
        path
    }

    /// 从磁盘加载缓存
    fn load_from_disk(path: &PathBuf) -> HashMap<CacheKey, i32> {
        match std::fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<CacheFile>(&content) {
                Ok(cache_file) => {
                    let map: HashMap<CacheKey, i32> = cache_file
                        .entries
                        .into_iter()
                        .map(|e| ((e.channel_id, e.key), e.value))
                        .collect();
                    debug!("从磁盘加载缓存成功，共 {} 条记录", map.len());
                    map
                }
                Err(e) => {
                    warn!("解析缓存文件失败: {}，将使用空缓存", e);
                    HashMap::new()
                }
            },
            Err(_) => {
                debug!("缓存文件不存在，将使用空缓存: {:?}", path);
                HashMap::new()
            }
        }
    }

    /// 将缓存保存到磁盘
    fn save_to_disk(&self) {
        let data = self.data.lock().unwrap();
        let cache_file = CacheFile {
            entries: data
                .iter()
                .map(|((channel_id, key), value)| CacheEntry {
                    channel_id: *channel_id,
                    key: *key,
                    value: *value,
                })
                .collect(),
        };
        drop(data); // 尽早释放锁

        match serde_json::to_string_pretty(&cache_file) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&self.file_path, json) {
                    warn!("保存缓存到磁盘失败: {}", e);
                } else {
                    debug!("缓存已保存到磁盘: {:?}", self.file_path);
                }
            }
            Err(e) => {
                warn!("序列化缓存失败: {}", e);
            }
        }
    }
}

// ============= 公开 API =============

/// 设置缓存值（同时持久化到磁盘）
///
/// # 参数
/// - `channel_id`: 通道 ID
/// - `key`: 键（如 node_id、寄存器地址等）
/// - `value`: 值
pub fn set(channel_id: u32, key: u32, value: i32) {
    let cache = &*CACHE;
    cache.data.lock().unwrap().insert((channel_id, key), value);
    cache.save_to_disk();
    debug!(
        "缓存已更新: channel={}, key={}, value={}",
        channel_id, key, value
    );
}

/// 获取缓存值
///
/// # 参数
/// - `channel_id`: 通道 ID
/// - `key`: 键
///
/// # 返回
/// 缓存的值，若无缓存则返回 None
pub fn get(channel_id: u32, key: u32) -> Option<i32> {
    CACHE
        .data
        .lock()
        .unwrap()
        .get(&(channel_id, key))
        .copied()
}

/// 获取缓存值，若无缓存则返回默认值
pub fn get_or(channel_id: u32, key: u32, default: i32) -> i32 {
    get(channel_id, key).unwrap_or(default)
}

/// 删除某条缓存（同时持久化到磁盘）
pub fn remove(channel_id: u32, key: u32) -> Option<i32> {
    let cache = &*CACHE;
    let old = cache.data.lock().unwrap().remove(&(channel_id, key));
    if old.is_some() {
        cache.save_to_disk();
    }
    old
}

/// 清除某个 channel 的全部缓存（同时持久化到磁盘）
pub fn clear_channel(channel_id: u32) {
    let cache = &*CACHE;
    let mut data = cache.data.lock().unwrap();
    let before = data.len();
    data.retain(|(ch, _), _| *ch != channel_id);
    let removed = before - data.len();
    drop(data);
    if removed > 0 {
        cache.save_to_disk();
        debug!("已清除 channel {} 的 {} 条缓存", channel_id, removed);
    }
}
