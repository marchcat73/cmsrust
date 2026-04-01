// src/handlers/admin.rs
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    http::StatusCode,
    Extension,
};
use serde_json::json;
use tera::Context;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, QueryOrder};
use crate::{AppState, utils::jwt::Claims, entities::post};

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
