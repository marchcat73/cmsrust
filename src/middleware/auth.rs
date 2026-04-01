// src/middleware/auth.rs
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{Response, IntoResponse, Redirect},
    http::{StatusCode},
};
use tower_cookies::Cookies;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use crate::entities::user::{self, Entity, UserRole};
use crate::services::user_service::UserService;
use crate::{AppState, utils::jwt};
use tracing::warn;


/// Гарантирует существование хотя бы одного администратора
pub async fn ensure_admin_user(db: &sea_orm::DatabaseConnection) {
    // Проверяем, есть ли вообще хоть один админ
    let admin_count = Entity::find()
        .filter(user::Column::Role.eq(UserRole::Admin))
        .count(db)
        .await
        .expect("Failed to count admins");

    if admin_count == 0 {
        println!("⚠️ Администраторы не найдены. Создаем первого...");

        let username = std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
        let email = std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string());
        let password = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string());

        match UserService::create_user_if_not_exists(
            db,
            &username,
            &email,
            &password,
            UserRole::Admin,
        ).await {
            Ok(_) => println!("🎉 Администратор успешно создан!"),
            Err(e) => eprintln!("❌ Ошибка создания админа: {}", e),
        }
    } else {
        println!("✅ Администраторы уже существуют (найдено: {})", admin_count);
    }
}



/// Функция проверки, используемая внутри middleware
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let cookies = request
        .extensions()
        .get::<Cookies>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let token = cookies
        .get(std::env::var("AUTH_COOKIE_NAME").unwrap_or_else(|_| "cms_auth_token".to_string()).as_str())
        .map(|c| c.value().to_string());

    match token {
        Some(token_str) => {
            match jwt::verify_token(&token_str, &state.jwt_secret) {
                Ok(claims) => {
                    request.extensions_mut().insert(claims);
                    Ok(next.run(request).await)
                }
                Err(_) => {
                    warn!("Invalid token");
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
        }
        None => {
            // Проверяем, ждет ли клиент HTML (страница админки) или JSON (API)
            let accepts_html = request
                .headers()
                .get(axum::http::header::ACCEPT)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.contains("text/html"))
                .unwrap_or(false);

            if accepts_html {
                // Редирект на логин для браузера
                Ok(Redirect::to("/login").into_response())
            } else {
                // 401 для API
                Err(StatusCode::UNAUTHORIZED)
            }
        },
    }
}



