// src/services/post_service.rs
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set
};
use crate::entities::{post, post_category, post_tag, category, tag, user};
use crate::handlers::posts::CreatePostRequest;
use uuid::Uuid;
use chrono::Utc;

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
        data: CreatePostRequest,
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
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        let post = active_post.insert(db).await?;

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

        if let Some(tag_ids) = data.tag_ids {
            for tag_id in tag_ids {
                post_tag::ActiveModel {
                    post_id: Set(post.id),
                    tag_id: Set(tag_id),
                }
                .insert(db)
                .await?;
            }
        }

        Ok(post)
    }

    pub fn generate_slug(title: &str) -> String {
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

    pub async fn get_post_author(
        db: &DatabaseConnection,
        post_id: Uuid,
    ) -> Result<Option<user::Model>, sea_orm::DbErr> {
        user::Entity::find()
            .join(
                JoinType::InnerJoin,
                post::Entity::belongs_to(user::Entity)
                    .from(post::Column::AuthorId)
                    .to(user::Column::Id)
                    .into()
            )
            .filter(post::Column::Id.eq(post_id))
            .one(db)
            .await
    }

    pub async fn get_post_categories(
        db: &DatabaseConnection,
        post_id: Uuid,
    ) -> Result<Vec<category::Model>, sea_orm::DbErr> {
        category::Entity::find()
            .join(
                JoinType::InnerJoin,
                post_category::Entity::belongs_to(category::Entity)
                    .from(post_category::Column::CategoryId)
                    .to(category::Column::Id)
                    .into()
            )
            .join(
                JoinType::InnerJoin,
                post::Entity::belongs_to(post_category::Entity)
                    .from(post::Column::Id)
                    .to(post_category::Column::PostId)
                    .into()
            )
            .filter(post::Column::Id.eq(post_id))
            .all(db)
            .await
    }

    pub async fn get_post_tags(
        db: &DatabaseConnection,
        post_id: Uuid,
    ) -> Result<Vec<tag::Model>, sea_orm::DbErr> {
        tag::Entity::find()
            .join(
                JoinType::InnerJoin,
                post_tag::Entity::belongs_to(tag::Entity)
                    .from(post_tag::Column::TagId)
                    .to(tag::Column::Id)
                    .into()
            )
            .join(
                JoinType::InnerJoin,
                post::Entity::belongs_to(post_tag::Entity)
                    .from(post::Column::Id)
                    .to(post_tag::Column::PostId)
                    .into()
            )
            .filter(post::Column::Id.eq(post_id))
            .all(db)
            .await
    }
}
