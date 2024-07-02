use crate::error::Error;
use crate::storage::Device;
use axum::extract::Path;
use axum::Json;
use ipnetwork::IpNetwork;
use mac_address::MacAddress;
use serde::Deserialize;
use serde_json::{json, Value};
use std::str::FromStr;
use tracing::{debug, info};
use utoipa::ToSchema;

#[utoipa::path(
    get,
    path = "/device/{id}",
    responses(
        (status = 200, description = "Get `Device` information", body = [Device])
    ),
    params(
        ("id" = String, Path, description = "device id")
    ),
    security((), ("api_key" = []))
)]
pub async fn get(Path(id): Path<String>) -> Result<Json<Value>, Error> {
    info!("get device from path {}", id);

    let device = Device::read(&id)?;

    debug!("got device {:?}", device);

    Ok(Json(json!(device)))
}

#[derive(Deserialize, ToSchema)]
pub struct DPayload {
    id: String,
    mac: String,
    broadcast_addr: String,
    ip: Option<String>,
}

#[utoipa::path(
    put,
    path = "/device",
    request_body = DPayload,
    responses(
        (status = 200, description = "add device to storage", body = [Device])
    ),
    security((), ("api_key" = []))
)]
pub async fn put(Json(payload): Json<DPayload>) -> Result<Json<Value>, Error> {
    info!(
        "add device {} ({}, {}, {:?})",
        payload.id, payload.mac, payload.broadcast_addr, payload.ip
    );

    let ip = if let Some(ip_s) = payload.ip {
        Some(IpNetwork::from_str(&ip_s)?)
    } else {
        None
    };
    let mac = MacAddress::from_str(&payload.mac)?;
    let device = Device {
        id: payload.id,
        mac,
        broadcast_addr: payload.broadcast_addr,
        ip,
        times: None,
    };
    device.write()?;

    Ok(Json(json!(device)))
}

#[utoipa::path(
    post,
    path = "/device",
    request_body = DPayload,
    responses(
        (status = 200, description = "update device in storage", body = [Device])
    ),
    security((), ("api_key" = []))
)]
pub async fn post(Json(payload): Json<DPayload>) -> Result<Json<Value>, Error> {
    info!(
        "edit device {} ({}, {}, {:?})",
        payload.id, payload.mac, payload.broadcast_addr, payload.ip
    );

    let ip = if let Some(ip_s) = payload.ip {
        Some(IpNetwork::from_str(&ip_s)?)
    } else {
        None
    };
    let mac = MacAddress::from_str(&payload.mac)?;
    let times = Device::read(&payload.id)?.times;

    let device = Device {
        id: payload.id,
        mac,
        broadcast_addr: payload.broadcast_addr,
        ip,
        times,
    };
    device.write()?;

    Ok(Json(json!(device)))
}
