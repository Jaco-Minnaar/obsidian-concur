use std::sync::Arc;

use axum::{debug_handler, extract::State, http::StatusCode, routing, Json, Router};

use crate::models::vault::Vault;

use super::ServerState;

pub fn vault() -> Router<Arc<ServerState>> {
    Router::new().route("/", routing::post(save))
}

#[debug_handler]
async fn save(
    State(state): State<Arc<ServerState>>,
    Json(value): Json<Vault>,
) -> (StatusCode, Json<Vault>) {
    let vault = sqlx::query_as!(Vault, "SELECT * FROM vault where name = ?", &value.name)
        .fetch_optional(&state.pool)
        .await
        .expect("Failed to execute query");

    if let Some(vault) = vault {
        log::debug!("Found vault {}.", vault.name);
        return (StatusCode::OK, Json(vault));
    }

    log::debug!("Creating vault {}", &value.name);
    sqlx::query!("INSERT INTO vault (name) VALUES (?)", &value.name)
        .execute(&state.pool)
        .await
        .expect("Failed to execute query");

    let vault = sqlx::query_as!(Vault, "SELECT * FROM vault where name = ?", &value.name)
        .fetch_one(&state.pool)
        .await
        .expect("Failed to execute query");

    (StatusCode::CREATED, Json(vault))
}
