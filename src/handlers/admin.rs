// src/handlers/admin.rs
use axum::{
    Extension, Json, extract::{Path, State}, http::StatusCode, response::{Html, IntoResponse}
};
use serde_json::json;
use serde::{Deserialize};
use tera::{Context };
use sea_orm::{EntityTrait, QueryOrder, ActiveModelTrait, Set};
use uuid::Uuid;
use chrono::Utc;

use crate::{AppState, utils::jwt::Claims, entities::post, entities::user, handlers::posts::ClaimsExtractor};

/// Главная страница админки (Список постов)
pub async fn dashboard(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>, // Получаем данные пользователя из middleware
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("current_theme", &state.current_theme);
    context.insert("user", &json!({
        "username": claims.username,
        "role": claims.role
    }));

    // Получаем все посты (можно добавить фильтрацию по автору, если не админ)
    let posts = match post::Entity::find()
        .order_by_desc(post::Column::CreatedAt)
        .all(&state.db)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("DB Error in admin dashboard: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Database Error</h1>")).into_response();
        }
    };

    context.insert("posts", &posts);

    match state.tera.render("admin/dashboard.html", &context) {
        Ok(body) => Html(body).into_response(),
        Err(e) => {
            tracing::error!("Template Error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<h1>Template Error</h1><p>{}</p>", e))).into_response()
        }
    }
}

/// Страница создания поста
pub async fn create_post_page(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("current_theme", &state.current_theme);
    context.insert("user", &json!({
        "username": claims.username,
        "role": claims.role
    }));
    context.insert("is_edit", &false);
    // post не передаем, так как создаем новый

    match state.tera.render("admin/post_form.html", &context) {
        Ok(body) => Html(body).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("Error: {}", e))).into_response(),
    }
}

/// Страница редактирования поста
pub async fn edit_post_page(
    State(state): State<AppState>,
    Path(id): Path<String>, // Берем как строку для простоты парсинга UUID
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    // Парсим UUID
    let uuid = match uuid::Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, Html("Invalid ID")).into_response(),
    };

    let post = match post::Entity::find_by_id(uuid).one(&state.db).await {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, Html("Post not found")).into_response(),
        Err(e) => {
            tracing::error!("DB Error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Html("DB Error")).into_response();
        }
    };

    let mut context = Context::new();
    context.insert("current_theme", &state.current_theme);
    context.insert("user", &json!({
        "username": claims.username,
        "role": claims.role
    }));
    context.insert("is_edit", &true);
    context.insert("post", &post);

    match state.tera.render("admin/post_form.html", &context) {
        Ok(body) => Html(body).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("Error: {}", e))).into_response(),
    }
}

/// Страница профиля текущего пользователя (/admin/users/me)
pub async fn profile_page(
    State(state): State<AppState>,
    claims: ClaimsExtractor,
) -> impl IntoResponse {
    // Получаем ID пользователя из токена
    let user_id = match Uuid::parse_str(&claims.0.sub) {
        Ok(id) => id,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Invalid User ID</h1>")).into_response(),
    };

    // Загружаем данные пользователя
    let user_model = match user::Entity::find_by_id(user_id).one(&state.db).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::NOT_FOUND, Html("<h1>User not found</h1>")).into_response(),
        Err(e) => {
            tracing::error!("DB Error loading profile: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Database Error</h1>")).into_response();
        }
    };

    let mut context = Context::new();
    context.insert("current_theme", &state.current_theme);
    context.insert("user", &json!({
        "username": claims.0.username,
        "role": claims.0.role,
        "email": user_model.email,
        "display_name": user_model.display_name,
        "bio": user_model.bio,
        "avatar_url": user_model.avatar_url,
        "id": user_model.id.to_string()
    }));
    context.insert("title", "Мой профиль");

    match state.tera.render("admin/profile.html", &context) {
        Ok(body) => Html(body).into_response(),
        Err(e) => {
            tracing::error!("Template Error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<h1>Template Error</h1><p>{}</p>", e))).into_response()
        }
    }
}

/// API endpoint для обновления профиля (PUT /api/users/me)
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    // Можно добавить поле current_password и new_password для смены пароля
}

pub async fn update_profile_api(
    State(state): State<AppState>,
    claims: ClaimsExtractor,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.0.sub)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid User ID".to_string()))?;

    let mut active_user = user::Entity::find_by_id(user_id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let mut user_active: user::ActiveModel = active_user.into();

    if let Some(name) = payload.display_name {
        user_active.display_name = Set(Some(name));
    }
    if let Some(bio) = payload.bio {
        user_active.bio = Set(Some(bio));
    }
    if let Some(avatar) = payload.avatar_url {
        user_active.avatar_url = Set(Some(avatar));
    }

    user_active.updated_at = Set(Utc::now());

    let updated = user_active.update(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update failed: {}", e)))?;

    Ok(Json(json!({
        "success": true,
        "message": "Profile updated successfully",
        "data": {
            "display_name": updated.display_name,
            "bio": updated.bio,
            "avatar_url": updated.avatar_url
        }
    })))
}
