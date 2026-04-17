use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::trip::domain::entity::{CsvPreviewRow, Trip};

// ── Requests ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListTripsQuery {
    pub driver_id: Option<Uuid>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTripRequest {
    pub driver_id: Uuid,
    pub vehicle_id: Option<Uuid>,
    pub trip_date: NaiveDate,
    pub cash_aed: Decimal,
    pub uber_cash_aed: Option<Decimal>,
    pub bolt_cash_aed: Option<Decimal>,
    pub card_aed: Option<Decimal>,
    /// Kept for backward compat — maps to uber_cash_aed if uber_cash_aed is absent.
    pub other_aed: Option<Decimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CsvPreviewRequest {
    pub driver_id: Uuid,
    /// Raw CSV text content
    pub csv_content: String,
}

#[derive(Debug, Deserialize)]
pub struct CsvImportRequest {
    pub driver_id: Uuid,
    /// Rows returned from /trips/csv/preview — only valid rows will be inserted
    pub rows: Vec<CsvPreviewRowDto>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CsvPreviewRowDto {
    pub row_num: usize,
    pub trip_date: String,
    pub cash_aed: Decimal,
    pub uber_cash_aed: Decimal,
    pub bolt_cash_aed: Decimal,
    pub card_aed: Decimal,
    pub notes: Option<String>,
    pub error: Option<String>,
    pub cap_warning: Option<String>,
}

// ── Responses ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TripResponse {
    pub id: Uuid,
    pub driver_id: Uuid,
    pub driver_name: String,
    pub vehicle_id: Option<Uuid>,
    pub trip_date: NaiveDate,
    pub cash_aed: Decimal,
    pub uber_cash_aed: Decimal,
    pub bolt_cash_aed: Decimal,
    pub card_aed: Decimal,
    /// Backward compat alias for uber_cash_aed
    pub other_aed: Decimal,
    pub total_aed: Decimal,
    pub source: String,
    pub notes: Option<String>,
    pub created_at: String,
}

impl From<Trip> for TripResponse {
    fn from(t: Trip) -> Self {
        let total_aed = t.total();
        let source = format!("{:?}", t.source).to_lowercase();
        Self {
            id: t.id,
            driver_id: t.driver_id,
            driver_name: t.driver_name,
            vehicle_id: t.vehicle_id,
            trip_date: t.trip_date,
            cash_aed: t.cash_aed,
            uber_cash_aed: t.uber_cash_aed,
            bolt_cash_aed: t.bolt_cash_aed,
            card_aed: t.card_aed,
            other_aed: t.uber_cash_aed,
            total_aed,
            source,
            notes: t.notes,
            created_at: t.created_at.to_rfc3339(),
        }
    }
}

/// Response returned by the create-trip endpoint; includes an optional
/// `conflict_warning` when another trip from a different source already
/// exists for the same driver on the same date.
#[derive(Debug, Serialize)]
pub struct CreateTripResponse {
    #[serde(flatten)]
    pub trip: TripResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_warning: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CsvPreviewResponse {
    pub rows: Vec<CsvPreviewRow>,
    pub valid_count: usize,
    pub error_count: usize,
}
