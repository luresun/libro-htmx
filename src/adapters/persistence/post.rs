use async_trait::async_trait;
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    adapters::persistence::PostgresPersistence,
    app_error::{AppError, AppResult},
    entities::post::{Comment, Post},
    use_cases::post::PostPersistence,
};

#[derive(Debug, Serialize, FromRow)]
pub struct PostDb {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub user_id: Uuid,
    pub username: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<PostDb> for Post {
    fn from(post_db: PostDb) -> Self {
        Post {
            id: post_db.id,
            title: post_db.title,
            content: post_db.content,
            user_id: post_db.user_id,
            username: post_db.username,
            created_at: post_db
                .created_at
                .unwrap_or_else(|| chrono::Local::now().naive_local()),
            updated_at: post_db
                .updated_at
                .unwrap_or_else(|| chrono::Local::now().naive_local()),
        }
    }
}

#[async_trait]
impl PostPersistence for PostgresPersistence {
    async fn create_post(&self, title: &str, content: &str, user_id: &uuid::Uuid) -> AppResult<uuid::Uuid> {
        let post_id = Uuid::new_v4();

        sqlx::query("INSERT INTO posts (id, title, content, user_id) VALUES ($1, $2, $3, $4)")
            .bind(post_id)
            .bind(title)
            .bind(content)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;

        Ok(post_id)
    }

    async fn get_post_by_id(&self, post_id: &uuid::Uuid) -> AppResult<Option<Post>> {
        let row = sqlx::query_as::<_, PostDb>(
            "SELECT p.id, p.title, p.content, p.user_id, u.username, p.created_at, p.updated_at FROM posts p LEFT JOIN users u ON p.user_id = u.id WHERE p.id = $1"
        )
        .bind(post_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(row.map(Post::from))
    }

    async fn get_posts_by_user_id(&self, user_id: &uuid::Uuid) -> AppResult<Vec<Post>> {
        let rows = sqlx::query_as::<_, PostDb>(
            "SELECT p.id, p.title, p.content, p.user_id, u.username, p.created_at, p.updated_at FROM posts p LEFT JOIN users u ON p.user_id = u.id WHERE p.user_id = $1 ORDER BY p.created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows.into_iter().map(Post::from).collect())
    }

    async fn get_all_posts(&self) -> AppResult<Vec<Post>> {
        let rows = sqlx::query_as::<_, PostDb>(
            "SELECT p.id, p.title, p.content, p.user_id, u.username, p.created_at, p.updated_at FROM posts p LEFT JOIN users u ON p.user_id = u.id ORDER BY p.created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows.into_iter().map(Post::from).collect())
    }

    async fn like_post(&self, user_id: &uuid::Uuid, post_id: &uuid::Uuid) -> AppResult<()> {
        let like_id = Uuid::new_v4();
        sqlx::query("INSERT INTO likes (id, user_id, post_id) VALUES ($1, $2, $3) ON CONFLICT (user_id, post_id) DO NOTHING")
            .bind(like_id)
            .bind(user_id)
            .bind(post_id)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    async fn unlike_post(&self, user_id: &uuid::Uuid, post_id: &uuid::Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM likes WHERE user_id = $1 AND post_id = $2")
            .bind(user_id)
            .bind(post_id)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    async fn get_like_count(&self, post_id: &uuid::Uuid) -> AppResult<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM likes WHERE post_id = $1")
            .bind(post_id)
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::from)?;
        Ok(row.0)
    }

    async fn is_liked_by_user(&self, user_id: &uuid::Uuid, post_id: &uuid::Uuid) -> AppResult<bool> {
        let row: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM likes WHERE user_id = $1 AND post_id = $2)")
            .bind(user_id)
            .bind(post_id)
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::from)?;
        Ok(row.0)
    }

    async fn add_comment(&self, user_id: &uuid::Uuid, post_id: &uuid::Uuid, content: &str) -> AppResult<()> {
        let comment_id = Uuid::new_v4();
        sqlx::query("INSERT INTO comments (id, user_id, post_id, content) VALUES ($1, $2, $3, $4)")
            .bind(comment_id)
            .bind(user_id)
            .bind(post_id)
            .bind(content)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    async fn get_comments_by_post(&self, post_id: &uuid::Uuid) -> AppResult<Vec<Comment>> {
        let rows = sqlx::query_as::<_, CommentDb>(
            "SELECT c.id, c.user_id, c.post_id, c.content, c.created_at, u.username \
             FROM comments c \
             LEFT JOIN users u ON u.id = c.user_id \
             WHERE c.post_id = $1 \
             ORDER BY c.created_at ASC"
        )
        .bind(post_id)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows.into_iter().map(Comment::from).collect())
    }
}

#[derive(Debug, Serialize, FromRow)]
pub struct CommentDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub post_id: Uuid,
    pub content: String,
    pub created_at: Option<NaiveDateTime>,
    pub username: Option<String>,
}

impl From<CommentDb> for Comment {
    fn from(comment_db: CommentDb) -> Self {
        Comment {
            id: comment_db.id,
            user_id: comment_db.user_id,
            post_id: comment_db.post_id,
            username: comment_db.username,
            content: comment_db.content,
            created_at: comment_db
                .created_at
                .unwrap_or_else(|| chrono::Local::now().naive_local()),
        }
    }
}
