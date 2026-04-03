use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::report::domain::entity::{
    AdvanceReportRow, CashFlowRow, DashboardKpis, DriverSummaryRow, FinanceReport,
    LeaveReportRow, SalaryReportRow, TripDetailRow, VehicleReportRow,
};

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

// ── Dashboard ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct InsuranceAlertResponse {
    pub vehicle_id: Uuid,
    pub plate_number: String,
    pub insurance_expiry: NaiveDate,
    pub days_left: i64,
    pub is_expired: bool,
}

#[derive(Debug, Serialize)]
pub struct DriverPerfResponse {
    pub driver_id: Uuid,
    pub driver_name: String,
    pub trips_count: i64,
    pub revenue_aed: Decimal,
}

#[derive(Debug, Serialize)]
pub struct DayRevenueResponse {
    pub date: NaiveDate,
    pub revenue_aed: Decimal,
    pub trips_count: i64,
}

#[derive(Debug, Serialize)]
pub struct DashboardKpisResponse {
    pub revenue_mtd: Decimal,
    pub trips_mtd: i64,
    pub active_drivers: i64,
    pub active_vehicles: i64,
    pub pending_advances: i64,
    pub pending_leave: i64,
    pub total_expenses_mtd: Decimal,
    pub net_profit: Decimal,
    pub insurance_expiring_soon: Vec<InsuranceAlertResponse>,
    pub top_drivers: Vec<DriverPerfResponse>,
    pub bottom_drivers: Vec<DriverPerfResponse>,
    pub revenue_trend: Vec<DayRevenueResponse>,
}

impl From<DashboardKpis> for DashboardKpisResponse {
    fn from(d: DashboardKpis) -> Self {
        Self {
            revenue_mtd: d.revenue_mtd,
            trips_mtd: d.trips_mtd,
            active_drivers: d.active_drivers,
            active_vehicles: d.active_vehicles,
            pending_advances: d.pending_advances,
            pending_leave: d.pending_leave,
            total_expenses_mtd: d.total_expenses_mtd,
            net_profit: d.net_profit,
            insurance_expiring_soon: d.insurance_expiring_soon.into_iter().map(|a| InsuranceAlertResponse {
                vehicle_id: a.vehicle_id,
                plate_number: a.plate_number,
                insurance_expiry: a.insurance_expiry,
                days_left: a.days_left,
                is_expired: a.is_expired,
            }).collect(),
            top_drivers: d.top_drivers.into_iter().map(|r| DriverPerfResponse {
                driver_id: r.driver_id,
                driver_name: r.driver_name,
                trips_count: r.trips_count,
                revenue_aed: r.revenue_aed,
            }).collect(),
            bottom_drivers: d.bottom_drivers.into_iter().map(|r| DriverPerfResponse {
                driver_id: r.driver_id,
                driver_name: r.driver_name,
                trips_count: r.trips_count,
                revenue_aed: r.revenue_aed,
            }).collect(),
            revenue_trend: d.revenue_trend.into_iter().map(|r| DayRevenueResponse {
                date: r.date,
                revenue_aed: r.revenue_aed,
                trips_count: r.trips_count,
            }).collect(),
        }
    }
}

// ── Advance Report ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AdvanceReportResponse {
    pub driver_name: String,
    pub total_requested: Decimal,
    pub total_approved: Decimal,
    pub total_paid: Decimal,
    pub outstanding_balance: Decimal,
}

impl From<AdvanceReportRow> for AdvanceReportResponse {
    fn from(r: AdvanceReportRow) -> Self {
        Self {
            driver_name: r.driver_name,
            total_requested: r.total_requested,
            total_approved: r.total_approved,
            total_paid: r.total_paid,
            outstanding_balance: r.outstanding_balance,
        }
    }
}

pub fn advance_report_csv(rows: &[AdvanceReportResponse]) -> String {
    let mut out = String::from("driver_name,total_requested,total_approved,total_paid,outstanding_balance\n");
    for r in rows {
        out.push_str(&format!(
            "{},{},{},{},{}\n",
            r.driver_name, r.total_requested, r.total_approved, r.total_paid, r.outstanding_balance
        ));
    }
    out
}

// ── Cash Flow Report ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CashFlowResponse {
    pub driver_name: String,
    pub total_cash_received: Decimal,
    pub total_cash_submitted: Decimal,
    pub shortfall: Decimal,
}

impl From<CashFlowRow> for CashFlowResponse {
    fn from(r: CashFlowRow) -> Self {
        Self {
            driver_name: r.driver_name,
            total_cash_received: r.total_cash_received,
            total_cash_submitted: r.total_cash_submitted,
            shortfall: r.shortfall,
        }
    }
}

pub fn cash_flow_csv(rows: &[CashFlowResponse]) -> String {
    let mut out = String::from("driver_name,total_cash_received,total_cash_submitted,shortfall\n");
    for r in rows {
        out.push_str(&format!(
            "{},{},{},{}\n",
            r.driver_name, r.total_cash_received, r.total_cash_submitted, r.shortfall
        ));
    }
    out
}

// ── Leave Report ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct LeaveReportResponse {
    pub driver_name: String,
    pub total_leave_days: i64,
    pub total_permissions: i64,
    pub pending_count: i64,
    pub approved_count: i64,
    pub rejected_count: i64,
}

impl From<LeaveReportRow> for LeaveReportResponse {
    fn from(r: LeaveReportRow) -> Self {
        Self {
            driver_name: r.driver_name,
            total_leave_days: r.total_leave_days,
            total_permissions: r.total_permissions,
            pending_count: r.pending_count,
            approved_count: r.approved_count,
            rejected_count: r.rejected_count,
        }
    }
}

pub fn leave_report_csv(rows: &[LeaveReportResponse]) -> String {
    let mut out = String::from("driver_name,total_leave_days,total_permissions,pending_count,approved_count,rejected_count\n");
    for r in rows {
        out.push_str(&format!(
            "{},{},{},{},{},{}\n",
            r.driver_name, r.total_leave_days, r.total_permissions,
            r.pending_count, r.approved_count, r.rejected_count
        ));
    }
    out
}

// ── Salary Summary Report ─────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SalaryReportResponse {
    pub driver_name: String,
    pub period: String,
    pub salary_type: String,
    pub gross: Decimal,
    pub deductions: Decimal,
    pub net_payable: Decimal,
}

impl From<SalaryReportRow> for SalaryReportResponse {
    fn from(r: SalaryReportRow) -> Self {
        Self {
            driver_name: r.driver_name,
            period: r.period,
            salary_type: r.salary_type,
            gross: r.gross,
            deductions: r.deductions,
            net_payable: r.net_payable,
        }
    }
}

pub fn salary_report_csv(rows: &[SalaryReportResponse]) -> String {
    let mut out = String::from("driver_name,period,salary_type,gross,deductions,net_payable\n");
    for r in rows {
        out.push_str(&format!(
            "{},{},{},{},{},{}\n",
            r.driver_name, r.period, r.salary_type, r.gross, r.deductions, r.net_payable
        ));
    }
    out
}

// ── Vehicle Report ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct VehicleReportResponse {
    pub plate_number: String,
    pub make: String,
    pub model: String,
    pub status: String,
    pub owner_name: Option<String>,
    pub current_driver: Option<String>,
    pub insurance_expiry: Option<NaiveDate>,
    pub service_count: i64,
    pub last_service_date: Option<NaiveDate>,
}

impl From<VehicleReportRow> for VehicleReportResponse {
    fn from(r: VehicleReportRow) -> Self {
        Self {
            plate_number: r.plate_number,
            make: r.make,
            model: r.model,
            status: r.status,
            owner_name: r.owner_name,
            current_driver: r.current_driver,
            insurance_expiry: r.insurance_expiry,
            service_count: r.service_count,
            last_service_date: r.last_service_date,
        }
    }
}

pub fn vehicle_report_csv(rows: &[VehicleReportResponse]) -> String {
    let mut out = String::from(
        "plate_number,make,model,status,owner_name,current_driver,insurance_expiry,service_count,last_service_date\n",
    );
    for r in rows {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            r.plate_number,
            r.make,
            r.model,
            r.status,
            r.owner_name.as_deref().unwrap_or(""),
            r.current_driver.as_deref().unwrap_or(""),
            r.insurance_expiry.map(|d| d.to_string()).unwrap_or_default(),
            r.service_count,
            r.last_service_date.map(|d| d.to_string()).unwrap_or_default(),
        ));
    }
    out
}
