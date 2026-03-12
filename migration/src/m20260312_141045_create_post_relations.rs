use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Таблица post_categories (many-to-many)
        manager
            .create_table(
                Table::create()
                    .table(PostCategories::Table)
                    .col(pk_uuid(PostCategories::PostId))
                    .col(pk_uuid(PostCategories::CategoryId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_categories_post")
                            .from(PostCategories::Table, PostCategories::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_categories_category")
                            .from(PostCategories::Table, PostCategories::CategoryId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .name("pk_post_categories")
                            .col(PostCategories::PostId)
                            .col(PostCategories::CategoryId),
                    )
                    .to_owned(),
            )
            .await?;

        // Таблица post_tags (many-to-many)
        manager
            .create_table(
                Table::create()
                    .table(PostTags::Table)
                    .col(pk_uuid(PostTags::PostId))
                    .col(pk_uuid(PostTags::TagId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_tags_post")
                            .from(PostTags::Table, PostTags::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_tags_tag")
                            .from(PostTags::Table, PostTags::TagId)
                            .to(Tags::Table, Tags::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .name("pk_post_tags")
                            .col(PostTags::PostId)
                            .col(PostTags::TagId),
                    )
                    .to_owned(),
            )
            .await?;

        // Индексы для производительности
        manager
            .create_index(
                Index::create()
                    .name("idx_post_categories_post_id")
                    .table(PostCategories::Table)
                    .col(PostCategories::PostId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_post_tags_post_id")
                    .table(PostTags::Table)
                    .col(PostTags::PostId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PostTags::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PostCategories::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum PostCategories {
    Table,
    PostId,
    CategoryId,
}

#[derive(Iden)]
pub enum PostTags {
    Table,
    PostId,
    TagId,
}

#[derive(Iden)]
pub enum Posts {
    Table,
    Id,
}

#[derive(Iden)]
pub enum Categories {
    Table,
    Id,
}

#[derive(Iden)]
pub enum Tags {
    Table,
    Id,
}
