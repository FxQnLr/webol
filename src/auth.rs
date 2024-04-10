use crate::AppState;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::Deserialize;
use tracing::trace;

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
    trace!(?auth.method, "auth request");
    match auth.method {
        Methods::Key => {
            if let Some(secret) = headers.get("authorization") {
                if auth.secret.as_str() != secret {
                    trace!("auth failed, unknown secret");
                    return Err(StatusCode::UNAUTHORIZED);
                };
                trace!("auth successfull");
                let response = next.run(request).await;
                Ok(response)
            } else {
                trace!("auth failed, no secret");
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        Methods::None => Ok(next.run(request).await),
    }
}
