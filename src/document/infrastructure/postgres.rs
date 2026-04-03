use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::document::domain::{
    entity::{CreateDocument, Document, DocumentEntity, DocumentType},
    repository::DocumentRepository,
};

pub struct PgDocumentRepository {
    pool: PgPool,
}

impl PgDocumentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DocumentRepository for PgDocumentRepository {
    async fn list_by_entity(
        &self,
        entity_type: DocumentEntity,
        entity_id: Uuid,
    ) -> Result<Vec<Document>, AppError> {
        let rows = sqlx::query_as!(
            Document,
            r#"
            SELECT
                id,
                entity_type AS "entity_type: DocumentEntity",
                entity_id,
                doc_type AS "doc_type: DocumentType",
                file_url,
                file_name,
                expiry_date,
                uploaded_by,
                notes,
                created_at
            FROM documents
            WHERE entity_type = $1
              AND entity_id = $2
            ORDER BY created_at DESC
            "#,
            entity_type as DocumentEntity,
            entity_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn create(&self, payload: CreateDocument) -> Result<Document, AppError> {
        let row = sqlx::query_as!(
            Document,
            r#"
            INSERT INTO documents (entity_type, entity_id, doc_type, file_url, file_name, expiry_date, uploaded_by, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id,
                entity_type AS "entity_type: DocumentEntity",
                entity_id,
                doc_type AS "doc_type: DocumentType",
                file_url,
                file_name,
                expiry_date,
                uploaded_by,
                notes,
                created_at
            "#,
            payload.entity_type as DocumentEntity,
            payload.entity_id,
            payload.doc_type as DocumentType,
            payload.file_url,
            payload.file_name,
            payload.expiry_date,
            payload.uploaded_by,
            payload.notes,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!("DELETE FROM documents WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Document {} not found", id)));
        }

        Ok(())
    }
}
