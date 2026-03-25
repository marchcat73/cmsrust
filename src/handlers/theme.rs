use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    http::StatusCode,
};
use serde_json::json;
use tera::Context;
use uuid::Uuid;

use crate::{AppState, entities::{post, user}};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, QuerySelect, QueryOrder};


// Главная страница (список постов)
pub async fn index(State(state): State<AppState>) -> impl IntoResponse {
    // Получаем посты из БД
    let posts = post::Entity::find()
        .filter(post::Column::Status.eq(post::PostStatus::Published))
        .order_by_desc(post::Column::PublishedAt)
        .limit(10)
        .all(&state.db)
        .await;

    match posts {
        Ok(posts_list) => {
            let mut context = Context::new();
            context.insert("posts", &posts_list);
            context.insert("current_theme", &state.current_theme);
            // Можно добавить текущего пользователя, если есть сессия

            match state.tera.render("index.html", &context) {
                Ok(body) => Html(body).into_response(),
                Err(e) => {
                    tracing::error!("Ошибка рендеринга: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Ошибка шаблона").into_response()
                }
            }
        }
        Err(e) => {
            tracing::error!("Ошибка БД: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Ошибка БД").into_response()
        }
    }
}

// Страница одного поста
pub async fn single_post(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let post_result = post::Entity::find()
        .filter(post::Column::Slug.eq(&slug))
        .filter(post::Column::Status.eq(post::PostStatus::Published))
        .one(&state.db)
        .await;

    match post_result {
        Ok(Some(post)) => {
            // Загружаем автора отдельно (так как у нас пока нет автоматического join в сущности для фронтенда)
            let author = user::Entity::find_by_id(post.author_id)
                .one(&state.db)
                .await
                .ok()
                .flatten();

            let mut context = Context::new();
            context.insert("post", &post);
            context.insert("author", &author);
            context.insert("current_theme", &state.current_theme);

            // Заглушка для категорий и комментариев (можно доработать через сервисы)
            context.insert("categories", &Vec::<serde_json::Value>::new());
            context.insert("comments", &Vec::<serde_json::Value>::new());

            match state.tera.render("single.html", &context) {
                Ok(body) => Html(body).into_response(),
                Err(e) => {
                    tracing::error!("Ошибка рендеринга поста: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Ошибка шаблона").into_response()
                }
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Пост не найден").into_response(),
        Err(e) => {
            tracing::error!("Ошибка БД: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Ошибка БД").into_response()
        }
    }
}
