use crate::storage::Device;
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
    path = "/start/{id}",
    request_body = Option<SPayload>,
    responses(
        (status = 200, description = "start device with the given id", body = [Response])
    ),
    params(
        ("id" = String, Path, description = "device id")
    ),
    security((), ("api_key" = []))
)]
pub async fn post(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<String>,
    payload: Option<Json<SPayload>>,
) -> Result<Json<Value>, Error> {
    send_wol(state, &id, payload)
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
    send_wol(state, &id, None)
}

fn send_wol(
    state: Arc<crate::AppState>,
    id: &str,
    payload: Option<Json<SPayload>>,
) -> Result<Json<Value>, Error> {
    info!("start request for {id}");
    let device = Device::read(id)?;

    info!("starting {}", device.id);

    let bind_addr = "0.0.0.0:0";

    send_packet(
        bind_addr,
        &device.broadcast_addr.to_string(),
        &create_buffer(&device.mac.to_string())?
    )?;
    let dev_id = device.id.clone();
    let uuid = if let Some(pl) = payload {
        if pl.ping.is_some_and(|ping| ping) {
            if device.ip.is_none() {
                return Err(Error::NoIpOnPing);
            }
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
    // Safe: Only called when ip is set
    let ip = device.ip.unwrap();
    for (key, value) in state.ping_map.clone() {
        if value.ip == ip {
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
            ip,
            eta: get_eta(device.clone().times),
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
        )
        .await;
    });

    uuid_ret
}

fn get_eta(times: Option<Vec<u64>>) -> u64 {
    let times = if let Some(times) = times {
        times
    } else {
        vec![0]
    };

    times.iter().sum::<u64>() / u64::try_from(times.len()).unwrap()
}

#[derive(Deserialize, ToSchema)]
pub struct SPayload {
    ping: Option<bool>,
}

#[derive(Serialize, ToSchema)]
pub struct Response {
    id: String,
    boot: bool,
    uuid: Option<String>,
}
