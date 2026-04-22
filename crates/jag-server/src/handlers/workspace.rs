use actix_web::{get, post, web, HttpResponse, Responder, HttpRequest, HttpMessage};
use crate::AppState;
use serde::Deserialize;
use jag_core::types::{WorkspaceId, WorkspaceRole, UserId};
use crate::middleware::auth::{AuthService, Claims, Permission};
use tracing::error;
use std::str::FromStr;

#[derive(Deserialize)]
pub struct InviteRequest {
    pub email: String,
    pub role: WorkspaceRole,
}

fn extract_claims(req: &HttpRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}

/// List all files in the safe workspace directory.
#[get("/workspace/files")]
pub async fn list_files(data: web::Data<AppState>) -> impl Responder {
    match data.workspace_mgr.list_files("") {
        Ok(files) => HttpResponse::Ok().json(files),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

/// Read the content of a specific file from the workspace.
#[get("/workspace/files/{path:.*}")]
pub async fn read_file(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let file_path = path.into_inner();
    match data.workspace_mgr.read_file(&file_path) {
        Ok(content) => HttpResponse::Ok().body(content),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

#[post("/workspaces/{id}/invite")]
pub async fn invite_member(
    path: web::Path<String>,
    req: HttpRequest,
    body: web::Json<InviteRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let claims = match extract_claims(&req) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    // Need to check if user has admin rights for this specific workspace
    // For now, we'll assume global admin or developer role has permission
    if !AuthService::has_permission(&claims, Permission::ManageUsers) {
        return HttpResponse::Forbidden().finish();
    }

    let workspace_id = match WorkspaceId::from_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let user_id = match UserId::from_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    // Find the user by email to add them immediately
    // In reality, if they don't exist, we'd add them to `workspace_invites` table
    match state.db.get_user_by_email(&body.email).await {
        Ok(Some(user)) => {
            match state.db.add_workspace_member(&workspace_id, &user.id, body.role.clone(), Some(&user_id)).await {
                Ok(_) => {
                    // Send audit log
                    let _ = state.audit_logger.log_signed(
                        Some(workspace_id),
                        Some(user_id),
                        None,
                        "workspace_invite_accepted",
                        "workspace_member",
                        &format!("User added to workspace with role {:?}", body.role),
                        jag_sandbox::audit::AuditResult::Success,
                    ).await;
                    HttpResponse::Ok().json(serde_json::json!({ "status": "success", "message": "User added" }))
                },
                Err(e) => {
                    error!("Failed to add workspace member: {}", e);
                    HttpResponse::InternalServerError().finish()
                }
            }
        },
        Ok(None) => {
            // User doesn't exist, would create an invite link here
            // Dummy implementation
            HttpResponse::Ok().json(serde_json::json!({ "status": "pending", "message": "Invitation sent" }))
        },
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/workspaces/{id}/members")]
pub async fn get_members(
    path: web::Path<String>,
    req: HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let _claims = match extract_claims(&req) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let workspace_id = match WorkspaceId::from_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match state.db.get_workspace_members(&workspace_id).await {
        Ok(members) => HttpResponse::Ok().json(members),
        Err(e) => {
            error!("Failed to fetch workspace members: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
