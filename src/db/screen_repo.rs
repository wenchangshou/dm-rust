//! Screen 数据仓库

use anyhow::Result;
use chrono::Utc;
use sqlx::MySqlPool;

use super::models::{CreateScreenRequest, Screen, UpdateScreenRequest};

/// Screen 仓库
#[derive(Clone)]
pub struct ScreenRepository {
    pool: MySqlPool,
}

impl ScreenRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 获取所有 Screen
    pub async fn find_all(&self) -> Result<Vec<Screen>> {
        let screens = sqlx::query_as::<_, Screen>(
            "SELECT id, type as screen_type, name, content, active, created_at, updated_at FROM lspc_screen ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(screens)
    }

    /// 根据 ID 获取 Screen
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Screen>> {
        let screen = sqlx::query_as::<_, Screen>(
            "SELECT id, type as screen_type, name, content, active, created_at, updated_at FROM lspc_screen WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(screen)
    }

    /// 根据类型获取 Screen
    pub async fn find_by_type(&self, screen_type: &str) -> Result<Vec<Screen>> {
        let screens = sqlx::query_as::<_, Screen>(
            "SELECT id, type as screen_type, name, content, active, created_at, updated_at FROM lspc_screen WHERE type = ? ORDER BY created_at DESC"
        )
        .bind(screen_type)
        .fetch_all(&self.pool)
        .await?;
        Ok(screens)
    }

    /// 获取激活的 Screen
    pub async fn find_active(&self) -> Result<Vec<Screen>> {
        let screens = sqlx::query_as::<_, Screen>(
            "SELECT id, type as screen_type, name, content, active, created_at, updated_at FROM lspc_screen WHERE active = 1 ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(screens)
    }

    /// 根据类型和激活状态获取 Screen
    pub async fn find_by_type_and_active(&self, screen_type: &str, active: bool) -> Result<Vec<Screen>> {
        let screens = sqlx::query_as::<_, Screen>(
            "SELECT id, type as screen_type, name, content, active, created_at, updated_at FROM lspc_screen WHERE type = ? AND active = ? ORDER BY created_at DESC"
        )
        .bind(screen_type)
        .bind(active)
        .fetch_all(&self.pool)
        .await?;
        Ok(screens)
    }

    /// 创建 Screen
    pub async fn create(&self, req: &CreateScreenRequest) -> Result<Screen> {
        let now = Utc::now();
        let id = if req.id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            req.id.clone()
        };

        sqlx::query(
            "INSERT INTO lspc_screen (id, type, name, content, active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&req.screen_type)
        .bind(&req.name)
        .bind(&req.content)
        .bind(req.active)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(Screen {
            id,
            screen_type: req.screen_type.clone(),
            name: req.name.clone(),
            content: req.content.clone(),
            active: req.active,
            created_at: now,
            updated_at: now,
        })
    }

    /// 更新 Screen
    pub async fn update(&self, id: &str, req: &UpdateScreenRequest) -> Result<Option<Screen>> {
        // 先检查是否存在
        let existing = self.find_by_id(id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let now = Utc::now();
        let screen_type = req.screen_type.as_ref().unwrap_or(&existing.screen_type);
        let name = req.name.as_ref().unwrap_or(&existing.name);
        let content = req.content.as_ref().unwrap_or(&existing.content);
        let active = req.active.unwrap_or(existing.active);

        sqlx::query(
            "UPDATE lspc_screen SET type = ?, name = ?, content = ?, active = ?, updated_at = ? WHERE id = ?"
        )
        .bind(screen_type)
        .bind(name)
        .bind(content)
        .bind(active)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(Some(Screen {
            id: id.to_string(),
            screen_type: screen_type.clone(),
            name: name.clone(),
            content: content.clone(),
            active,
            created_at: existing.created_at,
            updated_at: now,
        }))
    }

    /// 设置指定 Screen 为激活状态，同类型的其他 Screen 设置为非激活
    pub async fn set_active(&self, id: &str) -> Result<Option<Screen>> {
        // 先检查是否存在
        let existing = self.find_by_id(id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        let now = Utc::now();

        // 开启事务
        let mut tx = self.pool.begin().await?;

        // 将同类型的 Screen 设置为非激活
        sqlx::query("UPDATE lspc_screen SET active = 0, updated_at = ? WHERE type = ?")
            .bind(now)
            .bind(&existing.screen_type)
            .execute(&mut *tx)
            .await?;

        // 将指定 Screen 设置为激活
        sqlx::query("UPDATE lspc_screen SET active = 1, updated_at = ? WHERE id = ?")
            .bind(now)
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // 提交事务
        tx.commit().await?;

        // 返回更新后的 Screen
        self.find_by_id(id).await
    }

    /// 删除 Screen
    pub async fn delete(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM lspc_screen WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 删除所有 Screen
    pub async fn delete_all(&self) -> Result<u64> {
        let result = sqlx::query("DELETE FROM lspc_screen")
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// 批量创建 Screen
    pub async fn batch_create(&self, screens: &[CreateScreenRequest]) -> Result<Vec<Screen>> {
        let mut results = Vec::new();
        for req in screens {
            let screen = self.create(req).await?;
            results.push(screen);
        }
        Ok(results)
    }

    /// 批量覆盖（删除所有后重新创建）
    pub async fn replace_all(&self, screens: &[CreateScreenRequest]) -> Result<Vec<Screen>> {
        // 开启事务
        let mut tx = self.pool.begin().await?;

        // 删除所有
        sqlx::query("DELETE FROM lspc_screen")
            .execute(&mut *tx)
            .await?;

        // 批量插入
        let now = Utc::now();
        let mut results = Vec::new();

        for req in screens {
            let id = if req.id.is_empty() {
                uuid::Uuid::new_v4().to_string()
            } else {
                req.id.clone()
            };

            sqlx::query(
                "INSERT INTO lspc_screen (id, type, name, content, active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&id)
            .bind(&req.screen_type)
            .bind(&req.name)
            .bind(&req.content)
            .bind(req.active)
            .bind(now)
            .bind(now)
            .execute(&mut *tx)
            .await?;

            results.push(Screen {
                id,
                screen_type: req.screen_type.clone(),
                name: req.name.clone(),
                content: req.content.clone(),
                active: req.active,
                created_at: now,
                updated_at: now,
            });
        }

        // 提交事务
        tx.commit().await?;

        Ok(results)
    }
}
