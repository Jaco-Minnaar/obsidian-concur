mod models;
mod routes;

use std::sync::Arc;

use axum::Router;
use dotenvy::dotenv;
use routes::{file::file, vault::vault, ServerState};
use shuttle_axum::ShuttleAxum;
use shuttle_secrets::SecretStore;
use sqlx::MySqlPool;

#[shuttle_runtime::main]
async fn axum(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttleAxum {
    let database_url = secret_store
        .get("DATABASE_URL")
        .expect("Could not get database URL.");

    let router = start(&database_url).await;

    Ok(router.into())
}

async fn start(database_url: &str) -> Router {
    dotenv().ok();
    // tracing_subscriber::fmt::init();

    let pool = MySqlPool::connect(&database_url)
        .await
        .expect("Could not connect to database");

    let state = Arc::new(ServerState { pool });

    let app: Router = Router::new()
        .nest("/file", file())
        .nest("/vault", vault())
        .with_state(state);

    app
}
