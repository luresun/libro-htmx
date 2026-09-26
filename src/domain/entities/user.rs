use uuid::Uuid;

#[derive(Debug)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub picture_url: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}
