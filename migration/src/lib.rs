pub use sea_orm_migration::prelude::*;

mod m20260312_131843_create_users_table;
mod m20260312_132015_create_posts_table;
mod m20260312_132126_create_categories_table;
mod m20260312_140819_create_tags_table;
mod m20260312_141045_create_post_relations;
mod m20260312_141453_create_comments_table;
mod m20260312_141646_create_media_table;
mod migrator;

pub use migrator::Migrator;
