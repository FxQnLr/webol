use crate::db::Device;
use crate::error::Error;
use crate::services::ping::Value as PingValue;
use crate::wol::{create_buffer, send_packet};
use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, info};
use utoipa::ToSchema;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/start",
    request_body = PayloadOld,
    responses(
        (status = 200, description = "DEP", body = [Response])
    ),
    security((), ("api_key" = []))
)]
#[deprecated]
pub async fn start_payload(
    State(state): State<Arc<crate::AppState>>,
    Json(payload): Json<PayloadOld>,
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

#[utoipa::path(
    post,
    path = "/start/{id}",
    request_body = Option<Payload>,
    responses(
        (status = 200, description = "start the device with the given id", body = [Response])
    ),
    params(
        ("id" = String, Path, description = "device id")
    ),
    security((), ("api_key" = []))
)]
pub async fn post(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<String>,
    payload: Option<Json<Payload>>,
) -> Result<Json<Value>, Error> {
    send_wol(state, &id, payload).await
}

#[utoipa::path(
    get,
    path = "/start/{id}",
    responses(
        (status = 200, description = "start the device with the given id", body = [Response])
    ),
    params(
        ("id" = String, Path, description = "device id")
    ),
    security((), ("api_key" = []))
)]
pub async fn get(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, Error> {
    send_wol(state, &id, None).await
}

async fn send_wol(
    state: Arc<crate::AppState>,
    id: &str,
    payload: Option<Json<Payload>>,
) -> Result<Json<Value>, Error> {
    info!("Start request for {id}");
    let device = sqlx::query_as!(
        Device,
        r#"
        SELECT id, mac, broadcast_addr, ip, times
        FROM devices
        WHERE id = $1;
        "#,
        id
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
    let uuid = if let Some(pl) = payload {
        if pl.ping.is_some_and(|ping| ping) {
            Some(setup_ping(state, device))
        } else {
            None
        }
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

#[derive(Deserialize, ToSchema)]
#[deprecated]
pub struct PayloadOld {
    id: String,
    ping: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct Payload {
    ping: Option<bool>,
}

#[derive(Serialize, ToSchema)]
pub struct Response {
    id: String,
    boot: bool,
    uuid: Option<String>,
}
