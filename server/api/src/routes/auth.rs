use crate::auth::session::{AuthSession, Credentials};

use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};

use axum_login::tower_sessions::Session;
use oauth2::CsrfToken;
use serde::Deserialize;
use tracing::info;

pub const CSRF_STATE_KEY: &str = "oauth.csrf-state";
pub const NEXT_URL_KEY: &str = "auth.next-url";

#[derive(Debug, Clone, Deserialize)]
pub struct AuthzResp {
    code: String,
    state: CsrfToken,
}

// This allows us to extract the "next" field from the query string. We use this
// to redirect after log in.
#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

pub fn router() -> Router {
    Router::new()
        .route("/auth/google", get(google_login))
        .route("/auth/google_callback", get(google_callback))
}

async fn google_login(
    auth_session: AuthSession,
    session: Session,
    Query(NextUrl { next }): Query<NextUrl>,
) -> impl IntoResponse {
    let (auth_url, csrf_state) = auth_session.backend.authorize_url();

    session
        .insert(CSRF_STATE_KEY, csrf_state.secret())
        .await
        .expect("Serialization should not fail.");

    session
        .insert(NEXT_URL_KEY, next)
        .await
        .expect("Serialization should not fail.");

    Redirect::to(auth_url.as_str()).into_response()
}

async fn google_callback(
    mut auth_session: AuthSession,
    session: Session,
    Query(AuthzResp {
        code,
        state: new_state,
    }): Query<AuthzResp>,
) -> impl IntoResponse {
    let Ok(Some(old_state)) = session.get(CSRF_STATE_KEY).await else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let creds = Credentials {
        provider: "google".to_string(),
        code,
        old_state,
        new_state,
    };

    let user = match auth_session.authenticate(creds).await {
        Ok(Some(user)) => {
            info!("user found login route");
            user
        }
        Ok(None) => {
            info!("login UNAUTHORIZED");
            return (StatusCode::UNAUTHORIZED, "UNAUTHORIZED").into_response();
        }
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if let Ok(Some(next)) = session.remove::<String>(NEXT_URL_KEY).await {
        Redirect::to(&next).into_response()
    } else {
        Redirect::to("/").into_response()
    }
}
