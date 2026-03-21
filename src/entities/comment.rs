use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "comments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    pub post_id: Uuid,
    pub author_id: Option<Uuid>,

    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub author_url: Option<String>,

    #[sea_orm(column_type = "Text")]
    pub content: String,

    pub status: CommentModerationStatus,
    pub parent_id: Option<Uuid>,

    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::post::Entity",
        from = "Column::PostId",
        to = "super::post::Column::Id"
    )]
    Post,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::AuthorId",
        to = "super::user::Column::Id"
    )]
    Author,
    #[sea_orm(
        belongs_to = "super::comment::Entity",
        from = "Column::ParentId",
        to = "super::comment::Column::Id"
    )]
    Parent,
}

impl ActiveModelBehavior for ActiveModel {}


#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "comment_moderation_status")]
pub enum CommentModerationStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "approved")]
    Approved,
    #[sea_orm(string_value = "spam")]
    Spam,
    #[sea_orm(string_value = "trash")]
    Trash,
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Author.def()
    }
}
