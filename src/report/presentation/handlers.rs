use std::sync::Arc;

use actix_web::{web, HttpResponse};

use crate::auth::presentation::handlers::require_role;
use crate::common::{error::AppError, response::ApiResponse, types::{CurrentUser, Role}};
use crate::report::application::service::ReportService;
use super::dto::{
    advance_report_csv, cash_flow_csv, driver_summary_csv, finance_summary_csv,
    leave_report_csv, salary_report_csv, trip_detail_csv, vehicle_report_csv,
    AdvanceReportResponse, CashFlowResponse, DashboardKpisResponse, DriverFinancialResponse,
    DriverSummaryResponse, FinanceSummaryResponse, LeaveReportResponse, ReportQuery,
    SalaryReportResponse, TripDetailResponse, VehicleReportResponse,
};

fn csv_response(content: String, filename: &str) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/csv")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{filename}\""),
        ))
        .body(content)
}

fn validate_date_range(from: chrono::NaiveDate, to: chrono::NaiveDate) -> Result<(), AppError> {
    if from > to {
        return Err(AppError::BadRequest("from_date must be before to_date".into()));
    }
    let max_days = (to - from).num_days();
    if max_days > 366 {
        return Err(AppError::BadRequest("Date range cannot exceed 366 days".into()));
    }
    Ok(())
}

pub async fn driver_summary(
    user: CurrentUser,
    svc: web::Data<Arc<ReportService>>,
    query: web::Query<ReportQuery>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    validate_date_range(query.from, query.to)?;

    let rows = svc.driver_summary(query.from, query.to).await?;
    let resp: Vec<DriverSummaryResponse> = rows.into_iter().map(Into::into).collect();

    if query.format == "csv" {
        return Ok(csv_response(driver_summary_csv(&resp), "driver_summary.csv"));
    }
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}

pub async fn trip_detail(
    user: CurrentUser,
    svc: web::Data<Arc<ReportService>>,
    query: web::Query<ReportQuery>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant, Role::Hr])?;
    validate_date_range(query.from, query.to)?;

    let rows = svc.trip_detail(query.from, query.to, query.driver_id).await?;
    let resp: Vec<TripDetailResponse> = rows.into_iter().map(Into::into).collect();

    if query.format == "csv" {
        return Ok(csv_response(trip_detail_csv(&resp), "trip_detail.csv"));
    }
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}

pub async fn finance_summary(
    user: CurrentUser,
    svc: web::Data<Arc<ReportService>>,
    query: web::Query<ReportQuery>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    validate_date_range(query.from, query.to)?;

    let report = svc.finance_summary(query.from, query.to).await?;
    let resp = FinanceSummaryResponse::from(report);

    if query.format == "csv" {
        return Ok(csv_response(finance_summary_csv(&resp), "finance_summary.csv"));
    }
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}

pub async fn dashboard(
    user: CurrentUser,
    svc: web::Data<Arc<ReportService>>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant, Role::Hr])?;
    let kpis = svc.dashboard().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(DashboardKpisResponse::from(kpis))))
}

pub async fn driver_financials(
    user: CurrentUser,
    svc: web::Data<Arc<ReportService>>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant, Role::Hr])?;
    let rows = svc.driver_financials().await?;
    let resp: Vec<DriverFinancialResponse> = rows.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}

pub async fn advance_report(
    user: CurrentUser,
    svc: web::Data<Arc<ReportService>>,
    query: web::Query<ReportQuery>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    validate_date_range(query.from, query.to)?;

    let rows = svc.advance_report(query.from, query.to).await?;
    let resp: Vec<AdvanceReportResponse> = rows.into_iter().map(Into::into).collect();

    if query.format == "csv" {
        return Ok(csv_response(advance_report_csv(&resp), "advance_report.csv"));
    }
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}

pub async fn cash_flow_report(
    user: CurrentUser,
    svc: web::Data<Arc<ReportService>>,
    query: web::Query<ReportQuery>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    validate_date_range(query.from, query.to)?;

    let rows = svc.cash_flow_report(query.from, query.to).await?;
    let resp: Vec<CashFlowResponse> = rows.into_iter().map(Into::into).collect();

    if query.format == "csv" {
        return Ok(csv_response(cash_flow_csv(&resp), "cash_flow.csv"));
    }
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}

pub async fn leave_report(
    user: CurrentUser,
    svc: web::Data<Arc<ReportService>>,
    query: web::Query<ReportQuery>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant, Role::Hr])?;
    validate_date_range(query.from, query.to)?;

    let rows = svc.leave_report(query.from, query.to).await?;
    let resp: Vec<LeaveReportResponse> = rows.into_iter().map(Into::into).collect();

    if query.format == "csv" {
        return Ok(csv_response(leave_report_csv(&resp), "leave_report.csv"));
    }
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}

pub async fn salary_report(
    user: CurrentUser,
    svc: web::Data<Arc<ReportService>>,
    query: web::Query<ReportQuery>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    validate_date_range(query.from, query.to)?;

    let rows = svc.salary_report(query.from, query.to).await?;
    let resp: Vec<SalaryReportResponse> = rows.into_iter().map(Into::into).collect();

    if query.format == "csv" {
        return Ok(csv_response(salary_report_csv(&resp), "salary_report.csv"));
    }
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}

pub async fn vehicle_report(
    user: CurrentUser,
    svc: web::Data<Arc<ReportService>>,
    query: web::Query<ReportQuery>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant, Role::Hr])?;
    validate_date_range(query.from, query.to)?;

    let rows = svc.vehicle_report(query.from, query.to).await?;
    let resp: Vec<VehicleReportResponse> = rows.into_iter().map(Into::into).collect();

    if query.format == "csv" {
        return Ok(csv_response(vehicle_report_csv(&resp), "vehicle_report.csv"));
    }
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}
