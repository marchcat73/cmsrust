// src/services/category_service.rs
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter,
    QueryOrder, Set, TransactionTrait
};
use crate::entities::{category, post_category};
use uuid::Uuid;

pub struct CategoryService;

impl CategoryService {
    /// Получить все категории (плоский список)
    pub async fn list_categories(
        db: &sea_orm::DatabaseConnection,
    ) -> Result<Vec<category::Model>, DbErr> {
        category::Entity::find()
            .order_by_asc(category::Column::Name)
            .all(db)
            .await
    }

    /// Получить категорию по ID
    pub async fn get_category_by_id(
        db: &sea_orm::DatabaseConnection,
        id: Uuid,
    ) -> Result<category::Model, DbErr> {
        category::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Category not found".to_string()))
    }

    /// Создать новую категорию
    pub async fn create_category(
        db: &sea_orm::DatabaseConnection,
        name: String,
        slug: Option<String>,
        description: Option<String>,
        parent_id: Option<Uuid>,
    ) -> Result<category::Model, DbErr> {
        // Если slug не передан, генерируем из названия
        let final_slug = slug.unwrap_or_else(|| Self::generate_slug(&name));

        let active_category = category::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(name),
            slug: Set(final_slug),
            description: Set(description),
            parent_id: Set(parent_id),
            ..Default::default()
        };

        active_category.insert(db).await
    }

    /// Обновить категорию
    pub async fn update_category(
        db: &sea_orm::DatabaseConnection,
        id: Uuid,
        name: Option<String>,
        slug: Option<String>,
        description: Option<Option<String>>,
        parent_id: Option<Option<Uuid>>,
    ) -> Result<category::Model, DbErr> {
        let category_model = Self::get_category_by_id(db, id).await?;
        let mut active_category: category::ActiveModel = category_model.into();

        if let Some(n) = name {
            active_category.name = Set(n);
            // Если slug не передан явно, но изменили имя, можно перегенерировать slug (опционально)
            if slug.is_none() {
                // active_category.slug = Set(Self::generate_slug(&n));
            }
        }

        if let Some(s) = slug {
            active_category.slug = Set(s);
        }

        if let Some(d) = description {
            active_category.description = Set(d);
        }

        if let Some(p) = parent_id {
            // Защита от циклической ссылки (категория не может быть родителем самой себя)
            if p == Some(id) {
                return Err(DbErr::Custom("Category cannot be its own parent".to_string()));
            }
            active_category.parent_id = Set(p);
        }

        active_category.update(db).await
    }

    /// Удалить категорию
    pub async fn delete_category(
        db: &sea_orm::DatabaseConnection,
        id: Uuid,
    ) -> Result<(), DbErr> {

        let result = category::Entity::delete_by_id(id).exec(db).await?;

        if result.rows_affected == 0 {
            return Err(DbErr::RecordNotFound("Category not found".to_string()));
        }

        Ok(())
    }

    /// Вспомогательная функция для генерации slug
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
