use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
    Extension,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    EntityTrait, ColumnTrait, QueryFilter, QuerySelect,
    PaginatorTrait, ActiveModelTrait, Set, ActiveValue,
    RelationTrait, DbErr, DeleteResult
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
    entities::{post, user, category, tag, post_category, post_tag},
    services::post_service::PostService,
    services::comment_service::CommentService,
    utils::{jwt::Claims, response::AppResponse},
    AppState,
};

// ==================== REQUEST/RESPONSE DTO ====================

#[derive(Debug, Deserialize)]
pub struct ListPostsQuery {
    page: Option<u64>,
    per_page: Option<u64>,
    status: Option<post::PostStatus>,
    author_id: Option<Uuid>,
    category_id: Option<Uuid>,
    search: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 3, max = 200))]
    pub title: String,
    #[validate(length(min = 10))]
    pub content: String,
    pub excerpt: Option<String>,
    pub slug: Option<String>,
    pub status: Option<post::PostStatus>,
    pub category_ids: Option<Vec<Uuid>>,
    pub tag_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct GetPostQuery {
    /// Загружать ли связанные данные (автор, категории, теги)
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

// ==================== Кастомный экстрактор для Claims ====================

#[derive(Clone)]
pub struct ClaimsExtractor(pub Claims);

impl<S> axum::extract::FromRequestParts<S> for ClaimsExtractor
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut http::request::Parts,
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

pub fn posts_router() -> Router<AppState> {
    Router::new()
        // Публичные маршруты (можно вынести отдельно)
        .route("/posts", get(list_posts))
        .route("/posts/:id", get(get_post))
        .route("/posts/slug/:slug", get(get_post_by_slug))

        // Защищённые маршруты (требуют auth middleware)
        .route("/posts", post(create_post))
        .route("/posts/:id", put(update_post).delete(delete_post))
        .route("/posts/:id/restore", post(restore_post))  // восстановление
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

/// Получение поста по ID с опциональной загрузкой связанных данных
///
/// Пример запроса:
/// GET /api/posts/550e8400-e29b-41d4-a716-446655440000?with_relations=true
pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AppResponse<post::Model>>, StatusCode> {
    let post = post::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // ✅ Успешный ответ с данными
    Ok(Json(AppResponse::success(post)))
}

/// Получение поста по slug (для SEO-дружественных URL)
///
/// Пример: GET /api/posts/slug/moj-pervyj-post
pub async fn get_post_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<GetPostQuery>,
) -> Result<Json<AppResponse<PostWithRelations>>, (StatusCode, String)> {
    let post = post::Entity::find()
        .filter(post::Column::Slug.eq(&slug))
        .filter(post::Column::Status.eq(post::PostStatus::Published))
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    // Загружаем связанные данные (аналогично get_post)
    let author = post
        .find_related(user::Entity)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    let categories = post
        .find_related(category::Entity)
        .all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

    let tags = post
        .find_related(tag::Entity)
        .all(&state.db)
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

/// Обновление поста по ID
///
/// 🔐 Требует аутентификации и прав автора/администратора
///
/// Пример запроса:
/// PUT /api/posts/550e8400-e29b-41d4-a716-446655440000
/// {
///   "title": "Обновлённый заголовок",
///   "content": "Новый контент...",
///   "status": "published"
/// }
pub async fn update_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ClaimsExtractor(claims): ClaimsExtractor,
    Json(payload): Json<UpdatePostRequest>,
) -> Result<(StatusCode, Json<AppResponse<post::Model>>), (StatusCode, String)> {
    // 1. Находим существующий пост
    let existing_post = post::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    // 2. Проверка прав: только автор или админ может редактировать
    if existing_post.author_id.to_string() != claims.sub && claims.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "You can only edit your own posts".to_string()));
    }

    // 3. Подготовка активных полей для обновления
    let mut post_active: post::ActiveModel = existing_post.clone().into();

    // Обновляем только переданные поля (Option-поля позволяют частичное обновление)
    if let Some(title) = payload.title {
        post_active.title = Set(title);
        // Автогенерация slug если не передан явно
        if payload.slug.is_none() {
            post_active.slug = Set(PostService::generate_slug(&title));
        }
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

    if let Some(status) = payload.status {
        post_active.status = Set(status);

        // Авто-установка published_at при публикации
        if status == post::PostStatus::Published && existing_post.published_at.is_none() {
            post_active.published_at = Set(Some(Utc::now()));
        }
    }

    if let Some(featured_image) = payload.featured_image {
        post_active.featured_image = Set(featured_image);
    }

    if let Some(comment_status) = payload.comment_status {
        post_active.comment_status = Set(comment_status);
    }

    if let Some(published_at) = payload.published_at {
        post_active.published_at = Set(published_at);
    }

    // Всегда обновляем timestamp
    post_active.updated_at = Set(Utc::now());

    // 4. Сохраняем изменения
    let updated_post = post_active
        .update(&state.db)
        .await
        .map_err(|e| match e {
            DbErr::RecordNotFound(_) => (StatusCode::NOT_FOUND, "Post not found".to_string()),
            DbErr::Exec(_) => (StatusCode::CONFLICT, "Slug already exists".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Update failed: {}", e)),
        })?;

    // 5. Обновляем связи с категориями если переданы
    if let Some(cat_ids) = payload.category_ids {
        // Удаляем старые связи
        post_category::Entity::delete_many()
            .filter(post_category::Column::PostId.eq(id))
            .exec(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

        // Создаём новые
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

    // 6. Обновляем связи с тегами если переданы
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

/// Удаление поста по ID
///
/// 🔐 Требует аутентификации и прав автора/администратора
///
/// 💡 Реализовано как "мягкое удаление": пост помечается статусом "trash"
///    а не удаляется физически из БД (как в WordPress)
///
/// Пример: DELETE /api/posts/550e8400-e29b-41d4-a716-446655440000
pub async fn delete_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ClaimsExtractor(claims): ClaimsExtractor,
) -> Result<Json<AppResponse<String>>, (StatusCode, String)> {
    // 1. Находим пост
    let existing_post = post::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "Post not found".to_string()))?;

    // 2. Проверка прав
    if existing_post.author_id.to_string() != claims.sub && claims.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "You can only delete your own posts".to_string()));
    }

    // 3. Админ может удалить окончательно, остальные — только в "корзину"
    if claims.role == "admin" {
        // 🔥 Физическое удаление (только для админов!)
        let result: DeleteResult = post::Entity::delete_by_id(id)
            .exec(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete failed: {}", e)))?;

        if result.rows_affected == 0 {
            return Err((StatusCode::NOT_FOUND, "Post not found".to_string()));
        }

        Ok(Json(AppResponse::success("Post permanently deleted".to_string())))
    } else {
        // 🗂️ Мягкое удаление: меняем статус на Trash
        let mut post_active: post::ActiveModel = existing_post.into();
        post_active.status = Set(post::PostStatus::Trash);
        post_active.updated_at = Set(Utc::now());

        post_active
            .update(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update failed: {}", e)))?;

        Ok(Json(AppResponse::success("Post moved to trash".to_string())))
    }
}

/// Восстановление поста из "корзины" (trash → draft)
///
/// 🔐 Только автор или админ
///
/// POST /api/posts/:id/restore
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

    // Можно восстанавливать только посты из корзины
    if existing_post.status != post::PostStatus::Trash {
        return Err((StatusCode::BAD_REQUEST, "Post is not in trash".to_string()));
    }

    // Проверка прав
    if existing_post.author_id.to_string() != claims.sub && claims.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "Permission denied".to_string()));
    }

    // Меняем статус на Draft
    let mut post_active: post::ActiveModel = existing_post.into();
    post_active.status = Set(post::PostStatus::Draft);
    post_active.updated_at = Set(Utc::now());

    let restored = post_active
        .update(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Restore failed: {}", e)))?;

    Ok(Json(AppResponse::success(restored)))
}

pub async fn get_post_comments(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<AppResponse<Vec<CommentWithReplies>>>, StatusCode> {
    let comments = CommentService::get_comments_with_replies(&state.db, post_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AppResponse::success(comments)))
}
