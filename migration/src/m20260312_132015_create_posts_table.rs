use crate::sea_orm::Statement;
use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Создаём ENUM для статуса поста (PostStatus)
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("post_status"))
                    .values([
                        Alias::new("draft"),
                        Alias::new("published"),
                        Alias::new("archived"),
                        Alias::new("trash"),
                    ])
                    .to_owned(),
            )
            .await?;

        // 2. Создаём ENUM для статуса комментариев (CommentStatus)
        manager
            .create_type(
                Type::create()
                    .as_enum(Alias::new("comment_status"))
                    .values([Alias::new("open"), Alias::new("closed")])
                    .to_owned(),
            )
            .await?;

        // 3. Создаём таблицу posts
        manager
            .create_table(
                Table::create()
                    .table(Posts::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Posts::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Posts::Title).string().not_null())
                    .col(ColumnDef::new(Posts::Slug).string().not_null().unique_key())
                    .col(ColumnDef::new(Posts::Content).text().not_null())
                    .col(ColumnDef::new(Posts::Excerpt).string().null())
                    .col(ColumnDef::new(Posts::FeaturedImage).uuid().null())
                    .col(
                        ColumnDef::new(Posts::Status)
                            .enumeration(
                                Alias::new("post_status"),
                                [
                                    Alias::new("draft"),
                                    Alias::new("published"),
                                    Alias::new("archived"),
                                    Alias::new("trash"),
                                ],
                            )
                            .not_null()
                            .default("draft"),
                    )
                    .col(
                        ColumnDef::new(Posts::CommentStatus)
                            .enumeration(
                                Alias::new("comment_status"),
                                [Alias::new("open"), Alias::new("closed")],
                            )
                            .not_null()
                            .default("open"),
                    )
                    .col(ColumnDef::new(Posts::AuthorId).uuid().not_null())
                    .col(
                        ColumnDef::new(Posts::PublishedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Posts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Posts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    // Foreign Key: author_id -> users.id
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_posts_author")
                            .from(Posts::Table, Posts::AuthorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    // Foreign Key: featured_image -> media.id
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_posts_featured_image")
                            .from(Posts::Table, Posts::FeaturedImage)
                            .to(Media::Table, Media::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 4. Создаём индексы для производительности

        // Индекс по slug для быстрого поиска по URL
        manager
            .create_index(
                Index::create()
                    .name("idx_posts_slug")
                    .table(Posts::Table)
                    .col(Posts::Slug)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Индекс по author_id для фильтрации постов автора
        manager
            .create_index(
                Index::create()
                    .name("idx_posts_author_id")
                    .table(Posts::Table)
                    .col(Posts::AuthorId)
                    .to_owned(),
            )
            .await?;

        // Индекс по status для фильтрации опубликованных/черновиков
        manager
            .create_index(
                Index::create()
                    .name("idx_posts_status")
                    .table(Posts::Table)
                    .col(Posts::Status)
                    .to_owned(),
            )
            .await?;

        // Индекс по published_at для сортировки по дате публикации
        manager
            .create_index(
                Index::create()
                    .name("idx_posts_published_at")
                    .table(Posts::Table)
                    .col(Posts::PublishedAt)
                    .to_owned(),
            )
            .await?;

        // Составной индекс для частых запросов (статус + дата публикации)
        manager
            .create_index(
                Index::create()
                    .name("idx_posts_status_published_at")
                    .table(Posts::Table)
                    .col(Posts::Status)
                    .col(Posts::PublishedAt)
                    .to_owned(),
            )
            .await?;

        // 5. Создаём триггер для автоматического обновления updated_at
        // ✅ ИСПРАВЛЕНИЕ: используем Statement::from_string()
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "CREATE OR REPLACE FUNCTION update_posts_updated_at()
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
                "CREATE TRIGGER trigger_posts_updated_at
                     BEFORE UPDATE ON posts
                     FOR EACH ROW
                     EXECUTE FUNCTION update_posts_updated_at();"
                    .to_owned(),
            ))
            .await?;

        // 6. Добавляем полнотекстовый поиск (опционально, для PostgreSQL)
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "ALTER TABLE posts ADD COLUMN search_vector tsvector
                     GENERATED ALWAYS AS (
                         setweight(to_tsvector('russian', coalesce(title, '')), 'A') ||
                         setweight(to_tsvector('russian', coalesce(excerpt, '')), 'B') ||
                         setweight(to_tsvector('russian', coalesce(content, '')), 'C')
                     ) STORED;"
                    .to_owned(),
            ))
            .await?;

        // Индекс для полнотекстового поиска
        manager
            .create_index(
                Index::create()
                    .name("idx_posts_search_vector")
                    .table(Posts::Table)
                    .col(Posts::SearchVector)
                    // .using(sea_query::IndexType::Gin)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Удаляем триггер и функцию при откате
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DROP TRIGGER IF EXISTS trigger_posts_updated_at ON posts;".to_owned(),
            ))
            .await?;

        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DROP FUNCTION IF EXISTS update_posts_updated_at();".to_owned(),
            ))
            .await?;

        // Удаляем таблицу posts
        manager
            .drop_table(Table::drop().table(Posts::Table).to_owned())
            .await?;

        // Удаляем ENUM типы (если не используются в других таблицах)
        manager
            .drop_type(Type::drop().name(Alias::new("post_status")).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(Alias::new("comment_status")).to_owned())
            .await?;

        Ok(())
    }
}

// ==================== Iden для таблицы Posts ====================

#[derive(Iden)]
pub enum Posts {
    Table,
    Id,
    Title,
    Slug,
    Content,
    Excerpt,
    FeaturedImage,
    Status,
    CommentStatus,
    AuthorId,
    PublishedAt,
    CreatedAt,
    UpdatedAt,
    SearchVector,
}

// ==================== Iden для связанных таблиц ====================

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
}

#[derive(Iden)]
pub enum Media {
    Table,
    Id,
}
