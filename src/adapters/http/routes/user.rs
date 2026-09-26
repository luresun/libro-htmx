use askama::Template;
use axum::{
    Router,
    extract::{Form, Multipart, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_sessions::Session;
use tracing::info;
use uuid::Uuid;

use crate::{adapters::http::app_state::AppState, use_cases::user::UserUseCases};

#[derive(Debug, Clone, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserSession {
    user_id: Uuid,
    username: String,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    is_logged_in: bool,
    message: String,
    message_type: String,
}

#[derive(Template)]
#[template(path = "profile.html")]
struct ProfileTemplate {
    is_logged_in: bool,
    user_id: String,
    username: String,
    email: String,
    created_at: String,
    picture_url: String,
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {
    is_logged_in: bool,
    message: String,
    message_type: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login_page))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/profile", get(profile))
        .route("/profile/upload", post(upload_picture))
        .route("/register", post(register))
}

async fn upload_picture(
    State(user_use_cases): State<Arc<UserUseCases>>,
    session: Session,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Some(user_session) = session.get::<UserSession>("user").await.unwrap() {
        while let Some(field) = multipart.next_field().await.unwrap() {
            if field.name().unwrap() == "picture" {
                let filename = field.file_name().unwrap().to_string();
                let extension = filename.rsplit('.').next().unwrap_or("jpg");

                if !["jpg", "jpeg", "png", "gif", "webp"]
                    .contains(&extension.to_lowercase().as_str())
                {
                    return (
                        StatusCode::BAD_REQUEST,
                        "Invalid file type. Only JPG, PNG, GIF, WEBP allowed.",
                    )
                        .into_response();
                }

                let data = field.bytes().await.unwrap();
                if data.len() > 5 * 1024 * 1024 {
                    return (
                        StatusCode::BAD_REQUEST,
                        "File too large. Maximum size is 5MB.",
                    )
                        .into_response();
                }

                let user_id = user_session.user_id;
                let new_filename = format!("{}.{}", user_id, extension);
                let filepath = format!("uploads/{}", new_filename);

                tokio::fs::create_dir_all("uploads").await.unwrap();
                tokio::fs::write(&filepath, data).await.unwrap();

                let picture_url = format!("/uploads/{}", new_filename);
                user_use_cases
                    .update_picture(&user_id, &picture_url)
                    .await
                    .unwrap();

                info!("Profile picture uploaded for user {}", user_id);
            }
        }

        Redirect::to("/profile").into_response()
    } else {
        Redirect::to("/login").into_response()
    }
}

pub async fn login_page(session: Session) -> impl IntoResponse {
    if session.get::<UserSession>("user").await.unwrap().is_some() {
        return Redirect::to("/profile").into_response();
    }

    Html(
        LoginTemplate {
            is_logged_in: false,
            message: String::new(),
            message_type: String::new(),
        }
        .render()
        .unwrap(),
    )
    .into_response()
}

pub async fn login(
    State(user_use_cases): State<Arc<UserUseCases>>,
    session: Session,
    Form(payload): Form<LoginPayload>,
) -> impl IntoResponse {
    info!("Login attempt for user: {}", payload.username);

    match user_use_cases
        .authenticate(&payload.username, &payload.password)
        .await
    {
        Ok(user) => {
            info!("Creating session for user: {}", user.username);
            let user_session = UserSession {
                user_id: user.id,
                username: user.username.clone(),
            };
            session.insert("user", &user_session).await.unwrap();
            info!("Session created");

            Redirect::to("/").into_response()
        }
        Err(_) => {
            let template = LoginTemplate {
                is_logged_in: false,
                message: "Invalid username or password".to_string(),
                message_type: "error".to_string(),
            };
            Html(template.render().unwrap()).into_response()
        }
    }
}

pub async fn logout(session: Session) -> impl IntoResponse {
    session.flush().await.unwrap();
    Redirect::to("/login").into_response()
}

pub async fn register(
    State(user_use_cases): State<Arc<UserUseCases>>,
    Form(payload): Form<RegisterPayload>,
) -> impl IntoResponse {
    info!("Register attempt for user: {}", payload.username);

    let secret_password = SecretString::new(payload.password.into_boxed_str());

    match user_use_cases
        .add(&payload.username, &payload.email, &secret_password)
        .await
    {
        Ok(_) => Redirect::to("/login").into_response(),
        Err(_) => Html(
            RegisterTemplate {
                is_logged_in: false,
                message: "Registration failed. Username may already exist.".to_string(),
                message_type: "error".to_string(),
            }
            .render()
            .unwrap(),
        )
        .into_response(),
    }
}

pub async fn profile(
    State(user_use_cases): State<Arc<UserUseCases>>,
    session: Session,
) -> impl IntoResponse {
    info!("Profile page accessed, session ID: {:?}", session.id());
    let user_session = session.get::<UserSession>("user").await.unwrap();

    info!("Session user data: {:?}", user_session);

    if let Some(user_session) = user_session {
        info!("Found user in session: {}", user_session.username);
        if let Ok(user) = user_use_cases
            .persistence
            .find_by_username(&user_session.username)
            .await
        {
            if let Some(user) = user {
                let template = ProfileTemplate {
                    is_logged_in: true,
                    user_id: user.id.to_string(),
                    username: user.username,
                    email: user.email,
                    created_at: user.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    picture_url: user.picture_url.unwrap_or_default(),
                };
                return Html(template.render().unwrap());
            }
        }
    } else {
        info!("No user session found");
    }

    let template = LoginTemplate {
        is_logged_in: false,
        message: "Please log in to view your profile".to_string(),
        message_type: "error".to_string(),
    };
    Html(template.render().unwrap())
}
