use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct DriverContextResponse {
    pub profile_id: Uuid,
    pub full_name: String,
    pub email: String,
    pub driver_id: Uuid,
    pub salary_type: String,
    pub nationality: Option<String>,
    pub self_entry_enabled: bool,
    pub vehicle: Option<AssignedVehicle>,
}

#[derive(Debug, Serialize)]
pub struct AssignedVehicle {
    pub id: Uuid,
    pub plate_number: String,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EarningsQuery {
    /// YYYY-MM — defaults to current month
    pub month: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EarningsResponse {
    pub month: String,
    pub days: Vec<DayEarnings>,
    pub total_cash: Decimal,
    pub total_card: Decimal,
    pub total_other: Decimal,
    pub grand_total: Decimal,
}

#[derive(Debug, Serialize)]
pub struct DayEarnings {
    pub date: NaiveDate,
    pub cash_aed: Decimal,
    pub card_aed: Decimal,
    pub other_aed: Decimal,
    pub total_aed: Decimal,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct NotificationItem {
    pub id: Uuid,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}
