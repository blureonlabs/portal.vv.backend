use std::sync::Arc;

use uuid::Uuid;

use crate::audit::application::service::AuditService;
use crate::common::error::AppError;
use crate::common::types::Role;
use crate::config::domain::{
    entity::{ConfigItem, ConfigDocumentType},
    repository::ConfigRepository,
};

pub struct ConfigService {
    repo: Arc<dyn ConfigRepository>,
    audit: Arc<AuditService>,
}

impl ConfigService {
    pub fn new(repo: Arc<dyn ConfigRepository>, audit: Arc<AuditService>) -> Self {
        Self { repo, audit }
    }

    // ── Expense categories ──────────────────────────────────────────────────

    pub async fn list_expense_categories(&self, active_only: bool) -> Result<Vec<ConfigItem>, AppError> {
        self.repo.list_expense_categories(active_only).await
    }

    pub async fn create_expense_category(
        &self,
        name: &str,
        code: &str,
        actor_id: Uuid,
        actor_role: &Role,
    ) -> Result<ConfigItem, AppError> {
        validate_name(name)?;
        validate_code(code)?;
        let item = self.repo.create_expense_category(name.trim(), code.trim()).await?;
        let _ = self.audit.log(actor_id, actor_role, "config_expense_category", Some(item.id), "create", None).await;
        Ok(item)
    }

    pub async fn update_expense_category(
        &self,
        id: Uuid,
        name: &str,
        is_active: bool,
        sort_order: i32,
        actor_id: Uuid,
        actor_role: &Role,
    ) -> Result<(), AppError> {
        validate_name(name)?;
        self.repo.update_config_item("expense_categories", id, name.trim(), is_active, sort_order).await?;
        let _ = self.audit.log(actor_id, actor_role, "config_expense_category", Some(id), "update", None).await;
        Ok(())
    }

    pub async fn delete_expense_category(
        &self,
        id: Uuid,
        actor_id: Uuid,
        actor_role: &Role,
    ) -> Result<(), AppError> {
        self.repo.delete_config_item("expense_categories", id).await?;
        let _ = self.audit.log(actor_id, actor_role, "config_expense_category", Some(id), "delete", None).await;
        Ok(())
    }

    // ── Leave types ─────────────────────────────────────────────────────────

    pub async fn list_leave_types(&self, active_only: bool) -> Result<Vec<ConfigItem>, AppError> {
        self.repo.list_leave_types(active_only).await
    }

    pub async fn create_leave_type(
        &self,
        name: &str,
        code: &str,
        actor_id: Uuid,
        actor_role: &Role,
    ) -> Result<ConfigItem, AppError> {
        validate_name(name)?;
        validate_code(code)?;
        let item = self.repo.create_leave_type(name.trim(), code.trim()).await?;
        let _ = self.audit.log(actor_id, actor_role, "config_leave_type", Some(item.id), "create", None).await;
        Ok(item)
    }

    pub async fn update_leave_type(
        &self,
        id: Uuid,
        name: &str,
        is_active: bool,
        sort_order: i32,
        actor_id: Uuid,
        actor_role: &Role,
    ) -> Result<(), AppError> {
        validate_name(name)?;
        self.repo.update_config_item("leave_types", id, name.trim(), is_active, sort_order).await?;
        let _ = self.audit.log(actor_id, actor_role, "config_leave_type", Some(id), "update", None).await;
        Ok(())
    }

    pub async fn delete_leave_type(
        &self,
        id: Uuid,
        actor_id: Uuid,
        actor_role: &Role,
    ) -> Result<(), AppError> {
        self.repo.delete_config_item("leave_types", id).await?;
        let _ = self.audit.log(actor_id, actor_role, "config_leave_type", Some(id), "delete", None).await;
        Ok(())
    }

    // ── Document types ──────────────────────────────────────────────────────

    pub async fn list_document_types(&self, active_only: bool) -> Result<Vec<ConfigDocumentType>, AppError> {
        self.repo.list_document_types(active_only).await
    }

    pub async fn create_document_type(
        &self,
        name: &str,
        code: &str,
        applies_to: &str,
        actor_id: Uuid,
        actor_role: &Role,
    ) -> Result<ConfigDocumentType, AppError> {
        validate_name(name)?;
        validate_code(code)?;
        validate_applies_to(applies_to)?;
        let item = self.repo.create_document_type(name.trim(), code.trim(), applies_to.trim()).await?;
        let _ = self.audit.log(actor_id, actor_role, "config_document_type", Some(item.id), "create", None).await;
        Ok(item)
    }

    pub async fn update_document_type(
        &self,
        id: Uuid,
        name: &str,
        is_active: bool,
        sort_order: i32,
        actor_id: Uuid,
        actor_role: &Role,
    ) -> Result<(), AppError> {
        validate_name(name)?;
        self.repo.update_config_item("document_types", id, name.trim(), is_active, sort_order).await?;
        let _ = self.audit.log(actor_id, actor_role, "config_document_type", Some(id), "update", None).await;
        Ok(())
    }

    pub async fn delete_document_type(
        &self,
        id: Uuid,
        actor_id: Uuid,
        actor_role: &Role,
    ) -> Result<(), AppError> {
        self.repo.delete_config_item("document_types", id).await?;
        let _ = self.audit.log(actor_id, actor_role, "config_document_type", Some(id), "delete", None).await;
        Ok(())
    }
}

// ── Validation helpers ──────────────────────────────────────────────────────

fn validate_name(name: &str) -> Result<(), AppError> {
    let name = name.trim();
    if name.is_empty() || name.len() > 100 {
        return Err(AppError::BadRequest("Name must be 1-100 characters".into()));
    }
    Ok(())
}

fn validate_code(code: &str) -> Result<(), AppError> {
    let code = code.trim();
    if code.is_empty() || code.len() > 50 {
        return Err(AppError::BadRequest("Code must be 1-50 characters".into()));
    }
    if !code.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return Err(AppError::BadRequest("Code must be lowercase alphanumeric with underscores".into()));
    }
    Ok(())
}

fn validate_applies_to(applies_to: &str) -> Result<(), AppError> {
    match applies_to.trim() {
        "driver" | "vehicle" | "both" => Ok(()),
        _ => Err(AppError::BadRequest("applies_to must be 'driver', 'vehicle', or 'both'".into())),
    }
}
