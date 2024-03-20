mod models;
mod routes;

use std::sync::Arc;

use axum::Router;
use dotenvy::dotenv;
use libsql::Connection;
use routes::{file::file, vault::vault, ServerState};
use shuttle_axum::ShuttleAxum;

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_turso::Turso(
        addr = "{secrets.LIBSQL_URL}",
        token = "{secrets.LIBSQL_TOKEN}",
        local_addr = "{secrets.LIBSQL_LOCAL_URL}"
    )]
    client: Connection,
) -> ShuttleAxum {
    let router = start(client).await;

    Ok(router.into())
}

async fn start(connection: Connection) -> Router {
    dotenv().ok();
    // tracing_subscriber::fmt::init();

    let state = Arc::new(ServerState { connection });

    let app: Router = Router::new()
        .nest("/file", file())
        .nest("/vault", vault())
        .with_state(state);

    app
}
