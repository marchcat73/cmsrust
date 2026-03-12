use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(Categories::Table)
                    .col(pk_uuid(Categories::Id))
                    .col(string(Categories::Name))
                    .col(string_uniq(Categories::Slug))
                    .col(string_null(Categories::Description))
                    .col(uuid_null(Categories::ParentId))
                    .col(timestamp_with_time_zone(Categories::CreatedAt))
                    .col(timestamp_with_time_zone(Categories::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_categories_parent")
                            .from(Categories::Table, Categories::ParentId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Индекс для быстрого поиска по slug
        manager
            .create_index(
                Index::create()
                    .name("idx_categories_slug")
                    .table(Categories::Table)
                    .col(Categories::Slug)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Categories::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Categories {
    Table,
    Id,
    Name,
    Slug,
    Description,
    ParentId,
    CreatedAt,
    UpdatedAt,
}
