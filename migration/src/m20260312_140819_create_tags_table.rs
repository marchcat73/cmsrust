use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(Tags::Table)
                    .col(pk_uuid(Tags::Id))
                    .col(string(Tags::Name))
                    .col(string_uniq(Tags::Slug))
                    .col(timestamp_with_time_zone(Tags::CreatedAt))
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

#[derive(Iden)]
pub enum Tags {
    Table,
    Id,
    Name,
    Slug,
    CreatedAt,
}
