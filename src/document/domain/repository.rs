use async_trait::async_trait;
use uuid::Uuid;

use crate::common::error::AppError;
use super::entity::{CreateDocument, Document, DocumentEntity};

#[async_trait]
pub trait DocumentRepository: Send + Sync {
    async fn list_by_entity(
        &self,
        entity_type: DocumentEntity,
        entity_id: Uuid,
    ) -> Result<Vec<Document>, AppError>;

    async fn create(&self, payload: CreateDocument) -> Result<Document, AppError>;

    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}
