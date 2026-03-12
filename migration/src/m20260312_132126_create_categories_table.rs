use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};
use crate::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Categories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Categories::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Categories::Name).string().not_null())
                    .col(
                        ColumnDef::new(Categories::Slug)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Categories::Description).string().null())
                    // ✅ parent_id - ТОЛЬКО ОДИН РАЗ!
                    .col(ColumnDef::new(Categories::ParentId).uuid().null())
                    // ✅ created_at - ТОЛЬКО ОДИН РАЗ!
                    .col(
                        ColumnDef::new(Categories::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    // ✅ updated_at - ТОЛЬКО ОДИН РАЗ!
                    .col(
                        ColumnDef::new(Categories::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    // Foreign Key: parent_id -> categories.id (рекурсивная связь)
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_categories_parent")
                            .from(Categories::Table, Categories::ParentId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
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
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Индекс для поиска по parent_id
        manager
            .create_index(
                Index::create()
                    .name("idx_categories_parent_id")
                    .table(Categories::Table)
                    .col(Categories::ParentId)
                    .to_owned(),
            )
            .await?;

        // Триггер для авто-обновления updated_at
        manager
            .get_connection()
            .execute(
                Statement::from_string(
                    manager.get_database_backend(),
                    "CREATE OR REPLACE FUNCTION update_categories_updated_at()
                     RETURNS TRIGGER AS $$
                     BEGIN
                         NEW.updated_at = NOW();
                         RETURN NEW;
                     END;
                     $$ LANGUAGE plpgsql;".to_owned(),
                ),
            )
            .await?;

        manager
            .get_connection()
            .execute(
                Statement::from_string(
                    manager.get_database_backend(),
                    "CREATE TRIGGER trigger_categories_updated_at
                     BEFORE UPDATE ON categories
                     FOR EACH ROW
                     EXECUTE FUNCTION update_categories_updated_at();".to_owned(),
                ),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Удаляем триггер и функцию
        manager
            .get_connection()
            .execute(
                Statement::from_string(
                    manager.get_database_backend(),
                    "DROP TRIGGER IF EXISTS trigger_categories_updated_at ON categories;".to_owned(),
                ),
            )
            .await?;

        manager
            .get_connection()
            .execute(
                Statement::from_string(
                    manager.get_database_backend(),
                    "DROP FUNCTION IF EXISTS update_categories_updated_at();".to_owned(),
                ),
            )
            .await?;

        // Удаляем таблицу
        manager
            .drop_table(Table::drop().table(Categories::Table).to_owned())
            .await
    }
}

// ==================== Iden ====================

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
