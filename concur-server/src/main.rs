mod auth;
mod models;
mod routes;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{http, Router};
use dotenvy::dotenv;
use oauth2::{
    basic::BasicClient, url::Url, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, TokenUrl,
};
use routes::{auth::auth, file::file, vault::vault, ServerState};
use shuttle_axum::ShuttleAxum;
use shuttle_secrets::SecretStore;
use sqlx::MySqlPool;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, info};

struct Config {
    database_url: String,
    github_client_id: String,
    github_client_secret: String,
}

#[shuttle_runtime::main]
async fn axum(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttleAxum {
    debug!("Debug logging is enabled.");
    let database_url = secret_store
        .get("DATABASE_URL")
        .expect("Could not get database URL.");

    let github_client_id = secret_store
        .get("GITHUB_CLIENT_ID")
        .expect("Could not get client ID.");
    let github_client_secret = secret_store
        .get("GITHUB_CLIENT_SECRET")
        .expect("Could not get client secret.");

    let config = Config {
        database_url,
        github_client_id,
        github_client_secret,
    };

    let router = start(config).await;

    info!("Starting Server");
    Ok(router.into())
}

fn auth_init(client_id: String, client_secret: String) -> (Url, CsrfToken, BasicClient) {
    let client_id = ClientId::new(client_id);

    let client_secret = ClientSecret::new(client_secret);

    let auth_url =
        AuthUrl::new("https://github.com/login/oauth/authorize".into()).expect("Invalid auth URL.");
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".into())
        .expect("Invalid token URL.");

    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:8080/auth/redirect".into())
                .expect("Invalid redirect URL."),
        );

    let (url, token) = client.authorize_url(CsrfToken::new_random).url();

    (url, token, client)
}

async fn start(config: Config) -> Router {
    dotenv().ok();
    // tracing_subscriber::fmt::init();

    let pool = MySqlPool::connect(&config.database_url)
        .await
        .expect("Could not connect to database");

    let (auth_url, csrf_token, auth_client) =
        auth_init(config.github_client_id, config.github_client_secret);

    let state = Arc::new(ServerState {
        pool,
        auth_url,
        csrf_token,
        auth_client,
        client_ids: Mutex::new(HashMap::new()),
    });

    let app: Router = Router::new()
        .nest("/file", file())
        .nest("/vault", vault())
        .nest("/auth", auth())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION]),
        )
        .with_state(state);

    app
}
