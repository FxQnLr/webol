use std::sync::Arc;
use axum::extract::State;
use axum::headers::HeaderMap;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::auth::auth;
use crate::db::Device;
use crate::error::WebolError;

pub async fn get_device(State(state): State<Arc<crate::AppState>>, headers: HeaderMap, Json(payload): Json<GetDevicePayload>) -> Result<Json<Value>, WebolError> {
    let secret = headers.get("authorization");
    if auth(secret).map_err(WebolError::Auth)? {
        let device = sqlx::query_as!(
            Device,
            r#"
            SELECT id, mac, broadcast_addr
            FROM devices
            WHERE id = $1;
            "#,
            payload.id
        ).fetch_one(&state.db).await.map_err(|err| WebolError::Server(Box::new(err)))?;

        Ok(Json(json!(device)))
    } else {
        Err(WebolError::Generic)
    }
}

#[derive(Deserialize)]
pub struct GetDevicePayload {
    id: String,
}

pub async fn put_device(State(state): State<Arc<crate::AppState>>, headers: HeaderMap, Json(payload): Json<PutDevicePayload>) -> Result<Json<Value>, WebolError> {
    let secret = headers.get("authorization");
    if auth(secret).map_err(WebolError::Auth)? {
        sqlx::query!(
            r#"
            INSERT INTO devices (id, mac, broadcast_addr)
            VALUES ($1, $2, $3);
            "#,
            payload.id,
            payload.mac,
            payload.broadcast_addr
        ).execute(&state.db).await.map_err(|err| WebolError::Server(Box::new(err)))?;

        Ok(Json(json!(PutDeviceResponse { success: true })))
    } else {
        Err(WebolError::Generic)
    }
}

#[derive(Deserialize)]
pub struct PutDevicePayload {
    id: String,
    mac: String,
    broadcast_addr: String,
}

#[derive(Serialize)]
pub struct PutDeviceResponse {
    success: bool
}

pub async fn post_device(State(state): State<Arc<crate::AppState>>, headers: HeaderMap, Json(payload): Json<PostDevicePayload>) -> Result<Json<Value>, WebolError> {
    let secret = headers.get("authorization");
    if auth(secret).map_err(WebolError::Auth)? {
        let device = sqlx::query_as!(
            Device,
            r#"
            UPDATE devices
            SET mac = $1, broadcast_addr = $2 WHERE id = $3
            RETURNING id, mac, broadcast_addr;
            "#,
            payload.mac,
            payload.broadcast_addr,
            payload.id
        ).fetch_one(&state.db).await.map_err(|err| WebolError::Server(Box::new(err)))?;

        Ok(Json(json!(device)))
    } else {
        Err(WebolError::Generic)
    }
}

#[derive(Deserialize)]
pub struct PostDevicePayload {
    id: String,
    mac: String,
    broadcast_addr: String,
}