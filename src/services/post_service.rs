// src/services/post_service.rs
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter, QuerySelect, RelationTrait};
use uuid::Uuid;
use crate::entities::{post, post_category, category, post_tag, tag};

pub struct PostService;

impl PostService {
    /// Получить пост с категориями
    pub async fn get_post_with_categories(
        db: &DatabaseConnection,
        post_id: Uuid,
    ) -> Result<(post::Model, Vec<category::Model>), sea_orm::DbErr> {
        // 1. Найти пост
        let post = post::Entity::find_by_id(post_id)
            .one(db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("Post not found".to_string()))?;

        // 2. Загрузить категории через промежуточную таблицу
        let categories = post
            .find_related(post_category::Entity)  // Сначала промежуточная таблица
            .find_with_related(category::Entity)  // Затем целевая сущность
            .all(db)
            .await?;

        // Извлекаем категории из кортежей
        let categories: Vec<category::Model> = categories
            .into_iter()
            .map(|(_, cat)| cat)
            .collect();

        Ok((post, categories))
    }

    /// Получить пост с тегами
    pub async fn get_post_with_tags(
        db: &DatabaseConnection,
        post_id: Uuid,
    ) -> Result<(post::Model, Vec<tag::Model>), sea_orm::DbErr> {
        let post = post::Entity::find_by_id(post_id)
            .one(db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("Post not found".to_string()))?;

        let tags = post
            .find_related(post_tag::Entity)
            .find_with_related(tag::Entity)
            .all(db)
            .await?;

        let tags: Vec<tag::Model> = tags.into_iter().map(|(_, tag)| tag).collect();

        Ok((post, tags))
    }

    /// Альтернатива: прямой запрос через join
    pub async fn get_post_categories_direct(
        db: &DatabaseConnection,
        post_id: Uuid,
    ) -> Result<Vec<category::Model>, sea_orm::DbErr> {
        category::Entity::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                post_category::Entity::belongs_to(category::Entity)
                    .from(post_category::Column::CategoryId)
                    .to(category::Column::Id)
                    .into()
            )
            .join(
                sea_orm::JoinType::InnerJoin,
                post::Entity::belongs_to(post_category::Entity)
                    .from(post::Column::Id)
                    .to(post_category::Column::PostId)
                    .into()
            )
            .filter(post::Column::Id.eq(post_id))
            .all(db)
            .await
    }

    /// Получить теги поста
    pub async fn get_post_tags_direct(
        db: &DatabaseConnection,
        post_id: Uuid,
    ) -> Result<Vec<tag::Model>, sea_orm::DbErr> {
        tag::Entity::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                post_tag::Entity::belongs_to(tag::Entity)
                    .from(post_tag::Column::TagId)
                    .to(tag::Column::Id)
                    .into()
            )
            .join(
                sea_orm::JoinType::InnerJoin,
                post::Entity::belongs_to(post_tag::Entity)
                    .from(post::Column::Id)
                    .to(post_tag::Column::PostId)
                    .into()
            )
            .filter(post::Column::Id.eq(post_id))
            .all(db)
            .await
    }

    /// Привязать категории к посту
    pub async fn attach_categories_to_post(
        db: &DatabaseConnection,
        post_id: Uuid,
        category_ids: Vec<Uuid>,
    ) -> Result<(), sea_orm::DbErr> {
        // Удалить старые связи
        post_category::Entity::delete_many()
            .filter(post_category::Column::PostId.eq(post_id))
            .exec(db)
            .await?;

        // Создать новые связи
        for cat_id in category_ids {
            post_category::ActiveModel {
                post_id: Set(post_id),
                category_id: Set(cat_id),
            }
            .insert(db)
            .await?;
        }

        Ok(())
    }

    /// Привязать теги к посту
    pub async fn attach_tags_to_post(
        db: &DatabaseConnection,
        post_id: Uuid,
        tag_ids: Vec<Uuid>,
    ) -> Result<(), sea_orm::DbErr> {
        post_tag::Entity::delete_many()
            .filter(post_tag::Column::PostId.eq(post_id))
            .exec(db)
            .await?;

        for tag_id in tag_ids {
            post_tag::ActiveModel {
                post_id: Set(post_id),
                tag_id: Set(tag_id),
            }
            .insert(db)
            .await?;
        }

        Ok(())
    }
}
