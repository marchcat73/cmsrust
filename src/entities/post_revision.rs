// // src/entities/post_revision.rs
// #[derive(Clone, Debug, DeriveEntityModel)]
// #[sea_orm(table_name = "post_revisions")]
// pub struct Model {
//     #[sea_orm(primary_key)]
//     pub id: Uuid,
//     pub post_id: Uuid,
//     pub author_id: Uuid,
//     pub title: String,
//     pub content: String,
//     pub created_at: DateTime,
// }

// // Автоматическое создание ревизии при обновлении поста
// impl ActiveModelBehavior for ActiveModel {
//     async fn before_save<C>(
//         self,
//         db: &C,
//         insert: bool,
//     ) -> Result<Self, DbErr>
//     where
//         C: ConnectionTrait,
//     {
//         if !insert {
//             // Сохраняем текущую версию как ревизию
//             let current = self.clone().one(db).await?;
//             if let Some(post) = current {
//                 post_revision::ActiveModel {
//                     id: Set(Uuid::new_v4()),
//                     post_id: Set(post.id),
//                     author_id: Set(post.author_id),
//                     title: Set(post.title.clone()),
//                     content: Set(post.content.clone()),
//                     created_at: Set(chrono::Utc::now()),
//                 }
//                 .insert(db)
//                 .await?;
//             }
//         }
//         Ok(self)
//     }
// }
