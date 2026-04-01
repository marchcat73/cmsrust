// src/main.rs
use axum::{
    Router, http::{Method, StatusCode, header}, middleware as axum_middleware, response::{IntoResponse, Response}, routing::{delete, get, post, put}
};
use std::time::Duration;
use tower_http::{cors::{CorsLayer, AllowOrigin}, trace::TraceLayer};
use tower_http::services::ServeDir;
use tower_cookies::CookieManagerLayer;
use axum::middleware::from_fn_with_state;
use cookie::Key;
use tera::Tera;
use std::sync::Arc;
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
    pub tera: Arc<Tera>,
    pub current_theme: String,
    pub jwt_secret: String,
    pub cookie_key: Key,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    // Логирование
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();


    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set");

        // Получаем ключ шифрования кук из env
    let cookie_secret = std::env::var("COOKIE_SECRET_KEY")
        .expect("COOKIE_SECRET_KEY must be set in .env");
    let cookie_key = Key::from(cookie_secret.as_bytes());

    let db = config::database::connect_database(&database_url).await;


    // Инициализация Tera
    // Ищем шаблоны в папке themes/*/templates/
    // Для простоты пока берем одну тему "default"
    let tera = match Tera::new("themes/default/templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Ошибка парсинга шаблонов: {}", e);
            std::process::exit(1);
        }
    };

    // Создаем админа при старте (опционально)
    middleware::auth::ensure_admin_user(&db).await;

    let app_state = AppState {
        db,
        jwt_secret,
        cookie_key,
        tera: Arc::new(tera),
        current_theme: "default".to_string(),
    };

    // Создаем сервис для раздачи статических файлов из папки themes/default/static
    let static_files = ServeDir::new("themes/default/static")
        .append_index_html_on_directories(false);

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _req_head| {
            // Разрешаем все локальные адреса
            let origin_str = origin.as_bytes();
            origin_str == b"http://localhost:8000" ||
            origin_str == b"http://127.0.0.1:8000" ||
            // Можно добавить regex для всех localhost портов
            origin_str.starts_with(b"http://localhost:") ||
            origin_str.starts_with(b"http://127.0.0.1:")
        }))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::COOKIE,
            header::ORIGIN,
        ])
        .allow_credentials(true)
        .expose_headers([
            header::SET_COOKIE,
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
        ])
        .max_age(Duration::from_secs(3600)); // В продакшене указать конкретные домены!

    let public_routes = Router::new()
        .route("/health", get(|| async { "OK" }))
        // Публичные маршруты
        .route("/api/posts", get(handlers::posts::list_posts))
        .route("/api/posts/{id}", get(handlers::posts::get_post))
        .route("/api/posts/slug/{slug}", get(handlers::posts::get_post_by_slug))
        // Авторизация (Логин/Регистрация)
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/register", post(handlers::auth::register))
        .route("/api/auth/logout", post(handlers::auth::logout))
        .route("/", get(handlers::theme::index))
        .route("/login", get(handlers::theme::login_page))
        .route("/register", get(handlers::theme::register_page))
        .route("/post/{slug}", get(handlers::theme::single_post));

        // 2. Создаем роутер для ЗАЩИЩЕННЫХ маршрутов
    let protected_routes = Router::new()
        .route("/api/posts", post(handlers::posts::create_post))
        .route("/api/posts/{id}", put(handlers::posts::update_post).delete(handlers::posts::delete_post))
        .route("/api/posts/{id}/restore", post(handlers::posts::restore_post))
        // Слой кук (если используете) должен быть самым внешним или перед auth_middleware внутри protected_routes
        .layer(CookieManagerLayer::new())
        // Применяем middleware ТОЛЬКО к этому роутеру
        .layer(from_fn_with_state(app_state.clone(), middleware::auth::auth_middleware));

    // 3. Объединяем роутеры
    // Важно: порядок не имеет значения для merge, но логически мы сливаем их в один
    let app = public_routes
        .merge(protected_routes)
        .layer(cors)
        // Подключение статики
        .nest_service("/static/themes/default", static_files)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let port = std::env::var("APP_PORT").unwrap_or_else(|_| "8000".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    tracing::info!("🚀 CMS server running on http://localhost:{}", port);

    axum::serve(listener, app).await.unwrap();
}

