use std::sync::Arc;

use actix_web::{web, HttpResponse};
use chrono::NaiveDate;
use uuid::Uuid;

use crate::common::{
    error::AppError,
    response::ApiResponse,
    types::{CurrentUser, Role},
};
use crate::salary::application::service::{GenerateRequest, SalaryService};
use crate::salary::presentation::dto::{GenerateSalaryBody, ListSalaryQuery, SalaryResponse};

fn parse_month(s: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(&format!("{}-01", s), "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest(format!("Invalid month '{}' — use YYYY-MM", s)))
}

fn require_admin(user: &CurrentUser) -> Result<(), AppError> {
    match user.role {
        Role::SuperAdmin | Role::Accountant => Ok(()),
        _ => Err(AppError::Forbidden("Admin access required".into())),
    }
}

pub async fn generate_salary(
    user: CurrentUser,
    svc: web::Data<Arc<SalaryService>>,
    body: web::Json<GenerateSalaryBody>,
) -> Result<HttpResponse, AppError> {
    require_admin(&user)?;
    let period_month = parse_month(&body.period_month)?;
    let req = GenerateRequest {
        driver_id:               body.driver_id,
        period_month,
        salary_type:             body.salary_type.clone(),
        total_earnings_aed:      body.total_earnings_aed,
        total_cash_received_aed: body.total_cash_received_aed,
        total_cash_submit_aed:   body.total_cash_submit_aed,
        cash_not_handover_aed:   body.cash_not_handover_aed,
        car_charging_aed:        body.car_charging_aed,
        car_charging_used_aed:   body.car_charging_used_aed,
        salik_used_aed:          body.salik_used_aed,
        salik_refund_aed:        body.salik_refund_aed,
        rta_fine_aed:            body.rta_fine_aed,
        card_service_charges_aed: body.card_service_charges_aed,
        room_rent_aed:           body.room_rent_aed,
        generated_by:            user.id,
    };
    let salary = svc.generate(req).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(SalaryResponse::from(salary))))
}

pub async fn list_salaries(
    user: CurrentUser,
    svc: web::Data<Arc<SalaryService>>,
    query: web::Query<ListSalaryQuery>,
) -> Result<HttpResponse, AppError> {
    require_admin(&user)?;
    let month = query.month.as_deref().map(parse_month).transpose()?;
    let salaries = svc.list(query.driver_id, month).await?;
    let resp: Vec<SalaryResponse> = salaries.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}

pub async fn get_salary(
    user: CurrentUser,
    svc: web::Data<Arc<SalaryService>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_admin(&user)?;
    let salary = svc.get(*id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(SalaryResponse::from(salary))))
}
