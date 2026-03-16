// src/services/comment_service.rs
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;
use uuid::Uuid;
use crate::entities::comment::{self, CommentModerationStatus};

pub struct CommentService;

impl CommentService {
    /// Загрузить комментарии поста с ответами
    pub async fn get_comments_with_replies(
        db: &DatabaseConnection,
        post_id: Uuid,
    ) -> Result<Vec<CommentWithReplies>, DbErr> {
        // 1. Загрузить все корневые комментарии (без родителя)
        let root_comments = comment::Entity::find()
            .filter(comment::Column::PostId.eq(post_id))
            .filter(comment::Column::ParentId.is_null())
            .filter(comment::Column::Status.eq(CommentModerationStatus::Approved))
            .order_by_asc(comment::Column::CreatedAt)
            .all(db)
            .await?;

        // 2. Загрузить все ответы одним запросом (эффективно!)
        let comment_ids: Vec<Uuid> = root_comments.iter().map(|c| c.id).collect();
        let all_replies = comment::Entity::find()
            .filter(comment::Column::ParentId.is_in(comment_ids))
            .filter(comment::Column::Status.eq(CommentModerationStatus::Approved))
            .order_by_asc(comment::Column::CreatedAt)
            .all(db)
            .await?;

        // 3. Сгруппировать ответы по родительскому комментарию
        let mut replies_map: std::collections::HashMap<Uuid, Vec<comment::Model>> =
            std::collections::HashMap::new();

        for reply in all_replies {
            if let Some(parent_id) = reply.parent_id {
                replies_map.entry(parent_id).or_default().push(reply);
            }
        }

        // 4. Собрать результат
        let result = root_comments
            .into_iter()
            .map(|comment| {
                let replies = replies_map.remove(&comment.id).unwrap_or_default();
                CommentWithReplies { comment, replies }
            })
            .collect();

        Ok(result)
    }
}

#[derive(Debug, Serialize)]
pub struct CommentWithReplies {
    pub comment: comment::Model,
    pub replies: Vec<comment::Model>,
}
