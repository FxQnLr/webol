use ipnetwork::IpNetworkError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use mac_address::MacParseError;
use serde_json::json;
use std::io;
use tracing::{error, warn};
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error, ToSchema)]
pub enum Error {
    #[error("json: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },

    #[error("buffer parse: {source}")]
    ParseInt {
        #[from]
        source: std::num::ParseIntError,
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

    #[error("No ip set for device but ping requested")]
    NoIpOnPing,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::Json { source } => {
                // !THIS REALLY SHOULD NOT HAPPEN!:
                // Json file has to had been tampered with by an external force
                error!("{source}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            }
            Self::ParseInt { source } => {
                // !THIS REALLY SHOULD NOT HAPPEN!:
                // Mac Address `&str` can't be converted to hex, which should be impossible trough
                // `MacAddress` type-check on device registration and edit
                error!("{source}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
            }
            Self::Io { source } => {
                if source.kind() == io::ErrorKind::NotFound {
                    warn!("unknown device requested");
                    (StatusCode::NOT_FOUND, "Requested device not found")
                } else {
                    error!("{source}");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Server Error")
                }
            }
            Self::MacParse { source } => {
                warn!("{source}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "The given MAC-Address couldn't be parsed",
                )
            }
            Self::IpParse { source } => {
                warn!("{source}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "The given IP-Address couldn't be parsed",
                )
            }
            Self::NoIpOnPing => {
                warn!("Ping requested but no ip given");
                (
                    StatusCode::BAD_REQUEST,
                    "No IP saved for device, ping can't be executed. Device may be started anyway",
                )
            }
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
