// src/services/user_service.rs
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, Set};
use crate::entities::{user, user::UserRole};
use uuid::Uuid;
use chrono::Utc;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub struct UserService;

impl UserService {
    /// Хештирование пароля с использованием Argon2id (рекомендуемый вариант)
    fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
        // Генерируем случайную соль
        let salt = SaltString::generate(&mut OsRng);

        // Настраиваем параметры Argon2id
        // Memory cost: 19456 KiB (19 MB)
        // Time cost: 2 iterations
        // Parallelism: 1 thread
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(19456, 2, 1, None).unwrap(),
        );

        // Хешируем пароль
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;

        Ok(password_hash.to_string())
    }

    /// Проверка пароля
    pub fn verify_password(password: &str, hash: &str) -> bool {
        // 1. Парсим хеш из базы данных.
        // Хеш сам содержит информацию о версии, соли и параметрах (m, t, p).
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(h) => h,
            Err(_) => {
                eprintln!("❌ Ошибка парсинга хеша: неверный формат");
                return false;
            }
        };

        // Создаем экземпляр Argon2 с теми же параметрами (алгоритм определится из хеша)
        let argon2 = Argon2::default();

        // Верифицируем
        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => {
            println!("✅ Пароль верен!");
            true
        },
        Err(e) => {
            eprintln!("❌ Ошибка верификации: {}", e);
            false
        }
    }
    }

    /// Создает пользователя, если он еще не существует
    pub async fn create_user_if_not_exists(
        db: &sea_orm::DatabaseConnection,
        username: &str,
        email: &str,
        password: &str,
        role: UserRole,
    ) -> Result<user::Model, Box<dyn std::error::Error>> {
        // Проверяем существование
        if let Some(_existing) = user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(db)
            .await?
        {
            return Err("User already exists".into());
        }

        // Хешируем пароль через Argon2
        let password_hash = Self::hash_password(password)
            .map_err(|e| format!("Hashing error: {}", e))?;

        let new_user = user::ActiveModel {
            id: Set(Uuid::new_v4()),
            username: Set(username.to_string()),
            email: Set(email.to_string()),
            password_hash: Set(password_hash),
            display_name: Set(Some(username.to_string())),
            bio: Set(None),
            avatar_url: Set(None),
            role: Set(role.clone()),
            is_active: Set(true),
            last_login: Set(None),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        let user_model = new_user.insert(db).await?;
        println!("✅ Пользователь создан: {} (Роль: {:?})", email, role);

        Ok(user_model)
    }

    /// Проверка пароля при входе
    pub async fn verify_password_login(
        email: &str,
        password: &str,
        db: &sea_orm::DatabaseConnection
    ) -> Option<user::Model> {
        let user = user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(db)
            .await
            .ok()?
            .filter(|u| u.is_active)?;

        // Используем нашу функцию верификации
        if Self::verify_password(password, &user.password_hash) {
            Some(user)
        } else {
            None
        }
    }

    /// Получить пользователя по ID
    pub async fn get_user_by_id(
        db: &sea_orm::DatabaseConnection,
        user_id: Uuid,
    ) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find_by_id(user_id)
            .one(db)
            .await
    }

    /// Обновить профиль пользователя (имя, био, аватар)
    pub async fn update_profile(
        db: &sea_orm::DatabaseConnection,
        user_id: Uuid,
        display_name: Option<String>,
        bio: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<user::Model, DbErr> {
        let user = user::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("User not found".to_string()))?;

        let mut active_user: user::ActiveModel = user.into();

        if let Some(name) = display_name {
            active_user.display_name = Set(Some(name));
        }
        if let Some(b) = bio {
            active_user.bio = Set(Some(b));
        }
        if let Some(avatar) = avatar_url {
            active_user.avatar_url = Set(Some(avatar));
        }

        active_user.update(db).await
    }
}
