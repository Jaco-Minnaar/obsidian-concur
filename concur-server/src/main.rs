mod models;
mod routes;

use std::sync::Arc;

use axum::Router;
use dotenvy::dotenv;
use libsql::Connection;
use routes::{file::file, vault::vault, ServerState};
use shuttle_axum::ShuttleAxum;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode};

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
    CombinedLogger::init(vec![
        TermLogger::new(
            log::LevelFilter::Warn,
            ConfigBuilder::new().add_filter_ignore_str("concur").build(),
            TerminalMode::Stdout,
            ColorChoice::Always,
        ),
        TermLogger::new(
            log::LevelFilter::Debug,
            ConfigBuilder::new().add_filter_allow_str("concur").build(),
            TerminalMode::Stdout,
            ColorChoice::Always,
        ),
    ])
    .expect("Failed to initialize logger");

    let state = Arc::new(ServerState { connection });

    let app: Router = Router::new()
        .nest("/file", file())
        .nest("/vault", vault())
        .with_state(state);

    app
}
