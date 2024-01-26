use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Query, State},
    response::{Html, Redirect},
    routing, Json, Router,
};
use chrono::Utc;
use oauth2::{reqwest::async_http_client, AuthorizationCode, CsrfToken, TokenResponse};
use serde::Serialize;
use tokio::sync::oneshot;
use tracing::{debug, info};
use uuid::Uuid;

use super::{AppError, ServerState};

pub fn auth() -> Router<Arc<ServerState>> {
    Router::new()
        .route("/", routing::get(auth_page))
        .route("/client_id", routing::post(get_client_id))
        .route("/start", routing::get(start))
        .route("/redirect", routing::get(redirect))
        .route("/token", routing::post(token))
}

async fn auth_page(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Html<String>, AppError> {
    let auth_url = &state.auth_url;

    let client_id = query
        .get("client_id")
        .ok_or(anyhow::anyhow!("No client ID in params"))?;

    Ok(Html(format!(
        r#"<!DOCTYPE html>
        <html>
            <head>
                <title>Concur Auth</title>
            </head>
            <body>
                Redirecting to GitHub Auth...
                
                <script>
                    const clientId = "{}";
                    console.log("Setting client ID:", clientId);
                    window.localStorage.setItem("client_id", clientId);
                    window.location = "{}";
                </script>
            </body>
        </html>"#,
        client_id, auth_url,
    )))
}

async fn get_client_id(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<StartResponse>, AppError> {
    debug!("Getting client ID");
    let client_id = {
        let mut client_ids = state
            .client_ids
            .lock()
            .or(Err(anyhow::anyhow!("Failed to lock client IDs")))?;

        let client_id = Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel();

        client_ids.insert(client_id.clone(), (Utc::now(), Some(rx), Some(tx)));

        client_id
    };

    Ok(Json(StartResponse { client_id }))
}

async fn start(
    State(state): State<Arc<ServerState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ConcurTokenResponse>, AppError> {
    let client_id = params
        .get("client_id")
        .ok_or(anyhow::anyhow!("No client ID in params"))?;

    let rx = {
        let mut client_ids = state
            .client_ids
            .lock()
            .or(Err(anyhow::anyhow!("Failed to lock client IDs")))?;

        client_ids
            .get_mut(client_id)
            .ok_or(anyhow::anyhow!("No client ID found"))?
            .1
            .take()
            .ok_or(anyhow::anyhow!("No receiver found"))?
    };

    let token = rx
        .await
        .or(Err(anyhow::anyhow!("Failed to receive token")))?;

    debug!("Received token: {}", token);

    Ok(Json(ConcurTokenResponse {
        access_token: token,
    }))
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

    info!("Token: {}", token);

    Ok(Html(format!(
        r#"<!DOCTYPE html>
        <html>
            <head>
                <title>Concur Auth Success</title>
            </head>
            <body>
                <h1>Successfully authenticated.</h1>
                <p>You can now close this browser tab and return to your application</p>

                <script>
                    const client_id = window.localStorage.getItem("client_id");
                    fetch(new Request("token", {{
                        method: "POST",
                        headers: {{
                            "Content-Type": "application/json"
                        }},
                        body: JSON.stringify({{
                            client_id: client_id,
                            token: "{}"
                        }})
                    }})).then(() => {{
                        console.debug("Successfully sent token.")
                    }});
                </script>
            </body>
        </html>"#,
        token
    )))
}

async fn token(
    State(state): State<Arc<ServerState>>,
    Json(params): Json<HashMap<String, String>>,
) -> Result<(), AppError> {
    debug!("Received token: {:?}", params);
    let client_id = params
        .get("client_id")
        .ok_or(anyhow::anyhow!("No client ID in params"))?;

    let token = params
        .get("token")
        .ok_or(anyhow::anyhow!("No token in params"))?;

    let tx = {
        let mut client_ids = state
            .client_ids
            .lock()
            .or(Err(anyhow::anyhow!("Failed to lock client IDs")))?;

        client_ids
            .get_mut(client_id)
            .ok_or(anyhow::anyhow!("No client ID found"))?
            .2
            .take()
            .ok_or(anyhow::anyhow!("No sender found"))?
    };

    tx.send(token.to_string())
        .or(Err(anyhow::anyhow!("Failed to send token")))?;

    Ok(())
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct StartResponse {
    client_id: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ConcurTokenResponse {
    access_token: String,
    // expires_in: u64,
    // refresh_token: String,
}
