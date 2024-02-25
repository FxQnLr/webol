use crate::db::Device;
use crate::error::Error;
use axum::extract::State;
use axum::Json;
use mac_address::MacAddress;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::types::ipnetwork::IpNetwork;
use std::{sync::Arc, str::FromStr};
use tracing::{debug, info};

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

#[derive(Deserialize)]
pub struct GetDevicePayload {
    id: String,
}

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
    sqlx::query!(
        r#"
        INSERT INTO devices (id, mac, broadcast_addr, ip)
        VALUES ($1, $2, $3, $4);
        "#,
        payload.id,
        mac,
        payload.broadcast_addr,
        ip
    )
    .execute(&state.db)
    .await?;

    Ok(Json(json!(PutDeviceResponse { success: true })))
}

#[derive(Deserialize)]
pub struct PutDevicePayload {
    id: String,
    mac: String,
    broadcast_addr: String,
    ip: String,
}

#[derive(Serialize)]
pub struct PutDeviceResponse {
    success: bool,
}

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

#[derive(Deserialize)]
pub struct PostDevicePayload {
    id: String,
    mac: String,
    broadcast_addr: String,
    ip: String,
}
