use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Vault {
    pub id: Option<i32>,
    pub name: String,
}
