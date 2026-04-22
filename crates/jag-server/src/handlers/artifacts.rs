use actix_web::{get, post, web, HttpResponse, Responder, HttpRequest, HttpMessage};
use crate::AppState;
use crate::dto::*;
use crate::middleware::auth::{AuthService, Claims, Permission};
use jag_core::types::{VerificationStatus, ArtifactId};
use std::str::FromStr;

fn extract_claims(req: &HttpRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}

/// List all artifacts produced by agents.
#[get("/artifacts")]
pub async fn list_artifacts(_data: web::Data<AppState>) -> impl Responder {
    // In Phase 2, we return a list of artifact metadata from the store/DB.
    HttpResponse::Ok().json(Vec::<ArtifactMetadataDto>::new())
}

/// Get the content of a specific artifact.
#[get("/artifacts/{id}")]
pub async fn get_artifact(path: web::Path<String>, _data: web::Data<AppState>) -> impl Responder {
    let _id = path.into_inner();
    // (Logic to load from ArtifactStore will be wired here)
    HttpResponse::Ok().json(serde_json::json!({ 
        "id": _id, 
        "content_base64": "SGVsbG8gSmFnIElERQ==",
        "type": "PRD"
    }))
}

/// Approve an artifact (VerificationStatus::Approved).
#[post("/artifacts/{id}/approve")]
pub async fn approve_artifact(
    path: web::Path<String>,
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let claims = match extract_claims(&req) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if !AuthService::has_permission(&claims, Permission::ApproveArtifacts) {
        return HttpResponse::Forbidden().finish();
    }

    let id_str = path.into_inner();
    if let Ok(id) = ArtifactId::from_str(&id_str) {
        let status = VerificationStatus::Approved;
        match state.db.update_artifact_status(&id, &status).await {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "status": "Approved" })),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    } else {
        HttpResponse::BadRequest().finish()
    }
}

/// Reject an artifact (VerificationStatus::Rejected).
#[post("/artifacts/{id}/reject")]
pub async fn reject_artifact(
    path: web::Path<String>,
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let claims = match extract_claims(&req) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if !AuthService::has_permission(&claims, Permission::ApproveArtifacts) {
        return HttpResponse::Forbidden().finish();
    }

    let id_str = path.into_inner();
    if let Ok(id) = ArtifactId::from_str(&id_str) {
        let status = VerificationStatus::Rejected;
        match state.db.update_artifact_status(&id, &status).await {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "status": "Rejected" })),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    } else {
        HttpResponse::BadRequest().finish()
    }
}
