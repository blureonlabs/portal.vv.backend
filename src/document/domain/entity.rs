use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "document_type", rename_all = "snake_case")]
pub enum DocumentType {
    License,
    Visa,
    Passport,
    EmiratesId,
    Medical,
    RegistrationCard,
    InsuranceCertificate,
    Receipt,
    Other,
}

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "document_entity", rename_all = "snake_case")]
pub enum DocumentEntity {
    Driver,
    Vehicle,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: Uuid,
    pub entity_type: DocumentEntity,
    pub entity_id: Uuid,
    pub doc_type: DocumentType,
    pub file_url: String,
    pub file_name: String,
    pub expiry_date: Option<NaiveDate>,
    pub uploaded_by: Uuid,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct CreateDocument {
    pub entity_type: DocumentEntity,
    pub entity_id: Uuid,
    pub doc_type: DocumentType,
    pub file_url: String,
    pub file_name: String,
    pub expiry_date: Option<NaiveDate>,
    pub uploaded_by: Uuid,
    pub notes: Option<String>,
}
