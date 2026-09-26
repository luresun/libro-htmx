use crate::adapters::http::app_state::AppState;
use askama::Template;
use axum::{
    Router,
    extract::{Form, State},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::services::ServeDir;
use tower_sessions::Session;
use crate::use_cases::post::PostUseCases;

#[derive(serde::Deserialize)]
struct UserSession {
    username: String,
    user_id: uuid::Uuid,
}

#[derive(Debug, Clone, Deserialize)]
struct CreatePostPayload {
    pub title: String,
    pub content: String,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub is_logged_in: bool,
    pub username: Option<String>,
    pub posts: Vec<PostDisplay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostDisplay {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub username: String,
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    pub is_logged_in: bool,
    pub message: String,
    pub message_type: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index).post(create_post))
        .route("/register", get(register_page))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/uploads", ServeDir::new("uploads"))
}

pub async fn index(
    State(post_use_cases): State<Arc<PostUseCases>>,
    session: Session,
) -> impl IntoResponse {
    let (is_logged_in, username) =
        if let Some(user_session) = session.get::<UserSession>("user").await.unwrap() {
            (true, Some(user_session.username))
        } else {
            (false, None)
        };

    let posts_result = post_use_cases.get_all_posts().await;
    let posts = match posts_result {
        Ok(posts) => posts
            .into_iter()
            .map(|p| PostDisplay {
                id: p.id.to_string(),
                title: p.title,
                content: p.content.chars().take(200).collect::<String>() + if p.content.len() > 200 { "..." } else { "" },
                created_at: p.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                username: p.username.unwrap_or_else(|| "Anonymous".to_string()),
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    let template = IndexTemplate {
        is_logged_in,
        username,
        posts,
    };
    Html(template.render().unwrap())
}

async fn create_post(
    State(post_use_cases): State<Arc<PostUseCases>>,
    session: Session,
    Form(payload): Form<CreatePostPayload>,
) -> impl IntoResponse {
    let user_session = match session.get::<UserSession>("user").await.unwrap() {
        Some(s) => s,
        None => return Redirect::to("/login").into_response(),
    };

    if payload.title.trim().is_empty() || payload.content.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "Title and content cannot be empty").into_response();
    }

    match post_use_cases
        .create_post(&payload.title, &payload.content, &user_session.user_id)
        .await
    {
        Ok(_) => Redirect::to("/").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create post").into_response(),
    }
}

pub async fn register_page(session: Session) -> impl IntoResponse {
    if session.get::<UserSession>("user").await.unwrap().is_some() {
        return Redirect::to("/profile").into_response();
    }

    let template = RegisterTemplate {
        is_logged_in: false,
        message: String::new(),
        message_type: String::new(),
    };
    Html(template.render().unwrap()).into_response()
}

pub struct Html<T>(pub T);

impl<T> IntoResponse for Html<T>
where
    T: Into<axum::body::Body>,
{
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.0.into())
            .unwrap()
    }
}
