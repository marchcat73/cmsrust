// src/handlers/media.rs
use axum::{
    extract::{Multipart, Path, Query, State},
    http::{StatusCode, HeaderMap},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppState,
    entities::media,
    services::media_service::MediaService,
    utils::response::AppResponse,
    utils::jwt,
};

#[derive(Debug, Deserialize)]
pub struct ListMediaQuery {
    page: Option<u64>,
    per_page: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct MediaResponse {
    pub id: String,
    pub filename: String,
    pub filepath: String,
    pub mime_type: String,
    pub size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub description: Option<String>,
    pub url: String,
}

impl From<media::Model> for MediaResponse {
    fn from(model: media::Model) -> Self {
        let filename_for_url = model.filepath.split('/').last().unwrap_or("");
        Self {
            id: model.id.to_string(),
            filename: model.filename,
            filepath: model.filepath.clone(),
            mime_type: model.mime_type,
            size: model.size,
            width: model.width,
            height: model.height,
            alt_text: model.alt_text,
            caption: model.caption,
            description: model.description,
            url: format!("/static/uploads/{}", filename_for_url),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateMediaRequest {
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub description: Option<String>,
}

/// GET /api/media - Список всех медиафайлов
pub async fn list_media(
    State(state): State<AppState>,
    Query(query): Query<ListMediaQuery>,
) -> Result<Json<AppResponse<Vec<MediaResponse>>>, StatusCode> {
    let media_list = MediaService::list_media(
        &state.db,
        query.page.unwrap_or(1),
        query.per_page.unwrap_or(20),
        None, // Не фильтруем по uploader_id
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response: Vec<MediaResponse> = media_list.into_iter().map(|m| m.into()).collect();

    Ok(Json(AppResponse::success(response)))
}

/// Helper to extract JWT token from cookie header string
fn extract_token_from_cookie(cookie_header: &str, cookie_name: &str) -> Option<String> {
    cookie_header.split(';').find(|c| c.trim().starts_with(cookie_name))
        .map(|c| c.trim().trim_start_matches(cookie_name).trim_start_matches('=').to_string())
        .filter(|v| !v.is_empty())
}

/// POST /api/media/upload - Загрузка файла
pub async fn upload_media(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<AppResponse<MediaResponse>>, (StatusCode, String)> {
    // Извлекаем токен из cookie
    let cookie_name = std::env::var("AUTH_COOKIE_NAME").unwrap_or_else(|_| "cms_auth_token".to_string());

    // Получаем cookie из заголовка Cookie
    let auth_cookie_value = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookie_header| extract_token_from_cookie(cookie_header, &cookie_name));

    let token = match auth_cookie_value {
        Some(val) if !val.is_empty() => Some(val),
        _ => None
    };

    let claims = match token {
        Some(token_str) => jwt::verify_token(&token_str, &state.jwt_secret)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?,
        None => return Err((StatusCode::UNAUTHORIZED, "No token provided".to_string())),
    };

    let uploader_id = Uuid::parse_str(&claims.sub)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid user ID in token: {}", e)))?;

    // Получаем первый файл
    let file = multipart.next_field().await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read file: {}", e)))?
        .ok_or((StatusCode::BAD_REQUEST, "No file uploaded".to_string()))?;

    let media = MediaService::upload_file(
        &state.db,
        file,
        uploader_id,
        "uploads", // Директория для загрузок
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Upload failed: {}", e)))?;

    Ok(Json(AppResponse::success(media.into())))
}

/// GET /api/media/:id - Получить информацию о файле
pub async fn get_media(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AppResponse<MediaResponse>>, StatusCode> {
    let media = MediaService::get_media_by_id(&state.db, id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(AppResponse::success(media.into())))
}

/// PUT /api/media/:id - Обновить метаданные
pub async fn update_media(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMediaRequest>,
) -> Result<Json<AppResponse<MediaResponse>>, (StatusCode, String)> {
    let media = MediaService::update_media(
        &state.db,
        id,
        payload.alt_text,
        payload.caption,
        payload.description,
    )
    .await
    .map_err(|e| match e {
        sea_orm::DbErr::RecordNotFound(_) => (StatusCode::NOT_FOUND, "Media not found".to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "Update failed".to_string()),
    })?;

    Ok(Json(AppResponse::success(media.into())))
}

/// DELETE /api/media/:id - Удалить файл
pub async fn delete_media(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AppResponse<String>>, (StatusCode, String)> {
    MediaService::delete_media(&state.db, id)
        .await
        .map_err(|e| match e {
            sea_orm::DbErr::RecordNotFound(_) => (StatusCode::NOT_FOUND, "Media not found".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Delete failed".to_string()),
        })?;

    Ok(Json(AppResponse::success("Media deleted successfully".to_string())))
}

/// Роутер для медиа
pub fn media_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/media", get(list_media).post(upload_media))
        .route("/media/{id}", get(get_media).put(update_media).delete(delete_media))
        .with_state(state)
}
