// src/services/media_service.rs
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set
};
use tokio::io::AsyncWriteExt;
use futures_util::stream::StreamExt;
use crate::entities::media;
use uuid::Uuid;
use chrono::Utc;
use image::ImageReader;
use std::path::Path;
use axum::extract::multipart::{Field, Multipart};

pub struct MediaService;

impl MediaService {
    /// Разрешенные MIME типы для изображений
    const ALLOWED_MIME_TYPES: &'static [&'static str] = &[
        "image/jpeg",
        "image/png",
        "image/gif",
        "image/webp",
        "image/svg+xml",
    ];

    /// Максимальный размер файла (10 MB)
    const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

    /// Загрузка файла через axum multipart
    pub async fn upload_file(
        db: &sea_orm::DatabaseConnection,
        mut field: Field<'_>,
        uploader_id: Uuid,
        upload_dir: &str,
    ) -> Result<media::Model, Box<dyn std::error::Error + Send + Sync>> {
        // Проверка MIME типа
        let content_type: Option<String> = field
            .content_type()
            .map(|ct| ct.to_string());
        let content_type_str = content_type
            .unwrap_or(mime::APPLICATION_OCTET_STREAM.to_string());

        let mime_str = content_type_str.as_str();
        if !Self::ALLOWED_MIME_TYPES.contains(&mime_str) {
            return Err(format!("Invalid file type: {}", content_type_str).into());
        }

        // Генерация уникального имени файла
        // field.file_name() возвращает Option<&str> (синхронно)
        let original_filename = field
            .file_name()
            .unwrap_or("uploaded_file")
            .to_string();

        // Определяем расширение на основе MIME типа
        let extension = match content_type_str.as_str() {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            "image/svg+xml" => "svg",
            _ => "bin",
        };

        let unique_filename = format!(
            "{}_{}.{}",
            Uuid::new_v4(),
            Utc::now().timestamp(),
            extension
        );
        let filepath = Path::new(upload_dir).join(&unique_filename);

        // Создаем директорию если не существует
        if let Some(parent) = filepath.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Сохраняем файл на диск
        let mut dest = tokio::fs::File::create(&filepath).await?;

        // Читаем чанки и записываем в файл
        let mut total_size = 0u64;
        loop {
            match field.next().await {
                Some(Ok(chunk)) => {
                    dest.write_all(&chunk).await?;
                    total_size += chunk.len() as u64;
                }
                Some(Err(e)) => {
                    return Err(format!("Failed to read chunk: {}", e).into());
                }
                None => break,
            }
        }
        dest.flush().await?;

        // Получаем размеры изображения
        let (width, height) = Self::get_image_dimensions(&filepath)
            .unwrap_or((None, None));

        // Создаем запись в БД
        let active_media = media::ActiveModel {
            id: Set(Uuid::new_v4()),
            filename: Set(original_filename.clone()),
            filepath: Set(filepath.to_string_lossy().to_string()),
            mime_type: Set(content_type_str),
            size: Set(total_size as i64),
            width: Set(width),
            height: Set(height),
            alt_text: Set(None),
            caption: Set(None),
            description: Set(None),
            uploader_id: Set(uploader_id),
            created_at: Set(Utc::now()),
        };

        let media_model = active_media.insert(db).await?;
        Ok(media_model)
    }

    /// Загрузка нескольких файлов через axum multipart
    pub async fn upload_files(
        db: &sea_orm::DatabaseConnection,
        mut multipart: Multipart,
        uploader_id: Uuid,
        upload_dir: &str,
    ) -> Result<Vec<media::Model>, Box<dyn std::error::Error + Send + Sync>> {
        let mut uploaded_files = Vec::new();

        while let Some(field) = multipart.next_field().await? {
            let media = Self::upload_file(db, field, uploader_id, upload_dir).await?;
            uploaded_files.push(media);
        }

        Ok(uploaded_files)
    }

    /// Получение размеров изображения
    fn get_image_dimensions(path: &Path) -> Result<(Option<i32>, Option<i32>), Box<dyn std::error::Error>> {
        let img = ImageReader::open(path)?.with_guessed_format()?;
        let dimensions = img.into_dimensions()?;
        Ok((Some(dimensions.0 as i32), Some(dimensions.1 as i32)))
    }

    /// Получить все медиафайлы с пагинацией
    pub async fn list_media(
        db: &sea_orm::DatabaseConnection,
        page: u64,
        per_page: u64,
        uploader_id: Option<Uuid>,
    ) -> Result<Vec<media::Model>, sea_orm::DbErr> {
        let mut query = media::Entity::find()
            .order_by_desc(media::Column::CreatedAt);

        if let Some(uid) = uploader_id {
            query = query.filter(media::Column::UploaderId.eq(uid));
        }

        // ✅ ИСПРАВЛЕНИЕ: PaginatorTrait теперь в scope
        query
            .paginate(db, per_page)
            .fetch_page(page - 1)
            .await
    }

    /// Получить медиафайл по ID
    pub async fn get_media_by_id(
        db: &sea_orm::DatabaseConnection,
        id: Uuid,
    ) -> Result<media::Model, sea_orm::DbErr> {
        media::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound("Media not found".to_string()))
    }

    /// Обновить метаданные медиафайла
    pub async fn update_media(
        db: &sea_orm::DatabaseConnection,
        id: Uuid,
        alt_text: Option<String>,
        caption: Option<String>,
        description: Option<String>,
    ) -> Result<media::Model, sea_orm::DbErr> {
        let media_model = Self::get_media_by_id(db, id).await?;
        let mut active_media: media::ActiveModel = media_model.into();

        if let Some(alt) = alt_text {
            active_media.alt_text = Set(Some(alt));
        }
        if let Some(cap) = caption {
            active_media.caption = Set(Some(cap));
        }
        if let Some(desc) = description {
            active_media.description = Set(Some(desc));
        }

        active_media.update(db).await
    }

    /// Удалить медиафайл
    pub async fn delete_media(
        db: &sea_orm::DatabaseConnection,
        id: Uuid,
    ) -> Result<(), sea_orm::DbErr> {
        let media_model = Self::get_media_by_id(db, id).await?;

        // Удаляем файл с диска (игнорируем ошибки, если файл уже удален)
        let _ = tokio::fs::remove_file(&media_model.filepath).await;

        // Удаляем запись из БД
        media::Entity::delete_by_id(id).exec(db).await?;

        Ok(())
    }
}
