mod models;
mod routes;

use std::{env, sync::Arc};

use axum::Router;
use dotenvy::dotenv;
use routes::{file::file, vault::vault, ServerState};
use sqlx::MySqlPool;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("Could not get database URL");
    let pool = MySqlPool::connect(&database_url)
        .await
        .expect("Could not connect to database");

    let state = Arc::new(ServerState { pool });

    let app: Router = Router::new()
        .nest("/file", file())
        .nest("/vault", vault())
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .expect("Failed to init axum");
}
