use std::sync::Arc;

use uuid::Uuid;

use crate::common::error::AppError;
use crate::document::domain::{
    entity::{CreateDocument, Document, DocumentEntity, DocumentType},
    repository::DocumentRepository,
};

pub struct DocumentService {
    repo: Arc<dyn DocumentRepository>,
}

impl DocumentService {
    pub fn new(repo: Arc<dyn DocumentRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(
        &self,
        entity_type: DocumentEntity,
        entity_id: Uuid,
    ) -> Result<Vec<Document>, AppError> {
        self.repo.list_by_entity(entity_type, entity_id).await
    }

    pub async fn upload(
        &self,
        entity_type: DocumentEntity,
        entity_id: Uuid,
        doc_type: DocumentType,
        file_url: String,
        file_name: String,
        expiry_date: Option<chrono::NaiveDate>,
        uploaded_by: Uuid,
        notes: Option<String>,
    ) -> Result<Document, AppError> {
        if file_url.trim().is_empty() {
            return Err(AppError::BadRequest("file_url is required".into()));
        }
        if file_name.trim().is_empty() {
            return Err(AppError::BadRequest("file_name is required".into()));
        }

        let payload = CreateDocument {
            entity_type,
            entity_id,
            doc_type,
            file_url,
            file_name,
            expiry_date,
            uploaded_by,
            notes,
        };

        self.repo.create(payload).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.repo.delete(id).await
    }
}
