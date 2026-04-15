use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use std::sync::Arc;
use jag_core::config::Config;
use jag_db::Database;

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "healthy", "version": "0.1.0" }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting Jag IDE Backend Server...");

    // Load configuration
    let config = Config::from_env().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("Config error: {}", e))
    })?;

    // Initialize Database (SQLite)
    let db = Database::new(&config.database.url).await.map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("Database error: {}", e))
    })?;
    db.run_migrations().await.map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("Migration error: {}", e))
    })?;
    let db = Arc::new(db);

    info!("Server listening on http://127.0.0.1:8080");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(db.clone()))
            .service(health_check)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
