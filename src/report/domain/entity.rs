use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DriverSummaryRow {
    pub driver_id: Uuid,
    pub driver_name: String,
    pub trips_count: i64,
    pub total_revenue_aed: Decimal,
    pub total_expenses_aed: Decimal,
    pub net_aed: Decimal,
}

#[derive(Debug, Clone)]
pub struct TripDetailRow {
    pub trip_id: Uuid,
    pub driver_name: String,
    pub trip_date: NaiveDate,
    pub cash_aed: Decimal,
    pub card_aed: Decimal,
    pub other_aed: Decimal,
    pub total_aed: Decimal,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FinanceSummaryRow {
    pub category: String,
    pub total_aed: Decimal,
}

#[derive(Debug, Clone)]
pub struct FinanceReport {
    pub trip_revenue_cash: Decimal,
    pub trip_revenue_card: Decimal,
    pub trip_revenue_other: Decimal,
    pub trip_revenue_total: Decimal,
    pub expense_by_category: Vec<FinanceSummaryRow>,
    pub total_expenses: Decimal,
    pub total_handovers: Decimal,
    pub net_aed: Decimal,
}
