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
# Полная очистка ОСТОРОЖНО!!! Удалит и другие контейнеры
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

{"success":true,"message":null,"data":{"token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI1MThhNWM1My0xY2UwLTRmYjQtYWU5YS02MTRlZjFmOThiMmQiLCJ1c2VybmFtZSI6ImFkbWluIiwicm9sZSI6IkFkbWluIiwiZXhwIjoxNzc1MTE2NjIwfQ.MX112wITdDwhOk7xuyi5Mi8mj9tF68JgvT0yVV_ohO8","user":{"id":"518a5c53-1ce0-4fb4-ae9a-614ef1f98b2d","username":"admin","email":"admin@example.com","role":"Admin"}},"errors":null}

```bash
curl -X POST http://localhost:8000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"TestUser", "email":"user@example.com", "password":"StrongPassword1234!"}'
```

{"success":true,"message":null,"data":{"id":"addd63e0-af33-412b-9403-5413fd617089","username":"TestUser","email":"user@example.com","password_hash":"$argon2id$v=19$m=19456,t=2,p=1$WShNo9Vh7qHITn58Q90f9w$7zPh/T5wqDibXlQQeFYGCoJ7LUMXmxGFQCo041iO9AA","display_name":"TestUser","bio":null,"avatar_url":null,"role":"Subscriber","is_active":true,"last_login":null,"created_at":"2026-04-01T08:30:03.142493Z","updated_at":"2026-04-01T08:30:03.142496Z"},"errors":null}

```bash
curl -X POST http://localhost:8000/api/categories \
  -H "Content-Type: application/json" \
  -H "Cookie: cms_auth_token=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI1MThhNWM1My0xY2UwLTRmYjQtYWU5YS02MTRlZjFmOThiMmQiLCJ1c2VybmFtZSI6ImFkbWluIiwicm9sZSI6IkFkbWluIiwiZXhwIjoxNzc1MzAyNzczfQ.tmyuO4BAw9rk41uvFEHihGBze5w8XGx6mDkx28WSmFk" \
  -d '{
    "name": "Технологии1",
    "slug": "tech1",
    "description": "Все о технологиях",
    "parent_id": null
  }'
```
5afe4e3e-b57f-4336-97f0-2df0a99f1b6c
```bash
curl -X GET http://localhost:8000/api/categories \
  -H "Cookie: cms_auth_token=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI1MThhNWM1My0xY2UwLTRmYjQtYWU5YS02MTRlZjFmOThiMmQiLCJ1c2VybmFtZSI6ImFkbWluIiwicm9sZSI6IkFkbWluIiwiZXhwIjoxNzc1MzAyNzczfQ.tmyuO4BAw9rk41uvFEHihGBze5w8XGx6mDkx28WSmFk"
```

```bash
# Замените UUID на реальный ID из ответа шага 1
curl -X PUT http://localhost:8000/api/categories/5afe4e3e-b57f-4336-97f0-2df0a99f1b6c \
  -H "Content-Type: application/json" \
  -H "Cookie: cms_auth_token=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI1MThhNWM1My0xY2UwLTRmYjQtYWU5YS02MTRlZjFmOThiMmQiLCJ1c2VybmFtZSI6ImFkbWluIiwicm9sZSI6IkFkbWluIiwiZXhwIjoxNzc1MzAyNzczfQ.tmyuO4BAw9rk41uvFEHihGBze5w8XGx6mDkx28WSmFk" \
  -d '{
    "name": "IT и Технологии",
    "description": "Обновленное описание"
  }'
```

## Auth

```sh
# Сгенерировать ключ COOKIE_SECRET_KEY в терминале (Linux/Mac):
openssl rand -base64 64
```
