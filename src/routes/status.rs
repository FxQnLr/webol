use std::sync::Arc;
use axum::extract::{State, WebSocketUpgrade};
use axum::response::Response;
use crate::AppState;
use crate::services::ping::status_websocket;

#[axum_macros::debug_handler]
pub async fn status(State(state): State<Arc<AppState>>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| status_websocket(socket, state))
}
