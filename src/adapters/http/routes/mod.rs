pub mod user;
pub mod web;
pub mod post;

use crate::adapters::http::app_state::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(user::router())
        .merge(web::router())
        .merge(post::router())
}
