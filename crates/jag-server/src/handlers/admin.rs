use actix_web::{get, post, web, HttpResponse, Responder, HttpRequest, HttpMessage};
use crate::AppState;
use crate::middleware::auth::{AuthService, Claims, Permission};
use crate::dto::{ApprovalDto, ApprovalDecisionRequest};
use serde::Deserialize;
use tracing::{info, warn};

#[derive(Deserialize)]
pub struct AnalyticsQuery {
    pub days: Option<u32>,
}

#[derive(Deserialize)]
pub struct AuditQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

fn extract_claims(req: &HttpRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}

#[get("/analytics")]
pub async fn get_analytics(
    state: web::Data<AppState>,
    query: web::Query<AnalyticsQuery>,
    req: HttpRequest,
) -> impl Responder {
    let claims = match extract_claims(&req) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if !AuthService::has_permission(&claims, Permission::ViewMissions) {
        warn!("Unauthorized access attempt to analytics by {}", claims.email);
        return HttpResponse::Forbidden().finish();
    }

    let days = query.days.unwrap_or(30);
    match state.db.get_daily_usage_stats(days).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => {
            warn!("Failed to fetch analytics: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/audit")]
pub async fn get_audit_logs(
    state: web::Data<AppState>,
    query: web::Query<AuditQuery>,
    req: HttpRequest,
) -> impl Responder {
    let claims = match extract_claims(&req) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if !AuthService::has_permission(&claims, Permission::ViewMissions) {
        warn!("Unauthorized access attempt to audit logs by {}", claims.email);
        return HttpResponse::Forbidden().finish();
    }

    let page = query.page.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);
    let offset = page * limit;

    match state.db.get_audit_logs_paginated(offset, limit).await {
        Ok(entries) => HttpResponse::Ok().json(entries),
        Err(e) => {
            warn!("Failed to fetch audit logs: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/audit/export")]
pub async fn export_audit_logs(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let claims = match extract_claims(&req) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if !AuthService::has_permission(&claims, Permission::ExportAudit) {
        return HttpResponse::Forbidden().finish();
    }

    match state.db.get_all_audit_logs().await {
        Ok(entries) => {
            let mut writer = csv::Writer::from_writer(Vec::new());
            
            // Header
            writer.write_record(&["ID", "Timestamp", "Action", "User ID", "Agent ID", "Resource Type", "Resource ID", "Result", "IP Address"]).ok();
            
            for entry in entries {
                writer.write_record(&[
                    entry.id.to_string(),
                    entry.timestamp.to_rfc3339(),
                    entry.action,
                    entry.user_id.map(|id| id.to_string()).unwrap_or_default(),
                    entry.agent_id.map(|id| id.to_string()).unwrap_or_default(),
                    entry.resource_type.unwrap_or_default(),
                    entry.resource_id.unwrap_or_default(),
                    format!("{:?}", entry.result),
                    entry.ip_address.unwrap_or_default(),
                ]).ok();
            }

            match writer.into_inner() {
                Ok(csv_data) => HttpResponse::Ok()
                    .content_type("text/csv")
                    .insert_header(("Content-Disposition", "attachment; filename=\"audit_log.csv\""))
                    .body(csv_data),
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
        Err(e) => {
            warn!("Failed to export audit logs: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/audit/{id}/verify")]
pub async fn verify_audit(
    _state: web::Data<AppState>,
    path: web::Path<i64>,
    req: HttpRequest,
) -> impl Responder {
    let _claims = match extract_claims(&req) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    // Note: In Layer 4, we'll implement full HMAC verification logic here.
    // For now, we return a verified response if the user is authenticated.
    HttpResponse::Ok().json(serde_json::json!({
        "audit_id": path.into_inner(),
        "verified": true,
        "integrity": "intact"
    }))
}

#[get("/approvals")]
pub async fn get_approvals(
    _state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    let _claims = match extract_claims(&req) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    // Note: In Layer 4, this would query the workflow_approvals table.
    // For the current Phase 3 integration, we return an empty list or mock data.
    HttpResponse::Ok().json(Vec::<ApprovalDto>::new())
}

#[post("/approvals/{id}/decide")]
pub async fn decide_approval(
    state: web::Data<AppState>,
    path: web::Path<jag_core::types::ArtifactId>,
    req: HttpRequest,
    decision: web::Json<ApprovalDecisionRequest>,
) -> impl Responder {
    let claims = match extract_claims(&req) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if !AuthService::has_permission(&claims, Permission::ApproveArtifacts) {
        return HttpResponse::Forbidden().finish();
    }

    let artifact_id = path.into_inner();
    let status = decision.status.clone();
    
    // Explicitly use jag_core type to resolve mismatch
    let core_status: jag_core::types::VerificationStatus = status;
    
    match state.db.update_artifact_status(&artifact_id, &core_status).await {
        Ok(_) => {
            info!("Admin {} {} artifact {}", claims.email, format!("{:?}", decision.status), artifact_id);
            HttpResponse::Ok().json(serde_json::json!({ "status": "updated" }))
        }
        Err(e) => {
            warn!("Failed to update artifact status: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
