use actix_web::{get, web, HttpResponse, Responder};
use crate::AppState;
use crate::dto::*;

/// Aggregated dashboard state for single-poll frontend efficiency.
#[get("/dashboard")]
pub async fn get_dashboard(data: web::Data<AppState>) -> impl Responder {
    // 1. Get agent states
    let agent_state_dtos = Vec::new();
    // In a real impl, we'd query the agents from the registry. 
    // For Phase 2 mock, we'll return static roles with Idle status if not found.
    // (This will be expanded as we integrate the actual AgentRegistry in Layer 3 final)
    
    // 2. Get workflow status
    let engine = data.workflow_engine.read().await;
    let dag = engine.dag();
    
    let workflow_dto = WorkflowStatusDto {
        is_complete: dag.is_complete(),
        has_failures: dag.has_failures(),
        status_counts: dag.status_counts(),
    };

    // 3. Get recent artifacts
    let recent_artifacts = data.db.get_recent_artifacts(10).await.unwrap_or_default();

    // 4. Get available models
    let router = data.model_router.read().await;
    let available_models = router.available_models();

    HttpResponse::Ok().json(DashboardResponse {
        agents: agent_state_dtos,
        workflow: workflow_dto,
        recent_artifacts,
        available_models,
    })
}
