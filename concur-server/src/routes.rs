use std::{collections::HashMap, sync::Mutex};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use oauth2::{basic::BasicClient, url::Url, CsrfToken};
use sqlx::{MySql, Pool};
use tokio::sync::oneshot::{Receiver, Sender};

pub mod auth;
pub mod file;
pub mod vault;

pub struct ServerState {
    pub pool: Pool<MySql>,
    pub auth_url: Url,
    pub csrf_token: CsrfToken,
    pub auth_client: BasicClient,
    pub client_ids: Mutex<
        HashMap<
            String,
            (
                DateTime<Utc>,
                Option<Receiver<String>>,
                Option<Sender<String>>,
            ),
        >,
    >,
}

enum AppError {
    Status(StatusCode, String),
    Other(anyhow::Error),
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Status(status, body) => (status, body).into_response(),
            AppError::Other(err) => {
                tracing::error!("Unhandled error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            }
        }
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Other(err.into())
    }
}
