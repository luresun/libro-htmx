use async_trait::async_trait;
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    adapters::persistence::PostgresPersistence,
    app_error::{AppError, AppResult},
    entities::user::User,
    use_cases::user::UserPersistence,
};

// User struct as stored in the db.
#[derive(Debug, Serialize, FromRow)]
pub struct UserDb {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub picture_url: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

impl From<UserDb> for User {
    fn from(user_db: UserDb) -> Self {
        User {
            id: user_db.id,
            username: user_db.username,
            email: user_db.email,
            password_hash: user_db.password_hash,
            picture_url: user_db.picture_url,
            created_at: user_db
                .created_at
                .unwrap_or_else(|| chrono::Local::now().naive_local()),
        }
    }
}

#[async_trait]
impl UserPersistence for PostgresPersistence {
    async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> AppResult<()> {
        let uuid = Uuid::new_v4();

        sqlx::query("INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4)")
            .bind(uuid)
            .bind(username)
            .bind(email)
            .bind(password_hash)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let row = sqlx::query_as::<_, UserDb>("SELECT id, username, email, password_hash, picture_url, created_at FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(AppError::from)?;

        Ok(row.map(User::from))
    }

    async fn update_picture_url(&self, user_id: &uuid::Uuid, picture_url: &str) -> AppResult<()> {
        sqlx::query("UPDATE users SET picture_url = $1 WHERE id = $2")
            .bind(picture_url)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }
}
