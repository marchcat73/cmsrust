// src/entities/category.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "categories")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
        pub created_at: DateTime<Utc>,
        #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::category::Entity",
        from = "Column::ParentId",
        to = "super::category::Column::Id"
    )]
    Parent,
    #[sea_orm(has_many = "super::post_category::Entity")]
    PostCategories,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::post_category::Entity> for Entity {
    fn to() -> RelationDef {
        super::post_category::Relation::Category.def()
    }
}
