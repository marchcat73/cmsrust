use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, QuerySelect, PaginatorTrait, ActiveModelTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::{post, user},
    services::post_service::PostService,
    utils::response::AppResponse,
};

#[derive(Debug, Deserialize)]
pub struct ListPostsQuery {
    page: Option<u64>,
    per_page: Option<u64>,
    status: Option<post::PostStatus>,
    author_id: Option<Uuid>,
    category_id: Option<Uuid>,
    search: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub slug: Option<String>,
    pub status: Option<post::PostStatus>,
    pub category_ids: Option<Vec<Uuid>>,
    pub tag_ids: Option<Vec<Uuid>>,
}

pub fn posts_router() -> Router<AppState> {
    Router::new()
        .route("/posts", get(list_posts).post(create_post))
        .route("/posts/:id", get(get_post).put(update_post).delete(delete_post))
        .route("/posts/slug/:slug", get(get_post_by_slug))
}

pub async fn list_posts(
    State(state): State<AppState>,
    Query(query): Query<ListPostsQuery>,
) -> Result<Json<AppResponse<Vec<post::Model>>>, StatusCode> {
    let posts = PostService::list_posts(
        &state.db,
        query.page.unwrap_or(1),
        query.per_page.unwrap_or(10),
        query.status,
        query.author_id,
        query.search,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AppResponse::success(posts)))
}

pub async fn create_post(
    State(state): State<AppState>,
    claims: ClaimsExtractor,  // Custom extractor для claims
    Json(payload): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<AppResponse<post::Model>>), StatusCode> {
    let post = PostService::create_post(&state.db, claims.user_id, payload)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok((StatusCode::CREATED, Json(AppResponse::success(post))))
}

// ... остальные handlers (get_post, update_post, delete_post)
