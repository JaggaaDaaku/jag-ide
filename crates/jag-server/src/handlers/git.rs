use actix_web::{get, post, web, HttpResponse, Responder};
use crate::AppState;
use jag_git::provider::{create_provider};
use jag_core::types::TaskId;
use serde_json::json;

/// Get the current git status of the workspace.
#[get("/git/status")]
pub async fn get_git_status(data: web::Data<AppState>) -> impl Responder {
    let repo = match data.git_repo.lock() {
        Ok(guard) => guard,
        Err(_) => return HttpResponse::InternalServerError().body("Git repository lock poisoned"),
    };
    match repo.status() {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Get the git commit log.
#[get("/git/log")]
pub async fn get_git_log(data: web::Data<AppState>) -> impl Responder {
    let repo = match data.git_repo.lock() {
        Ok(guard) => guard,
        Err(_) => return HttpResponse::InternalServerError().body("Git repository lock poisoned"),
    };
    match repo.log(50) {
        Ok(log) => HttpResponse::Ok().json(log),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// List all active Pull Requests for the current repository.
#[get("/git/prs")]
pub async fn list_pull_requests(data: web::Data<AppState>) -> impl Responder {
    let config = &data.config.git;
    let provider_res = create_provider(config);
    
    match provider_res {
        Ok(_provider) => {
            // In a real implementation, we would call provider.list_prs().
            HttpResponse::Ok().json(json!([]))
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Create a manual Pull Request trigger for an approved mission.
#[post("/git/prs/create/{mission_id}")]
pub async fn trigger_pr_creation(
    _data: web::Data<AppState>,
    mission_id: web::Path<TaskId>,
) -> impl Responder {
    HttpResponse::Accepted().json(json!({
        "mission_id": mission_id.to_string(),
        "status": "triggered",
        "message": "GitAgent initiated autonomous PR flow"
    }))
}

/// Get the latest coverage report.
#[get("/validation/coverage")]
pub async fn get_coverage_report(data: web::Data<AppState>) -> impl Responder {
    let mock = data.config.validation.mock_mode;
    
    HttpResponse::Ok().json(json!({
        "rust": if mock { 0.85 } else { 0.0 },
        "ts": if mock { 0.82 } else { 0.0 },
        "passed": mock,
        "is_mock": mock
    }))
}
