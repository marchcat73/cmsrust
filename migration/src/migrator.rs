use crate::m20260312_131843_create_users_table;
use crate::m20260312_132015_create_posts_table;
use crate::m20260312_132126_create_categories_table;
use crate::m20260312_140819_create_tags_table;
use crate::m20260312_141045_create_post_relations;
use crate::m20260312_141453_create_comments_table;
use crate::m20260312_141646_create_media_table;
use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260312_131843_create_users_table::Migration),
            Box::new(m20260312_141646_create_media_table::Migration),
            Box::new(m20260312_132015_create_posts_table::Migration),
            Box::new(m20260312_132126_create_categories_table::Migration),
            Box::new(m20260312_140819_create_tags_table::Migration),
            Box::new(m20260312_141045_create_post_relations::Migration),
            Box::new(m20260312_141453_create_comments_table::Migration),
            // Добавляйте новые миграции сюда
        ]
    }
}
