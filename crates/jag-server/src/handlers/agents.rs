use actix_web::{get, post, web, HttpResponse, Responder};
use crate::AppState;
use crate::dto::*;
use jag_core::types::*;

/// List all registered agents and their current status.
#[get("/agents")]
pub async fn list_agents(_data: web::Data<AppState>) -> impl Responder {
    // In Phase 2, we return the state of the 4 core agents from the engine.
    HttpResponse::Ok().json(Vec::<AgentStateDto>::new())
}

/// Get detailed status for a single agent.
#[get("/agents/{id}")]
pub async fn get_agent(path: web::Path<String>, _data: web::Data<AppState>) -> impl Responder {
    let _id = path.into_inner();
    HttpResponse::Ok().json(serde_json::json!({ "id": _id, "status": "Idle" }))
}

/// Submit a manual task to an agent by its role.
#[post("/agents/{role}/task")]
pub async fn submit_task(
    role: web::Path<String>,
    req: web::Json<SubmitTaskRequest>,
    data: web::Data<AppState>
) -> impl Responder {
    let role_str = role.into_inner();
    
    // Convert string to AgentRole enum
    let _agent_role = match role_str.to_lowercase().as_str() {
        "planner" => AgentRole::Planner,
        "backend" => AgentRole::Backend,
        "frontend" => AgentRole::Frontend,
        "integration" => AgentRole::Integration,
        _ => return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Invalid role" })),
    };

    let task = Task {
        id: TaskId::new(),
        agent_id: None, // Engine will assign
        task_type: req.task_type.clone(),
        status: TaskStatus::Pending,
        priority: Priority::Normal,
        payload: req.payload.clone(),
        dependencies: vec![],
    };

    let mut engine = data.workflow_engine.write().await;
    engine.dag_mut().add_task(task.clone());

    HttpResponse::Accepted().json(serde_json::json!({ "task_id": task.id.to_string() }))
}
