use axum::{
    Router, http::Method, middleware::from_fn, routing::{get, post, put}
};
use sea_orm::DatabaseConnection;
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod handlers;
mod middleware;
mod services;
mod utils;
mod entities;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() {
    // Инициализация логирования
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Загрузка конфигурации
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set");

    // Подключение к БД
    let db = config::database::connect_database(&database_url).await;

    // Применение миграций при старте (опционально)
    // sea_orm_migration::Migrator::up(&db, None).await.unwrap();

    let app_state = AppState { db, jwt_secret };

    // CORS настройка
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_origin(Any);  // В продакшене указать конкретные домены!

    // Роуты
    let public_routes = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/register", post(handlers::auth::register))
        .route("/api/posts", get(handlers::posts::list_posts))
        .route("/api/posts/:id", get(handlers::posts::get_post))
        .route("/api/posts/slug/:slug", get(handlers::posts::get_post_by_slug));

    let protected_routes = Router::new()
        .route("/api/posts", post(handlers::posts::create_post))
        .route("/api/posts/:id", put(handlers::posts::update_post).delete(handlers::posts::delete_post))
        .route("/api/media", post(handlers::media::upload))
        .route("/api/users/me", get(handlers::users::get_current));

    let admin_routes = Router::new()
        .route("/api/users", get(handlers::users::list_users))
        .route("/api/settings", get(handlers::settings::get).put(handlers::settings::update));

    // Сборка приложения
    let app = Router::new()
        .merge(public_routes)
        .merge(
            protected_routes
                .route_layer(from_fn(middleware::auth::auth_middleware))
        )
        .merge(
            admin_routes
                .route_layer(from_fn(middleware::auth::auth_middleware))
                .route_layer(from_fn(|req, next| middleware::auth::require_role(req, next, "admin")))
        )
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
