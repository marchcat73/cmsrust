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
                    .table(Tags::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tags::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tags::Name).string().not_null())
                    .col(
                        ColumnDef::new(Tags::Slug)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    // ✅ created_at - ТОЛЬКО ОДИН РАЗ!
                    .col(
                        ColumnDef::new(Tags::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Индекс для быстрого поиска по slug
        manager
            .create_index(
                Index::create()
                    .name("idx_tags_slug")
                    .table(Tags::Table)
                    .col(Tags::Slug)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Индекс для поиска по name
        manager
            .create_index(
                Index::create()
                    .name("idx_tags_name")
                    .table(Tags::Table)
                    .col(Tags::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tags::Table).to_owned())
            .await
    }
}

// ==================== Iden ====================

#[derive(Iden)]
pub enum Tags {
    Table,
    Id,
    Name,
    Slug,
    CreatedAt,
}
