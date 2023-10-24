use axum::headers::HeaderMap;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use axum::extract::State;
use serde_json::{json, Value};
use tracing::{debug, info};
use crate::auth::auth;
use crate::config::SETTINGS;
use crate::wol::{create_buffer, send_packet};
use crate::db::Device;
use crate::error::WebolError;

pub async fn start(State(state): State<Arc<crate::AppState>>, headers: HeaderMap, Json(payload): Json<StartPayload>) -> Result<Json<Value>, WebolError> {
    info!("POST request");
    let secret = headers.get("authorization");
    let authorized = auth(secret).map_err(WebolError::Auth)?;
    if authorized {
        let device = sqlx::query_as!(
            Device,
            r#"
            SELECT id, mac, broadcast_addr
            FROM devices
            WHERE id = $1;
            "#,
            payload.id
        ).fetch_one(&state.db).await.map_err(WebolError::DB)?;

        info!("starting {}", device.id);

        let bind_addr = SETTINGS
            .get_string("bindaddr")
            .unwrap_or("0.0.0.0:1111".to_string());

        let _ = send_packet(
            &bind_addr.parse().map_err(WebolError::IpParse)?,
            &device.broadcast_addr.parse().map_err(WebolError::IpParse)?,
            create_buffer(&device.mac)?
        )?;

        if payload.ping.is_some_and(|ping| ping) {
            debug!("ping true");
            tokio::spawn(async move {
                debug!("Init ping service");
                crate::services::ping::spawn(state.ping_send.clone()).await
            });
        };
        Ok(Json(json!(StartResponse { id: device.id, boot: true })))
    } else {
        Err(WebolError::Generic)
    }
}

#[derive(Deserialize)]
pub struct StartPayload {
    id: String,
    ping: Option<bool>,
}

#[derive(Serialize)]
struct StartResponse {
    id: String,
    boot: bool,
}
