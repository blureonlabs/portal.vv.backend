use std::sync::Arc;

use actix_web::{web, HttpResponse};
use chrono::{Datelike, Local, NaiveDate};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::common::{error::AppError, response::{ApiResponse, PaginatedResponse}, types::{CurrentUser, Role}};
use crate::trip::application::service::TripService;
use crate::trip::domain::entity::CsvPreviewRow;
use crate::trip::presentation::dto::{
    CreateTripRequest, CreateTripResponse, CsvImportRequest, CsvPreviewRequest, CsvPreviewResponse,
    ListTripsQuery, TripResponse,
};

/// Resolve the driver_id for the calling user (driver role only).
async fn resolve_driver_id(
    svc: &TripService,
    user: &CurrentUser,
) -> Result<Option<Uuid>, AppError> {
    if user.role == Role::Driver {
        svc.repo.find_driver_id_by_profile(user.id).await
    } else {
        Ok(None)
    }
}

pub async fn list_trips(
    user: CurrentUser,
    svc: web::Data<Arc<TripService>>,
    query: web::Query<ListTripsQuery>,
) -> Result<HttpResponse, AppError> {
    let today = Local::now().date_naive();
    let from = query.from.unwrap_or_else(|| {
        NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today)
    });
    let to = query.to.unwrap_or(today);

    let limit = query.limit.unwrap_or(20).min(100).max(1);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let actor_driver_id = resolve_driver_id(&svc, &user).await?;
    let all: Vec<TripResponse> = svc
        .list(&user.role, actor_driver_id, query.driver_id, from, to)
        .await?
        .into_iter()
        .map(TripResponse::from)
        .collect();
    let total = all.len() as i64;
    let page_data = all.into_iter().skip(offset as usize).take(limit as usize).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(PaginatedResponse::ok(page_data, page, limit, total)))
}

pub async fn create_trip(
    user: CurrentUser,
    svc: web::Data<Arc<TripService>>,
    body: web::Json<CreateTripRequest>,
) -> Result<HttpResponse, AppError> {
    let body = body.into_inner();
    let actor_driver_id = resolve_driver_id(&svc, &user).await?;

    let (trip, conflict_warning) = svc.create(
        user.id,
        &user.role,
        actor_driver_id,
        body.driver_id,
        body.vehicle_id,
        body.trip_date,
        body.cash_aed,
        body.card_aed.unwrap_or(Decimal::ZERO),
        body.other_aed.unwrap_or(Decimal::ZERO),
        body.notes,
    ).await?;

    let resp = CreateTripResponse {
        trip: TripResponse::from(trip),
        conflict_warning,
    };
    Ok(HttpResponse::Created().json(ApiResponse::ok(resp)))
}

pub async fn delete_trip(
    user: CurrentUser,
    svc: web::Data<Arc<TripService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    svc.delete(user.id, &user.role, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(())))
}

pub async fn csv_template(
    user: CurrentUser,
) -> Result<HttpResponse, AppError> {
    match user.role {
        Role::SuperAdmin | Role::Accountant => {}
        _ => return Err(AppError::Forbidden("Only super_admin or accountant can download the CSV template".into())),
    }
    let csv = "driver_id,trip_date,cash_aed,card_aed,other_aed,notes\n";
    Ok(HttpResponse::Ok()
        .content_type("text/csv")
        .append_header(("Content-Disposition", "attachment; filename=\"trips_template.csv\""))
        .body(csv))
}

pub async fn csv_preview(
    user: CurrentUser,
    svc: web::Data<Arc<TripService>>,
    body: web::Json<CsvPreviewRequest>,
) -> Result<HttpResponse, AppError> {
    // Only admin/accountant can do CSV import on behalf of any driver
    match user.role {
        Role::SuperAdmin | Role::Accountant => {}
        _ => return Err(AppError::Forbidden("Only super_admin or accountant can import CSV trips".into())),
    }
    let body = body.into_inner();
    let rows = svc.csv_preview(body.driver_id, &body.csv_content).await?;
    let valid_count = rows.iter().filter(|r| r.error.is_none()).count();
    let error_count = rows.len() - valid_count;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(CsvPreviewResponse {
        rows,
        valid_count,
        error_count,
    })))
}

pub async fn csv_import(
    user: CurrentUser,
    svc: web::Data<Arc<TripService>>,
    body: web::Json<CsvImportRequest>,
) -> Result<HttpResponse, AppError> {
    match user.role {
        Role::SuperAdmin | Role::Accountant => {}
        _ => return Err(AppError::Forbidden("Only super_admin or accountant can import CSV trips".into())),
    }
    let body = body.into_inner();

    // Convert DTO rows to domain CsvPreviewRows
    let preview_rows: Vec<CsvPreviewRow> = body.rows.into_iter().map(|r| CsvPreviewRow {
        row_num: r.row_num,
        trip_date: r.trip_date,
        cash_aed: r.cash_aed,
        card_aed: r.card_aed,
        other_aed: r.other_aed,
        notes: r.notes,
        error: r.error,
        cap_warning: r.cap_warning,
    }).collect();

    let trips = svc.csv_import(user.id, &user.role, body.driver_id, preview_rows).await?;
    let resp: Vec<TripResponse> = trips.into_iter().map(TripResponse::from).collect();
    Ok(HttpResponse::Created().json(ApiResponse::ok(resp)))
}
