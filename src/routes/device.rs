use std::sync::Arc;
use axum::extract::State;
use axum::Json;
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, info};
use crate::auth::auth;
use crate::db::Device;
use crate::error::Error;

pub async fn get(State(state): State<Arc<crate::AppState>>, headers: HeaderMap, Json(payload): Json<GetDevicePayload>) -> Result<Json<Value>, Error> {
    info!("add device {}", payload.id);
    let secret = headers.get("authorization");
    if auth(&state.config, secret).map_err(Error::Auth)? {
        let device = sqlx::query_as!(
            Device,
            r#"
            SELECT id, mac, broadcast_addr, ip, times
            FROM devices
            WHERE id = $1;
            "#,
            payload.id
        ).fetch_one(&state.db).await.map_err(Error::DB)?;

        debug!("got device {:?}", device);

        Ok(Json(json!(device)))
    } else {
        Err(Error::Generic)
    }
}

#[derive(Deserialize)]
pub struct GetDevicePayload {
    id: String,
}

pub async fn put(State(state): State<Arc<crate::AppState>>, headers: HeaderMap, Json(payload): Json<PutDevicePayload>) -> Result<Json<Value>, Error> {
    info!("add device {} ({}, {}, {})", payload.id, payload.mac, payload.broadcast_addr, payload.ip);
    let secret = headers.get("authorization");
    if auth(&state.config, secret).map_err(Error::Auth)? {
        sqlx::query!(
            r#"
            INSERT INTO devices (id, mac, broadcast_addr, ip)
            VALUES ($1, $2, $3, $4);
            "#,
            payload.id,
            payload.mac,
            payload.broadcast_addr,
            payload.ip
        ).execute(&state.db).await.map_err(Error::DB)?;

        Ok(Json(json!(PutDeviceResponse { success: true })))
    } else {
        Err(Error::Generic)
    }
}

#[derive(Deserialize)]
pub struct PutDevicePayload {
    id: String,
    mac: String,
    broadcast_addr: String,
    ip: String
}

#[derive(Serialize)]
pub struct PutDeviceResponse {
    success: bool
}

pub async fn post(State(state): State<Arc<crate::AppState>>, headers: HeaderMap, Json(payload): Json<PostDevicePayload>) -> Result<Json<Value>, Error> {
    info!("edit device {} ({}, {}, {})", payload.id, payload.mac, payload.broadcast_addr, payload.ip);
    let secret = headers.get("authorization");
    if auth(&state.config, secret).map_err(Error::Auth)? {
        let device = sqlx::query_as!(
            Device,
            r#"
            UPDATE devices
            SET mac = $1, broadcast_addr = $2, ip = $3 WHERE id = $4
            RETURNING id, mac, broadcast_addr, ip, times;
            "#,
            payload.mac,
            payload.broadcast_addr,
            payload.ip,
            payload.id
        ).fetch_one(&state.db).await.map_err(Error::DB)?;

        Ok(Json(json!(device)))
    } else {
        Err(Error::Generic)
    }
}

#[derive(Deserialize)]
pub struct PostDevicePayload {
    id: String,
    mac: String,
    broadcast_addr: String,
    ip: String,
}
