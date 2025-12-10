//! 数据库模块

pub mod models;
pub mod screen_repo;
pub mod material_repo;

use sqlx::mysql::MySqlPool;
use anyhow::Result;

pub use models::*;
pub use screen_repo::ScreenRepository;
pub use material_repo::MaterialRepository;

/// 数据库连接池
#[derive(Clone)]
pub struct Database {
    pub pool: MySqlPool,
    /// 资源文件存储路径
    resource_path: Option<String>,
}

impl Database {
    /// 创建数据库连接
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = MySqlPool::connect(database_url).await?;
        tracing::info!("数据库连接成功");
        Ok(Self { pool, resource_path: None })
    }

    /// 设置资源文件存储路径
    pub fn with_resource_path(mut self, path: String) -> Self {
        self.resource_path = Some(path);
        self
    }

    /// 获取 Screen 仓库
    pub fn screens(&self) -> ScreenRepository {
        ScreenRepository::new(self.pool.clone())
    }

    /// 获取 Material 仓库
    pub fn materials(&self) -> MaterialRepository {
        let repo = MaterialRepository::new(self.pool.clone());
        if let Some(ref path) = self.resource_path {
            repo.with_resource_path(path.clone())
        } else {
            repo
        }
    }
}
