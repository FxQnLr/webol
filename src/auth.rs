use crate::AppState;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum Methods {
    Key,
    None,
}

pub async fn auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth = state.config.auth;
    match auth.method {
        Methods::Key => {
            if let Some(secret) = headers.get("authorization") {
                if auth.secret.as_str() != secret {
                    return Err(StatusCode::UNAUTHORIZED);
                };
                let response = next.run(request).await;
                Ok(response)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        Methods::None => Ok(next.run(request).await),
    }
}
