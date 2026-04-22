use actix_web::{get, post, web, HttpResponse, Responder};
use crate::AppState;
// use jag_core::types::*;
use jag_benchmarks::BenchmarkRunner;
use jag_models::router::ModelRouting;

/// List recent benchmarks for a specific model.
#[get("/benchmarks/{model_name}")]
pub async fn get_benchmarks(
    data: web::Data<AppState>,
    model_name: web::Path<String>,
) -> impl Responder {
    match data.db.get_model_benchmarks(&model_name).await {
        Ok(benches) => HttpResponse::Ok().json(benches),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Trigger a benchmark run for a specific model.
#[post("/benchmarks/{model_name}/run")]
pub async fn run_benchmark(
    data: web::Data<AppState>,
    model_name: web::Path<String>,
) -> impl Responder {
    let runner = BenchmarkRunner::new(data.model_router.read().await.clone_box());
    
    let prompt = "Explain quantum computing in 50 words.";
    match runner.benchmark_prompt(&model_name, prompt).await {
        Ok(result) => {
            if let Err(e) = data.db.create_benchmark(&result).await {
                return HttpResponse::InternalServerError().body(format!("Failed to save benchmark: {}", e));
            }
            HttpResponse::Ok().json(result)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Benchmark failed: {}", e)),
    }
}
