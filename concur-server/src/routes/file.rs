use std::sync::Arc;

use anyhow::anyhow;
use axum::{
    debug_handler,
    extract::{Query, State},
    routing, Json, Router,
};
use base64ct::{Base64, Encoding};
use chrono::{DateTime, Utc};

use futures::TryStreamExt;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

use crate::models::file::File;

use super::{AppError, ServerState};

pub fn file() -> Router<Arc<ServerState>> {
    Router::new().route("/", routing::get(get_unsynced).post(add_file))
}

async fn add_file(
    State(state): State<Arc<ServerState>>,
    Json(file): Json<File>,
) -> Result<(), AppError> {
    let now = Utc::now().timestamp_millis();

    let query_str = r#"
            INSERT INTO file (path, vault_id, content, hash, last_sync)
                VALUES (?1, ?2, ?3, ?4, ?5)
        "#;

    let hash = Base64::encode_string(Md5::digest(&file.content).as_slice());

    state.connection.execute(
        &query_str,
        libsql::params!(file.path, file.vault_id, file.content, hash, now),
    );

    Ok(())
}

#[debug_handler]
async fn get_unsynced(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<LastSync>,
) -> Result<Json<Files>, AppError> {
    log::info!("Getting unsynced files for vault {}", query.vault_id);
    dbg!(&query);

    let unsynced = state
        .connection
        .query(
            r#"
            SELECT id, path, vault_id, content, hash, last_sync
            FROM file
            WHERE last_sync > ?1
            AND vault_id = ?2
        "#,
            libsql::params!(&query.last_sync, &query.vault_id),
        )
        .await?;

    let files = unsynced
        .into_stream()
        .and_then(|row| async move {
            let id: i32 = row.get(0)?;
            let path: String = row.get(1)?;
            let vault_id: i32 = row.get(2)?;
            let content: String = row.get(3)?;
            let hash: String = row.get(4)?;
            let last_sync: i64 = row.get(5)?;

            let file = File {
                id: Some(id),
                path,
                vault_id,
                content,
                hash,
                last_sync,
            };

            Ok(file)
        })
        .try_collect::<Vec<File>>()
        .await?;

    let resp = Files { files };

    Ok(Json(resp))
}

#[derive(Deserialize, Debug)]
struct LastSync {
    last_sync: i64,
    vault_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FileHashes {
    hashes: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Files {
    files: Vec<File>,
}
