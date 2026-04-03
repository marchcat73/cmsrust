// src/handlers/user.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppState,
    entities::user,
    services::user_service::UserService,
    utils::jwt::Claims,
    utils::response::AppResponse,
    handlers::posts::ClaimsExtractor, // Используем тот же экстрактор
};

/// DTO для ответа (скрываем пароль и лишние поля)
#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub role: String,
    pub is_active: bool,
}

impl From<user::Model> for UserProfileResponse {
    fn from(model: user::Model) -> Self {
        Self {
            id: model.id.to_string(),
            username: model.username,
            email: model.email,
            display_name: model.display_name,
            bio: model.bio,
            avatar_url: model.avatar_url,
            role: format!("{:?}", model.role),
            is_active: model.is_active,
        }
    }
}

/// GET /api/users/me
/// Получить профиль текущего пользователя
pub async fn get_current_user(
    State(state): State<AppState>,
    claims: ClaimsExtractor,
) -> Result<Json<AppResponse<UserProfileResponse>>, (StatusCode, String)> {
    // Парсим UUID из claims.sub
    let user_id = Uuid::parse_str(&claims.0.sub)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid user ID in token".to_string()))?;

    let user = UserService::get_user_by_id(&state.db, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    Ok(Json(AppResponse::success(user.into())))
}

/// PUT /api/users/me
/// Обновить профиль текущего пользователя
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

pub async fn update_current_user(
    State(state): State<AppState>,
    claims: ClaimsExtractor,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<AppResponse<UserProfileResponse>>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.0.sub)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid user ID in token".to_string()))?;

    let user = UserService::update_profile(
        &state.db,
        user_id,
        payload.display_name,
        payload.bio,
        payload.avatar_url,
    )
    .await
    .map_err(|e| match e {
        sea_orm::DbErr::RecordNotFound(_) => (StatusCode::NOT_FOUND, "User not found".to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Update failed: {}", e)),
    })?;

    Ok(Json(AppResponse::success(user.into())))
}

// --- Админские функции (опционально) ---

/// GET /api/users/{id}
/// Получить любого пользователя по ID (только для админов)
pub async fn get_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    claims: ClaimsExtractor,
) -> Result<Json<AppResponse<UserProfileResponse>>, (StatusCode, String)> {
    // Проверка прав: только админ может смотреть других
    if claims.0.role != "Admin" && claims.0.role != "Editor" {
         // Разрешаем смотреть только свой профиль через этот эндпоинт, если не админ
         let my_id = Uuid::parse_str(&claims.0.sub).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid ID".to_string()))?;
         if my_id != id {
             return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
         }
    }

    let user = UserService::get_user_by_id(&state.db, id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "User not found".to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    Ok(Json(AppResponse::success(user.into())))
}
