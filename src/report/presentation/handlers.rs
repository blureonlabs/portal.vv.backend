use std::sync::Arc;

use actix_web::{web, HttpResponse};

use crate::auth::presentation::handlers::require_role;
use crate::common::{error::AppError, response::ApiResponse, types::{CurrentUser, Role}};
use crate::report::application::service::ReportService;
use super::dto::{
    driver_summary_csv, finance_summary_csv, trip_detail_csv, DashboardKpisResponse,
    DriverSummaryResponse, FinanceSummaryResponse, ReportQuery, TripDetailResponse,
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
