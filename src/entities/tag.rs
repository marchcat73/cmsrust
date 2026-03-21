// src/entities/tag.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "tags")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::post_tag::Entity")]
    PostTags,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::post_tag::Entity> for Entity {
    fn to() -> RelationDef {
        super::post_tag::Relation::Tag.def()
    }
}
