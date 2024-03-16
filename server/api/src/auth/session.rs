use axum::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use oauth2::{basic::BasicClient, url::Url, CsrfToken};
use sea_orm::{DatabaseConnection, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::entities::{providers, users};

use super::{
    error::BackendError,
    google::{google_callback, GoogleRes},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    id: Uuid,
    pub email: String,
    pub access_token: String,
}
// Here we've implemented `Debug` manually to avoid accidentally logging the
// access token.
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("email", &self.email)
            .field("access_token", &"[redacted]")
            .finish()
    }
}

impl AuthUser for User {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.access_token.as_bytes()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub provider: String,
    pub code: String,
    pub old_state: CsrfToken,
    pub new_state: CsrfToken,
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: DatabaseConnection,
    google_client: BasicClient,
}

impl Backend {
    pub fn new(db: DatabaseConnection, google_client: BasicClient) -> Self {
        Self { db, google_client }
    }

    pub fn authorize_url(&self) -> (Url, CsrfToken) {
        self.google_client
            .authorize_url(CsrfToken::new_random)
            .url()
    }
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = BackendError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        // Ensure the CSRF state has not been tampered with.
        if creds.old_state.secret() != creds.new_state.secret() {
            return Ok(None);
        };

        let GoogleRes {
            access_token,
            profile,
        } = google_callback(&self.google_client, creds.code)
            .await
            .unwrap();
        // Persist user in our database so we can use `get_user`.
        let user = users::ActiveModel {
            first_name: Set(profile.given_name),
            last_name: Set(profile.family_name),
            email: Set(profile.email),
            email_verified: Set(profile.email_verified),
            image: Set(Some(profile.picture)),
            ..Default::default()
        };
        let res = users::Entity::insert(user)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(users::Column::Email)
                    .update_column(users::Column::Image)
                    .to_owned(),
            )
            .exec_with_returning(&self.db)
            .await
            .map_err(BackendError::SeaOrm)?;

        let provider = providers::ActiveModel {
            name: Set("google".to_string()),
            provider_id: Set(profile.sub),
            user_id: Set(res.id),
            access_token: Set(access_token.to_string()),
            ..Default::default()
        };

        let _p = providers::Entity::insert(provider)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(providers::Column::ProviderId)
                    .update_column(providers::Column::AccessToken)
                    .to_owned(),
            )
            .exec(&self.db)
            .await;

        Ok(Some(User {
            id: res.id,
            email: res.email,
            access_token: access_token.to_string(),
        }))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<User>, Self::Error> {
        info!("UserID-----{}", user_id);
        let user_res = users::Entity::find_by_id(user_id.clone())
            .find_also_related(providers::Entity)
            .one(&self.db)
            .await
            .map_err(BackendError::SeaOrm)?;

        match user_res {
            Some(u) => Ok(Some(User {
                access_token: u.1.unwrap().access_token,
                email: u.0.email,
                id: u.0.id,
            })),
            None => {
                info!("User not found -----------");
                todo!();
            }
        }
    }
}

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<Backend>;
