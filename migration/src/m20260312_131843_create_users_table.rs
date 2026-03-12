use crate::sea_orm::Statement;
use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Создаём ENUM для ролей пользователей
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("user_role"))
                    .values([
                        Alias::new("admin"),
                        Alias::new("editor"),
                        Alias::new("author"),
                        Alias::new("subscriber"),
                    ])
                    .to_owned(),
            )
            .await?;

        // 2. Создаём таблицу users
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Users::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(ColumnDef::new(Users::DisplayName).string().null())
                    .col(ColumnDef::new(Users::Bio).string().null())
                    .col(ColumnDef::new(Users::AvatarUrl).string().null())
                    .col(
                        ColumnDef::new(Users::Role)
                            .enumeration(
                                Alias::new("user_role"),
                                [
                                    Alias::new("admin"),
                                    Alias::new("editor"),
                                    Alias::new("author"),
                                    Alias::new("subscriber"),
                                ],
                            )
                            .not_null()
                            .default("subscriber"),
                    )
                    .col(
                        ColumnDef::new(Users::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Users::LastLogin)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    // ✅ created_at - только ОДИН РАЗ!
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    // ✅ updated_at - только ОДИН РАЗ!
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 3. Создаём индексы
        manager
            .create_index(
                Index::create()
                    .name("idx_users_email")
                    .table(Users::Table)
                    .col(Users::Email)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_username")
                    .table(Users::Table)
                    .col(Users::Username)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 4. Триггер для авто-обновления updated_at
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "CREATE OR REPLACE FUNCTION update_users_updated_at()
                     RETURNS TRIGGER AS $$
                     BEGIN
                         NEW.updated_at = NOW();
                         RETURN NEW;
                     END;
                     $$ LANGUAGE plpgsql;"
                    .to_owned(),
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "CREATE TRIGGER trigger_users_updated_at
                     BEFORE UPDATE ON users
                     FOR EACH ROW
                     EXECUTE FUNCTION update_users_updated_at();"
                    .to_owned(),
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Удаляем триггер и функцию
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DROP TRIGGER IF EXISTS trigger_users_updated_at ON users;".to_owned(),
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DROP FUNCTION IF EXISTS update_users_updated_at();".to_owned(),
            ))
            .await?;

        // Удаляем таблицу
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        // Удаляем ENUM
        manager
            .drop_type(Type::drop().name(Alias::new("user_role")).to_owned())
            .await?;

        Ok(())
    }
}

// ==================== Iden ====================

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    Username,
    Email,
    PasswordHash,
    DisplayName,
    Bio,
    AvatarUrl,
    Role,
    IsActive,
    LastLogin,
    CreatedAt,
    UpdatedAt,
}
