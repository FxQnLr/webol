use axum::headers::HeaderMap;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use axum::extract::State;
use serde_json::{json, Value};
use tracing::{error, info};
use crate::auth::{auth, AuthError};
use crate::config::SETTINGS;
use crate::wol::{create_buffer, send_packet};
use crate::db::Device;

pub async fn start(State(state): State<Arc<crate::AppState>>, headers: HeaderMap, Json(payload): Json<StartPayload>) -> Result<Json<Value>, StartError> {
    let secret = headers.get("authorization");
    if auth(secret).map_err(StartError::Auth)? {
        let device = sqlx::query_as!(
            Device,
            r#"
            SELECT id, mac, broadcast_addr
            FROM devices
            WHERE id = ?1;
            "#,
            payload.id
        ).fetch_one(&state.db).await.map_err(|err| StartError::Server(Box::new(err)))?;

        info!("starting {}", device.id);

        let bind_addr = SETTINGS
            .get_string("bindaddr")
            .map_err(|err| StartError::Server(Box::new(err)))?;

        let _ = send_packet(
            &bind_addr.parse().map_err(|err| StartError::Server(Box::new(err)))?,
            &device.broadcast_addr.parse().map_err(|err| StartError::Server(Box::new(err)))?,
            create_buffer(&device.mac).map_err(|err| StartError::Server(Box::new(err)))?
        ).map_err(|err| StartError::Server(Box::new(err)));
        Ok(Json(json!(StartResponse { id: device.id, boot: true })))
    } else {
        Err(StartError::Generic)
    }
}

#[derive(Deserialize)]
pub struct StartPayload {
    id: String,
    _test: Option<bool>,
}

#[derive(Serialize)]
struct StartResponse {
    id: String,
    boot: bool,
}

pub enum StartError {
    Auth(AuthError),
    Generic,
    Server(Box<dyn Error>),
}

impl IntoResponse for StartError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            StartError::Auth(err) => err.get(),
            StartError::Generic => (StatusCode::INTERNAL_SERVER_ERROR, ""),
            StartError::Server(err) => {
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