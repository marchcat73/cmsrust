// src/entities/category.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "categories")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    pub name: String,
    pub slug: String,
    pub description: Option<String>,

    pub parent_id: Option<Uuid>,  // Для вложенных категорий

    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::category::Entity",
        from = "Column::ParentId",
        to = "super::category::Column::Id"
    )]
    Parent,
    #[sea_orm(has_many = "super::category::Entity")]
    Children,
    #[sea_orm(has_many = "super::post_category::Entity")]
    PostCategories,
    #[sea_orm(has_many = "super::post::Entity")]
    Posts,
}

impl ActiveModelBehavior for ActiveModel {}
