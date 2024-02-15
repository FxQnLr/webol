use crate::auth::Error as AuthError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::io;
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("generic error")]
    Generic,

    #[error("auth: {source}")]
    Auth {
        #[from]
        source: AuthError,
    },

    #[error("db: {source}")]
    Db {
        #[from]
        source: sqlx::Error,
    },

    #[error("buffer parse: {source}")]
    ParseInt {
        #[from]
        source: std::num::ParseIntError,
    },

    #[error("io: {source}")]
    Io {
        #[from]
        source: io::Error,
    },
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("{}", self.to_string());
        let (status, error_message) = match self {
            Self::Auth { source } => source.get(),
            Self::Generic => (StatusCode::INTERNAL_SERVER_ERROR, ""),
            Self::Db { source } => {
                error!("{source}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            }
            Self::Io { source } => {
                error!("{source}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            }
            Self::ParseInt { source } => {
                error!("{source}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            }
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
