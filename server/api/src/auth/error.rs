use oauth2::{basic::BasicRequestTokenError, reqwest::AsyncHttpClientError};
use sea_orm::DbErr;

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error(transparent)]
    SeaOrm(DbErr),

    #[error(transparent)]
    Reqwest(reqwest::Error),

    #[error(transparent)]
    OAuth2(BasicRequestTokenError<AsyncHttpClientError>),
}
