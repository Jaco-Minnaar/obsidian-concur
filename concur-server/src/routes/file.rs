use std::sync::Arc;

use axum::{
    debug_handler,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing, Json, Router,
};
use chrono::Utc;
use log::info;
use serde::{Deserialize, Serialize};

use crate::models::file::File;

use super::{AppError, ServerState};

pub fn file() -> Router<Arc<ServerState>> {
    Router::new().route(
        "/",
        routing::get(get_unsynced).post(add_file).put(update_file),
    )
}

async fn add_file(
    State(state): State<Arc<ServerState>>,
    Json(file): Json<File>,
) -> Result<(), AppError> {
    info!("Adding file {:?}", file.path);
    let now = Utc::now().timestamp_millis();
    dbg!(&file);

    let query_str = r#"
            INSERT INTO file (path, vault_id, content, last_sync)
                VALUES (?1, ?2, ?3, ?4)
        "#;

    state
        .connection
        .execute(
            &query_str,
            libsql::params!(file.path, file.vault_id, file.content, now),
        )
        .await?;

    Ok(())
}

async fn update_file(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<FileUpdateRequest>,
) -> Result<impl IntoResponse, AppError> {
    info!("Updating file {:?}", request.path);

    let now = Utc::now().timestamp_millis();

    let query_str = r#"
            UPDATE file
            SET path = ?1, content = ?2, last_sync = ?3
            WHERE path = ?4 AND vault_id = ?5
        "#;

    state
        .connection
        .execute(
            &query_str,
            libsql::params!(
                request.file.path,
                request.file.content,
                now,
                request.path,
                request.file.vault_id
            ),
        )
        .await?;

    Ok(StatusCode::OK)
}

#[debug_handler]
async fn get_unsynced(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<LastSync>,
) -> Result<Json<Files>, AppError> {
    log::info!("Getting unsynced files for vault {}", query.vault_id);
    dbg!(&query);

    let mut unsynced = state
        .connection
        .query(
            r#"
            SELECT id, path, vault_id, content, last_sync
            FROM file
            WHERE last_sync > ?1
            AND vault_id = ?2
        "#,
            libsql::params!(&query.last_sync, &query.vault_id),
        )
        .await?;

    let mut files = Vec::new();

    while let Some(row) = unsynced.next()? {
        let id: i32 = row.get(0)?;
        let path: String = row.get(1)?;
        let vault_id: i32 = row.get(2)?;
        let content: String = row.get(3)?;
        let last_sync: i64 = row.get(5)?;

        let file = File {
            id: Some(id),
            path,
            vault_id,
            content,
            last_sync,
        };

        files.push(file);
    }

    let resp = Files { files };

    Ok(Json(resp))
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FileUpdateRequest {
    path: String,
    file: File,
}

#[derive(Deserialize, Debug)]
struct LastSync {
    last_sync: i64,
    vault_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Files {
    files: Vec<File>,
}
