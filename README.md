# CMS Rust

## Database

```bash
sudo -u postgres psql

CREATE USER cmsrust WITH PASSWORD 'cmsrust';
CREATE DATABASE cmsrustdb OWNER cmsrust;
GRANT ALL PRIVILEGES ON DATABASE cmsrustdb TO cmsrust;
```

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
cargo new migration --bin

cd migration
sea-orm-cli migrate generate create_users_table
sea-orm-cli migrate generate create_posts_table
sea-orm-cli migrate generate create_categories_table
sea-orm-cli migrate generate create_tags_table
sea-orm-cli migrate generate create_post_relations
sea-orm-cli migrate generate create_comments_table
sea-orm-cli migrate generate create_media_table


sea-orm-cli migrate up -d /home/user/projects/cmsrust/migration
sea-orm-cli migrate up -d /Users/pirs/Documents/www/cmsrust/migration

```

# Errors

```bash

cargo check 2>&1 | tee build_error.log.txt

```
