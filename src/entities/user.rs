use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    #[sea_orm(unique)]
    pub username: String,

    #[sea_orm(unique)]
    pub email: String,

    pub password_hash: String,

    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,

    pub role: UserRole, // Enum: Admin, Editor, Author, Subscriber

    pub is_active: bool,
    pub last_login: Option<DateTime>,

    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::post::Entity")]
    Posts,
    #[sea_orm(has_many = "super::media::Entity")]
    Media,
}

impl ActiveModelBehavior for ActiveModel {}

// Enum для ролей
#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_role")]
pub enum UserRole {
    #[sea_orm(string_value = "admin")]
    Admin,
    #[sea_orm(string_value = "editor")]
    Editor,
    #[sea_orm(string_value = "author")]
    Author,
    #[sea_orm(string_value = "subscriber")]
    Subscriber,
}
