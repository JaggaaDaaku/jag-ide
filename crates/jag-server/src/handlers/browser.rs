use actix_web::{post, web, HttpResponse, Responder};
use jag_agents::browser::{
    BrowserAgent, BrowserTester, 
    EnsembleVisualRequest
};
use jag_core::types::{ViewportSpec, DesignSpec, Priority, BrowserConfig};
use serde::{Deserialize, Serialize};
use crate::AppState;
use tracing::{info, error};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct BrowserTestRequest {
    pub url: String,
    pub artifact_id: String,
    pub selector: Option<String>,
}

#[derive(Deserialize)]
pub struct EnsembleVisualTestRequest {
    pub prompt: String,
    pub artifact_id: String,
    pub design_reference: Option<DesignSpec>,
    pub candidate_models: Vec<String>,
    pub viewport: Option<ViewportSpec>,
    pub priority: Option<Priority>,
    pub enable_judge: Option<bool>,
}

#[derive(Serialize)]
pub struct TaskAccepted {
    pub task_id: String,
    pub message: String,
}

#[post("/browser/test")]
pub async fn run_browser_test(
    state: web::Data<AppState>,
    req: web::Json<BrowserTestRequest>,
) -> impl Responder {
    let task_id = Uuid::new_v4().to_string();
    info!("Accepted browser test task {} for artifact: {}", task_id, req.artifact_id);

    // Context for background task
    let workspace_root = state.workspace_mgr.root().to_path_buf();
    let vision_analyzer = state.vision_analyzer.clone();
    let reference_store = state.reference_store.clone();
    let model_router = state.model_router.clone();
    let db = state.db.clone();
    let cost_config = state.config.cost.clone();
    
    let url = req.url.clone();
    let artifact_id = req.artifact_id.clone();
    let task_id_clone = task_id.clone();

    // Spawn background task
    tokio::spawn(async move {
        info!("Background task {}: Initializing browser agent", task_id_clone);
        
        let agent = BrowserAgent::new(workspace_root, BrowserConfig::default());
        let model_router_read = model_router.read().await;
        // In local scope, model_router_read won't work easily if we need Sync/Send.
        // We'll clone the router before moving into the thread is safer.
        let router_clone = std::sync::Arc::new((*model_router_read).clone());
        
        let tester = BrowserTester::new(
            agent, 
            Some(vision_analyzer), 
            reference_store, 
            router_clone,
            db,
            cost_config,
        );

        match tester.run_ui_smoke_test(&artifact_id, &url).await {
            Ok(result) => {
                info!("Background task {}: Success. Screenshot: {:?}, Vision Result: {:?}", 
                    task_id_clone, result.screenshot_path, result.vision_analysis);
            }
            Err(e) => {
                error!("Background task {}: Failed: {}", task_id_clone, e);
            }
        }
    });

    HttpResponse::Accepted().json(TaskAccepted {
        task_id,
        message: "Visual verification task queued".into(),
    })
}

#[post("/browser/ensemble-test")]
pub async fn run_ensemble_visual_test(
    state: web::Data<AppState>,
    req: web::Json<EnsembleVisualTestRequest>,
) -> impl Responder {
    let task_id = Uuid::new_v4().to_string();
    info!("Accepted ensemble visual test task {} for artifact: {}", task_id, req.artifact_id);

    let workspace_root = state.workspace_mgr.root().to_path_buf();
    let vision_analyzer = state.vision_analyzer.clone();
    let reference_store = state.reference_store.clone();
    let model_router = state.model_router.clone();
    
    let payload = req.into_inner();
    let task_id_clone = task_id.clone();
    let db = state.db.clone();
    let cost_config = state.config.cost.clone();

    tokio::spawn(async move {
        info!("Background task {}: Starting ensemble evaluation", task_id_clone);
        
        let agent = BrowserAgent::new(workspace_root, BrowserConfig::default());
        let model_router_read = model_router.read().await;
        let router_clone = std::sync::Arc::new((*model_router_read).clone());

        let tester = BrowserTester::new(
            agent,
            Some(vision_analyzer),
            reference_store,
            router_clone,
            db,
            cost_config,
        );

        let request = EnsembleVisualRequest {
            prompt: payload.prompt,
            artifact_id: payload.artifact_id.parse().unwrap_or_default(),
            design_reference: payload.design_reference,
            candidate_models: payload.candidate_models,
            viewport: payload.viewport.unwrap_or_default(),
            priority: payload.priority.unwrap_or(Priority::Normal),
            enable_judge: payload.enable_judge,
        };

        match tester.run_ensemble_visual_test(request).await {
            Ok(result) => {
                info!("Background task {}: Ensemble successful. Winner: {}", 
                    task_id_clone, result.winning_candidate.model_name);
            }
            Err(e) => {
                error!("Background task {}: Ensemble failed: {}", task_id_clone, e);
            }
        }
    });

    HttpResponse::Accepted().json(TaskAccepted {
        task_id,
        message: "Ensemble visual evaluation queued".into(),
    })
}
