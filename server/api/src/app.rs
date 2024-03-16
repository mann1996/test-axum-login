use axum::{http::StatusCode, response::IntoResponse, routing::get, Extension};
use axum_login::{
    login_required,
    tower_sessions::{cookie::SameSite, Expiry, MemoryStore, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use dotenvy::dotenv;
use oauth2::basic::BasicClient;
use sea_orm::DatabaseConnection;
use time::Duration;

use crate::{
    auth::{google::build_google_client, session::Backend},
    db::{connect_db, run_migration},
    graphql::{self, build_schema},
    routes::{self, protected},
};

use std::{env, net::SocketAddr, str::FromStr};

pub struct App {
    db: DatabaseConnection,
    google_client: BasicClient,
}

impl App {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();
        let db = connect_db().await;
        run_migration(&db).await;

        // google oauth client
        let google_client = build_google_client();

        Ok(Self { db, google_client })
    }

    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        // Session layer.
        //
        // This uses `tower-sessions` to establish a layer that will provide the session
        // as a request extension.
        let session_store = MemoryStore::default();
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_same_site(SameSite::Lax) // Ensure we send the cookie from the OAuth redirect.
            .with_expiry(Expiry::OnInactivity(Duration::days(1)));

        // Auth service.
        //
        // This combines the session layer with our backend to establish the auth
        // service which will provide the auth session as a request extension.

        let backend = Backend::new(self.db, self.google_client);
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        // let schema = build_schema();

        let router = protected::router()
            // .route(
            //     "/graphql",
            //     get(graphql::graphiql).post(graphql::graphql_handler),
            // )
            .route_layer(login_required!(Backend))
            .merge(routes::auth::router())
            // .layer(Extension(schema))
            .layer(auth_layer)
            .fallback(handler_404);

        // listen
        let addr = SocketAddr::from_str(&env::var("LISTEN_HOST_PORT").unwrap())
            .expect("Invalid LISTEN_HOST_PORT environment variable");
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, router.into_make_service()).await?;

        Ok(())
    }
}
async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 - Not Found")
}
