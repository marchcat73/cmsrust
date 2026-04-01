// src/handlers/auth.rs
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use axum::response::Result as AxumResult;
use cookie::time::Duration;
use serde::{Deserialize, Serialize};
use axum_extra::extract::CookieJar;
use crate::{
    AppState,
    services::user_service::UserService,
    utils::{jwt, response::AppResponse},
    entities::user,
};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub message: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
}

pub async fn login(
    State(state): State<AppState>,
        mut jar: CookieJar, // Магическим образом извлекает и позволяет менять куки
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), (StatusCode, String)> {
    let user = UserService::verify_password_login(&payload.email, &payload.password, &state.db)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()))?;

    let token = jwt::create_token(
        user.id.to_string(),
        user.username.clone(),
        format!("{:?}", user.role),
        &state.jwt_secret,
    ).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Token generation failed".to_string()))?;

    // Создаем куку
    let cookie = cookie::Cookie::build((std::env::var("AUTH_COOKIE_NAME").unwrap_or_else(|_| "cms_auth_token".to_string()), token))
        .path("/")
        .http_only(true) // Защита от XSS
        .secure(false) // В продакшене ставьте true (требует HTTPS)
        .same_site(cookie::SameSite::Lax)
        .max_age(Duration::days(1)) // Срок жизни 1 день
        .finish();

    // Добавляем куку в Jar (она автоматически зашифруется благодаря PrivateCookieLayer)
    jar = jar.add(cookie);

    let response = LoginResponse {
        message: "Login successful".to_string(),
        user: UserInfo {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            role: format!("{:?}", user.role),
        },
    };

    Ok((jar, Json(response)))
}

pub async fn logout(jar: CookieJar) -> AxumResult<CookieJar> {
    // Удаляем токен cookie
    let jar = jar.remove(
        axum_extra::extract::cookie::Cookie::build((std::env::var("AUTH_COOKIE_NAME").unwrap_or_else(|_| "cms_auth_token".to_string()), ""))
            .path("/")
    );

    Ok(jar)
}


// Опционально: Регистрация нового пользователя (обычно только Subscriber)
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AppResponse<user::Model>>), (StatusCode, String)> {
    // Простая валидация можно добавить здесь

    match UserService::create_user_if_not_exists(
        &state.db,
        &payload.username,
        &payload.email,
        &payload.password,
        user::UserRole::Subscriber, // Новые пользователи всегда подписчики
    ).await {
        Ok(user) => Ok((StatusCode::CREATED, Json(AppResponse::success(user)))),
        Err(_) => Err((StatusCode::CONFLICT, "User already exists".to_string())),
    }
}


