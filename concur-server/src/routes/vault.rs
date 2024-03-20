use std::sync::Arc;

use anyhow::anyhow;
use axum::{debug_handler, extract::State, http::StatusCode, routing, Json, Router};

use crate::models::vault::Vault;

use super::{AppError, ServerState};

pub fn vault() -> Router<Arc<ServerState>> {
    Router::new().route("/", routing::post(save))
}

#[debug_handler]
async fn save(
    State(state): State<Arc<ServerState>>,
    Json(value): Json<Vault>,
) -> Result<(StatusCode, Json<Vault>), AppError> {
    let mut results = state
        .connection
        .query(
            "SELECT * FROM vault where name = ?1",
            libsql::params!(value.name.as_str()),
        )
        .await
        .expect("Failed to execute query");

    if let Some(row) = results.next().await? {
        log::debug!("Vault {} already exists. Returning it.", &value.name);
        let vault = Vault {
            id: row.get(0)?,
            name: row.get(1)?,
        };
        return Ok((StatusCode::OK, Json(vault)));
    }

    log::debug!("Creating vault {}", &value.name);
    let mut results = state
        .connection
        .query(
            "INSERT INTO vault (name) VALUES (?1) RETURNING id, name",
            libsql::params!(value.name.as_str()),
        )
        .await?;

    if let Some(row) = results.next().await? {
        log::debug!("Vault {} already exists. Returning it.", &value.name);
        let vault = Vault {
            id: row.get(0)?,
            name: row.get(1)?,
        };
        return Ok((StatusCode::CREATED, Json(vault)));
    }

    return Err(anyhow!("Could not insert vault {}", value.name).into());
}
