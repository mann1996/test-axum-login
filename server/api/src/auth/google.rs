use axum::http::header::{AUTHORIZATION, USER_AGENT};

use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, RedirectUrl, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use std::env;

use super::error::BackendError as Error;

pub fn build_google_client() -> BasicClient {
    let redirect_url = env::var("GOOGLE_CALLBACK_URL").unwrap();

    let auth_url = AuthUrl::new(
        "https://accounts.google.com/o/oauth2/v2/auth?scope=openid%20profile%20email".to_string(),
    )
    .expect("Invalid authorization endpoint URL");

    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");

    let google_client_id = env::var("GOOGLE_CLIENT_ID").unwrap();
    let google_client_secret = env::var("GOOGLE_CLIENT_SECRET").unwrap();

    let client: BasicClient = BasicClient::new(
        ClientId::new(google_client_id),
        Some(ClientSecret::new(google_client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap());

    client
}

#[derive(Debug, Deserialize)]
pub struct UserProfile {
    pub email: String,
    pub email_verified: bool,
    pub given_name: String,
    pub family_name: String,
    pub sub: String,
    pub picture: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleRes {
    pub access_token: String,
    pub profile: UserProfile,
}

pub async fn google_callback(client: &BasicClient, code: String) -> Result<GoogleRes, Error> {
    // Process authorization code, expecting a token response back.
    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await
        .map_err(Error::OAuth2)?;

    // Use access token to request user info.
    let profile = reqwest::Client::new()
        .get("https://openidconnect.googleapis.com/v1/userinfo")
        .header(USER_AGENT.as_str(), "axum-login")
        .header(
            AUTHORIZATION.as_str(),
            format!("Bearer {}", token.access_token().secret()),
        )
        .send()
        .await
        .map_err(Error::Reqwest)?
        .json::<UserProfile>()
        .await
        .map_err(Error::Reqwest)?;

    Ok(GoogleRes {
        access_token: token.access_token().secret().to_string(),
        profile,
    })
}
