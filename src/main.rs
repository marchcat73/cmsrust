// src/main.rs
use axum::{Router, http::Method, routing::{get, post, put, delete}};
use tower_http::{cors::{CorsLayer, Any}, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod handlers;
mod middleware;
mod services;
mod utils;
mod entities;

#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set");

    let db = config::database::connect_database(&database_url).await;

    let app_state = AppState { db, jwt_secret };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        // Публичные маршруты
        .route("/api/posts", get(handlers::posts::list_posts))
        .route("/api/posts/:id", get(handlers::posts::get_post))
        .route("/api/posts/slug/:slug", get(handlers::posts::get_post_by_slug))
        // Защищённые маршруты
        .route("/api/posts", post(handlers::posts::create_post))
        .route("/api/posts/:id", put(handlers::posts::update_post).delete(handlers::posts::delete_post))
        .route("/api/posts/:id/restore", post(handlers::posts::restore_post))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(app_state);

    let port = std::env::var("APP_PORT").unwrap_or_else(|_| "8000".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    tracing::info!("🚀 CMS server running on http://localhost:{}", port);

    axum::serve(listener, app).await.unwrap();
}
