use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize, Serialize, Debug, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: Option<i32>,
    pub vault_id: i32,
    pub path: String,
    pub content: String,

    #[serde(skip_deserializing, skip_serializing)]
    pub hash: String,

    #[serde(skip_deserializing)]
    pub last_sync: NaiveDateTime,
}
