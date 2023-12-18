use std::sync::Arc;

use anyhow::anyhow;
use axum::{
    debug_handler,
    extract::{Query, State},
    routing, Json, Router,
};
use base64ct::{Base64, Encoding};
use chrono::{DateTime, Utc};
use futures::future;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

use crate::models::file::File;

use super::{AppError, ServerState};

pub fn file() -> Router<Arc<ServerState>> {
    Router::new().route("/", routing::get(get_unsynced).post(save))
}

async fn save(
    State(state): State<Arc<ServerState>>,
    Json(values): Json<Vec<File>>,
) -> Result<(), AppError> {
    let mut tasks = Vec::with_capacity(values.len());
    let now = Utc::now().naive_utc();
    for value in values {
        let mut hasher = Md5::new();
        hasher.update(&value.content);
        let hash = Base64::encode_string(&hasher.finalize());
        tasks.push(
            sqlx::query!(
                r#"
                    INSERT INTO file (path, vault_id, content, hash, last_sync) 
                    VALUES (?, ?, ?, ?, ?) 
                    ON DUPLICATE KEY 
                        UPDATE content = ?, hash = ?, last_sync = ?
                "#,
                &value.path,
                &value.vault_id,
                &value.content,
                &hash,
                &now,
                &value.content,
                &hash,
                &now
            )
            .execute(&state.pool),
        );
    }

    future::try_join_all(tasks).await?;

    Ok(())
}

#[debug_handler]
async fn get_unsynced(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<LastSync>,
) -> Result<Json<Files>, AppError> {
    dbg!(&query);
    let last_sync = DateTime::from_timestamp(query.last_sync, 0)
        .ok_or(anyhow!("Invalid timestamp"))?
        .naive_utc();

    let unsynced = sqlx::query_as!(
        File,
        r#"
            SELECT id, path, vault_id, content, hash, last_sync
            FROM file
            WHERE last_sync > ?
        "#,
        &last_sync
    )
    .fetch_all(&state.pool)
    .await?;

    let resp = Files { files: unsynced };

    Ok(Json(resp))
}

#[derive(Deserialize, Debug)]
struct LastSync {
    last_sync: i64,
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
