# CMS Rust

```bash
cargo install sea-orm-cli@^1.0

sea-orm-cli generate entity \
    --database-url postgres://cmsrust:cmsrust@localhost/cmsrustdb \
    --output-dir src/entities \
    --with-serde both \
    --expanded-format

cargo run

```

# Migrations

```bash
cargo new migrations --bin

cd migrations
sea-orm-cli migrate generate create_users_table
sea-orm-cli migrate generate create_posts_table
sea-orm-cli migrate generate create_categories_table
sea-orm-cli migrate generate create_tags_table
sea-orm-cli migrate generate create_post_relations
sea-orm-cli migrate generate create_comments_table
sea-orm-cli migrate generate create_media_table
```
