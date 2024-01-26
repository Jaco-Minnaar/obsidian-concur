use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Query, State},
    response::{Html, Redirect},
    routing, Router,
};
use oauth2::{reqwest::async_http_client, AuthorizationCode, CsrfToken, TokenResponse};

use super::{AppError, ServerState};

pub fn auth() -> Router<Arc<ServerState>> {
    Router::new()
        .route("/", routing::get(auth_page))
        .route("/redirect", routing::get(redirect))
}

async fn auth_page(State(state): State<Arc<ServerState>>) -> Result<Redirect, AppError> {
    let auth_url = &state.auth_url;

    Ok(Redirect::to(auth_url.as_str()))
}

async fn redirect(
    State(state): State<Arc<ServerState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, AppError> {
    let code = params
        .get("code")
        .ok_or(anyhow::anyhow!("No code in params"))?;

    let code = AuthorizationCode::new(code.to_string());

    let csrf_state = params
        .get("state")
        .ok_or(anyhow::anyhow!("No state in params"))?;

    let csrf_state = CsrfToken::new(csrf_state.to_string());

    if csrf_state.secret() != state.csrf_token.secret() {
        return Err(anyhow::anyhow!("CSRF token mismatch").into());
    }

    let token_response = state
        .auth_client
        .exchange_code(code)
        .request_async(async_http_client)
        .await?;

    let token = token_response.access_token().secret();

    log::info!("Token: {}", token);

    Ok(Html(format!(
        r#"
        <html>
            <head>
                <title>Concur Auth Success</title>
            </head>
            <body>
                <h1>Successfully authenticated.</h1>
                <p>You can now close this browser tab and return to your application</p>
            </body>
        </html>"#
    )))
}
