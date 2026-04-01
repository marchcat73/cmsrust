# CMS Rust

## Database

```bash
sudo -u postgres psql

CREATE USER cmsrust WITH PASSWORD 'dev_password';
CREATE DATABASE cmsrustdb OWNER cmsrust;
GRANT ALL PRIVILEGES ON DATABASE cmsrustdb TO cmsrust;
```

## Use Podman

```sh
podman-compose up -d

# check
podman ps -a
# start
podman start cmsrust_postgres_1
# Очистка
podman stop cmsrust_postgres_1
podman rm cmsrust_postgres_1
podman-compose down --volumes --rmi all
```

```bash
cargo install sea-orm-cli@^1.0

# sea-orm-cli generate entity \
#     --database-url postgres://cmsrust:cmsrust@localhost/cmsrustdb \
#     --output-dir src/entities \
#     --with-serde both \
#     --expanded-format

cargo run

```

# Migrations

```bash
# cargo new migration --bin

cd migration
# sea-orm-cli migrate generate create_users_table
# sea-orm-cli migrate generate create_posts_table
# sea-orm-cli migrate generate create_categories_table
# sea-orm-cli migrate generate create_tags_table
# sea-orm-cli migrate generate create_post_relations
# sea-orm-cli migrate generate create_comments_table
# sea-orm-cli migrate generate create_media_table


sea-orm-cli migrate up -d /home/{user}/projects/cmsrust/migration
sea-orm-cli migrate up -d ./migration

```

# Errors

```bash

cargo check 2>&1 | tee build_error.log.txt

```

```bash
curl -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com", "password":"StrongPassword123!"}'
```
