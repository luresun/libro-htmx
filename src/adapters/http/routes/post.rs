use askama::Template;
use axum::{
    Router,
    extract::{State, Path, Form},
    http::{HeaderMap, StatusCode, Response},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use axum::body::Body;
use std::sync::Arc;
use tower_sessions::Session;
use uuid::Uuid;

use crate::{adapters::http::app_state::AppState, use_cases::post::PostUseCases};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserSession {
    user_id: Uuid,
    username: String,
}

#[derive(Template)]
#[template(path = "view_post.html")]
struct ViewPostTemplate {
    is_logged_in: bool,
    post: Option<PostDetail>,
    comments: Vec<CommentDisplay>,
}

#[derive(Template)]
#[template(path = "post_fragment.html")]
struct PostFragmentTemplate {
    is_logged_in: bool,
    post: Option<PostDetail>,
    comments: Vec<CommentDisplay>,
}

#[derive(Debug, Clone, Serialize)]
struct PostDetail {
    id: String,
    title: String,
    content: String,
    created_at: String,
    username: String,
    like_count: i64,
    is_liked_by_user: bool,
}

#[derive(Debug, Clone, Serialize)]
struct CommentDisplay {
    id: String,
    username: String,
    content: String,
    created_at: String,
}

async fn render_post_view(
    post_use_cases: &Arc<PostUseCases>,
    session: &Session,
    post_id: Uuid,
    fragment: bool,
) -> Response<Body> {
    let user_session = session.get::<UserSession>("user").await.unwrap();
    let is_logged_in = user_session.is_some();
    let user_id = user_session.as_ref().map(|s| s.user_id);

    let (post_detail, comments) = match post_use_cases.get_post(&post_id).await {
        Ok(Some(post)) => {
            let like_count = post_use_cases.get_like_count(&post_id).await.unwrap_or(0);
            let is_liked = match user_id {
                Some(uid) => post_use_cases
                    .is_liked_by_user(&uid, &post_id)
                    .await
                    .unwrap_or(false),
                None => false,
            };

            let comments = post_use_cases.get_comments(&post_id).await.unwrap_or_default();
            let comment_displays = comments
                .into_iter()
                .map(|c| CommentDisplay {
                    id: c.id.to_string(),
                    username: c
                        .username
                        .unwrap_or_else(|| "Anonymous".to_string()),
                    content: c.content,
                    created_at: c.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                })
                .collect();

            let post_detail = PostDetail {
                id: post.id.to_string(),
                title: post.title,
                content: post.content,
                created_at: post.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                username: post.username.unwrap_or_else(|| "Anonymous".to_string()),
                like_count,
                is_liked_by_user: is_liked,
            };

            (Some(post_detail), comment_displays)
        }
        Ok(None) => (None, Vec::new()),
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Failed to fetch post".to_string()),
            )
                .into_response();
        }
    };

    let html = if fragment {
        PostFragmentTemplate {
            is_logged_in,
            post: post_detail,
            comments,
        }
        .render()
    } else {
        ViewPostTemplate {
            is_logged_in,
            post: post_detail,
            comments,
        }
        .render()
    };

    match html {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html("Failed to render post".to_string()),
        )
            .into_response(),
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/posts/{id}", get(view_post))
        .route("/posts/{id}/like", post(toggle_like))
        .route("/posts/{id}/comment", post(add_comment))
}

pub async fn view_post(
    State(post_use_cases): State<Arc<PostUseCases>>,
    session: Session,
    Path(post_id): Path<Uuid>,
) -> Response<Body> {
    render_post_view(&post_use_cases, &session, post_id, false).await
}

pub async fn toggle_like(
    State(post_use_cases): State<Arc<PostUseCases>>,
    session: Session,
    Path(post_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response<Body> {
    let user_session = match session.get::<UserSession>("user").await.unwrap() {
        Some(s) => s,
        None => return (StatusCode::UNAUTHORIZED, "Not logged in").into_response(),
    };

    if let Err(err) = post_use_cases
        .toggle_like(&user_session.user_id, &post_id)
        .await
    {
        tracing::error!(?err, %post_id, "failed to toggle like");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to toggle like").into_response();
    }

    render_post_view(&post_use_cases, &session, post_id, is_htmx(&headers)).await
}

pub async fn add_comment(
    State(post_use_cases): State<Arc<PostUseCases>>,
    session: Session,
    Path(post_id): Path<Uuid>,
    headers: HeaderMap,
    Form(payload): Form<CommentPayload>,
) -> Response<Body> {
    let user_session = match session.get::<UserSession>("user").await.unwrap() {
        Some(s) => s,
        None => return (StatusCode::UNAUTHORIZED, "Not logged in").into_response(),
    };

    let content = payload.content.trim();
    if content.is_empty() {
        return (StatusCode::BAD_REQUEST, "Comment cannot be empty").into_response();
    }

    if let Err(err) = post_use_cases
        .add_comment(&user_session.user_id, &post_id, content)
        .await
    {
        tracing::error!(?err, %post_id, "failed to add comment");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to add comment").into_response();
    }

    render_post_view(&post_use_cases, &session, post_id, is_htmx(&headers)).await
}

fn is_htmx(headers: &HeaderMap) -> bool {
    headers
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case("true"))
}

#[derive(Deserialize)]
pub struct CommentPayload {
    content: String,
}

