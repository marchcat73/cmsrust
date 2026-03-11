// src/entities/post.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    pub title: String,
    pub slug: String,  // URL-friendly version of title

    #[sea_orm(column_type = "Text")]
    pub content: String,

    pub excerpt: Option<String>,

    pub featured_image: Option<Uuid>,  // Reference to media

    pub status: PostStatus,  // Draft, Published, Archived, Trash

    pub comment_status: CommentStatus,  // Open, Closed

    pub author_id: Uuid,

    pub published_at: Option<DateTime>,

    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::AuthorId",
        to = "super::user::Column::Id"
    )]
    Author,
    #[sea_orm(has_many = "super::category::Entity")]
    Categories,
    #[sea_orm(has_many = "super::tag::Entity")]
    Tags,
    #[sea_orm(has_many = "super::comment::Entity")]
    Comments,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "post_status")]
pub enum PostStatus {
    #[sea_orm(string_value = "draft")]
    Draft,
    #[sea_orm(string_value = "published")]
    Published,
    #[sea_orm(string_value = "archived")]
    Archived,
    #[sea_orm(string_value = "trash")]
    Trash,
}
