// src/handlers/category.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppState,
    services::category_service::CategoryService,
    utils::response::AppResponse,
    entities::category,
};

// ==================== DTO ====================

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<Option<String>>,
    pub parent_id: Option<Option<Uuid>>,
}

// ==================== Handlers ====================

/// GET /api/categories - Список всех категорий
pub async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<AppResponse<Vec<category::Model>>>, StatusCode> {
    let categories = CategoryService::list_categories(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AppResponse::success(categories)))
}

/// GET /api/categories/:id - Получить одну категорию
pub async fn get_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AppResponse<category::Model>>, StatusCode> {
    let category = CategoryService::get_category_by_id(&state.db, id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(AppResponse::success(category)))
}

/// POST /api/categories - Создать категорию
pub async fn create_category(
    State(state): State<AppState>,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<AppResponse<category::Model>>), StatusCode> {
    let new_category = CategoryService::create_category(
        &state.db,
        payload.name,
        payload.slug,
        payload.description,
        payload.parent_id,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create category: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    Ok((StatusCode::CREATED, Json(AppResponse::success(new_category))))
}

/// PUT /api/categories/:id - Обновить категорию
pub async fn update_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCategoryRequest>,
) -> Result<Json<AppResponse<category::Model>>, (StatusCode, String)> {
    let updated = CategoryService::update_category(
        &state.db,
        id,
        payload.name,
        payload.slug,
        payload.description,
        payload.parent_id,
    )
    .await
    .map_err(|e| match e {
        sea_orm::DbErr::RecordNotFound(_) => (StatusCode::NOT_FOUND, "Category not found".to_string()),
        sea_orm::DbErr::Custom(msg) => (StatusCode::BAD_REQUEST, msg),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Update failed: {}", e)),
    })?;

    Ok(Json(AppResponse::success(updated)))
}

/// DELETE /api/categories/:id - Удалить категорию
pub async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AppResponse<String>>, (StatusCode, String)> {
    CategoryService::delete_category(&state.db, id)
        .await
        .map_err(|e| match e {
            sea_orm::DbErr::RecordNotFound(_) => (StatusCode::NOT_FOUND, "Category not found".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete failed: {}", e)),
        })?;

    Ok(Json(AppResponse::success("Category deleted successfully".to_string())))
}

// ==================== Router ====================

pub fn categories_router() -> Router<AppState> {
    Router::new()
        .route("/api/categories", get(list_categories).post(create_category))
        .route("/api/categories/{id}", get(get_category).put(update_category).delete(delete_category))
}
