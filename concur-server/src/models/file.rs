use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: Option<i32>,
    pub vault_id: i32,
    pub path: String,
    pub content: String,
    pub last_sync: i64,
}
