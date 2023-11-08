use axum::headers::HeaderMap;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use axum::extract::State;
use serde_json::{json, Value};
use tracing::{debug, info};
use uuid::Uuid;
use crate::auth::auth;
use crate::config::SETTINGS;
use crate::wol::{create_buffer, send_packet};
use crate::db::Device;
use crate::error::WebolError;
use crate::services::ping::PingValue;

#[axum_macros::debug_handler]
pub async fn start(State(state): State<Arc<crate::AppState>>, headers: HeaderMap, Json(payload): Json<StartPayload>) -> Result<Json<Value>, WebolError> {
    info!("POST request");
    let secret = headers.get("authorization");
    let authorized = auth(secret).map_err(WebolError::Auth)?;
    if authorized {
        let device = sqlx::query_as!(
            Device,
            r#"
            SELECT id, mac, broadcast_addr, ip, times
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
        let dev_id = device.id.clone();
        let uuid = if payload.ping.is_some_and(|ping| ping) {
            let uuid_gen = Uuid::new_v4().to_string();
            let uuid_genc = uuid_gen.clone();
            // TODO: Check if service already runs
            tokio::spawn(async move {
                debug!("init ping service");
                state.ping_map.insert(uuid_gen.clone(), PingValue { ip: device.ip.clone(), online: false });

                crate::services::ping::spawn(state.ping_send.clone(), device, uuid_gen.clone(), &state.ping_map, &state.db).await
            });
            Some(uuid_genc)
        } else { None };
        Ok(Json(json!(StartResponse { id: dev_id, boot: true, uuid })))
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
    uuid: Option<String>,
}
