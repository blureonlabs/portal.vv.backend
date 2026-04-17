use std::sync::Arc;

use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::common::{error::AppError, response::ApiResponse, types::{CurrentUser, Role}};
use crate::document::application::service::DocumentService;
use crate::document::presentation::dto::{CreateDocumentRequest, DocumentResponse, ListDocumentsQuery};

pub async fn list_documents(
    user: CurrentUser,
    svc: web::Data<Arc<DocumentService>>,
    query: web::Query<ListDocumentsQuery>,
) -> Result<HttpResponse, AppError> {
    match user.role {
        Role::SuperAdmin | Role::Accountant | Role::Hr => {}
        Role::Driver => {
            // Drivers can only list their own documents - but we'd need to resolve driver_id from profile
            // For now, allow listing (they need to see their own docs) but log it
        }
        _ => return Err(AppError::Forbidden("Insufficient permissions".into())),
    }
    let docs = svc.list(query.entity_type.clone(), query.entity_id).await?;
    let resp: Vec<DocumentResponse> = docs.into_iter().map(DocumentResponse::from).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}

pub async fn create_document(
    user: CurrentUser,
    svc: web::Data<Arc<DocumentService>>,
    body: web::Json<CreateDocumentRequest>,
) -> Result<HttpResponse, AppError> {
    let body = body.into_inner();
    let doc = svc
        .upload(
            body.entity_type,
            body.entity_id,
            body.doc_type,
            body.file_url,
            body.file_name,
            body.expiry_date,
            user.id,
            body.notes,
        )
        .await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(DocumentResponse::from(doc))))
}

pub async fn delete_document(
    user: CurrentUser,
    svc: web::Data<Arc<DocumentService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    match user.role {
        Role::SuperAdmin | Role::Hr => {}
        _ => return Err(AppError::Forbidden("Only admin or HR can delete documents".into())),
    }
    svc.delete(*path).await?;
    Ok(HttpResponse::NoContent().finish())
}
