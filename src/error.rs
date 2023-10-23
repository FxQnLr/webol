use std::error::Error;
use std::io;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use tracing::error;
use crate::auth::AuthError;

#[derive(Debug)]
pub enum WebolError {
    Generic,
    Auth(AuthError),
    Ping(surge_ping::SurgeError),
    DB(sqlx::Error),
    IpParse(<std::net::IpAddr as std::str::FromStr>::Err),
    BufferParse(std::num::ParseIntError),
    Broadcast(io::Error),
    Axum(axum::Error)
}

impl IntoResponse for WebolError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::Auth(err) => err.get(),
            Self::Generic => (StatusCode::INTERNAL_SERVER_ERROR, ""),
            Self::Ping(err) => {
                error!("Ping: {}", err.source().unwrap());
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            },
            Self::IpParse(err) => {
                error!("server error: {}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            },
            Self::DB(err) => {
                error!("server error: {}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            },
            Self::Broadcast(err) => {
                error!("server error: {}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            },
            Self::BufferParse(err) => {
                error!("server error: {}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            },
            Self::Axum(err) => {
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
