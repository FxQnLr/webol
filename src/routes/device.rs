use crate::db::Device;
use crate::error::Error;
use axum::extract::{Path, State};
use axum::Json;
use mac_address::MacAddress;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::types::ipnetwork::IpNetwork;
use std::{str::FromStr, sync::Arc};
use tracing::{debug, info};
use utoipa::ToSchema;

#[utoipa::path(
    get,
    path = "/device",
    request_body = GetDevicePayload,
    responses(
        (status = 200, description = "Get `Device` information", body = [Device])
    ),
    security(("api_key" = []))
)]
#[deprecated]
pub async fn get(
    State(state): State<Arc<crate::AppState>>,
    Json(payload): Json<GetDevicePayload>,
) -> Result<Json<Value>, Error> {
    info!("get device {}", payload.id);
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

    debug!("got device {:?}", device);

    Ok(Json(json!(device)))
}

#[utoipa::path(
    get,
    path = "/device/{id}",
    responses(
        (status = 200, description = "Get `Device` information", body = [Device])
    ),
    params(
        ("id" = String, Path, description = "Device id")
    ),
    security(("api_key" = []))
)]
pub async fn get_path(
    State(state): State<Arc<crate::AppState>>,
    Path(path): Path<String>,
) -> Result<Json<Value>, Error> {
    info!("get device from path {}", path);
    let device = sqlx::query_as!(
        Device,
        r#"
        SELECT id, mac, broadcast_addr, ip, times
        FROM devices
        WHERE id = $1;
        "#,
        path
    )
    .fetch_one(&state.db)
    .await?;

    debug!("got device {:?}", device);

    Ok(Json(json!(device)))
}

#[derive(Deserialize, ToSchema)]
pub struct GetDevicePayload {
    id: String,
}

#[utoipa::path(
    put,
    path = "/device",
    request_body = PutDevicePayload,
    responses(
        (status = 200, description = "List matching todos by query", body = [DeviceSchema])
    ),
    security(("api_key" = []))
)]
pub async fn put(
    State(state): State<Arc<crate::AppState>>,
    Json(payload): Json<PutDevicePayload>,
) -> Result<Json<Value>, Error> {
    info!(
        "add device {} ({}, {}, {})",
        payload.id, payload.mac, payload.broadcast_addr, payload.ip
    );

    let ip = IpNetwork::from_str(&payload.ip)?;
    let mac = MacAddress::from_str(&payload.mac)?;
    let device = sqlx::query_as!(
        Device,
        r#"
        INSERT INTO devices (id, mac, broadcast_addr, ip)
        VALUES ($1, $2, $3, $4)
        RETURNING id, mac, broadcast_addr, ip, times;
        "#,
        payload.id,
        mac,
        payload.broadcast_addr,
        ip
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!(device)))
}

#[derive(Deserialize, ToSchema)]
pub struct PutDevicePayload {
    id: String,
    mac: String,
    broadcast_addr: String,
    ip: String,
}

#[utoipa::path(
    post,
    path = "/device",
    request_body = PostDevicePayload,
    responses(
        (status = 200, description = "List matching todos by query", body = [DeviceSchema])
    ),
    security(("api_key" = []))
)]
pub async fn post(
    State(state): State<Arc<crate::AppState>>,
    Json(payload): Json<PostDevicePayload>,
) -> Result<Json<Value>, Error> {
    info!(
        "edit device {} ({}, {}, {})",
        payload.id, payload.mac, payload.broadcast_addr, payload.ip
    );
    let ip = IpNetwork::from_str(&payload.ip)?;
    let mac = MacAddress::from_str(&payload.mac)?;
    let device = sqlx::query_as!(
        Device,
        r#"
        UPDATE devices
        SET mac = $1, broadcast_addr = $2, ip = $3 WHERE id = $4
        RETURNING id, mac, broadcast_addr, ip, times;
        "#,
        mac,
        payload.broadcast_addr,
        ip,
        payload.id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!(device)))
}

#[derive(Deserialize, ToSchema)]
pub struct PostDevicePayload {
    id: String,
    mac: String,
    broadcast_addr: String,
    ip: String,
}
