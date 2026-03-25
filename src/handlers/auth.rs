// src/handlers/auth.rs
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
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
    pub token: String,
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
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AppResponse<LoginResponse>>, (StatusCode, String)> {
    // 1. Проверяем пароль
    let user = UserService::verify_password_login(&payload.email, &payload.password, &state.db)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()))?;

    // 2. Генерируем JWT токен
    let token = jwt::create_token(
        user.id.to_string(),
        user.username.clone(),
        format!("{:?}", user.role),
        &state.jwt_secret,
    )
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Token generation failed".to_string()))?;

    let response = LoginResponse {
        token,
        user: UserInfo {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            role: format!("{:?}", user.role),
        },
    };

    Ok(Json(AppResponse::success(response)))
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
