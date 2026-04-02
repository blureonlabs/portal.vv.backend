use std::sync::Arc;
use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::auth::presentation::handlers::require_role;
use crate::common::{error::AppError, response::ApiResponse, types::{CurrentUser, Role}};
use crate::vehicle::application::service::VehicleService;
use crate::vehicle::presentation::dto::{
    AddServiceRecordRequest, AssignDriverRequest, CreateVehicleRequest,
    UpdateVehicleRequest, VehicleAssignmentResponse, VehicleResponse, VehicleServiceResponse,
};

pub async fn list_vehicles(
    user: CurrentUser,
    svc: web::Data<Arc<VehicleService>>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant, Role::Hr])?;
    let vehicles: Vec<VehicleResponse> = svc.list().await?.into_iter().map(VehicleResponse::from).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(vehicles)))
}

pub async fn get_vehicle(
    user: CurrentUser,
    svc: web::Data<Arc<VehicleService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant, Role::Hr])?;
    let v = svc.get(path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(VehicleResponse::from(v))))
}

pub async fn create_vehicle(
    user: CurrentUser,
    svc: web::Data<Arc<VehicleService>>,
    body: web::Json<CreateVehicleRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let b = body.into_inner();
    let role = user.role.clone();
    let v = svc.create(
        user.id, &role,
        b.plate_number, b.make, b.model, b.year, b.color,
        b.registration_date, b.registration_expiry, b.insurance_expiry,
    ).await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(VehicleResponse::from(v))))
}

pub async fn update_vehicle(
    user: CurrentUser,
    svc: web::Data<Arc<VehicleService>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateVehicleRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    let b = body.into_inner();
    let role = user.role.clone();
    let v = svc.update(
        user.id, &role, path.into_inner(),
        b.plate_number, b.make, b.model, b.year, b.color,
        b.registration_date, b.registration_expiry, b.insurance_expiry,
    ).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(VehicleResponse::from(v))))
}

pub async fn assign_driver(
    user: CurrentUser,
    svc: web::Data<Arc<VehicleService>>,
    path: web::Path<Uuid>,
    body: web::Json<AssignDriverRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Hr])?;
    let role = user.role.clone();
    svc.assign_driver(user.id, &role, path.into_inner(), body.driver_id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(())))
}

pub async fn unassign_driver(
    user: CurrentUser,
    svc: web::Data<Arc<VehicleService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Hr])?;
    let role = user.role.clone();
    svc.unassign_driver(user.id, &role, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(())))
}

pub async fn list_assignments(
    user: CurrentUser,
    svc: web::Data<Arc<VehicleService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant, Role::Hr])?;
    let assignments: Vec<VehicleAssignmentResponse> = svc.list_assignments(path.into_inner()).await?
        .into_iter().map(VehicleAssignmentResponse::from).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(assignments)))
}

pub async fn add_service_record(
    user: CurrentUser,
    svc: web::Data<Arc<VehicleService>>,
    path: web::Path<Uuid>,
    body: web::Json<AddServiceRecordRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    let b = body.into_inner();
    let role = user.role.clone();
    let record = svc.add_service_record(
        user.id, &role, path.into_inner(),
        b.service_date, b.service_type, b.description, b.cost, b.next_due,
    ).await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(VehicleServiceResponse::from(record))))
}

pub async fn list_service_records(
    user: CurrentUser,
    svc: web::Data<Arc<VehicleService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant, Role::Hr])?;
    let records: Vec<VehicleServiceResponse> = svc.list_service_records(path.into_inner()).await?
        .into_iter().map(VehicleServiceResponse::from).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(records)))
}
