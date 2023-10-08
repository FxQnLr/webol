use axum::headers::HeaderMap;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::auth::auth;

pub async fn start(headers: HeaderMap, Json(payload): Json<StartPayload>) -> Json<Value> {
    let mut res = StartResponse { id: payload.id, boot: false };
    if let Some(secret) = headers.get("authorization") {
        if !auth(secret.to_str().unwrap()) { Json(json!(res)) } else {
            res.boot = true;
            Json(json!(res))
        }
    } else {
        Json(json!(res))
    }
}

#[derive(Deserialize)]
pub struct StartPayload {
    id: String,
    _test: Option<bool>,
}

#[derive(Serialize)]
struct StartResponse {
    id: String,
    boot: bool,
}
