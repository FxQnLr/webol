use ::ipnetwork::IpNetworkError;
use axum::http::header::ToStrError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use mac_address::MacParseError;
use serde_json::json;
use utoipa::ToSchema;
use std::io;
use tracing::error;

#[derive(Debug, thiserror::Error, ToSchema)]
pub enum Error {
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

    #[error("header parse: {source}")]
    ParseHeader {
        #[from]
        source: ToStrError,
    },

    #[error("string parse: {source}")]
    IpParse {
        #[from]
        source: IpNetworkError,
    },

    #[error("mac parse: {source}")]
    MacParse {
        #[from]
        source: MacParseError,
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
            Self::Db { source } => {
                error!("{source}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            }
            Self::Io { source } => {
                error!("{source}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            }
            Self::ParseHeader { source } => {
                error!("{source}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            }
            Self::ParseInt { source } => {
                error!("{source}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            }
            Self::MacParse { source } => {
                error!("{source}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            }
            Self::IpParse { source } => {
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
