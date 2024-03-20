mod models;
mod routes;

use std::sync::Arc;

use axum::Router;
use dotenvy::dotenv;
use libsql::Connection;
use routes::{file::file, vault::vault, ServerState};
use shuttle_axum::ShuttleAxum;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_turso::Turso(
        addr = "{secrets.LIBSQL_URL}",
        token = "{secrets.LIBSQL_TOKEN}"
    )]
    client: Connection,
) -> ShuttleAxum {
    let router = start(client).await;

    Ok(router.into())
}

async fn start(connection: Connection) -> Router {
    dotenv().ok();
    TermLogger::init(
        log::LevelFilter::Debug,
        Config::default(),
        TerminalMode::Stdout,
        ColorChoice::Auto,
    )
    .expect("Failed to initialize logger");

    let state = Arc::new(ServerState { connection });

    let app: Router = Router::new()
        .nest("/file", file())
        .nest("/vault", vault())
        .with_state(state);

    app
}
