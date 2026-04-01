// src/middleware/auth.rs
use axum::{
    extract::{Request, State, Extension},
    middleware::Next,
    response::Response,
    http::{StatusCode},
};
use tower_cookies::Cookies;
use cookie::Cookie;
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

/// Middleware-обертка для защищенных хендлеров
/// Принимает хендлер и возвращает новый хендлер, который сначала проверяет авторизацию
pub fn require_auth<H, T, S>(handler: H) -> impl Fn(State<AppState>, Extension<Cookies>, Request) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<Response, StatusCode>> + Send>>
where
    H: axum::handler::Handler<T, S> + Clone + Send + 'static,
    T: 'static,
    S: Send + Sync + 'static,
{
    move |state, Extension(cookies), request| {
        let handler = handler.clone();
        let state = state.clone();

        Box::pin(async move {
            // 1. Пытаемся получить токен из куки
            let token = cookies
                .get(std::env::var("AUTH_COOKIE_NAME").unwrap_or_else(|_| "cms_auth_token".to_string()).as_str())
                .and_then(|c| Some(c.value().to_string()));

            match token {
                Some(token_str) => {
                    // 2. Верифицируем JWT
                    match jwt::verify_token(&token_str, &state.jwt_secret) {
                        Ok(claims) => {
                            // 3. Добавляем Claims в Extensions, чтобы хендлер мог их взять
                            let mut request = request;
                            request.extensions_mut().insert(claims);

                            // 4. Вызываем оригинальный хендлер
                            // Примечание: Для упрощения здесь используется прямой вызов,
                            // но в реальном проекте лучше использовать axum::middleware::from_fn_with_state
                            // и отдельную функцию проверки.
                            // Ниже приведен более правильный вариант через from_fn.

                            // Так как синтаксис обертки хендлера сложен,
                            // давайте используем стандартный подход axum::middleware::from_fn_with_state
                            // См. обновленный main.rs ниже для правильного подключения.
                            Ok(Response::new(axum::body::Body::empty())) // Заглушка, см. ниже правильный код
                        }
                        Err(_) => {
                            warn!("Invalid token in cookie");
                            Err(StatusCode::UNAUTHORIZED)
                        }
                    }
                }
                None => Err(StatusCode::UNAUTHORIZED),
            }
        })
    }
}

// --- ПРАВИЛЬНЫЙ ПОДХОД ЧЕРЕЗ from_fn ---

use axum::middleware::from_fn_with_state;

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
        None => Err(StatusCode::UNAUTHORIZED),
    }
}



