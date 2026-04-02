use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DashboardKpis {
    pub revenue_mtd: Decimal,
    pub trips_mtd: i64,
    pub active_drivers: i64,
    pub active_vehicles: i64,
    pub pending_advances: i64,
    pub pending_leave: i64,
    pub insurance_expiring_soon: Vec<InsuranceAlert>,
    pub top_drivers: Vec<DriverPerfRow>,
    pub revenue_trend: Vec<DayRevenue>,
}

#[derive(Debug, Clone)]
pub struct InsuranceAlert {
    pub vehicle_id: Uuid,
    pub plate_number: String,
    pub insurance_expiry: NaiveDate,
    pub days_left: i64,
}

#[derive(Debug, Clone)]
pub struct DriverPerfRow {
    pub driver_id: Uuid,
    pub driver_name: String,
    pub trips_count: i64,
    pub revenue_aed: Decimal,
}

#[derive(Debug, Clone)]
pub struct DayRevenue {
    pub date: NaiveDate,
    pub revenue_aed: Decimal,
    pub trips_count: i64,
}

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
