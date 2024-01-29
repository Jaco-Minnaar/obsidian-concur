use axum::{extract::Request, middleware::Next, response::Response};
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub sub: String,
}

pub async fn jwt_middleware(request: Request, next: Next) -> Response {
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .unwrap_or_default();

    let secret = include_bytes!("../key");

    match jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    ) {
        Err(err) => {
            tracing::error!("Failed to decode token: {:?}", err);
            return Response::builder()
                .status(401)
                .body("Invalid token".into())
                .unwrap();
        }
        _ => {}
    };

    next.run(request).await
}
