use actix_web::{get, web, HttpResponse, Responder};
use crate::AppState;
use jag_core::types::*;
use std::str::FromStr;

/// Get the current status and result of a specific task.
#[get("/tasks/{id}")]
pub async fn get_task(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let task_id_str = path.into_inner();
    let task_id = match TaskId::from_str(&task_id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({ "error": "Invalid TaskId" })),
    };

    let engine = data.workflow_engine.read().await;
    let dag = engine.dag();

    match dag.get_task(&task_id) {
        Some(task) => HttpResponse::Ok().json(task),
        None => HttpResponse::NotFound().json(serde_json::json!({ "error": "Task not found" })),
    }
}
