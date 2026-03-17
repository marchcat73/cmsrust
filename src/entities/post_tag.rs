// src/entities/post_tag.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "post_tags")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub post_id: Uuid,
    #[sea_orm(primary_key)]
    pub tag_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // ✅ Обязательно: связь с post
    #[sea_orm(
        belongs_to = "super::post::Entity",
        from = "Column::PostId",
        to = "super::post::Column::Id"
    )]
    Post,
    // ✅ Обязательно: связь с tag (без этого будет ошибка!)
    #[sea_orm(
        belongs_to = "super::tag::Entity",
        from = "Column::TagId",
        to = "super::tag::Column::Id"
    )]
    Tag,
}

impl ActiveModelBehavior for ActiveModel {}
