use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use oauth2::{basic::BasicClient, url::Url, CsrfToken};
use sqlx::{MySql, Pool};

pub mod auth;
pub mod file;
pub mod vault;

pub struct ServerState {
    pub pool: Pool<MySql>,
    pub auth_url: Url,
    pub csrf_token: CsrfToken,
    pub auth_client: BasicClient,
}

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
