use crate::m20260312_132015_create_posts_table::Posts;
use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ENUM для статуса комментариев
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("comment_status"))
                    .values([
                        Alias::new("pending"),
                        Alias::new("approved"),
                        Alias::new("spam"),
                        Alias::new("trash"),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                table_auto(Comments::Table)
                    .col(pk_uuid(Comments::Id))
                    .col(uuid(Comments::PostId))
                    .col(string_null(Comments::AuthorName))
                    .col(string_null(Comments::AuthorEmail))
                    .col(string_null(Comments::AuthorUrl))
                    .col(text(Comments::Content))
                    .col(
                        ColumnDef::new(Comments::Status)
                            .enumeration(
                                Alias::new("comment_status"),
                                [
                                    Alias::new("pending"),
                                    Alias::new("approved"),
                                    Alias::new("spam"),
                                    Alias::new("trash"),
                                ],
                            )
                            .not_null()
                            .default("pending"),
                    )
                    .col(uuid_null(Comments::ParentId))
                    .col(timestamp_with_time_zone(Comments::CreatedAt))
                    .col(timestamp_with_time_zone(Comments::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comments_post")
                            .from(Comments::Table, Comments::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comments_parent")
                            .from(Comments::Table, Comments::ParentId)
                            .to(Comments::Table, Comments::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Comments::Table).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(Alias::new("comment_status")).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Comments {
    Table,
    Id,
    PostId,
    AuthorName,
    AuthorEmail,
    AuthorUrl,
    Content,
    Status,
    ParentId,
    CreatedAt,
    UpdatedAt,
}
