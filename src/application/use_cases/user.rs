use std::sync::Arc;

use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};
use tracing::{info, instrument, warn};

use crate::{
    app_error::{AppError, AppResult},
    entities::user::User,
};

#[async_trait]
pub trait UserPersistence: Send + Sync {
    async fn create_user(&self, username: &str, email: &str, password_hash: &str) -> AppResult<()>;
    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;
    async fn update_picture_url(&self, user_id: &uuid::Uuid, picture_url: &str) -> AppResult<()>;
}

pub trait UserCredentialsHasher: Send + Sync {
    fn hash_password(&self, password: &str) -> AppResult<String>;
    fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct UserUseCases {
    hasher: Arc<dyn UserCredentialsHasher>,
    pub persistence: Arc<dyn UserPersistence>,
}

impl UserUseCases {
    pub fn new(
        hasher: Arc<dyn UserCredentialsHasher>,
        persistence: Arc<dyn UserPersistence>,
    ) -> Self {
        Self {
            hasher,
            persistence,
        }
    }

    #[instrument(skip(self))]
    pub async fn add(&self, username: &str, email: &str, password: &SecretString) -> AppResult<()> {
        info!("Adding user...");

        let hash = &self.hasher.hash_password(password.expose_secret())?;
        self.persistence.create_user(username, email, hash).await?;

        info!("Adding user finished.");

        Ok(())
    }

    #[instrument(skip(self, password))]
    pub async fn authenticate(&self, username: &str, password: &str) -> AppResult<User> {
        info!("Authenticating user {}", username);

        let user = self
            .persistence
            .find_by_username(username)
            .await?
            .ok_or_else(|| AppError::AuthenticationError("Invalid credentials".to_string()))?;

        if self.hasher.verify_password(password, &user.password_hash)? {
            info!("Authentication successful for {}", username);
            Ok(user)
        } else {
            warn!("Authentication failed for {}", username);
            Err(AppError::AuthenticationError(
                "Invalid credentials".to_string(),
            ))
        }
    }

    #[instrument(skip(self))]
    pub async fn update_picture(&self, user_id: &uuid::Uuid, picture_url: &str) -> AppResult<()> {
        info!("Updating picture for user {}", user_id);
        self.persistence
            .update_picture_url(user_id, picture_url)
            .await?;
        info!("Picture updated for user {}", user_id);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use async_trait::async_trait;

    use super::*;

    struct MockUserPersistence;

    #[async_trait]
    impl UserPersistence for MockUserPersistence {
        async fn create_user(
            &self,
            username: &str,
            email: &str,
            _password_hash: &str,
        ) -> AppResult<()> {
            assert_eq!(username, "testuser");
            assert_eq!(email, "testuser@gmail.com");

            Ok(())
        }

        async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
            if username == "testuser" {
                Ok(Some(User {
                    id: uuid::Uuid::new_v4(),
                    username: username.to_string(),
                    email: "testuser@gmail.com".to_string(),
                    password_hash: "testuser_pw_hash".to_string(),
                    picture_url: None,
                    created_at: chrono::Local::now().naive_local(),
                }))
            } else {
                Ok(None)
            }
        }

        async fn update_picture_url(
            &self,
            _user_id: &uuid::Uuid,
            _picture_url: &str,
        ) -> AppResult<()> {
            Ok(())
        }
    }

    struct MockUserCredentialsHasher;

    impl UserCredentialsHasher for MockUserCredentialsHasher {
        fn hash_password(&self, password: &str) -> AppResult<String> {
            Ok(format!("{}_hash", password))
        }

        fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool> {
            Ok(hash == format!("{}_hash", password))
        }
    }

    #[tokio::test]
    async fn add_user_works() {
        let user_use_cases = UserUseCases::new(
            Arc::new(MockUserCredentialsHasher),
            Arc::new(MockUserPersistence),
        );

        let result = user_use_cases
            .add("testuser", "testuser@gmail.com", &"testuser_pw".into())
            .await;

        assert!(result.is_ok());
    }
}
