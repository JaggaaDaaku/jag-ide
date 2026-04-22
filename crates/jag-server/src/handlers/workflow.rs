use actix_web::{get, post, web, HttpResponse, Responder, HttpRequest, HttpMessage};
use crate::AppState;
use crate::dto::*;
use crate::middleware::auth::{AuthService, Claims, Permission};
use jag_core::types::*;
use uuid::Uuid;

fn extract_claims(req: &HttpRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}

/// Start a new workflow based on a project description.
/// This initializes the DAG with the first task (GeneratePRD).
#[post("/workflow/start")]
pub async fn start_workflow(
    req: web::Json<StartWorkflowRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let claims = match extract_claims(&http_req) {
        Some(c) => c,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if !AuthService::has_permission(&claims, Permission::StartMission) {
        return HttpResponse::Forbidden().finish();
    }

    jag_workflow::engine::ACTIVE_MISSIONS.inc();

    let mut engine = data.workflow_engine.write().await;
    
    // Clear existing DAG for Phase 2 demo simplicity (In Phase 3 we'd manage multiple sessions)
    // engine.reset(); 

    let prd_task = Task {
        id: TaskId::new(),
        agent_id: None,
        task_type: TaskType::GeneratePRD,
        status: TaskStatus::Pending,
        priority: Priority::High,
        payload: serde_json::json!({
            "description": req.description,
            "tech_stack": "React, TypeScript, Rust, SQLite"
        }),
        dependencies: vec![],
    };

    engine.dag_mut().add_task(prd_task.clone());

    HttpResponse::Accepted().json(serde_json::json!({ 
        "workflow_id": Uuid::new_v4().to_string(),
        "initial_task_id": prd_task.id.to_string() 
    }))
}

/// Get the current status of the workflow DAG.
#[get("/workflow/status")]
pub async fn get_workflow_status(data: web::Data<AppState>) -> impl Responder {
    let engine = data.workflow_engine.read().await;
    let dag = engine.dag();

    HttpResponse::Ok().json(WorkflowStatusDto {
        is_complete: dag.is_complete(),
        has_failures: dag.has_failures(),
        status_counts: dag.status_counts(),
    })
}

/// Get history of past workflows.
#[get("/workflow/history")]
pub async fn get_workflow_history(_state: web::Data<AppState>) -> impl Responder {
    // Placeholder for database query
    HttpResponse::Ok().json(serde_json::json!([]))
}

/// Get visual verification results for a specific workflow.
#[get("/workflow/{id}/visual-results")]
pub async fn get_visual_results(
    _state: web::Data<AppState>,
    _id: web::Path<TaskId>,
) -> impl Responder {
    // Placeholder for visual result retrieval
    HttpResponse::Ok().json(serde_json::json!([]))
}
