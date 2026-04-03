use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::document::domain::entity::{Document, DocumentEntity, DocumentType};

// ── Helpers ────────────────────────────────────────────────────────────────────

fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    Ok(s.filter(|v| !v.trim().is_empty()))
}

fn empty_string_as_none_date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        None => Ok(None),
        Some(ref v) if v.trim().is_empty() => Ok(None),
        Some(v) => NaiveDate::parse_from_str(&v, "%Y-%m-%d")
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

// ── Query ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub entity_type: DocumentEntity,
    pub entity_id: Uuid,
}

// ── Request ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub entity_type: DocumentEntity,
    pub entity_id: Uuid,
    pub doc_type: DocumentType,
    pub file_url: String,
    pub file_name: String,
    #[serde(default, deserialize_with = "empty_string_as_none_date")]
    pub expiry_date: Option<NaiveDate>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub notes: Option<String>,
}

// ── Response ───────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub doc_type: String,
    pub file_url: String,
    pub file_name: String,
    pub expiry_date: Option<NaiveDate>,
    pub uploaded_by: Uuid,
    pub notes: Option<String>,
    pub created_at: String,
}

impl From<Document> for DocumentResponse {
    fn from(d: Document) -> Self {
        // Use serde to serialize enums respecting rename_all = "snake_case"
        let entity_type = serde_json::to_value(&d.entity_type)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default();
        let doc_type = serde_json::to_value(&d.doc_type)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default();
        Self {
            id: d.id,
            entity_type,
            entity_id: d.entity_id,
            doc_type,
            file_url: d.file_url,
            file_name: d.file_name,
            expiry_date: d.expiry_date,
            uploaded_by: d.uploaded_by,
            notes: d.notes,
            created_at: d.created_at.to_rfc3339(),
        }
    }
}
