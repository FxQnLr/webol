use std::error::Error;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use tracing::error;
use crate::auth::AuthError;

#[derive(Debug)]
pub enum WebolError {
    Auth(AuthError),
    Generic,
    Server(Box<dyn Error>),
}

impl IntoResponse for WebolError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            WebolError::Auth(err) => err.get(),
            WebolError::Generic => (StatusCode::INTERNAL_SERVER_ERROR, ""),
            WebolError::Server(err) => {
                error!("server error: {}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            },

        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
