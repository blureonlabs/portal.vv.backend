use std::sync::Arc;

use actix_web::{web, HttpResponse};

use crate::auth::presentation::handlers::require_role;
use crate::common::{error::AppError, response::ApiResponse, types::{CurrentUser, Role}};
use crate::report::application::service::ReportService;
use super::dto::{
    advance_report_csv, cash_flow_csv, driver_summary_csv, finance_summary_csv,
    leave_report_csv, salary_report_csv, trip_detail_csv, AdvanceReportResponse,
    CashFlowResponse, DashboardKpisResponse, DriverSummaryResponse, FinanceSummaryResponse,
    LeaveReportResponse, ReportQuery, SalaryReportResponse, TripDetailResponse,
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

pub async fn driver_summary(
    user: CurrentUser,
    svc: web::Data<Arc<ReportService>>,
    query: web::Query<ReportQuery>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;

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

pub async fn advance_report(
    user: CurrentUser,
    svc: web::Data<Arc<ReportService>>,
    query: web::Query<ReportQuery>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;

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

    let rows = svc.salary_report(query.from, query.to).await?;
    let resp: Vec<SalaryReportResponse> = rows.into_iter().map(Into::into).collect();

    if query.format == "csv" {
        return Ok(csv_response(salary_report_csv(&resp), "salary_report.csv"));
    }
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}
