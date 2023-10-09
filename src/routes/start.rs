use axum::headers::HeaderMap;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use axum::extract::State;
use serde_json::{json, Value};
use tracing::info;
use crate::auth::auth;
use crate::config::SETTINGS;
use crate::wol::{create_buffer, send_packet};
use crate::db::Device;
use crate::error::WebolError;

pub async fn start(State(state): State<Arc<crate::AppState>>, headers: HeaderMap, Json(payload): Json<StartPayload>) -> Result<Json<Value>, WebolError> {
    let secret = headers.get("authorization");
    if auth(secret).map_err(WebolError::Auth)? {
        let device = sqlx::query_as!(
            Device,
            r#"
            SELECT id, mac, broadcast_addr
            FROM devices
            WHERE id = $1;
            "#,
            payload.id
        ).fetch_one(&state.db).await.map_err(|err| WebolError::Server(Box::new(err)))?;

        info!("starting {}", device.id);

        let bind_addr = SETTINGS
            .get_string("bindaddr")
            .map_err(|err| WebolError::Server(Box::new(err)))?;

        let _ = send_packet(
            &bind_addr.parse().map_err(|err| WebolError::Server(Box::new(err)))?,
            &device.broadcast_addr.parse().map_err(|err| WebolError::Server(Box::new(err)))?,
            create_buffer(&device.mac).map_err(|err| WebolError::Server(Box::new(err)))?
        ).map_err(|err| WebolError::Server(Box::new(err)));
        Ok(Json(json!(StartResponse { id: device.id, boot: true })))
    } else {
        Err(WebolError::Generic)
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