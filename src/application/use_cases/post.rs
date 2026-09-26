use std::sync::Arc;

use async_trait::async_trait;
use tracing::{info, instrument};

use crate::{
    app_error::AppResult,
    entities::post::{Comment, Post},
};

#[async_trait]
pub trait PostPersistence: Send + Sync {
    async fn create_post(&self, title: &str, content: &str, user_id: &uuid::Uuid) -> AppResult<uuid::Uuid>;
    async fn get_post_by_id(&self, post_id: &uuid::Uuid) -> AppResult<Option<Post>>;
    async fn get_posts_by_user_id(&self, user_id: &uuid::Uuid) -> AppResult<Vec<Post>>;
    async fn get_all_posts(&self) -> AppResult<Vec<Post>>;
    async fn like_post(&self, user_id: &uuid::Uuid, post_id: &uuid::Uuid) -> AppResult<()>;
    async fn unlike_post(&self, user_id: &uuid::Uuid, post_id: &uuid::Uuid) -> AppResult<()>;
    async fn get_like_count(&self, post_id: &uuid::Uuid) -> AppResult<i64>;
    async fn is_liked_by_user(&self, user_id: &uuid::Uuid, post_id: &uuid::Uuid) -> AppResult<bool>;
    async fn add_comment(&self, user_id: &uuid::Uuid, post_id: &uuid::Uuid, content: &str) -> AppResult<()>;
    async fn get_comments_by_post(&self, post_id: &uuid::Uuid) -> AppResult<Vec<crate::entities::post::Comment>>;
}

#[derive(Clone)]
pub struct PostUseCases {
    pub persistence: Arc<dyn PostPersistence>,
}

impl PostUseCases {
    pub fn new(persistence: Arc<dyn PostPersistence>) -> Self {
        Self { persistence }
    }

    #[instrument(skip(self))]
    pub async fn create_post(&self, title: &str, content: &str, user_id: &uuid::Uuid) -> AppResult<uuid::Uuid> {
        info!("Creating post for user {}", user_id);
        let post_id = self.persistence.create_post(title, content, user_id).await?;
        info!("Post created with ID {}", post_id);
        Ok(post_id)
    }

    #[instrument(skip(self))]
    pub async fn get_post(&self, post_id: &uuid::Uuid) -> AppResult<Option<Post>> {
        info!("Getting post {}", post_id);
        self.persistence.get_post_by_id(post_id).await
    }

    #[instrument(skip(self))]
    pub async fn get_user_posts(&self, user_id: &uuid::Uuid) -> AppResult<Vec<Post>> {
        info!("Getting posts for user {}", user_id);
        self.persistence.get_posts_by_user_id(user_id).await
    }

    #[instrument(skip(self))]
    pub async fn get_all_posts(&self) -> AppResult<Vec<Post>> {
        info!("Getting all posts");
        self.persistence.get_all_posts().await
    }

    #[instrument(skip(self))]
    pub async fn like_post(&self, user_id: &uuid::Uuid, post_id: &uuid::Uuid) -> AppResult<()> {
        info!("Liking post {} by user {}", post_id, user_id);
        self.persistence.like_post(user_id, post_id).await
    }

    #[instrument(skip(self))]
    pub async fn unlike_post(&self, user_id: &uuid::Uuid, post_id: &uuid::Uuid) -> AppResult<()> {
        info!("Unliking post {} by user {}", post_id, user_id);
        self.persistence.unlike_post(user_id, post_id).await
    }

    #[instrument(skip(self))]
    pub async fn toggle_like(&self, user_id: &uuid::Uuid, post_id: &uuid::Uuid) -> AppResult<bool> {
        let currently_liked = self.persistence.is_liked_by_user(user_id, post_id).await?;
        if currently_liked {
            self.unlike_post(user_id, post_id).await?;
            Ok(false)
        } else {
            self.like_post(user_id, post_id).await?;
            Ok(true)
        }
    }

    #[instrument(skip(self))]
    pub async fn get_like_count(&self, post_id: &uuid::Uuid) -> AppResult<i64> {
        self.persistence.get_like_count(post_id).await
    }

    #[instrument(skip(self))]
    pub async fn is_liked_by_user(&self, user_id: &uuid::Uuid, post_id: &uuid::Uuid) -> AppResult<bool> {
        self.persistence.is_liked_by_user(user_id, post_id).await
    }

    #[instrument(skip(self))]
    pub async fn add_comment(&self, user_id: &uuid::Uuid, post_id: &uuid::Uuid, content: &str) -> AppResult<()> {
        info!("Adding comment to post {} by user {}", post_id, user_id);
        self.persistence.add_comment(user_id, post_id, content).await
    }

    #[instrument(skip(self))]
    pub async fn get_comments(&self, post_id: &uuid::Uuid) -> AppResult<Vec<Comment>> {
        info!("Getting comments for post {}", post_id);
        self.persistence.get_comments_by_post(post_id).await
    }
}

#[cfg(test)]
mod test {
    use async_trait::async_trait;
    use super::*;

    struct MockPostPersistence;

    #[async_trait]
    impl PostPersistence for MockPostPersistence {
        async fn create_post(&self, _title: &str, _content: &str, _user_id: &uuid::Uuid) -> AppResult<uuid::Uuid> {
            Ok(uuid::Uuid::new_v4())
        }
        async fn get_post_by_id(&self, _post_id: &uuid::Uuid) -> AppResult<Option<Post>> {
            Ok(None)
        }
        async fn get_posts_by_user_id(&self, _user_id: &uuid::Uuid) -> AppResult<Vec<Post>> {
            Ok(vec![])
        }
        async fn get_all_posts(&self) -> AppResult<Vec<Post>> {
            Ok(vec![])
        }
        async fn like_post(&self, _user_id: &uuid::Uuid, _post_id: &uuid::Uuid) -> AppResult<()> {
            Ok(())
        }
        async fn unlike_post(&self, _user_id: &uuid::Uuid, _post_id: &uuid::Uuid) -> AppResult<()> {
            Ok(())
        }
        async fn get_like_count(&self, _post_id: &uuid::Uuid) -> AppResult<i64> {
            Ok(0)
        }
        async fn is_liked_by_user(&self, _user_id: &uuid::Uuid, _post_id: &uuid::Uuid) -> AppResult<bool> {
            Ok(false)
        }
        async fn add_comment(&self, _user_id: &uuid::Uuid, _post_id: &uuid::Uuid, _content: &str) -> AppResult<()> {
            Ok(())
        }
        async fn get_comments_by_post(&self, _post_id: &uuid::Uuid) -> AppResult<Vec<Comment>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn toggle_like_works() {
        let persistence = Arc::new(MockPostPersistence);
        let use_cases = PostUseCases::new(persistence);
        let post_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();

        let result = use_cases.toggle_like(&user_id, &post_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
}
