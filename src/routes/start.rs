use crate::db::Device;
use crate::error::Error;
use crate::services::ping::Value as PingValue;
use crate::wol::{create_buffer, send_packet};
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

pub async fn start(
    State(state): State<Arc<crate::AppState>>,
    Json(payload): Json<Payload>,
) -> Result<Json<Value>, Error> {
    info!("POST request");
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
    .await?;

    info!("starting {}", device.id);

    let bind_addr = "0.0.0.0:0";

    let _ = send_packet(
        bind_addr,
        &device.broadcast_addr,
        &create_buffer(&device.mac.to_string())?,
    )?;
    let dev_id = device.id.clone();
    let uuid = if payload.ping.is_some_and(|ping| ping) {
        Some(setup_ping(state, device))
    } else {
        None
    };
    Ok(Json(json!(Response {
        id: dev_id,
        boot: true,
        uuid
    })))
}

fn setup_ping(state: Arc<crate::AppState>, device: Device) -> String {
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
    let uuid_ret = uuid_gen.clone();

    debug!("init ping service");
    state.ping_map.insert(
        uuid_gen.clone(),
        PingValue {
            ip: device.ip,
            online: false,
        },
    );

    tokio::spawn(async move {
        crate::services::ping::spawn(
            state.ping_send.clone(),
            &state.config,
            device,
            uuid_gen,
            &state.ping_map,
            &state.db,
        )
        .await;
    });

    uuid_ret
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
