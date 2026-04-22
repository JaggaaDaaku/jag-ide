use actix_web::{get, web, HttpResponse, Responder};
use crate::AppState;

/// List all available Ollama models.
#[get("/models")]
pub async fn list_models(data: web::Data<AppState>) -> impl Responder {
    let router = data.model_router.read().await;
    let models = router.available_models();
    
    HttpResponse::Ok().json(serde_json::json!({
        "models": models
    }))
}
