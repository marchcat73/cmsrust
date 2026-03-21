// src/entities/media.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "media")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    pub filename: String,
    pub filepath: String,
    pub mime_type: String,
    pub size: i64,

    pub width: Option<i32>,
    pub height: Option<i32>,

    pub alt_text: Option<String>,
    pub caption: Option<String>,

    pub uploader_id: Uuid,

    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UploaderId",
        to = "super::user::Column::Id"
    )]
    Uploader,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Uploader.def()
    }
}
