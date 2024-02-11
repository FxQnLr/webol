use crate::auth::auth;
use crate::config::SETTINGS;
use crate::db::Device;
use crate::error::Error;
use crate::services::ping::Value as PingValue;
use crate::wol::{create_buffer, send_packet};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

#[axum_macros::debug_handler]
pub async fn start(
    State(state): State<Arc<crate::AppState>>,
    headers: HeaderMap,
    Json(payload): Json<Payload>,
) -> Result<Json<Value>, Error> {
    info!("POST request");
    let secret = headers.get("authorization");
    let authorized = auth(secret).map_err(Error::Auth)?;
    if authorized {
        let device = sqlx::query_as!(
            Device,
            r#"
            SELECT id, mac, broadcast_addr, ip, times
            FROM devices
            WHERE id = $1;
            "#,
            payload.id
        )
        .fetch_one(&state.db)
        .await
        .map_err(Error::DB)?;

        info!("starting {}", device.id);

        let bind_addr = SETTINGS
            .get_string("bindaddr")
            .unwrap_or("0.0.0.0:1111".to_string());

        let _ = send_packet(
            &bind_addr.parse().map_err(Error::IpParse)?,
            &device.broadcast_addr.parse().map_err(Error::IpParse)?,
            &create_buffer(&device.mac)?,
        )?;
        let dev_id = device.id.clone();
        let uuid = if payload.ping.is_some_and(|ping| ping) {
            let mut uuid: Option<String> = None;
            for (key, value) in state.ping_map.clone() {
                if value.ip == device.ip {
                    debug!("service already exists");
                    uuid = Some(key);
                    break;
                }
            }
            let uuid_gen = match uuid {
                Some(u) => u,
                None => Uuid::new_v4().to_string(),
            };
            let uuid_genc = uuid_gen.clone();

            tokio::spawn(async move {
                debug!("init ping service");
                state.ping_map.insert(
                    uuid_gen.clone(),
                    PingValue {
                        ip: device.ip.clone(),
                        online: false,
                    },
                );

                crate::services::ping::spawn(
                    state.ping_send.clone(),
                    device,
                    uuid_gen.clone(),
                    &state.ping_map,
                    &state.db,
                )
                .await;
            });
            Some(uuid_genc)
        } else {
            None
        };
        Ok(Json(json!(Response {
            id: dev_id,
            boot: true,
            uuid
        })))
    } else {
        Err(Error::Generic)
    }
}

#[derive(Deserialize)]
pub struct Payload {
    id: String,
    ping: Option<bool>,
}

#[derive(Serialize)]
struct Response {
    id: String,
    boot: bool,
    uuid: Option<String>,
}
