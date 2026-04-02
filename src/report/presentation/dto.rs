use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::report::domain::entity::{DriverSummaryRow, FinanceReport, TripDetailRow};

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub driver_id: Option<Uuid>,
    /// "json" (default) or "csv"
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "json".to_string()
}

// ── Driver Summary ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct DriverSummaryResponse {
    pub driver_id: Uuid,
    pub driver_name: String,
    pub trips_count: i64,
    pub total_revenue_aed: Decimal,
    pub total_expenses_aed: Decimal,
    pub net_aed: Decimal,
}

impl From<DriverSummaryRow> for DriverSummaryResponse {
    fn from(r: DriverSummaryRow) -> Self {
        Self {
            driver_id: r.driver_id,
            driver_name: r.driver_name,
            trips_count: r.trips_count,
            total_revenue_aed: r.total_revenue_aed,
            total_expenses_aed: r.total_expenses_aed,
            net_aed: r.net_aed,
        }
    }
}

pub fn driver_summary_csv(rows: &[DriverSummaryResponse]) -> String {
    let mut out = String::from("driver_name,trips_count,total_revenue_aed,total_expenses_aed,net_aed\n");
    for r in rows {
        out.push_str(&format!(
            "{},{},{},{},{}\n",
            r.driver_name, r.trips_count, r.total_revenue_aed, r.total_expenses_aed, r.net_aed
        ));
    }
    out
}

// ── Trip Detail ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TripDetailResponse {
    pub trip_id: Uuid,
    pub driver_name: String,
    pub trip_date: NaiveDate,
    pub cash_aed: Decimal,
    pub card_aed: Decimal,
    pub other_aed: Decimal,
    pub total_aed: Decimal,
    pub notes: Option<String>,
}

impl From<TripDetailRow> for TripDetailResponse {
    fn from(r: TripDetailRow) -> Self {
        Self {
            trip_id: r.trip_id,
            driver_name: r.driver_name,
            trip_date: r.trip_date,
            cash_aed: r.cash_aed,
            card_aed: r.card_aed,
            other_aed: r.other_aed,
            total_aed: r.total_aed,
            notes: r.notes,
        }
    }
}

pub fn trip_detail_csv(rows: &[TripDetailResponse]) -> String {
    let mut out = String::from("driver_name,trip_date,cash_aed,card_aed,other_aed,total_aed,notes\n");
    for r in rows {
        out.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            r.driver_name,
            r.trip_date,
            r.cash_aed,
            r.card_aed,
            r.other_aed,
            r.total_aed,
            r.notes.as_deref().unwrap_or("")
        ));
    }
    out
}

// ── Finance Summary ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct FinanceSummaryResponse {
    pub trip_revenue_cash: Decimal,
    pub trip_revenue_card: Decimal,
    pub trip_revenue_other: Decimal,
    pub trip_revenue_total: Decimal,
    pub expense_by_category: Vec<CategoryTotal>,
    pub total_expenses: Decimal,
    pub total_handovers: Decimal,
    pub net_aed: Decimal,
}

#[derive(Debug, Serialize)]
pub struct CategoryTotal {
    pub category: String,
    pub total_aed: Decimal,
}

impl From<FinanceReport> for FinanceSummaryResponse {
    fn from(r: FinanceReport) -> Self {
        Self {
            trip_revenue_cash: r.trip_revenue_cash,
            trip_revenue_card: r.trip_revenue_card,
            trip_revenue_other: r.trip_revenue_other,
            trip_revenue_total: r.trip_revenue_total,
            expense_by_category: r
                .expense_by_category
                .into_iter()
                .map(|c| CategoryTotal {
                    category: c.category,
                    total_aed: c.total_aed,
                })
                .collect(),
            total_expenses: r.total_expenses,
            total_handovers: r.total_handovers,
            net_aed: r.net_aed,
        }
    }
}

pub fn finance_summary_csv(r: &FinanceSummaryResponse) -> String {
    let mut out = String::from("metric,amount_aed\n");
    out.push_str(&format!("trip_revenue_cash,{}\n", r.trip_revenue_cash));
    out.push_str(&format!("trip_revenue_card,{}\n", r.trip_revenue_card));
    out.push_str(&format!("trip_revenue_other,{}\n", r.trip_revenue_other));
    out.push_str(&format!("trip_revenue_total,{}\n", r.trip_revenue_total));
    for cat in &r.expense_by_category {
        out.push_str(&format!("expense_{},{}\n", cat.category, cat.total_aed));
    }
    out.push_str(&format!("total_expenses,{}\n", r.total_expenses));
    out.push_str(&format!("total_handovers,{}\n", r.total_handovers));
    out.push_str(&format!("net_aed,{}\n", r.net_aed));
    out
}
