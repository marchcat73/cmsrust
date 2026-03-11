use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter,
    QuerySelect, PaginatorTrait, Set, ActiveValue,
};
use crate::entities::{post, post_category, category};
use uuid::Uuid;

pub struct PostService;

impl PostService {
    pub async fn list_posts(
        db: &DatabaseConnection,
        page: u64,
        per_page: u64,
        status: Option<post::PostStatus>,
        author_id: Option<Uuid>,
        search: Option<String>,
    ) -> Result<Vec<post::Model>, sea_orm::DbErr> {
        let mut query = post::Entity::find();

        if let Some(s) = status {
            query = query.filter(post::Column::Status.eq(s));
        }
        if let Some(aid) = author_id {
            query = query.filter(post::Column::AuthorId.eq(aid));
        }
        if let Some(q) = search {
            query = query.filter(
                post::Column::Title.contains(&q)
                    .or(post::Column::Content.contains(&q))
            );
        }

        query
            .order_by_desc(post::Column::PublishedAt)
            .paginate(db, per_page)
            .fetch_page(page - 1)
            .await
    }

    pub async fn create_post(
        db: &DatabaseConnection,
        author_id: Uuid,
        CreatePostRequest: CreatePostRequest,
    ) -> Result<post::Model, sea_orm::DbErr> {
        let slug = data.slug.unwrap_or_else(|| Self::generate_slug(&data.title));

        let active_post = post::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(data.title),
            slug: Set(slug),
            content: Set(data.content),
            excerpt: Set(data.excerpt),
            status: Set(data.status.unwrap_or(post::PostStatus::Draft)),
            author_id: Set(author_id),
            published_at: Set(None),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        let post = active_post.insert(db).await?;

        // Привязка категорий если указаны
        if let Some(cat_ids) = data.category_ids {
            for cat_id in cat_ids {
                post_category::ActiveModel {
                    post_id: Set(post.id),
                    category_id: Set(cat_id),
                }
                .insert(db)
                .await?;
            }
        }

        Ok(post)
    }

    fn generate_slug(title: &str) -> String {
        title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
}
