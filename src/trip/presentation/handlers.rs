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
    use crate::common::validation::validate_amount;
    let body = body.into_inner();
    let actor_driver_id = resolve_driver_id(&svc, &user).await?;

    // Validate monetary amounts
    validate_amount("cash_aed", body.cash_aed)?;
    if let Some(v) = body.uber_cash_aed { validate_amount("uber_cash_aed", v)?; }
    if let Some(v) = body.bolt_cash_aed { validate_amount("bolt_cash_aed", v)?; }
    if let Some(v) = body.card_aed { validate_amount("card_aed", v)?; }
    if let Some(v) = body.other_aed { validate_amount("other_aed", v)?; }

    // uber_cash_aed takes priority; fall back to other_aed for backward compat
    let uber_cash = body.uber_cash_aed
        .or(body.other_aed)
        .unwrap_or(Decimal::ZERO);

    let (trip, conflict_warning) = svc.create(
        user.id,
        &user.role,
        actor_driver_id,
        body.driver_id,
        body.vehicle_id,
        body.trip_date,
        body.cash_aed,
        uber_cash,
        body.bolt_cash_aed.unwrap_or(Decimal::ZERO),
        body.card_aed.unwrap_or(Decimal::ZERO),
        body.notes,
    ).await?;

    let resp = CreateTripResponse {
        trip: TripResponse::from(trip),
        conflict_warning,
    };
    Ok(HttpResponse::Created().json(ApiResponse::ok(resp)))
}

pub async fn update_trip(
    user: CurrentUser,
    svc: web::Data<Arc<TripService>>,
    path: web::Path<Uuid>,
    body: web::Json<CreateTripRequest>,
) -> Result<HttpResponse, AppError> {
    let trip_id = path.into_inner();
    let body = body.into_inner();

    let uber_cash = body.uber_cash_aed
        .or(body.other_aed)
        .unwrap_or(Decimal::ZERO);

    let trip = svc.update(
        user.id,
        &user.role,
        trip_id,
        body.driver_id,
        body.vehicle_id,
        body.trip_date,
        body.cash_aed,
        uber_cash,
        body.bolt_cash_aed.unwrap_or(Decimal::ZERO),
        body.card_aed.unwrap_or(Decimal::ZERO),
        body.notes,
    ).await?;

    Ok(HttpResponse::Ok().json(ApiResponse::ok(TripResponse::from(trip))))
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
    let csv = "driver_id,trip_date,cash_aed,uber_cash_aed,bolt_cash_aed,card_aed,notes\n";
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
        uber_cash_aed: r.uber_cash_aed,
        bolt_cash_aed: r.bolt_cash_aed,
        card_aed: r.card_aed,
        notes: r.notes,
        error: r.error,
        cap_warning: r.cap_warning,
    }).collect();

    let trips = svc.csv_import(user.id, &user.role, body.driver_id, preview_rows).await?;
    let resp: Vec<TripResponse> = trips.into_iter().map(TripResponse::from).collect();
    Ok(HttpResponse::Created().json(ApiResponse::ok(resp)))
}

pub async fn export_csv(
    user: CurrentUser,
    svc: web::Data<Arc<TripService>>,
    query: web::Query<ListTripsQuery>,
) -> Result<HttpResponse, AppError> {
    match user.role {
        Role::SuperAdmin | Role::Accountant => {}
        _ => return Err(AppError::Forbidden("Only super_admin or accountant can export trips".into())),
    }

    let today = Local::now().date_naive();
    let from = query.from.unwrap_or_else(|| {
        NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today)
    });
    let to = query.to.unwrap_or(today);

    let trips = svc.list(&user.role, None, query.driver_id, from, to).await?;

    let mut csv = String::from("Date,Driver,Cash,Uber Cash,Bolt Cash,Card,Total,Source,Notes\n");
    for t in &trips {
        let notes = t.notes.as_deref().unwrap_or("").replace(',', ";");
        let driver = t.driver_name.replace(',', ";");
        let source = format!("{:?}", t.source).to_lowercase();
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            t.trip_date,
            driver,
            t.cash_aed,
            t.uber_cash_aed,
            t.bolt_cash_aed,
            t.card_aed,
            t.total(),
            source,
            notes,
        ));
    }

    let filename = format!("trips_{}_{}.csv", from, to);
    Ok(HttpResponse::Ok()
        .content_type("text/csv")
        .append_header(("Content-Disposition", format!("attachment; filename=\"{filename}\"")))
        .body(csv))
}
