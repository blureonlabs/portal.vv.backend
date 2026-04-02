use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "trip_source", rename_all = "snake_case")]
pub enum TripSource {
    Manual,
    CsvImport,
    UberApi,
}

#[derive(Debug, Clone, Serialize)]
pub struct Trip {
    pub id: Uuid,
    pub driver_id: Uuid,
    pub driver_name: String,
    pub vehicle_id: Option<Uuid>,
    pub entered_by: Uuid,
    pub trip_date: NaiveDate,
    pub cash_aed: Decimal,
    pub card_aed: Decimal,
    pub other_aed: Decimal,
    pub source: TripSource,
    pub notes: Option<String>,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
}

impl Trip {
    pub fn total(&self) -> Decimal {
        self.cash_aed + self.card_aed + self.other_aed
    }
}

/// A CSV row parsed from uploaded content, with validation annotations.
#[derive(Debug, Clone, Serialize)]
pub struct CsvPreviewRow {
    pub row_num: usize,
    pub trip_date: String,
    pub cash_aed: Decimal,
    pub card_aed: Decimal,
    pub other_aed: Decimal,
    pub notes: Option<String>,
    pub error: Option<String>,
    pub cap_warning: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateTrip {
    pub driver_id: Uuid,
    pub vehicle_id: Option<Uuid>,
    pub entered_by: Uuid,
    pub trip_date: NaiveDate,
    pub cash_aed: Decimal,
    pub card_aed: Decimal,
    pub other_aed: Decimal,
    pub source: TripSource,
    pub notes: Option<String>,
}
