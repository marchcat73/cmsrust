// src/handlers/posts.rs
use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, request::Parts},
    response::Json,
    // routing::{get, put, post},
    // Router,
};
use axum::extract::FromRequestParts;
use axum::extract::Json as AxumJson;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, DeleteResult, EntityTrait,
    QueryFilter, Set
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppState,
    entities::{category, post, post_category, post_tag, tag, user},
    services::post_service::PostService,
    utils::{jwt::Claims, response::AppResponse},
};

// ==================== REQUEST/RESPONSE DTO ====================

#[derive(Debug, Deserialize)]
pub struct ListPostsQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub status: Option<post::PostStatus>,
    pub author_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub search: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct GetPostQuery {
    #[serde(default)]
    pub with_relations: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub excerpt: Option<Option<String>>,
    pub slug: Option<String>,
    pub status: Option<post::PostStatus>,
    pub featured_image: Option<Option<Uuid>>,
    pub comment_status: Option<post::CommentStatus>,
    pub published_at: Option<Option<DateTime<Utc>>>,
    pub category_ids: Option<Vec<Uuid>>,
    pub tag_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize)]
pub struct PostWithRelations {
    #[serde(flatten)]
    pub post: post::Model,
    pub author: Option<user::Model>,
    pub categories: Vec<category::Model>,
    pub tags: Vec<tag::Model>,
}

// ==================== ClaimsExtractor ====================

#[derive(Clone)]
pub struct ClaimsExtractor(pub Claims);

impl<S> FromRequestParts<S> for ClaimsExtractor
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(ClaimsExtractor)
            .ok_or((StatusCode::UNAUTHORIZED, "Claims not found"))
    }
}

impl ClaimsExtractor {
    pub fn user_id(&self) -> Uuid {
        Uuid::parse_str(&self.0.sub).unwrap_or_else(|_| Uuid::new_v4())
    }
}

// ==================== Router ====================

// pub fn posts_router() -> Router<AppState> {
//     Router::new()
//         .route("/posts", get(list_posts))
//         .route("/posts/{id}", get(get_post))
//         .route("/posts/slug/{slug}", get(get_post_by_slug))
//         .route("/posts", post(create_post))
//         .route("/posts/{id}", put(update_post).delete(delete_post))
//         .route("/posts/{id}/restore", post(restore_post))
// }

// ==================== Handlers ====================

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
    claims: ClaimsExtractor,
    AxumJson(payload): AxumJson<CreatePostRequest>,
) -> Result<(StatusCode, Json<AppResponse<post::Model>>), StatusCode> {
    let post = PostService::create_post(&state.db, claims.user_id(), payload)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok((StatusCode::CREATED, Json(AppResponse::success(post))))
}

pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AppResponse<post::Model>>, StatusCode> {
    let post = post::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(AppResponse::success(post)))
}

pub async fn get_post_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(_query): Query<GetPostQuery>,
) -> Result<Json<AppResponse<PostWithRelations>>, (StatusCode, String)> {
    let post = post::Entity::find()
        .filter(post::Column::Slug.eq(&slug))
        .filter(post::Column::Status.eq(post::PostStatus::Published))
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    // Загружаем автора через сервис
    let author = PostService::get_post_author(&state.db, post.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    // Загружаем категории через сервис
    let categories = PostService::get_post_categories(&state.db, post.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    // Загружаем теги через сервис
    let tags = PostService::get_post_tags(&state.db, post.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    let response = PostWithRelations {
        post,
        author,
        categories,
        tags,
    };

    Ok(Json(AppResponse::success(response)))
}

pub async fn update_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ClaimsExtractor(claims): ClaimsExtractor,
    AxumJson(payload): AxumJson<UpdatePostRequest>,
) -> Result<(StatusCode, Json<AppResponse<post::Model>>), (StatusCode, String)> {
    let existing_post = post::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    if existing_post.author_id.to_string() != claims.sub && claims.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "You can only edit your own posts".to_string()));
    }

    let mut post_active: post::ActiveModel = existing_post.clone().into();

    // ✅ ИСПРАВЛЕНИЕ: клонируем title перед использованием
    if let Some(title) = payload.title {
        if payload.slug.is_none() {
            post_active.slug = Set(PostService::generate_slug(&title));
        }
        post_active.title = Set(title);
    }

    if let Some(slug) = payload.slug {
        post_active.slug = Set(slug);
    }

    if let Some(content) = payload.content {
        post_active.content = Set(content);
    }

    if let Some(excerpt) = payload.excerpt {
        post_active.excerpt = Set(excerpt);
    }

    // ✅ ИСПРАВЛЕНИЕ: клонируем status перед использованием
    if let Some(status) = payload.status {
        if status == post::PostStatus::Published && existing_post.published_at.is_none() {
            post_active.published_at = Set(Some(Utc::now().naive_utc()));
        }
        post_active.status = Set(status);
    }

    if let Some(featured_image) = payload.featured_image {
        post_active.featured_image = Set(featured_image);
    }

    if let Some(comment_status) = payload.comment_status {
        post_active.comment_status = Set(comment_status);
    }

    if let Some(published_at) = payload.published_at {
        post_active.published_at = Set(published_at.map(|dt| dt.naive_utc()));
    }

    post_active.updated_at = Set(Utc::now().naive_utc());

    let updated_post = post_active
        .update(&state.db)
        .await
        .map_err(|e| match e {
            DbErr::RecordNotFound(_) => (StatusCode::NOT_FOUND, "Post not found".to_string()),
            DbErr::Exec(_) => (StatusCode::CONFLICT, "Slug already exists".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Update failed: {}", e)),
        })?;

    // Обновляем связи с категориями
    if let Some(cat_ids) = payload.category_ids {
        post_category::Entity::delete_many()
            .filter(post_category::Column::PostId.eq(id))
            .exec(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

        for cat_id in cat_ids {
            post_category::ActiveModel {
                post_id: Set(id),
                category_id: Set(cat_id),
            }
            .insert(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;
        }
    }

    // Обновляем связи с тегами
    if let Some(tag_ids) = payload.tag_ids {
        post_tag::Entity::delete_many()
            .filter(post_tag::Column::PostId.eq(id))
            .exec(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

        for tag_id in tag_ids {
            post_tag::ActiveModel {
                post_id: Set(id),
                tag_id: Set(tag_id),
            }
            .insert(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;
        }
    }

    Ok((StatusCode::OK, Json(AppResponse::success(updated_post))))
}

pub async fn delete_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ClaimsExtractor(claims): ClaimsExtractor,
) -> Result<Json<AppResponse<String>>, (StatusCode, String)> {
    let existing_post = post::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    if existing_post.author_id.to_string() != claims.sub && claims.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "You can only delete your own posts".to_string()));
    }

    if claims.role == "admin" {
        let result: DeleteResult = post::Entity::delete_by_id(id)
            .exec(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete failed: {}", e)))?;

        if result.rows_affected == 0 {
            return Err((StatusCode::NOT_FOUND, "Post not found".to_string()));
        }

        Ok(Json(AppResponse::success("Post permanently deleted".to_string())))
    } else {
        let mut post_active: post::ActiveModel = existing_post.into();
        post_active.status = Set(post::PostStatus::Trash);
        post_active.updated_at = Set(Utc::now().naive_utc());

        post_active
            .update(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update failed: {}", e)))?;

        Ok(Json(AppResponse::success("Post moved to trash".to_string())))
    }
}

pub async fn restore_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ClaimsExtractor(claims): ClaimsExtractor,
) -> Result<Json<AppResponse<post::Model>>, (StatusCode, String)> {
    let existing_post = post::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    if existing_post.status != post::PostStatus::Trash {
        return Err((StatusCode::BAD_REQUEST, "Post is not in trash".to_string()));
    }

    if existing_post.author_id.to_string() != claims.sub && claims.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "Permission denied".to_string()));
    }

    let mut post_active: post::ActiveModel = existing_post.into();
    post_active.status = Set(post::PostStatus::Draft);
    post_active.updated_at = Set(Utc::now().naive_utc());

    let restored = post_active
        .update(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Restore failed: {}", e)))?;

    Ok(Json(AppResponse::success(restored)))
}
