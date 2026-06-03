use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::config::domain::{
    entity::{ConfigItem, ConfigDocumentType},
    repository::ConfigRepository,
};

pub struct PgConfigRepository {
    pool: PgPool,
}

impl PgConfigRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Validate table name against a whitelist to prevent SQL injection.
fn validated_table(table: &str) -> Result<&'static str, AppError> {
    match table {
        "expense_categories" => Ok("config_expense_categories"),
        "leave_types" => Ok("config_leave_types"),
        "document_types" => Ok("config_document_types"),
        _ => Err(AppError::BadRequest("Invalid config type".into())),
    }
}

#[async_trait]
impl ConfigRepository for PgConfigRepository {
    async fn list_expense_categories(&self, active_only: bool) -> Result<Vec<ConfigItem>, AppError> {
        let rows = if active_only {
            sqlx::query_as::<_, ConfigItem>(
                "SELECT id, name, code, is_active, sort_order, created_at \
                 FROM config_expense_categories WHERE is_active = true ORDER BY sort_order, name"
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ConfigItem>(
                "SELECT id, name, code, is_active, sort_order, created_at \
                 FROM config_expense_categories ORDER BY sort_order, name"
            )
            .fetch_all(&self.pool)
            .await?
        };
        Ok(rows)
    }

    async fn list_leave_types(&self, active_only: bool) -> Result<Vec<ConfigItem>, AppError> {
        let rows = if active_only {
            sqlx::query_as::<_, ConfigItem>(
                "SELECT id, name, code, is_active, sort_order, created_at \
                 FROM config_leave_types WHERE is_active = true ORDER BY sort_order, name"
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ConfigItem>(
                "SELECT id, name, code, is_active, sort_order, created_at \
                 FROM config_leave_types ORDER BY sort_order, name"
            )
            .fetch_all(&self.pool)
            .await?
        };
        Ok(rows)
    }

    async fn list_document_types(&self, active_only: bool) -> Result<Vec<ConfigDocumentType>, AppError> {
        let rows = if active_only {
            sqlx::query_as::<_, ConfigDocumentType>(
                "SELECT id, name, code, applies_to, is_active, sort_order, created_at \
                 FROM config_document_types WHERE is_active = true ORDER BY sort_order, name"
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ConfigDocumentType>(
                "SELECT id, name, code, applies_to, is_active, sort_order, created_at \
                 FROM config_document_types ORDER BY sort_order, name"
            )
            .fetch_all(&self.pool)
            .await?
        };
        Ok(rows)
    }

    async fn create_expense_category(&self, name: &str, code: &str) -> Result<ConfigItem, AppError> {
        let row = sqlx::query_as::<_, ConfigItem>(
            "INSERT INTO config_expense_categories (name, code) VALUES ($1, $2) \
             RETURNING id, name, code, is_active, sort_order, created_at"
        )
        .bind(name)
        .bind(code)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn create_leave_type(&self, name: &str, code: &str) -> Result<ConfigItem, AppError> {
        let row = sqlx::query_as::<_, ConfigItem>(
            "INSERT INTO config_leave_types (name, code) VALUES ($1, $2) \
             RETURNING id, name, code, is_active, sort_order, created_at"
        )
        .bind(name)
        .bind(code)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn create_document_type(&self, name: &str, code: &str, applies_to: &str) -> Result<ConfigDocumentType, AppError> {
        let row = sqlx::query_as::<_, ConfigDocumentType>(
            "INSERT INTO config_document_types (name, code, applies_to) VALUES ($1, $2, $3) \
             RETURNING id, name, code, applies_to, is_active, sort_order, created_at"
        )
        .bind(name)
        .bind(code)
        .bind(applies_to)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn update_config_item(&self, table: &str, id: Uuid, name: &str, is_active: bool, sort_order: i32) -> Result<(), AppError> {
        let real_table = validated_table(table)?;
        // Use match to select the correct static query per table
        let rows_affected = match real_table {
            "config_expense_categories" => {
                sqlx::query(
                    "UPDATE config_expense_categories SET name = $1, is_active = $2, sort_order = $3 WHERE id = $4"
                )
                .bind(name).bind(is_active).bind(sort_order).bind(id)
                .execute(&self.pool).await?.rows_affected()
            }
            "config_leave_types" => {
                sqlx::query(
                    "UPDATE config_leave_types SET name = $1, is_active = $2, sort_order = $3 WHERE id = $4"
                )
                .bind(name).bind(is_active).bind(sort_order).bind(id)
                .execute(&self.pool).await?.rows_affected()
            }
            "config_document_types" => {
                sqlx::query(
                    "UPDATE config_document_types SET name = $1, is_active = $2, sort_order = $3 WHERE id = $4"
                )
                .bind(name).bind(is_active).bind(sort_order).bind(id)
                .execute(&self.pool).await?.rows_affected()
            }
            _ => return Err(AppError::BadRequest("Invalid config type".into())),
        };
        if rows_affected == 0 {
            return Err(AppError::NotFound("Config item not found".into()));
        }
        Ok(())
    }

    async fn delete_config_item(&self, table: &str, id: Uuid) -> Result<(), AppError> {
        let real_table = validated_table(table)?;
        let rows_affected = match real_table {
            "config_expense_categories" => {
                sqlx::query("DELETE FROM config_expense_categories WHERE id = $1")
                    .bind(id).execute(&self.pool).await?.rows_affected()
            }
            "config_leave_types" => {
                sqlx::query("DELETE FROM config_leave_types WHERE id = $1")
                    .bind(id).execute(&self.pool).await?.rows_affected()
            }
            "config_document_types" => {
                sqlx::query("DELETE FROM config_document_types WHERE id = $1")
                    .bind(id).execute(&self.pool).await?.rows_affected()
            }
            _ => return Err(AppError::BadRequest("Invalid config type".into())),
        };
        if rows_affected == 0 {
            return Err(AppError::NotFound("Config item not found".into()));
        }
        Ok(())
    }
}
