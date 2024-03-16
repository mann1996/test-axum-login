use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use tracing::info;

use crate::auth::session::AuthSession;

pub fn router() -> Router {
    Router::new().route("/", get(protected))
}

async fn protected(auth_session: AuthSession) -> impl IntoResponse {
    info!("protected route--------");
    match auth_session.user {
        Some(user) => {
            return user.email.to_string();
        }

        None => "You're not logged in.\nVisit `/auth/google` to do so.".to_string(),
    }
}
