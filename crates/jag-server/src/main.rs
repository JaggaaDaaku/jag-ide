use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::RwLock;
use std::time::Instant;
use clap::Parser;

use jag_core::config::Config;
use jag_db::Database;
use jag_workflow::engine::WorkflowEngine;
use jag_workflow::dag::WorkflowDag;
use jag_workspace::manager::WorkspaceManager;
use jag_sandbox::audit::AuditLogger;
use jag_models::router::ModelRouter;
use jag_models::ollama_client::OllamaClient;
use jag_git::repository::GitRepository;
use jag_core::telemetry::CrashReporter;

pub mod dto;
pub mod handlers;
pub mod middleware;

use jag_agents::browser::VisionAnalyzer;
 
 /// Shared application state across all API handlers.
 pub struct AppState {
     pub db: Arc<Database>,
     pub workflow_engine: Arc<RwLock<WorkflowEngine>>,
     pub workspace_mgr: Arc<WorkspaceManager>,
     pub audit_logger: Arc<AuditLogger>,
     pub model_router: Arc<RwLock<ModelRouter>>,
     pub vision_analyzer: Arc<VisionAnalyzer>,
     pub reference_store: jag_agents::browser::ReferenceStore,
     pub git_repo: Arc<Mutex<GitRepository>>,
     pub auth_service: Arc<middleware::auth::AuthService>,
     pub config: Arc<Config>,
     pub crash_reporter: Arc<CrashReporter>,
     pub start_time: Instant,
     pub broadcast_tx: tokio::sync::broadcast::Sender<serde_json::Value>,
 }
 
 static START_TIME: OnceLock<Instant> = OnceLock::new();
 
 #[derive(Parser, Debug)]
 #[command(author, version, about, long_about = None)]
 struct Args {
     /// Validate the configuration file and exit
     #[arg(long)]
     validate_config: bool,
 }

#[get("/health")]
async fn health_check(state: web::Data<AppState>) -> impl Responder {
    let uptime = START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0);
    
    // Quick DB check
    let db_status = if state.db.get_daily_usage_stats(1).await.is_ok() { "ok" } else { "error" };
    
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": "1.0.0",
        "uptime_seconds": uptime,
        "components": {
            "database": db_status,
            "ollama": "ok" // Simplified for now
        }
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting Jag IDE Backend Server...");

    let args = Args::parse();

    // Load configuration
    let config = Config::from_env().map_err(|e| {
        std::io::Error::other(format!("Config error: {}", e))
    })?;

    if args.validate_config {
        info!("Validating configuration...");
        match config.validate_llm_config().await {
            Ok(_) => {
                info!("✅ Configuration is valid.");
                return Ok(());
            }
            Err(e) => {
                error!("❌ Configuration validation failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Initialize Database (SQLite)
    let db = Database::new(&config.database.url).await.map_err(|e| {
        std::io::Error::other(format!("Database error: {}", e))
    })?;
    db.run_migrations().await.map_err(|e| {
        std::io::Error::other(format!("Migration error: {}", e))
    })?;
    let db = Arc::new(db);

    // Initialize components
    let workspace_path = std::path::Path::new("workspace");
    if !workspace_path.exists() {
        std::fs::create_dir_all(workspace_path)?;
    }
    
    let workspace_mgr = Arc::new(WorkspaceManager::new(workspace_path.to_path_buf()).map_err(|e| {
        std::io::Error::other(format!("Workspace error: {}", e))
    })?);
    let audit_logger = Arc::new(AuditLogger::new(db.clone(), config.auth.hmac_secret.clone()));
    
    // Initialize LLM Router
    let ollama_client = OllamaClient::new("http://localhost:11434");
    let mut model_router = ModelRouter::new(ollama_client);
    let _ = model_router.auto_configure().await;
    let model_router = Arc::new(RwLock::new(model_router));
    
    // Initialize Vision Analyzer
    let vision_analyzer = Arc::new(VisionAnalyzer::new(model_router.clone()));

    // Initialize Reference Store
    let reference_store = jag_agents::browser::ReferenceStore::new(".jag/references");

    // Initialize Workflow Engine
    let dag = WorkflowDag::new();
    let engine = WorkflowEngine::new(dag);
    let workflow_engine = Arc::new(RwLock::new(engine));

    // Initialize Git
    let git_repo = Arc::new(Mutex::new(GitRepository::init(workspace_path).map_err(|e| {
        std::io::Error::other(format!("Git error: {}", e))
    })?));

    // Initialize Auth Service
    let auth_service = Arc::new(middleware::auth::AuthService::new(&config.auth.jwt_secret));

    // Initialize Telemetry
    let crash_reporter = Arc::new(CrashReporter::new(&config));
    crash_reporter.init();

    let start_time = Instant::now();
    START_TIME.set(start_time).ok();

    let prometheus = actix_web_prometheus::PrometheusMetricsBuilder::new("jag")
        .endpoint("/metrics")
        .build()
        .unwrap();

    let (broadcast_tx, _) = tokio::sync::broadcast::channel(100);

    let state = web::Data::new(AppState {
        db,
        workflow_engine,
        workspace_mgr,
        audit_logger,
        model_router,
        vision_analyzer,
        reference_store,
        git_repo,
        auth_service,
        config: Arc::new(config),
        crash_reporter,
        start_time,
        broadcast_tx,
    });

    info!("Server listening on http://127.0.0.1:8080");

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(prometheus.clone())
            .app_data(state.clone())
            .service(health_check)
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/auth")
                            .service(handlers::auth::login)
                            .service(handlers::auth::callback)
                            .service(handlers::auth::refresh)
                    )
                    .service(handlers::dashboard::get_dashboard)
                    .service(
                        web::scope("/admin")
                            .service(handlers::admin::get_analytics)
                            .service(handlers::admin::get_audit_logs)
                            .service(handlers::admin::export_audit_logs)
                            .service(handlers::admin::verify_audit)
                            .service(handlers::admin::get_approvals)
                            .service(handlers::admin::decide_approval)
                    )
                    .service(handlers::agents::list_agents)
                    .service(handlers::agents::get_agent)
                    .service(handlers::agents::submit_task)
                    .service(handlers::workflow::start_workflow)
                    .service(handlers::workflow::get_workflow_status)
                    .service(handlers::workflow::get_workflow_history)
                    .service(handlers::workflow::get_visual_results)
                    .service(handlers::artifacts::list_artifacts)
                    .service(handlers::artifacts::get_artifact)
                    .service(handlers::artifacts::approve_artifact)
                    .service(handlers::artifacts::reject_artifact)
                    .service(handlers::workspace::list_files)
                    .service(handlers::workspace::read_file)
                    .service(handlers::workspace::invite_member)
                    .service(handlers::workspace::get_members)
                    .service(handlers::tasks::get_task)
                    .service(handlers::models::list_models)
                    .service(handlers::benchmarks::get_benchmarks)
                    .service(handlers::benchmarks::run_benchmark)
                    .service(handlers::git::get_git_status)
                    .service(handlers::git::get_git_log)
                    .service(handlers::git::list_pull_requests)
                    .service(handlers::git::trigger_pr_creation)
                    .service(handlers::git::get_coverage_report)
                    .service(handlers::browser::run_browser_test)
                    .service(handlers::browser::run_ensemble_visual_test)
                    .service(handlers::sync::workspace_sync)
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run();

    let server_handle = server.handle();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("Received shutdown signal, stopping server...");
        server_handle.stop(true).await;
    });

    server.await
}
