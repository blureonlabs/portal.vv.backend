use std::sync::Arc;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::owner::domain::{entity::Owner, repository::OwnerRepository};

pub struct OwnerService {
    repo: Arc<dyn OwnerRepository>,
}

impl OwnerService {
    pub fn new(repo: Arc<dyn OwnerRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(&self) -> Result<Vec<Owner>, AppError> {
        self.repo.list().await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Owner, AppError> {
        self.repo.find_by_id(id).await?.ok_or_else(|| AppError::NotFound("Owner not found".into()))
    }

    pub async fn find_by_profile_id(&self, profile_id: Uuid) -> Result<Owner, AppError> {
        self.repo.find_by_profile_id(profile_id).await?.ok_or_else(|| AppError::NotFound("Owner not found".into()))
    }

    pub async fn create(&self, profile_id: Uuid, company_name: Option<&str>, notes: Option<&str>) -> Result<Owner, AppError> {
        self.repo.create(profile_id, company_name, notes).await
    }

    pub async fn update(&self, id: Uuid, company_name: Option<&str>, notes: Option<&str>) -> Result<Owner, AppError> {
        self.repo.update(id, company_name, notes).await
    }
}
