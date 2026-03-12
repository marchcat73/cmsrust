use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Media::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Media::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Media::Filename).string().not_null())
                    .col(ColumnDef::new(Media::Filepath).string().not_null())
                    .col(ColumnDef::new(Media::MimeType).string().not_null())
                    .col(ColumnDef::new(Media::Size).big_integer().not_null())
                    .col(ColumnDef::new(Media::Width).integer().null())
                    .col(ColumnDef::new(Media::Height).integer().null())
                    .col(ColumnDef::new(Media::AltText).string().null())
                    .col(ColumnDef::new(Media::Caption).string().null())
                    .col(ColumnDef::new(Media::UploaderId).uuid().not_null())
                    // ✅ created_at - ТОЛЬКО ОДИН РАЗ!
                    .col(
                        ColumnDef::new(Media::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    // Foreign Key: uploader_id -> users.id
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_media_uploader")
                            .from(Media::Table, Media::UploaderId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Индекс для быстрого поиска по uploader
        manager
            .create_index(
                Index::create()
                    .name("idx_media_uploader_id")
                    .table(Media::Table)
                    .col(Media::UploaderId)
                    .to_owned(),
            )
            .await?;

        // Индекс для поиска по mime_type
        manager
            .create_index(
                Index::create()
                    .name("idx_media_mime_type")
                    .table(Media::Table)
                    .col(Media::MimeType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Media::Table).to_owned())
            .await
    }
}

// ==================== Iden ====================

#[derive(Iden)]
pub enum Media {
    Table,
    Id,
    Filename,
    Filepath,
    MimeType,
    Size,
    Width,
    Height,
    AltText,
    Caption,
    UploaderId,
    CreatedAt,
}

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
}
