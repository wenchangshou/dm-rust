//! Material 数据仓库（合并了素材和资源信息）

use anyhow::Result;
use chrono::Utc;
use sqlx::MySqlPool;
use std::path::Path;

use super::models::{CreateMaterialRequest, Material, UpdateMaterialRequest};

/// Material 仓库
#[derive(Clone)]
pub struct MaterialRepository {
    pool: MySqlPool,
    /// 资源文件存储根目录
    resource_path: Option<String>,
}

impl MaterialRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool, resource_path: None }
    }

    /// 设置资源文件存储路径
    pub fn with_resource_path(mut self, path: String) -> Self {
        self.resource_path = Some(path);
        self
    }

    /// 获取所有 Material
    pub async fn find_all(&self) -> Result<Vec<Material>> {
        let materials = sqlx::query_as::<_, Material>(
            r#"SELECT id, name, screen_id, preset, path, resource_type, size, mime_type, original_name, created_at
               FROM lspc_material
               ORDER BY created_at DESC"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(materials)
    }

    /// 根据 ID 获取 Material
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Material>> {
        let material = sqlx::query_as::<_, Material>(
            r#"SELECT id, name, screen_id, preset, path, resource_type, size, mime_type, original_name, created_at
               FROM lspc_material
               WHERE id = ?"#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(material)
    }

    /// 根据名称搜索 Material
    pub async fn find_by_name(&self, name: &str) -> Result<Vec<Material>> {
        let materials = sqlx::query_as::<_, Material>(
            r#"SELECT id, name, screen_id, preset, path, resource_type, size, mime_type, original_name, created_at
               FROM lspc_material
               WHERE name LIKE ?
               ORDER BY created_at DESC"#
        )
        .bind(format!("%{}%", name))
        .fetch_all(&self.pool)
        .await?;
        Ok(materials)
    }

    /// 根据 screen_id 获取所有 Material
    pub async fn find_by_screen_id(&self, screen_id: &str) -> Result<Vec<Material>> {
        let materials = sqlx::query_as::<_, Material>(
            r#"SELECT id, name, screen_id, preset, path, resource_type, size, mime_type, original_name, created_at
               FROM lspc_material
               WHERE screen_id = ?
               ORDER BY created_at DESC"#
        )
        .bind(screen_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(materials)
    }

    /// 根据 preset 过滤获取 Material
    pub async fn find_by_preset(&self, preset: bool) -> Result<Vec<Material>> {
        let materials = sqlx::query_as::<_, Material>(
            r#"SELECT id, name, screen_id, preset, path, resource_type, size, mime_type, original_name, created_at
               FROM lspc_material
               WHERE preset = ?
               ORDER BY created_at DESC"#
        )
        .bind(preset)
        .fetch_all(&self.pool)
        .await?;
        Ok(materials)
    }

    /// 创建 Material
    pub async fn create(&self, req: &CreateMaterialRequest) -> Result<Material> {
        let now = Utc::now();
        let id = if req.id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            req.id.clone()
        };

        sqlx::query(
            r#"INSERT INTO lspc_material (id, name, screen_id, preset, path, resource_type, size, mime_type, original_name, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&id)
        .bind(&req.name)
        .bind(&req.screen_id)
        .bind(&req.preset)
        .bind(&req.path)
        .bind(&req.resource_type)
        .bind(&req.size)
        .bind(&req.mime_type)
        .bind(&req.original_name)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(Material {
            id,
            name: req.name.clone(),
            screen_id: req.screen_id.clone(),
            preset: req.preset,
            path: req.path.clone(),
            resource_type: req.resource_type.clone(),
            size: req.size,
            mime_type: req.mime_type.clone(),
            original_name: req.original_name.clone(),
            created_at: now,
        })
    }

    /// 更新 Material
    pub async fn update(&self, id: &str, req: &UpdateMaterialRequest) -> Result<Option<Material>> {
        // 先检查是否存在
        let existing = self.find_by_id(id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let name = req.name.as_ref().unwrap_or(&existing.name);
        let screen_id = req.screen_id.as_ref().unwrap_or(&existing.screen_id);
        let preset = req.preset.unwrap_or(existing.preset);

        sqlx::query(
            "UPDATE lspc_material SET name = ?, screen_id = ?, preset = ? WHERE id = ?"
        )
        .bind(name)
        .bind(screen_id)
        .bind(preset)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(Some(Material {
            id: id.to_string(),
            name: name.clone(),
            screen_id: screen_id.clone(),
            preset,
            path: existing.path,
            resource_type: existing.resource_type,
            size: existing.size,
            mime_type: existing.mime_type,
            original_name: existing.original_name,
            created_at: existing.created_at,
        }))
    }

    /// 删除 Material（同时删除物理文件）
    pub async fn delete(&self, id: &str) -> Result<bool> {
        // 先获取 material 信息
        let material = self.find_by_id(id).await?;
        if material.is_none() {
            return Ok(false);
        }
        let material = material.unwrap();

        // 删除数据库记录
        sqlx::query("DELETE FROM lspc_material WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        // 删除物理文件
        if !material.path.is_empty() {
            self.delete_physical_file(&material.path).await;
        }

        Ok(true)
    }

    /// 删除物理文件
    async fn delete_physical_file(&self, relative_path: &str) {
        if let Some(ref base_path) = self.resource_path {
            let full_path = Path::new(base_path).join(relative_path);
            if full_path.exists() {
                if let Err(e) = tokio::fs::remove_file(&full_path).await {
                    tracing::warn!("删除文件失败: {:?}, 错误: {}", full_path, e);
                } else {
                    tracing::info!("已删除文件: {:?}", full_path);
                }
            }
        }
    }

    /// 删除所有 Material（同时删除物理文件）
    pub async fn delete_all(&self) -> Result<u64> {
        // 获取所有 material 的 path
        let paths: Vec<(String,)> = sqlx::query_as(
            "SELECT path FROM lspc_material WHERE path != ''"
        )
        .fetch_all(&self.pool)
        .await?;

        // 删除数据库记录
        let result = sqlx::query("DELETE FROM lspc_material")
            .execute(&self.pool)
            .await?;
        let affected = result.rows_affected();

        // 删除物理文件
        for (path,) in paths {
            self.delete_physical_file(&path).await;
        }

        Ok(affected)
    }

    /// 批量覆盖（删除所有后重新创建）
    pub async fn replace_all(&self, materials: &[CreateMaterialRequest]) -> Result<Vec<Material>> {
        // 先删除所有
        self.delete_all().await?;

        // 批量插入
        let mut results = Vec::new();
        for req in materials {
            let material = self.create(req).await?;
            results.push(material);
        }

        Ok(results)
    }
}
